use futures::FutureExt;

use crate::{
    sealer::SizedSealer,
    tests::common::{transform, Tx},
    Sealer,
};
use std::error::Error;

const SEAL_SIZE: usize = 6;

/// Test FIFO ordering and correct sealing
#[tokio::test]
async fn test_fifo() -> Result<(), Box<dyn Error>> {
    let mut sealer = SizedSealer::<Tx>::new(SEAL_SIZE);
    let test_txs = transform(vec![true, false, true, false, true, false]);
    for test_tx in &test_txs {
        sealer.update(*test_tx, 1);
    }
    let txs = sealer.await;

    // Check that the FIFO ordering is respected
    assert_eq!(test_txs, txs);
    Ok(())
}

/// Check whether the sealer returns when less transactions are present
#[tokio::test]
async fn test_correct_size() -> Result<(), Box<dyn Error>> {
    let mut sealer = SizedSealer::<Tx>::new(SEAL_SIZE);
    let test_txs = transform(vec![true, false, true, false, true, false]);
    let res = (&mut sealer).now_or_never();

    assert!(res.is_none(), "Sealer should not be ready when empty");

    for test_tx in &test_txs {
        sealer.update(*test_tx, 1);
    }
    let res = (&mut sealer).now_or_never();
    assert!(res.is_some(), "Sealer should have been ready");
    let txs = res.unwrap();

    // Check that the FIFO ordering is respected
    assert_eq!(test_txs, txs);
    Ok(())
}

/// Check whether after sealing the transactions are cleared correctly
#[tokio::test]
async fn test_multiple_seals() -> Result<(), Box<dyn Error>> {
    let mut sealer = SizedSealer::<Tx>::new(SEAL_SIZE);
    let test_txs = transform(vec![true, false, true, false, true, false]);
    let test_txs2 = transform(vec![true, true, true, false, true, false]);
    let test_txs3 = transform(vec![true, false, false, false, true, false]);
    for test_tx in &test_txs {
        sealer.update(*test_tx, 1);
    }
    let txs = (&mut sealer).await;
    assert_eq!(test_txs, txs);

    for test_tx in &test_txs2 {
        sealer.update(*test_tx, 1);
    }
    let txs = (&mut sealer).await;
    assert_eq!(test_txs2, txs);

    for test_tx in &test_txs3 {
        sealer.update(*test_tx, 1);
    }
    let txs = (&mut sealer).await;
    assert_eq!(test_txs3, txs);

    Ok(())
}
