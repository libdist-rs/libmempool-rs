use std::{future::Future, pin::Pin, task::{Context, Poll}};

use crate::Sealer;

/// The SizedSealer notifies when the size of the transactions in the mempool exceeds a threshold size 
pub struct SizedSealer<Tx>
{
    max_size: usize,
    current_size: usize,
    txs: Vec<Tx>,
}

impl<Tx> SizedSealer<Tx> {
    pub fn new(max_size: usize) -> Self 
    { 
        Self { max_size, current_size: 0, txs: Vec::new() } 
    }

    pub fn update(&mut self, tx: Tx, tx_size: usize)
    {
        self.current_size += tx_size;
        self.txs.push(tx);
    }

}

impl<Tx> Sealer<Tx> for SizedSealer<Tx> {
    /// Resets the sealer with current size as 0 and returns all the transactions
    fn seal(&mut self) -> Vec<Tx> {
        self.current_size = 0;
        std::mem::take(&mut self.txs)
    }
}

impl<Tx> Unpin for SizedSealer<Tx> {}

impl<Tx> Future for SizedSealer<Tx>
{
    type Output = Vec<Tx>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> 
    {
        if self.current_size >= self.max_size {
            return Poll::Ready(self.seal());
        }
        Poll::Pending
    }
}