use std::net::SocketAddr;
use fnv::FnvHashMap;
use libstorage::Storage;
use network::{plaintcp::TcpSimpleSender, Acknowledgement, NetSender};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
use libcrypto::hash::Hash;

use crate::{Mempool, Config, MempoolMsg, Batcher, sealer::SizedSealer};

use super::{get_peers, Id, Round, Tx};

const NUM_NODES: usize = 16;
const MEMPOOL_BASE_PORT: u16 = 10_000;
const CLIENT_BASE_PORT: u16 = 9_000;

#[tokio::test]
async fn test_mempool() -> anyhow::Result<()>
{
    let peers = get_peers(NUM_NODES, 10_000);
    let all_ids: Vec<Id> = peers.keys().cloned().collect();
    let mut params = Config::<Round>::default();
    params.gc_depth = 2.into();

    let mut receivers = Vec::<UnboundedReceiver<Hash>>::new();

    for i in 0..NUM_NODES {
        let my_name: Id = i.into();
        let store = Storage::new(format!(".mempool_tests-{}.db", i).as_str())?;
        let mempool_sender = TcpSimpleSender::<Id, MempoolMsg<Id, Tx>, Acknowledgement>::with_peers(peers.clone());

        let (_tx_consensus, rx_consensus) = unbounded_channel();
        let (tx_batcher, rx_batcher) = unbounded_channel();
        let (tx_processor, rx_processor) = unbounded_channel();
        let (tx_in_consensus, rx_in_consensus) = unbounded_channel();

        Batcher::spawn(
            rx_batcher, 
            tx_processor.clone(), 
            SizedSealer::new(2),
        );
    
        Mempool::spawn(
            my_name.clone(), 
            all_ids.clone(), 
            params.clone(), 
            store.clone(), 
            mempool_sender, 
            rx_consensus, 
            tx_batcher, 
            tx_processor, 
            rx_processor, 
            tx_in_consensus, 
            format!("0.0.0.0:{}", MEMPOOL_BASE_PORT + i as u16).parse()?, 
            format!("0.0.0.0:{}", CLIENT_BASE_PORT + i as u16).parse()?, 
        );
        
        receivers.push(rx_in_consensus);
    }

    let mut client_peers = FnvHashMap::<Id, SocketAddr>::default();
    for i in 0..NUM_NODES {
        client_peers.insert(i.into(), format!("127.0.0.1:{}", CLIENT_BASE_PORT + i as u16).parse()?);
    }

    let mut client_sender = TcpSimpleSender::<Id, Tx, Acknowledgement>::with_peers(client_peers);
    let test_tx = Tx::dummy();
    for i in 0..NUM_NODES {
        client_sender.send(i.into(), test_tx).await;
        client_sender.send(i.into(), test_tx).await;
        client_sender.send(i.into(), test_tx).await;
        client_sender.send(i.into(), test_tx).await;
    }

    for mut receiver in receivers {
        let processed_hash = receiver.recv().await;
        assert!(processed_hash.is_some(), "Got empty processed_hash");
        let processed_hash = processed_hash.unwrap();
        // Todo: Check with expected processed_hash
    }
    Ok(())
}