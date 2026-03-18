use crate::{Batch, BatchHash, MempoolMsg, Transaction};
use bytes::Bytes;
use serde::Serialize;
use std::fmt::Debug;
use tcp_sender::TcpSimpleSender;
use tokio::sync::mpsc::UnboundedReceiver;

/// The responsibility of this struct is to help other mempools by responding to
/// their sync requests
pub struct Helper<Id, Storage, Tx>
where
    Tx: Transaction,
{
    mempool_sender: TcpSimpleSender<Id, MempoolMsg<Id, Tx>>,
    rx_request: UnboundedReceiver<(Id, Vec<BatchHash<Tx>>)>,
    store: Storage,
}

impl<Id, Storage, Tx> Helper<Id, Storage, Tx>
where
    Id: Debug + Clone + Eq + std::hash::Hash + Send + Sync + Serialize + 'static,
    Storage: libstorage::Store,
    Tx: Transaction,
{
    pub fn spawn(
        mempool_sender: TcpSimpleSender<Id, MempoolMsg<Id, Tx>>,
        rx_request: UnboundedReceiver<(Id, Vec<BatchHash<Tx>>)>,
        store: Storage,
    ) {
        tokio::spawn(async move {
            Self {
                mempool_sender,
                rx_request,
                store,
            }
            .run()
            .await
        });
    }

    async fn run(&mut self) {
        while let Some((source, digests)) = self.rx_request.recv().await {
            for digest in digests {
                match self.store.read(digest.to_vec()).await {
                    Ok(Some(data)) => {
                        let b: Batch<Tx> = bincode::deserialize(&data).unwrap();
                        let msg: MempoolMsg<Id, Tx> = MempoolMsg::Batch(b);
                        let serialized = Bytes::from(bincode::serialize(&msg).unwrap());
                        if let Err(e) = self.mempool_sender.send(source.clone(), serialized).await {
                            log::warn!("Helper send error: {}", e);
                        }
                    }
                    Ok(None) => log::debug!("Digest: {} not found", digest),
                    Err(e) => log::warn!("Store Error: {}", e),
                }
            }
        }
        log::warn!("Helper is quitting");
    }
}
