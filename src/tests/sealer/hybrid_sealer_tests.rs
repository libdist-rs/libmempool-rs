use std::{collections::HashMap, time::Duration, pin::Pin, task::{Context, Poll}, error::Error};

use futures::{Future, FutureExt};
use tokio::time::Instant;

use crate::{sealer::{TimedSealer, SizedSealer}, Sealer};

struct HybridSealer<Tx>
{
    timed_sealer: TimedSealer<usize>,
    sized_sealer: SizedSealer<usize>,
    map: HashMap<usize, Tx>,
    counter: usize,
}

impl<Tx> HybridSealer<Tx> {
    fn new(timeout: Duration, tx_size: usize) -> Self 
    { 
        Self { 
            timed_sealer: TimedSealer::new(timeout),
            sized_sealer: SizedSealer::new(tx_size),
            map: HashMap::new(),
            counter: 0,
        }
    }

    fn update(&mut self, tx: Tx, tx_size: usize)
    {
        self.map.insert(self.counter, tx);
        self.sized_sealer.update(self.counter, tx_size);
        self.timed_sealer.update(self.counter);
        self.counter += 1;
    }

    fn reset(&mut self)
    {
        self.timed_sealer.seal();
        self.sized_sealer.seal();
        self.map.clear();
        self.counter = 0;
    }
}

impl<Tx> Sealer<Tx> for HybridSealer<Tx>
{
    fn seal(&mut self) -> Vec<Tx> {
        panic!("Do not call .seal() for Hybrid Sealer\nUse .reset() and .await");
    }
}

impl<Tx> Unpin for HybridSealer<Tx> {}

impl<Tx> Future for HybridSealer<Tx>
{
    type Output = Vec<Tx>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
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
async fn test_hybrid_fifo() -> Result<(), Box<dyn Error>>
{
    let mut sealer = HybridSealer::<bool>::new(SEAL_TIME, SEAL_SIZE);
    let start = Instant::now();
    sealer.reset();
    let test_txs = vec![true, false, true, false, true];
    // First, check for the timeout case
    for test_tx in &test_txs {
        sealer.update(*test_tx, 1);
    }
    let txs = (&mut sealer).await;
    let end = Instant::now();
    assert!(txs == test_txs, "The output and the input do not match");
    assert!(end - start > SEAL_TIME, "Did not seal by timeout");

    // Next, check for the size case
    sealer.reset();
    let test_txs = vec![true, false, true, false, true, false];
    for test_tx in &test_txs {
        sealer.update(*test_tx, 1);
    }
    let txs = (&mut sealer).await;
    assert!(txs == test_txs, "The output and the input do not match");
    assert!(txs.len() == SEAL_SIZE);

    Ok(())
}