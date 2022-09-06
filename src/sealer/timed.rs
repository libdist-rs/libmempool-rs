use std::{time::Duration, future::Future, pin::Pin, task::{Context, Poll}};

use tokio::time::{Sleep, sleep, Instant};

use crate::Sealer;

pub struct TimedSealer<Tx>
{
    timeout: Duration,
    timer: Pin<Box<Sleep>>,
    txs: Vec<Tx>,
}

impl<Tx> TimedSealer<Tx> {
    pub fn new(timeout: Duration) -> Self { 
        Self { 
            timeout, 
            txs: Vec::new(), 
            timer: Box::pin(sleep(timeout)) 
        } 
    }

    pub fn update(&mut self, tx: Tx) {
        self.txs.push(tx);
    }

    pub fn reset_timer(&mut self) {
        self.timer
            .as_mut()
            .reset(Instant::now() + self.timeout);
    }
}

impl<Tx> Sealer<Tx> for TimedSealer<Tx>
{
    /// Resets the timer and returns all the transactions
    fn seal(&mut self) -> Vec<Tx> {
        self.reset_timer();
        std::mem::take(&mut self.txs)
    }
}

impl<Tx> Unpin for TimedSealer<Tx> {}

impl<Tx> Future for TimedSealer<Tx> {
    type Output = Vec<Tx>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Poll::Ready(()) = self.timer.as_mut().poll(cx) {
            return Poll::Ready(self.seal());
        }
        Poll::Pending
    }
}