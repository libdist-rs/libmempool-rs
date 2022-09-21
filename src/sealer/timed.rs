use crate::{Sealer, Transaction};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::time::{sleep, Instant, Sleep};

pub struct Timed<Tx> {
    timeout: Duration,
    timer: Pin<Box<Sleep>>,
    txs: Vec<Tx>,
}

impl<Tx> Timed<Tx> {
    pub fn new(timeout: Duration) -> Self {
        Self {
            timeout,
            txs: Vec::new(),
            timer: Box::pin(sleep(timeout)),
        }
    }

    pub fn reset_timer(&mut self) {
        self.timer.as_mut().reset(Instant::now() + self.timeout);
    }
}

impl<Tx> Sealer<Tx> for Timed<Tx>
where
    Tx: Transaction,
{
    /// Resets the timer and returns all the transactions
    fn seal(&mut self) -> Vec<Tx> {
        self.reset_timer();
        std::mem::take(&mut self.txs)
    }

    fn update(
        &mut self,
        tx: Tx,
        _tx_size: usize,
    ) {
        self.txs.push(tx);
    }
}

impl<Tx> Unpin for Timed<Tx> {}

impl<Tx> Future for Timed<Tx>
where
    Tx: Transaction,
{
    type Output = Vec<Tx>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        if let Poll::Ready(()) = self.timer.as_mut().poll(cx) {
            return Poll::Ready(self.seal());
        }
        Poll::Pending
    }
}
