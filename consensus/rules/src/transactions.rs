use std::cmp::Ordering;

use curve25519_dalek::EdwardsPoint;
use monero_serai::{
    io::decompress_point,
    ringct::RctType,
    transaction::{Input, Output, Timelock, Transaction},
};

use crate::{
    batch_verifier::BatchVerifier, blocks::penalty_free_zone, check_point_canonically_encoded,
    is_decomposed_amount, HardFork,
};

// re-export.
pub use cuprate_types::TxVersion;

mod contextual_data;
mod ring_ct;
mod ring_signatures;
#[cfg(test)]
mod tests;

pub use contextual_data::*;
pub use ring_ct::RingCTError;

const MAX_BULLETPROOFS_OUTPUTS: usize = 16;
const MAX_TX_BLOB_SIZE: usize = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum TransactionError {
    #[error("The transactions version is incorrect.")]
    TransactionVersionInvalid,
    #[error("The transactions is too big.")]
    TooBig,
    //-------------------------------------------------------- OUTPUTS
    #[error("Output is not a valid point.")]
    OutputNotValidPoint,
    #[error("The transaction has an invalid output type.")]
    OutputTypeInvalid,
    #[error("The transaction is v1 with a 0 amount output.")]
    ZeroOutputForV1,
    #[error("The transaction is v2 with a non 0 amount output.")]
    NonZeroOutputForV2,
    #[error("The transaction has an output which is not decomposed.")]
    AmountNotDecomposed,
    #[error("The transactions outputs overflow.")]
    OutputsOverflow,
    #[error("The transactions outputs too much.")]
    OutputsTooHigh,
    #[error("The transactions has too many outputs.")]
    InvalidNumberOfOutputs,
    //-------------------------------------------------------- INPUTS
    #[error("One or more inputs don't have the expected number of decoys.")]
    InputDoesNotHaveExpectedNumbDecoys,
    #[error("The transaction has more than one mixable input with unmixable inputs.")]
    MoreThanOneMixableInputWithUnmixable,
    #[error("The key-image is not in the prime sub-group.")]
    KeyImageIsNotInPrimeSubGroup,
    #[error("Key-image is already spent.")]
    KeyImageSpent,
    #[error("The input is not the expected type.")]
    IncorrectInputType,
    #[error("The transaction has a duplicate ring member.")]
    DuplicateRingMember,
    #[error("The transaction inputs are not ordered.")]
    InputsAreNotOrdered,
    #[error("The transaction spends a decoy which is too young.")]
    OneOrMoreRingMembersLocked,
    #[error("The transaction inputs overflow.")]
    InputsOverflow,
    #[error("The transaction has no inputs.")]
    NoInputs,
    #[error("Ring member not in database or is not valid.")]
    RingMemberNotFoundOrInvalid,
    //-------------------------------------------------------- Ring Signatures
    #[error("Ring signature incorrect.")]
    RingSignatureIncorrect,
    //-------------------------------------------------------- RingCT
    #[error("RingCT Error: {0}.")]
    RingCTError(#[from] RingCTError),
}

//----------------------------------------------------------------------------------------------------------- OUTPUTS

/// Checks the output keys are canonically encoded points.
///
/// <https://monero-book.cuprate.org/consensus_rules/transactions/outputs.html#output-keys-canonical>
fn check_output_keys(outputs: &[Output]) -> Result<(), TransactionError> {
    for out in outputs {
        if !check_point_canonically_encoded(&out.key) {
            return Err(TransactionError::OutputNotValidPoint);
        }
    }

    Ok(())
}

/// Checks the output types are allowed for the given hard-fork.
///
/// This is also used during miner-tx verification.
///
/// <https://monero-book.cuprate.org/consensus_rules/transactions/outputs.html#output-type>
/// <https://monero-book.cuprate.org/consensus_rules/blocks/miner_tx.html#output-type>
pub(crate) fn check_output_types(outputs: &[Output], hf: HardFork) -> Result<(), TransactionError> {
    if hf == HardFork::V15 {
        for outs in outputs.windows(2) {
            if outs[0].view_tag.is_some() != outs[1].view_tag.is_some() {
                return Err(TransactionError::OutputTypeInvalid);
            }
        }
        return Ok(());
    }

    for out in outputs {
        if hf <= HardFork::V14 && out.view_tag.is_some()
            || hf >= HardFork::V16 && out.view_tag.is_none()
        {
            return Err(TransactionError::OutputTypeInvalid);
        }
    }
    Ok(())
}

/// Checks the individual outputs amount for version 1 txs.
///
/// ref: <https://monero-book.cuprate.org/consensus_rules/transactions/outputs.html#output-amount>
fn check_output_amount_v1(amount: u64, hf: HardFork) -> Result<(), TransactionError> {
    if amount == 0 {
        return Err(TransactionError::ZeroOutputForV1);
    }

    if hf >= HardFork::V2 && !is_decomposed_amount(&amount) {
        return Err(TransactionError::AmountNotDecomposed);
    }

    Ok(())
}

/// Checks the individual outputs amount for version 2 txs.
///
/// ref: <https://monero-book.cuprate.org/consensus_rules/transactions/outputs.html#output-amount>
const fn check_output_amount_v2(amount: u64) -> Result<(), TransactionError> {
    if amount == 0 {
        Ok(())
    } else {
        Err(TransactionError::NonZeroOutputForV2)
    }
}

/// Sums the outputs, checking for overflow and other consensus rules.
///
/// ref: <https://monero-book.cuprate.org/consensus_rules/transactions/outputs.html#output-amount>
/// &&   <https://monero-book.cuprate.org/consensus_rules/transactions/outputs.html#outputs-must-not-overflow>
fn sum_outputs(
    outputs: &[Output],
    hf: HardFork,
    tx_version: TxVersion,
) -> Result<u64, TransactionError> {
    let mut sum: u64 = 0;

    for out in outputs {
        let raw_amount = out.amount.unwrap_or(0);

        match tx_version {
            TxVersion::RingSignatures => check_output_amount_v1(raw_amount, hf)?,
            TxVersion::RingCT => check_output_amount_v2(raw_amount)?,
        }
        sum = sum
            .checked_add(raw_amount)
            .ok_or(TransactionError::OutputsOverflow)?;
    }

    Ok(sum)
}

/// Checks the number of outputs is allowed.
///
/// ref: <https://monero-book.cuprate.org/consensus_rules/transactions/outputs.html#2-outputs>
/// &&   <https://monero-book.cuprate.org/consensus_rules/transactions/ring_ct/bulletproofs.html#max-outputs>
/// &&   <https://monero-book.cuprate.org/consensus_rules/transactions/ring_ct/bulletproofs+.html#max-outputs>
fn check_number_of_outputs(
    outputs: usize,
    hf: HardFork,
    tx_version: TxVersion,
    bp_or_bpp: bool,
) -> Result<(), TransactionError> {
    if tx_version == TxVersion::RingSignatures {
        return Ok(());
    }

    if hf >= HardFork::V12 && outputs < 2 {
        return Err(TransactionError::InvalidNumberOfOutputs);
    }

    if bp_or_bpp && outputs > MAX_BULLETPROOFS_OUTPUTS {
        Err(TransactionError::InvalidNumberOfOutputs)
    } else {
        Ok(())
    }
}

/// Checks the outputs against all output consensus rules, returning the sum of the output amounts.
///
/// ref: <https://monero-book.cuprate.org/consensus_rules/transactions/outputs.html>
/// &&   <https://monero-book.cuprate.org/consensus_rules/transactions/ring_ct/bulletproofs.html#max-outputs>
/// &&   <https://monero-book.cuprate.org/consensus_rules/transactions/ring_ct/bulletproofs+.html#max-outputs>
fn check_outputs_semantics(
    outputs: &[Output],
    hf: HardFork,
    tx_version: TxVersion,
    bp_or_bpp: bool,
) -> Result<u64, TransactionError> {
    check_output_types(outputs, hf)?;
    check_output_keys(outputs)?;
    check_number_of_outputs(outputs.len(), hf, tx_version, bp_or_bpp)?;

    sum_outputs(outputs, hf, tx_version)
}

//----------------------------------------------------------------------------------------------------------- TIME LOCKS

/// Checks if an outputs unlock time has passed.
///
/// <https://monero-book.cuprate.org/consensus_rules/transactions/unlock_time.html>
pub const fn output_unlocked(
    time_lock: &Timelock,
    current_chain_height: usize,
    current_time_lock_timestamp: u64,
    hf: HardFork,
) -> bool {
    match *time_lock {
        Timelock::None => true,
        Timelock::Block(unlock_height) => {
            check_block_time_lock(unlock_height, current_chain_height)
        }
        Timelock::Time(unlock_time) => {
            check_timestamp_time_lock(unlock_time, current_time_lock_timestamp, hf)
        }
    }
}

/// Returns if a locked output, which uses a block height, can be spent.
///
/// ref: <https://monero-book.cuprate.org/consensus_rules/transactions/unlock_time.html#block-height>
const fn check_block_time_lock(unlock_height: usize, current_chain_height: usize) -> bool {
    // current_chain_height = 1 + top height
    unlock_height <= current_chain_height
}

/// Returns if a locked output, which uses a block height, can be spent.
///
/// ref: <https://monero-book.cuprate.org/consensus_rules/transactions/unlock_time.html#timestamp>
const fn check_timestamp_time_lock(
    unlock_timestamp: u64,
    current_time_lock_timestamp: u64,
    hf: HardFork,
) -> bool {
    current_time_lock_timestamp + hf.block_time().as_secs() >= unlock_timestamp
}

/// Checks all the time locks are unlocked.
///
/// `current_time_lock_timestamp` must be: <https://monero-book.cuprate.org/consensus_rules/transactions/unlock_time.html#getting-the-current-time>
///
/// <https://monero-book.cuprate.org/consensus_rules/transactions/unlock_time.html>
/// <https://monero-book.cuprate.org/consensus_rules/transactions/inputs.html#the-output-must-not-be-locked>
fn check_all_time_locks(
    time_locks: &[Timelock],
    current_chain_height: usize,
    current_time_lock_timestamp: u64,
    hf: HardFork,
) -> Result<(), TransactionError> {
    time_locks.iter().try_for_each(|time_lock| {
        if output_unlocked(
            time_lock,
            current_chain_height,
            current_time_lock_timestamp,
            hf,
        ) {
            Ok(())
        } else {
            tracing::debug!("Transaction invalid: one or more inputs locked, lock: {time_lock:?}.");
            Err(TransactionError::OneOrMoreRingMembersLocked)
        }
    })
}

//----------------------------------------------------------------------------------------------------------- INPUTS

/// Checks the decoys are allowed.
///
/// ref: <https://monero-book.cuprate.org/consensus_rules/transactions/inputs.html#minimum-decoys>
/// &&   <https://monero-book.cuprate.org/consensus_rules/transactions/inputs.html#equal-number-of-decoys>
pub fn check_decoy_info(decoy_info: &DecoyInfo, hf: HardFork) -> Result<(), TransactionError> {
    if hf == HardFork::V15 {
        // Hard-fork 15 allows both v14 and v16 rules
        return check_decoy_info(decoy_info, HardFork::V14)
            .or_else(|_| check_decoy_info(decoy_info, HardFork::V16));
    }

    let current_minimum_decoys = minimum_decoys(hf);

    if decoy_info.min_decoys < current_minimum_decoys {
        // Only allow rings without enough decoys if there aren't enough decoys to mix with.
        if decoy_info.not_mixable == 0 {
            return Err(TransactionError::InputDoesNotHaveExpectedNumbDecoys);
        }
        // Only allow upto 1 mixable input with unmixable inputs.
        if decoy_info.mixable > 1 {
            return Err(TransactionError::MoreThanOneMixableInputWithUnmixable);
        }
    } else if hf >= HardFork::V8 && decoy_info.min_decoys != current_minimum_decoys {
        // From V8 enforce the minimum used number of rings is the default minimum.
        return Err(TransactionError::InputDoesNotHaveExpectedNumbDecoys);
    }

    // From v12 all inputs must have the same number of decoys.
    if hf >= HardFork::V12 && decoy_info.min_decoys != decoy_info.max_decoys {
        return Err(TransactionError::InputDoesNotHaveExpectedNumbDecoys);
    }

    Ok(())
}

/// Checks the inputs key images for torsion.
///
/// ref: <https://monero-book.cuprate.org/consensus_rules/transactions/inputs.html#torsion-free-key-image>
fn check_key_images(input: &Input) -> Result<(), TransactionError> {
    match input {
        Input::ToKey { key_image, .. } => {
            // this happens in monero-serai but we may as well duplicate the check.
            if !decompress_point(*key_image)
                .as_ref()
                .is_some_and(EdwardsPoint::is_torsion_free)
            {
                return Err(TransactionError::KeyImageIsNotInPrimeSubGroup);
            }
        }
        Input::Gen(_) => return Err(TransactionError::IncorrectInputType),
    }

    Ok(())
}

/// Checks that the input is of type [`Input::ToKey`] aka `txin_to_key`.
///
/// ref: <https://monero-book.cuprate.org/consensus_rules/transactions/inputs.html#input-type>
const fn check_input_type(input: &Input) -> Result<(), TransactionError> {
    match input {
        Input::ToKey { .. } => Ok(()),
        Input::Gen(_) => Err(TransactionError::IncorrectInputType),
    }
}

/// Checks that the input has decoys.
///
/// ref: <https://monero-book.cuprate.org/consensus_rules/transactions/inputs.html#no-empty-decoys>
const fn check_input_has_decoys(input: &Input) -> Result<(), TransactionError> {
    match input {
        Input::ToKey { key_offsets, .. } => {
            if key_offsets.is_empty() {
                Err(TransactionError::InputDoesNotHaveExpectedNumbDecoys)
            } else {
                Ok(())
            }
        }
        Input::Gen(_) => Err(TransactionError::IncorrectInputType),
    }
}

/// Checks that the ring members for the input are unique after hard-fork 6.
///
/// ref: <https://monero-book.cuprate.org/consensus_rules/transactions/inputs.html#unique-ring-members>
fn check_ring_members_unique(input: &Input, hf: HardFork) -> Result<(), TransactionError> {
    if hf >= HardFork::V6 {
        match input {
            Input::ToKey { key_offsets, .. } => key_offsets.iter().skip(1).try_for_each(|offset| {
                if *offset == 0 {
                    Err(TransactionError::DuplicateRingMember)
                } else {
                    Ok(())
                }
            }),
            Input::Gen(_) => Err(TransactionError::IncorrectInputType),
        }
    } else {
        Ok(())
    }
}

/// Checks that from hf 7 the inputs are sorted by key image.
///
/// ref: <https://monero-book.cuprate.org/consensus_rules/transactions/inputs.html#sorted-inputs>
fn check_inputs_sorted(inputs: &[Input], hf: HardFork) -> Result<(), TransactionError> {
    let get_ki = |inp: &Input| match inp {
        Input::ToKey { key_image, .. } => Ok(key_image.to_bytes()),
        Input::Gen(_) => Err(TransactionError::IncorrectInputType),
    };

    if hf >= HardFork::V7 {
        for inps in inputs.windows(2) {
            match get_ki(&inps[0])?.cmp(&get_ki(&inps[1])?) {
                Ordering::Greater => (),
                _ => return Err(TransactionError::InputsAreNotOrdered),
            }
        }
    }

    Ok(())
}

/// Checks the youngest output is at least 10 blocks old.
///
/// ref: <https://monero-book.cuprate.org/consensus_rules/transactions/inputs.html#10-block-lock>
fn check_10_block_lock(
    youngest_used_out_height: usize,
    current_chain_height: usize,
    hf: HardFork,
) -> Result<(), TransactionError> {
    if hf >= HardFork::V12 {
        if youngest_used_out_height + 10 > current_chain_height {
            tracing::debug!(
                "Transaction invalid: One or more ring members younger than 10 blocks."
            );
            Err(TransactionError::OneOrMoreRingMembersLocked)
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

/// Sums the inputs checking for overflow.
///
/// ref: <https://monero-book.cuprate.org/consensus_rules/transactions/inputs.html#inputs-must-not-overflow>
fn sum_inputs_check_overflow(inputs: &[Input]) -> Result<u64, TransactionError> {
    let mut sum: u64 = 0;
    for inp in inputs {
        match inp {
            Input::ToKey { amount, .. } => {
                sum = sum
                    .checked_add(amount.unwrap_or(0))
                    .ok_or(TransactionError::InputsOverflow)?;
            }
            Input::Gen(_) => return Err(TransactionError::IncorrectInputType),
        }
    }

    Ok(sum)
}

/// Checks the inputs semantically validity, returning the sum of the inputs.
///
/// Semantic rules are rules that don't require blockchain context, the hard-fork does not require blockchain context as:
/// - The tx-pool will use the current hard-fork
/// - When syncing the hard-fork is in the block header.
fn check_inputs_semantics(inputs: &[Input], hf: HardFork) -> Result<u64, TransactionError> {
    // <https://monero-book.cuprate.org/consensus_rules/transactions/inputs.html#no-empty-inputs>
    if inputs.is_empty() {
        return Err(TransactionError::NoInputs);
    }

    for input in inputs {
        check_input_type(input)?;
        check_input_has_decoys(input)?;

        check_ring_members_unique(input, hf)?;
    }

    check_inputs_sorted(inputs, hf)?;

    sum_inputs_check_overflow(inputs)
}

/// Checks the inputs contextual validity.
///
/// Contextual rules are rules that require blockchain context to check.
///
/// This function does not check signatures or for duplicate key-images.
fn check_inputs_contextual(
    inputs: &[Input],
    tx_ring_members_info: &TxRingMembersInfo,
    current_chain_height: usize,
    hf: HardFork,
) -> Result<(), TransactionError> {
    // This rule is not contained in monero-core explicitly, but it is enforced by how Monero picks ring members.
    // When picking ring members monerod will only look in the DB at past blocks so an output has to be younger
    // than this transaction to be used in this tx.
    if tx_ring_members_info.youngest_used_out_height >= current_chain_height {
        tracing::debug!("Transaction invalid: One or more ring members too young.");
        return Err(TransactionError::OneOrMoreRingMembersLocked);
    }

    check_10_block_lock(
        tx_ring_members_info.youngest_used_out_height,
        current_chain_height,
        hf,
    )?;

    if let Some(decoys_info) = &tx_ring_members_info.decoy_info {
        check_decoy_info(decoys_info, hf)?;
    } else {
        assert_eq!(hf, HardFork::V1);
    }

    for input in inputs {
        check_key_images(input)?;
    }

    Ok(())
}

//----------------------------------------------------------------------------------------------------------- OVERALL

/// Checks the version is in the allowed range.
///
/// <https://monero-book.cuprate.org/consensus_rules/transactions.html#version>
fn check_tx_version(
    decoy_info: &Option<DecoyInfo>,
    version: TxVersion,
    hf: HardFork,
) -> Result<(), TransactionError> {
    if let Some(decoy_info) = decoy_info {
        let max = max_tx_version(hf);
        if version > max {
            return Err(TransactionError::TransactionVersionInvalid);
        }

        let min = min_tx_version(hf);
        if version < min && decoy_info.not_mixable == 0 {
            return Err(TransactionError::TransactionVersionInvalid);
        }
    } else {
        // This will only happen for hard-fork 1 when only RingSignatures are allowed.
        if version != TxVersion::RingSignatures {
            return Err(TransactionError::TransactionVersionInvalid);
        }
    }

    Ok(())
}

/// Returns the default maximum tx version for the given hard-fork.
fn max_tx_version(hf: HardFork) -> TxVersion {
    if hf <= HardFork::V3 {
        TxVersion::RingSignatures
    } else {
        TxVersion::RingCT
    }
}

/// Returns the default minimum tx version for the given hard-fork.
fn min_tx_version(hf: HardFork) -> TxVersion {
    if hf >= HardFork::V6 {
        TxVersion::RingCT
    } else {
        TxVersion::RingSignatures
    }
}

fn transaction_weight_limit(hf: HardFork) -> usize {
    penalty_free_zone(hf) / 2 - 600
}

/// Checks the transaction is semantically valid.
///
/// Semantic rules are rules that don't require blockchain context, the hard-fork does not require blockchain context as:
/// - The tx-pool will use the current hard-fork
/// - When syncing the hard-fork is in the block header.
///
/// To fully verify a transaction this must be accompanied by [`check_transaction_contextual`]
///
pub fn check_transaction_semantic(
    tx: &Transaction,
    tx_blob_size: usize,
    tx_weight: usize,
    tx_hash: &[u8; 32],
    hf: HardFork,
    verifier: impl BatchVerifier,
) -> Result<u64, TransactionError> {
    // <https://monero-book.cuprate.org/consensus_rules/transactions.html#transaction-size>
    if tx_blob_size > MAX_TX_BLOB_SIZE
        || (hf >= HardFork::V8 && tx_weight > transaction_weight_limit(hf))
    {
        return Err(TransactionError::TooBig);
    }

    let tx_version =
        TxVersion::from_raw(tx.version()).ok_or(TransactionError::TransactionVersionInvalid)?;

    let bp_or_bpp = match tx {
        Transaction::V2 {
            proofs: Some(proofs),
            ..
        } => match proofs.rct_type() {
            RctType::AggregateMlsagBorromean | RctType::MlsagBorromean => false,
            RctType::MlsagBulletproofs
            | RctType::MlsagBulletproofsCompactAmount
            | RctType::ClsagBulletproof
            | RctType::ClsagBulletproofPlus => true,
        },
        Transaction::V2 { proofs: None, .. } | Transaction::V1 { .. } => false,
    };

    let outputs_sum = check_outputs_semantics(&tx.prefix().outputs, hf, tx_version, bp_or_bpp)?;
    let inputs_sum = check_inputs_semantics(&tx.prefix().inputs, hf)?;

    let fee = match tx {
        Transaction::V1 { .. } => {
            if outputs_sum >= inputs_sum {
                return Err(TransactionError::OutputsTooHigh);
            }
            inputs_sum - outputs_sum
        }
        Transaction::V2 { proofs, .. } => {
            let proofs = proofs
                .as_ref()
                .ok_or(TransactionError::TransactionVersionInvalid)?;

            ring_ct::ring_ct_semantic_checks(proofs, tx_hash, verifier, hf)?;

            proofs.base.fee
        }
    };

    Ok(fee)
}

/// Checks the transaction is contextually valid.
///
/// To fully verify a transaction this must be accompanied by [`check_transaction_semantic`].
///
/// This function also does _not_ check for duplicate key-images: <https://monero-book.cuprate.org/consensus_rules/transactions/inputs.html#unique-key-image>.
///
/// `current_time_lock_timestamp` must be: <https://monero-book.cuprate.org/consensus_rules/transactions/unlock_time.html#getting-the-current-time>.
pub fn check_transaction_contextual(
    tx: &Transaction,
    tx_ring_members_info: &TxRingMembersInfo,
    current_chain_height: usize,
    current_time_lock_timestamp: u64,
    hf: HardFork,
) -> Result<(), TransactionError> {
    let tx_version =
        TxVersion::from_raw(tx.version()).ok_or(TransactionError::TransactionVersionInvalid)?;

    check_inputs_contextual(
        &tx.prefix().inputs,
        tx_ring_members_info,
        current_chain_height,
        hf,
    )?;
    check_tx_version(&tx_ring_members_info.decoy_info, tx_version, hf)?;

    check_all_time_locks(
        &tx_ring_members_info.time_locked_outs,
        current_chain_height,
        current_time_lock_timestamp,
        hf,
    )?;

    match &tx {
        Transaction::V1 { prefix, signatures } => ring_signatures::check_input_signatures(
            &prefix.inputs,
            signatures,
            &tx_ring_members_info.rings,
            // This will only return None on v2 miner txs.
            &tx.signature_hash()
                .ok_or(TransactionError::TransactionVersionInvalid)?,
        ),
        Transaction::V2 { prefix, proofs } => Ok(ring_ct::check_input_signatures(
            &tx.signature_hash()
                .ok_or(TransactionError::TransactionVersionInvalid)?,
            &prefix.inputs,
            proofs
                .as_ref()
                .ok_or(TransactionError::TransactionVersionInvalid)?,
            &tx_ring_members_info.rings,
        )?),
    }
}
