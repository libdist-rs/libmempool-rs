use super::{get_peers, Id, Round, Tx};
use crate::batcher::Batcher;
use crate::Batch;
use crate::{sealer::Sized, Config, Mempool, MempoolMsg};
use libcrypto::hash::Hash;
use libstorage::rocksdb::Storage;
use network::{plaintcp::TcpSimpleSender, Acknowledgement, NetSender};
use std::time::Duration;
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedReceiver},
    time::{self, Instant},
};

const CLIENT_BASE_PORT: u16 = 9_000;
const MEMPOOL_BASE_PORT: u16 = 10_000;

#[tokio::test]
async fn test_mempool() -> anyhow::Result<()> {
    let num_nodes = vec![4, 9, 16, 31, 67, 129];
    let mut ports_so_far: u16 = 0;
    for num_node in num_nodes {
        do_test_mempool(
            num_node,
            CLIENT_BASE_PORT + ports_so_far,
            MEMPOOL_BASE_PORT + ports_so_far,
        )
        .await?;
        time::sleep(Duration::from_millis(1_000)).await;

        ports_so_far += num_node as u16;
    }
    Ok(())
}

async fn do_test_mempool(
    num_nodes: usize,
    client_base_port: u16,
    mempool_base_port: u16,
) -> anyhow::Result<()> {
    let (mempool_peers, client_peers) = (
        get_peers(num_nodes, mempool_base_port),
        get_peers(num_nodes, client_base_port),
    );
    let all_ids: Vec<Id> = mempool_peers.keys().cloned().collect();

    let params = Config::<Round> {
        gc_depth: 2.into(),
        ..Default::default()
    };

    let mut receivers = Vec::<UnboundedReceiver<Hash<Batch<Tx>>>>::new();

    for i in 0..num_nodes {
        let my_name: Id = i;
        let store = Storage::new(format!(".mempool_tests{}-{}.db", num_nodes, i).as_str())?;
        let mempool_sender = TcpSimpleSender::<Id, MempoolMsg<Id, Tx>, Acknowledgement>::with_peers(
            mempool_peers.clone(),
        );

        // tx_consensus is to be used inside consensus for synchronization or garbage
        // collection
        let (_tx_consensus, rx_consensus) = unbounded_channel();
        let (tx_batcher, rx_batcher) = unbounded_channel();
        let (tx_processor, rx_processor) = unbounded_channel();
        let (tx_in_consensus, rx_in_consensus) = unbounded_channel();

        Batcher::spawn(rx_batcher, tx_processor.clone(), Sized::new(2));

        let (my_mempool_addr, my_client_addr) = {
            let mut mempool_addr = mempool_peers[&my_name];
            let mut client_addr = client_peers[&my_name];
            mempool_addr.set_ip("0.0.0.0".parse()?);
            client_addr.set_ip("0.0.0.0".parse()?);
            (mempool_addr, client_addr)
        };

        Mempool::spawn(
            my_name,
            all_ids.clone(),
            params.clone(),
            store.clone(),
            mempool_sender,
            rx_consensus,
            tx_batcher,
            tx_processor,
            rx_processor,
            tx_in_consensus,
            my_mempool_addr,
            my_client_addr,
        );

        receivers.push(rx_in_consensus);
    }

    let mut client_sender = TcpSimpleSender::<Id, Tx, Acknowledgement>::with_peers(client_peers);
    let test_tx = crate::tests::dummy_tx();

    let start = Instant::now();

    for i in 0..num_nodes {
        client_sender.send(i, test_tx).await;
        client_sender.send(i, test_tx).await;
        client_sender.send(i, test_tx).await;
        client_sender.send(i, test_tx).await;
    }

    for mut receiver in receivers {
        let processed_hash = receiver.recv().await;
        assert!(processed_hash.is_some(), "Got empty processed_hash");
        let _ = processed_hash.unwrap();
        // TODO: Check with expected processed_hash
    }
    let bench_time = (Instant::now() - start).as_micros();
    println!("Bench {}: {} ms", num_nodes, bench_time);

    Ok(())
}
