use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use super::{Sized, Timed};
use crate::{Sealer, Transaction};
use fnv::FnvHashMap as HashMap;
use futures::{Future, FutureExt};

pub type Counter = usize;

pub struct HybridSealer<Tx> {
    timed_sealer: Timed<Counter>,
    sized_sealer: Sized<Counter>,
    map: HashMap<Counter, Tx>,
    counter: Counter,
}

impl<Tx> HybridSealer<Tx>
where
    Tx: Transaction,
{
    pub fn new(
        timeout: Duration,
        tx_size: usize,
    ) -> Self {
        Self {
            timed_sealer: Timed::new(timeout),
            sized_sealer: Sized::new(tx_size),
            map: HashMap::default(),
            counter: 0,
        }
    }

    pub fn reset(&mut self) {
        self.timed_sealer.seal();
        self.sized_sealer.seal();
        self.map.clear();
        self.counter = 0;
    }
}

impl<Tx> Sealer<Tx> for HybridSealer<Tx>
where
    Tx: Transaction,
{
    fn seal(&mut self) -> Vec<Tx> {
        panic!("Do not call .seal() for Hybrid Sealer\nUse .reset() and .await");
    }

    fn update(
        &mut self,
        tx: Tx,
        tx_size: usize,
    ) {
        self.map.insert(self.counter, tx);
        self.sized_sealer.update(self.counter, tx_size);
        self.timed_sealer.update(self.counter, tx_size);
        self.counter += 1;
    }
}

impl<Tx> Unpin for HybridSealer<Tx> where Tx: Transaction {}

impl<Tx> Future for HybridSealer<Tx>
where
    Tx: Transaction,
{
    type Output = Vec<Tx>;

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Self::Output> {
        if let Poll::Ready(txs) = self.timed_sealer.poll_unpin(cx) {
            let mut out = Vec::new();
            for tx_id in txs {
                let tx = self.map.remove(&tx_id).unwrap();
                out.push(tx);
            }
            // Discard all the transactions in the sealer
            self.reset();
            return Poll::Ready(out);
        }
        if let Poll::Ready(txs) = self.sized_sealer.poll_unpin(cx) {
            let mut out = Vec::new();
            for tx_id in txs {
                let tx = self.map.remove(&tx_id).unwrap();
                out.push(tx);
            }
            // Discard all the transactions in the sealer
            self.reset();
            return Poll::Ready(out);
        }
        Poll::Pending
    }
}
