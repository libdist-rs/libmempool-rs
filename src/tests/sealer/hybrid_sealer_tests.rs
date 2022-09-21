use crate::{
    sealer::{Sized, Timed},
    tests::common::transform,
    Sealer, Transaction,
};
use futures::{Future, FutureExt};
use network::Message;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    error::Error,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::time::Instant;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Hash, Eq, Copy)]
pub struct Counter(usize);

impl Message for Counter {}
impl Transaction for Counter {}

struct HybridSealer<Tx> {
    timed_sealer: Timed<Counter>,
    sized_sealer: Sized<Counter>,
    map: HashMap<Counter, Tx>,
    counter: Counter,
}

impl<Tx> HybridSealer<Tx>
where
    Tx: Transaction,
{
    fn new(
        timeout: Duration,
        tx_size: usize,
    ) -> Self {
        Self {
            timed_sealer: Timed::new(timeout),
            sized_sealer: Sized::new(tx_size),
            map: HashMap::new(),
            counter: Counter { 0: 0 },
        }
    }

    fn reset(&mut self) {
        self.timed_sealer.seal();
        self.sized_sealer.seal();
        self.map.clear();
        self.counter.0 = 0;
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
        self.counter.0 += 1;
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

const SEAL_TIME: Duration = Duration::from_millis(50);
const SEAL_SIZE: usize = 6;

#[tokio::test]
async fn test_hybrid_fifo() -> Result<(), Box<dyn Error>> {
    let mut sealer = HybridSealer::<crate::tests::common::Tx>::new(SEAL_TIME, SEAL_SIZE);
    let start = Instant::now();
    sealer.reset();
    let test_txs = transform(vec![true, false, true, false, true]);
    // First, check for the timeout case
    for test_tx in &test_txs {
        sealer.update(test_tx.clone(), 1);
    }
    let txs = (&mut sealer).await;
    let end = Instant::now();
    assert_eq!(txs, test_txs, "The output and the input do not match");
    assert!(end - start > SEAL_TIME, "Did not seal by timeout");

    // Next, check for the size case
    sealer.reset();
    let test_txs = transform(vec![true, false, true, false, true, false]);
    for test_tx in &test_txs {
        sealer.update(test_tx.clone(), 1);
    }
    let txs = (&mut sealer).await;
    assert_eq!(txs, test_txs, "The output and the input do not match");
    assert_eq!(txs.len(), SEAL_SIZE);

    Ok(())
}
