use std::pin::Pin;

use crate::{Batch, Transaction};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

/// The Batcher will collect transactions and make batches from them
pub struct Batcher<Tx, Sealer> {
    rx_transaction: UnboundedReceiver<(Tx, usize)>,
    tx_output: UnboundedSender<Batch<Tx>>,
    sealer: Pin<Box<Sealer>>,
}

impl<Tx, Sealer> Batcher<Tx, Sealer>
where
    Tx: Transaction,
    Sealer: crate::Sealer<Tx>,
{
    pub fn spawn(
        rx_transaction: UnboundedReceiver<(Tx, usize)>,
        tx_output: UnboundedSender<Batch<Tx>>,
        sealer: Sealer,
    ) {
        tokio::spawn(async move {
            Self {
                rx_transaction,
                tx_output,
                sealer: Box::pin(sealer),
            }
            .run()
            .await
        });
    }

    async fn run(mut self) {
        loop {
            tokio::select! {
                Some((tx, tx_size)) = self.rx_transaction.recv() => {
                    log::debug!("Got a transaction");
                    self.sealer.as_mut().get_mut().update(tx, tx_size);
                }
                batch = (&mut self.sealer) => {
                    if let Err(e) = self.tx_output.send(batch.into()) {
                        log::error!("Batcher Error: {}", e);
                        break;
                    }
                }
            }
        }
        log::info!("Batcher is shutting down!");
    }
}
