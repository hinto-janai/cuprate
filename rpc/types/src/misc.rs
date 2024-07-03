//! TODO

//---------------------------------------------------------------------------------------------------- Lints
#![allow(
    missing_docs, // Docs are at: <https://www.getmonero.org/resources/developer-guides/daemon-rpc.html>
    clippy::struct_excessive_bools, // hey man, tell that to the people who wrote `monerod`
)]

//---------------------------------------------------------------------------------------------------- Import
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "epee")]
use cuprate_epee_encoding::epee_object;

use crate::macros::monero_definition_link;

//---------------------------------------------------------------------------------------------------- Macros
/// TODO
macro_rules! define_struct_and_impl_epee {
    (
        $( #[$struct_attr:meta] )*
        $struct_name:ident {
            // And any fields.
            $(
                $( #[$field_attr:meta] )*
                $field_name:ident: $field_type:ty,
            )*
        }
    ) => {
        $( #[$struct_attr] )*
        pub struct $struct_name {
            $(
                $( #[$field_attr] )*
                pub $field_name: $field_type,
            )*
        }


        #[cfg(feature = "epee")]
        epee_object! {
            $struct_name,
            $(
                $field_name: $field_type,
            )*
        }
    };
}

//---------------------------------------------------------------------------------------------------- BlockHeader
define_struct_and_impl_epee! {
    #[doc = monero_definition_link!(
        cc73fe71162d564ffda8e549b79a350bca53c454,
        "rpc/core_rpc_server_commands_defs.h",
        1163..=1212
    )]
    /// TODO.
    #[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    BlockHeader {
        block_size: u64,
        block_weight: u64,
        cumulative_difficulty_top64: u64,
        cumulative_difficulty: u64,
        depth: u64,
        difficulty_top64: u64,
        difficulty: u64,
        hash: String,
        height: u64,
        long_term_weight: u64,
        major_version: u8,
        miner_tx_hash: String,
        minor_version: u8,
        nonce: u32,
        num_txes: u64,
        orphan_status: bool,
        pow_hash: String,
        prev_hash: String,
        reward: u64,
        timestamp: u64,
        wide_cumulative_difficulty: String,
        wide_difficulty: String,
    }
}

//---------------------------------------------------------------------------------------------------- ConnectionInfo
define_struct_and_impl_epee! {
    #[doc = monero_definition_link!(
        cc73fe71162d564ffda8e549b79a350bca53c454,
        "cryptonote_protocol/cryptonote_protocol_defs.h",
        47..=116
    )]
    /// TODO.
    #[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
    ConnectionInfo {
        address: String,
        address_type: u8,
        avg_download: u64,
        avg_upload: u64,
        connection_id: String,
        current_download: u64,
        current_upload: u64,
        height: u64,
        host: String,
        incoming: bool,
        ip: String,
        live_time: u64,
        localhost: bool,
        local_ip: bool,
        peer_id: String,
        port: String,
        pruning_seed: u32,
        recv_count: u64,
        recv_idle_time: u64,
        rpc_credits_per_hash: u32,
        rpc_port: u16,
        send_count: u64,
        send_idle_time: u64,
        ssl: bool,
        state: String,
        support_flags: u32,
    }
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod test {
    // use super::*;
}