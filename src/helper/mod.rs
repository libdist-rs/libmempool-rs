use libcrypto::hash::Hash;
use network::{plaintcp::TcpSimpleSender, Acknowledgement, Identifier, Message, NetSender};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::{Batch, MempoolMsg, Transaction};

/// The responsibility of this struct is to help other mempools by responding to
/// their sync requests
pub struct Helper<Id, Storage, Tx>
where
    Tx: Transaction,
{
    mempool_sender: TcpSimpleSender<Id, MempoolMsg<Tx>, Acknowledgement>,
    rx_request: UnboundedReceiver<(Vec<Hash>, Id)>,
    store: Storage,
}

impl<Id, Storage, Tx> Helper<Id, Storage, Tx>
where
    Id: Identifier,
    Storage: libstorage::Store,
    Tx: Transaction,
{
    pub fn spawn(
        mempool_sender: TcpSimpleSender<Id, MempoolMsg<Tx>, Acknowledgement>,
        rx_request: UnboundedReceiver<(Vec<Hash>, Id)>,
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
        while let Some((digests, source)) = self.rx_request.recv().await {
            for digest in digests {
                match self.store.read(digest.to_vec()).await {
                    Ok(Some(data)) => {
                        let b: Batch<Tx> = Batch::from_bytes(&data);
                        let msg = MempoolMsg::Batch(b);
                        self.mempool_sender.send(source.clone(), msg).await;
                    }
                    Ok(None) => (),
                    Err(e) => log::warn!("Store Error: {}", e),
                }
            }
        }
        log::warn!("Helper is quitting");
    }
}
