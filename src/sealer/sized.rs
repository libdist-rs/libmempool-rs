use crate::{Sealer, Transaction};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

/// The SizedSealer notifies when the size of the transactions in the mempool
/// exceeds a threshold size
pub struct Sized<Tx> {
    max_size: usize,
    current_size: usize,
    txs: Vec<Tx>,
}

impl<Tx> Sized<Tx> {
    pub fn new(max_size: usize) -> Self {
        Self {
            max_size,
            current_size: 0,
            txs: Vec::new(),
        }
    }
}

impl<Tx> Sealer<Tx> for Sized<Tx>
where
    Tx: Transaction,
{
    /// Resets the sealer with current size as 0 and returns all the
    /// transactions
    fn seal(&mut self) -> Vec<Tx> {
        self.current_size = 0;
        std::mem::take(&mut self.txs)
    }

    fn update(
        &mut self,
        tx: Tx,
        tx_size: usize,
    ) {
        self.current_size += tx_size;
        self.txs.push(tx);
    }
}

impl<Tx> Unpin for Sized<Tx> {}

impl<Tx> Future for Sized<Tx>
where
    Tx: Transaction,
{
    type Output = Vec<Tx>;

    fn poll(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        if self.current_size >= self.max_size {
            return Poll::Ready(self.seal());
        }
        Poll::Pending
    }
}
