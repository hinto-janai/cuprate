//! Functions for TODO: doc enum message.

use std::convert::Infallible;

use anyhow::{anyhow, Error};
use tower::ServiceExt;

use cuprate_helper::cast::usize_to_u64;
use cuprate_p2p_core::{
    services::{AddressBookRequest, AddressBookResponse},
    AddressBook, NetworkZone,
};

// FIXME: use `anyhow::Error` over `tower::BoxError` in address book.

/// [`AddressBookRequest::PeerlistSize`]
pub(crate) async fn peerlist_size<Z: NetworkZone>(
    address_book: &mut impl AddressBook<Z>,
) -> Result<(u64, u64), Error> {
    let AddressBookResponse::PeerlistSize { white, grey } = address_book
        .ready()
        .await
        .map_err(|e| anyhow!(e))?
        .call(AddressBookRequest::PeerlistSize)
        .await
        .map_err(|e| anyhow!(e))?
    else {
        unreachable!();
    };

    Ok((usize_to_u64(white), usize_to_u64(grey)))
}

/// [`AddressBookRequest::ConnectionCount`]
pub(crate) async fn connection_count<Z: NetworkZone>(
    address_book: &mut impl AddressBook<Z>,
) -> Result<(u64, u64), Error> {
    let AddressBookResponse::ConnectionCount { incoming, outgoing } = address_book
        .ready()
        .await
        .map_err(|e| anyhow!(e))?
        .call(AddressBookRequest::ConnectionCount)
        .await
        .map_err(|e| anyhow!(e))?
    else {
        unreachable!();
    };

    Ok((usize_to_u64(incoming), usize_to_u64(outgoing)))
}

/// [`AddressBookRequest::SetBan`]
pub(crate) async fn set_ban<Z: NetworkZone>(
    address_book: &mut impl AddressBook<Z>,
    peer: cuprate_p2p_core::ban::SetBan<Z::Addr>,
) -> Result<(), Error> {
    let AddressBookResponse::Ok = address_book
        .ready()
        .await
        .map_err(|e| anyhow!(e))?
        .call(AddressBookRequest::SetBan(peer))
        .await
        .map_err(|e| anyhow!(e))?
    else {
        unreachable!();
    };

    Ok(())
}

/// [`AddressBookRequest::GetBan`]
pub(crate) async fn get_ban<Z: NetworkZone>(
    address_book: &mut impl AddressBook<Z>,
    peer: Z::Addr,
) -> Result<Option<std::time::Instant>, Error> {
    let AddressBookResponse::GetBan { unban_instant } = address_book
        .ready()
        .await
        .map_err(|e| anyhow!(e))?
        .call(AddressBookRequest::GetBan(peer))
        .await
        .map_err(|e| anyhow!(e))?
    else {
        unreachable!();
    };

    Ok(unban_instant)
}

/// [`AddressBookRequest::GetBans`]
pub(crate) async fn get_bans<Z: NetworkZone>(
    address_book: &mut impl AddressBook<Z>,
) -> Result<(), Error> {
    let AddressBookResponse::GetBans(bans) = address_book
        .ready()
        .await
        .map_err(|e| anyhow!(e))?
        .call(AddressBookRequest::GetBans)
        .await
        .map_err(|e| anyhow!(e))?
    else {
        unreachable!();
    };

    Ok(todo!())
}
