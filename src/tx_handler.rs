use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone)]
/// Forwards received transactions to the batcher
pub struct TxReceiveHandler<Tx> {
    tx_batcher: UnboundedSender<(Tx, usize)>,
}

impl<Tx> TxReceiveHandler<Tx> {
    pub fn new(tx_batcher: UnboundedSender<(Tx, usize)>) -> Self {
        Self { tx_batcher }
    }

    /// Dispatch a received transaction to the batcher channel
    pub fn dispatch(&self, msg: Tx)
    where
        Tx: serde::Serialize,
    {
        let size = bincode::serialized_size(&msg).unwrap() as usize;
        if let Err(e) = self.tx_batcher.send((msg, size)) {
            log::error!("Tx Handler error: {}", e);
        }
    }
}
