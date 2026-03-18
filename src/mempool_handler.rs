use crate::{Batch, BatchHash, MempoolMsg, Transaction};
use std::fmt::Debug;
use std::marker::PhantomData;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone)]
pub struct MempoolHandler<Id, Tx> {
    tx_helper: UnboundedSender<(Id, Vec<BatchHash<Tx>>)>,
    tx_processor: UnboundedSender<Batch<Tx>>,
    _x: PhantomData<Id>,
}

impl<Id, Tx> MempoolHandler<Id, Tx>
where
    Tx: Transaction,
    Id: Debug + Clone + Send + Sync + 'static,
{
    pub fn new(
        tx_helper: UnboundedSender<(Id, Vec<BatchHash<Tx>>)>,
        tx_processor: UnboundedSender<Batch<Tx>>,
    ) -> Self {
        Self {
            tx_helper,
            tx_processor,
            _x: PhantomData,
        }
    }

    /// Dispatch a received mempool message to the appropriate channel
    pub fn dispatch(&self, msg: MempoolMsg<Id, Tx>) {
        match msg {
            MempoolMsg::Batch(batch) => {
                let _ = self.tx_processor.send(batch);
            }
            MempoolMsg::RequestBatch(source, hashes) => {
                let _ = self.tx_helper.send((source, hashes));
            }
        }
    }
}
