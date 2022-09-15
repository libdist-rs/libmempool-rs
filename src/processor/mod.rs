use crate::{Batch, Transaction};
use libcrypto::hash::{do_hash, Hash};
use std::marker::PhantomData;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub struct Processor<Storage, Tx> {
    _x: PhantomData<(Storage, Tx)>,
}

impl<Storage, Tx> Processor<Storage, Tx>
where
    Storage: libstorage::Store,
    Tx: Transaction,
{
    pub fn spawn(
        mut store: Storage,
        // Input channel to receive batches.
        mut rx_processor: UnboundedReceiver<Batch<Tx>>,
        // Output channel to send out batches' digests.
        tx_hash: UnboundedSender<Hash>,
    ) {
        tokio::spawn(async move {
            while let Some(batch) = rx_processor.recv().await {
                let serialized_batch =
                    bincode::serialize(&batch).expect("Failed to serialize batch");
                // Hash the batch
                let hash = do_hash(&serialized_batch);
                store.write(hash.to_vec(), serialized_batch).await;

                let _ = tx_hash.send(hash);
            }
        });
    }
}
