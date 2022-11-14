use crate::{
    tests::common::transform, sealer::HybridSealer, Sealer,
};
use std::{
    error::Error,
    time::Duration,
};
use tokio::time::Instant;

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
