use futures::Future;
use network::Message;
// use network::Message;
// use serde::{de::DeserializeOwned, Serialize};

pub trait Transaction: Message {}

pub trait Round:
    Send
    + Sync
    + 'static
    + Ord
    + Default
    + core::fmt::Debug
    + std::ops::Sub<Output = Self>
    + Copy
    + core::fmt::Display
{
    const MIN: Self;
}

pub trait Sealer<Tx>: Send + Sync + 'static + Future<Output = Vec<Tx>> + Unpin {
    /// Cleans the sealer and returns all transactions
    fn seal(&mut self) -> Vec<Tx>;

    /// Updates the sealer with a new transaction
    fn update(
        &mut self,
        _tx: Tx,
        _tx_size: usize,
    ) {
    }
}
