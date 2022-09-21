use std::{
    error::Error,
    time::{Duration, Instant},
};

use crate::{
    sealer::Timed,
    tests::common::{transform, Tx},
    Sealer,
};

const SEAL_TIME: Duration = Duration::from_millis(50);

/// Test FIFO ordering and correct sealing
#[tokio::test]
async fn test_fifo() -> Result<(), Box<dyn Error>> {
    let mut sealer = Timed::<Tx>::new(SEAL_TIME);
    let test_txs: Vec<Tx> = transform(vec![true, false, true, false, true, false]);
    for test_tx in &test_txs {
        sealer.update(*test_tx, 1);
    }
    sealer.reset_timer();
    let txs = sealer.await;

    // Check that the FIFO ordering is respected
    assert_eq!(test_txs, txs);
    Ok(())
}

/// Check whether the sealer wait for approximately the right amount of time
#[tokio::test]
async fn test_correct_timing() -> Result<(), Box<dyn Error>> {
    let mut sealer = Timed::<Tx>::new(SEAL_TIME);
    let test_txs: Vec<Tx> = transform(vec![true, false, true, false, true, false]);
    for test_tx in &test_txs {
        sealer.update(*test_tx, 1);
    }
    let start = Instant::now();
    sealer.reset_timer();
    sealer.await;
    let end = Instant::now();
    // Check that we are indeed waiting for the right amount of time
    assert!(end - start > SEAL_TIME);
    Ok(())
}

/// Check whether after sealing the transactions are cleared correctly
#[tokio::test]
async fn test_multiple_seals() -> Result<(), Box<dyn Error>> {
    let mut sealer = Timed::<Tx>::new(SEAL_TIME);
    let test_txs = transform(vec![true, false, true, false, true, false]);
    let test_txs2 = transform(vec![true, true, true, false, true, false]);
    let test_txs3 = transform(vec![true, false, false, false, true, false]);
    for test_tx in &test_txs {
        sealer.update(*test_tx, 1);
    }
    sealer.reset_timer();
    let txs = (&mut sealer).await;
    assert_eq!(test_txs, txs);

    for test_tx in &test_txs2 {
        sealer.update(*test_tx, 1);
    }
    sealer.reset_timer();
    let txs = (&mut sealer).await;
    assert_eq!(test_txs2, txs);

    for test_tx in &test_txs3 {
        sealer.update(*test_tx, 1);
    }
    sealer.reset_timer();
    let txs = (&mut sealer).await;
    assert_eq!(test_txs3, txs);
    Ok(())
}
