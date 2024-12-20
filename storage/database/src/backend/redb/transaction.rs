//! Implementation of `trait TxRo/TxRw` for `redb`.

//---------------------------------------------------------------------------------------------------- Import
use crate::{
    error::DbResult,
    transaction::{TxRo, TxRw},
};

//---------------------------------------------------------------------------------------------------- TxRo
impl TxRo<'_> for redb::ReadTransaction {
    /// This function is infallible.
    fn commit(self) -> DbResult<()> {
        // `redb`'s read transactions cleanup automatically when all references are dropped.
        //
        // There is `close()`:
        // <https://docs.rs/redb/2.0.0/redb/struct.ReadTransaction.html#method.close>
        // but this will error if there are outstanding references, i.e. an open table.
        // This is unwanted behavior in our case, so we don't call this.
        Ok(())
    }
}

//---------------------------------------------------------------------------------------------------- TxRw
impl TxRw<'_> for redb::WriteTransaction {
    fn commit(self) -> DbResult<()> {
        Ok(self.commit()?)
    }

    fn abort(self) -> DbResult<()> {
        Ok(self.abort()?)
    }
}

//---------------------------------------------------------------------------------------------------- Tests
#[cfg(test)]
mod test {
    // use super::*;
}
