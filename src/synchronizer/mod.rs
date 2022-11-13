use std::time::Duration;
use crate::{ConsensusMempoolMsg, MempoolMsg, Transaction, Batch, BatchHash};
use fnv::FnvHashMap;
use futures::{stream::FuturesUnordered, StreamExt};
use libcrypto::hash::Hash;
use network::{plaintcp::TcpSimpleSender, Acknowledgement, Identifier, NetSender};
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    time::{sleep, Instant},
};
pub use waiter::*;

mod waiter;

#[derive(Debug)]
pub struct Synchronizer<Id, Round, Tx, Storage> {
    /// Id of this node
    my_name: Id,

    /// This is the channel used to get messages from the consensus layer
    rx_consensus: UnboundedReceiver<ConsensusMempoolMsg<Id, Round, Tx>>,

    /// The number of history rounds we need to maintain in the storage
    gc_depth: Round,

    /// Keeps track of the latest gc round
    latest_gc_round: Round,

    /// Keeps the digests (of batches) that are waiting to be processed by the
    /// consensus. Their processing will resume when we get the missing
    /// batches in the store or we no longer need them. It also keeps the
    /// round number and a timestamp (`u128`) of each request we sent.
    pending: FnvHashMap<BatchHash<Tx>, (Round, UnboundedSender<()>, Instant)>,

    /// Used to send sync messages to the network
    mempool_sender: TcpSimpleSender<Id, MempoolMsg<Id, Tx>, Acknowledgement>,

    /// Storage to clean
    storage: Storage,

    /// Synchronization wait time
    wait_time: Duration,

    /// Current round
    round: Round,

    /// All IDs
    all_ids: Vec<Id>,

    /// The number of nodes to send the request to, after failing to get it from
    /// the original sender
    sync_retry_nodes: usize,
}

impl<Id, Round, Tx, Storage> Synchronizer<Id, Round, Tx, Storage>
where
    Tx: Transaction,
    Id: Identifier,
    Round: crate::Round,
    Storage: libstorage::Store,
{
    pub fn spawn(
        my_name: Id,
        rx_consensus: UnboundedReceiver<ConsensusMempoolMsg<Id, Round, Tx>>,
        gc_depth: Round,
        mempool_sender: TcpSimpleSender<Id, MempoolMsg<Id, Tx>, Acknowledgement>,
        storage: Storage,
        wait_time: Duration,
        all_ids: Vec<Id>,
        sync_retry_nodes: usize,
    ) {
        tokio::spawn(async move {
            Self {
                my_name,
                rx_consensus,
                gc_depth,
                latest_gc_round: Round::MIN,
                pending: FnvHashMap::default(),
                mempool_sender,
                storage,
                wait_time,
                round: Round::MIN,
                all_ids,
                sync_retry_nodes,
            }
            .run()
            .await;
        });
    }

    pub async fn run(&mut self) {
        // This queue waits for synchronization messages to be resolved
        let mut sync_waiting = FuturesUnordered::new();
        let timer = sleep(self.wait_time);
        let mut timer = Box::pin(timer);

        loop {
            tokio::select! {
                // Handle messages from consensus
                Some(message) = self.rx_consensus.recv() => match message {
                    ConsensusMempoolMsg::UnknownBatch(source, hashes) => {
                        // Check pending and obtain all hashes for which we have not already requested a batch
                        let missing = hashes.iter()
                            .filter_map(|hash| {
                                if !self.pending.contains_key(hash) {
                                    return Some(hash.clone());
                                }
                                None
                            })
                            .collect::<Vec<_>>();

                        for missing_hash in &missing {
                            log::debug!("Request sync for {}", missing_hash);
                            // Add the digest to the waiter.
                            let (tx_cancel, rx_cancel) = unbounded_channel();
                            let fut = wait::<Storage, Batch<Tx>>(self.storage.clone(), missing_hash.clone(), rx_cancel);
                            sync_waiting.push(fut);
                            self.pending.insert(missing_hash.clone(), (self.round, tx_cancel, Instant::now()));
                        }

                        // Send sync request to a single node. If this fails, we will send it
                        // to other nodes when a timer times out.
                        let message = MempoolMsg::<Id, Tx>::RequestBatch(self.my_name.clone(), missing);
                        self.mempool_sender.send(source, message).await;
                    }

                    ConsensusMempoolMsg::End(round) => {
                        self.round = round;

                        if self.latest_gc_round > round {
                            log::debug!("Already cleaned {:?}", round);
                            continue;
                        }

                        self.latest_gc_round = round - self.gc_depth;
                        for (r, handler, _) in self.pending.values() {
                            if r < &self.latest_gc_round {
                                let _ = handler.send(());
                            }
                        }
                        self.pending.retain(|_, (r, _, _)| r > &mut self.latest_gc_round);
                    }
                },

                // Some request which we were waiting for has been resolved
                Some(result) = sync_waiting.next() => match result {
                    Ok(None) => {
                        log::debug!("Sync request was cancelled!");
                    },
                    Ok(Some(hash_vec)) => {
                        // We got the batch, remove it from the pending list.
                        let hash: Hash<Batch<Tx>> = hash_vec[0..32].try_into().unwrap();
                        self.pending.remove(&hash);
                    },
                    Err(e) => {
                        log::error!("Got error while synchronizing: {}", e);
                    },
                },

                // Triggered when the sync timer expires
                () = (&mut timer) => {
                    // We optimistically sent sync requests to a single node. If this timer triggers,
                    // it means we were wrong to trust it. We are done waiting for a reply and we now
                    // broadcast the request to a bunch of other nodes (selected at random).
                    let now = Instant::now();
                    let mut retry = Vec::new();
                    for (hash, (_, _, req_time)) in &self.pending {
                        if now - self.wait_time > *req_time {
                            log::debug!("Requesting sync for batch {:?} (retry)", hash);
                            retry.push(hash.clone());
                        }
                    }
                    if !retry.is_empty() {
                        let message = MempoolMsg::<Id, Tx>::RequestBatch(self.my_name.clone(), retry);
                        self.mempool_sender
                            .randcast(message, self.all_ids.clone(), self.sync_retry_nodes)
                            .await;
                    }

                    // Reschedule the timer.
                    timer.as_mut().reset(Instant::now() + self.wait_time);
                }
            }
        }
        // log::warn!("Synchronizer is shutting down!");
    }
}
