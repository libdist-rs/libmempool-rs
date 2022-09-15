use std::time::Duration;

#[derive(Debug)]
pub struct Config<Round> {
    /// The depth of the garbage collection (Denominated in number of rounds).
    pub gc_depth: Round,
    /// The delay after which the synchronizer retries to send sync requests.
    /// Denominated in ms.
    pub sync_retry_delay: Duration,
    /// Determine with how many nodes to sync when re-trying to send
    /// sync-request. These nodes are picked at random from the committee.
    pub sync_retry_nodes: usize,
}

impl<Round> Config<Round> 
where 
    Round: crate::Round,
{
    pub fn log(&self) {
        log::info!("GC Depth: {}", self.gc_depth);
        log::info!("Sync retry delay: {} ms", self.sync_retry_delay.as_millis());
        log::info!("Sync retry nodes: {}", self.sync_retry_nodes);
    }
}

impl<Round> Default for Config<Round>
where
    Round: crate::Round,
{
    fn default() -> Self {
        Self {
            gc_depth: Default::default(),
            sync_retry_delay: Duration::from_millis(100),
            sync_retry_nodes: 3,
        }
    }
}
