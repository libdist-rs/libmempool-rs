use std::marker::PhantomData;
use async_trait::async_trait;
use futures::SinkExt;
use network::{Acknowledgement, Handler, Identifier};
use tokio::sync::mpsc::UnboundedSender;
use crate::{Batch, MempoolMsg, Transaction, BatchHash};

#[derive(Debug, Clone)]
pub struct MempoolHandler<Id, Tx> {
    tx_helper: UnboundedSender<(Id, Vec<BatchHash<Tx>>)>,
    tx_processor: UnboundedSender<Batch<Tx>>,
    _x: PhantomData<Id>,
}

#[async_trait]
impl<Id, Tx> Handler<Acknowledgement, MempoolMsg<Id, Tx>> for MempoolHandler<Id, Tx>
where
    Tx: Transaction,
    Id: Identifier,
{
    async fn dispatch(
        &self,
        msg: MempoolMsg<Id, Tx>,
        writer: &mut network::Writer<Acknowledgement>,
    ) {
        match msg {
            MempoolMsg::Batch(batch) => {
                let _ = self.tx_processor.send(batch);
            }
            MempoolMsg::RequestBatch(source, hashes) => {
                let _ = self.tx_helper.send((source, hashes));
            }
        }
        let _ = writer.send(Acknowledgement::Pong).await;
    }
}

impl<Id, Tx> MempoolHandler<Id, Tx> {
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
}
