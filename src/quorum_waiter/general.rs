use anyhow::Result;
use fnv::FnvHashMap;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

/// The Quorum waiter follows the exactly once semantics.  
/// When the Quorum waiter has exactly `num_of_ids_to_wait_for`
/// acknowledgements, it will notify the sender.
///
/// NOTE: The RecvMsg is the type receiving type used in the corresponding
/// mempool_sender. Usually, it is `network::Acknowledgement`
pub struct General<MsgToWaitFor, Ack> {
    num_of_ids_to_wait_for: usize,
    ack_in: UnboundedReceiver<(MsgToWaitFor, Ack)>,
    notify: UnboundedSender<(MsgToWaitFor, Vec<Ack>)>,
    count_map: FnvHashMap<MsgToWaitFor, Vec<Ack>>,
}

impl<MsgToWaitFor, Ack> General<MsgToWaitFor, Ack>
where
    Ack: std::fmt::Debug + Send + Sync + 'static + Clone,
    MsgToWaitFor: std::fmt::Debug + Send + Sync + 'static + std::hash::Hash + std::cmp::Eq + Clone,
{
    pub fn spawn(
        num_of_ids_to_wait_for: usize,
        ack_in: UnboundedReceiver<(MsgToWaitFor, Ack)>,
        notify: UnboundedSender<(MsgToWaitFor, Vec<Ack>)>,
    ) {
        tokio::spawn(async move {
            let mut obj = Self {
                num_of_ids_to_wait_for,
                ack_in,
                notify,
                count_map: FnvHashMap::default(),
            };
            match obj.run().await {
                Ok(()) => {}
                Err(e) => log::error!("Quorum waiter terminated with {}", e),
            }
        });
    }

    async fn run(&mut self) -> Result<()> {
        while let Some((msg, ack)) = self.ack_in.recv().await {
            let val = self.count_map
                .entry(msg.clone())
                .or_insert_with(Vec::default);

            val.push(ack);

            if val.len() == self.num_of_ids_to_wait_for {
                self.notify.send((msg, val.clone()))?;
            }
        }
        log::info!("Quorum waiter is shutting down");
        Ok(())
    }
}
