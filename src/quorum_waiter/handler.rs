use anyhow::Result;
use futures::{stream::FuturesUnordered, StreamExt};
use network::plaintcp::CancelHandler;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

pub struct Message<MsgId> {
    batch: MsgId,
    handlers: Vec<CancelHandler<MsgId>>,
}

/// This struct serially waits for to obtain a quorum of acknowledgements before
/// moving to handling another quorum waiting.
pub struct HandlerWaiter<MsgId> {
    num_of_ids_to_wait_for: usize,
    notify: UnboundedSender<MsgId>,
    rx: UnboundedReceiver<Message<MsgId>>,
}

impl<MsgId> HandlerWaiter<MsgId>
where
    MsgId: Send + Sync + 'static + std::fmt::Debug,
{
    pub fn spawn(
        num_of_ids_to_wait_for: usize,
        notify: UnboundedSender<MsgId>,
        rx: UnboundedReceiver<Message<MsgId>>,
    ) {
        tokio::spawn(async move {
            let res = Self {
                num_of_ids_to_wait_for,
                notify,
                rx,
            }
            .run()
            .await;
            if res.is_err() {
                log::error!("HandlerWaiter error: {}", res.unwrap_err());
            }
        });
    }

    /// Helper function. It waits for a future to complete and then delivers a
    /// value.
    async fn waiter(wait_for: CancelHandler<MsgId>) {
        let _ = wait_for.await;
    }

    async fn run(&mut self) -> Result<()> {
        while let Some(Message { batch, handlers }) = self.rx.recv().await {
            let mut wait_future: FuturesUnordered<_> = handlers
                .into_iter()
                .map(|handle| Self::waiter(handle))
                .collect();

            let mut count: usize = 0;
            while let Some(()) = wait_future.next().await {
                count += 1;
                if count >= self.num_of_ids_to_wait_for {
                    self.notify.send(batch)?;
                    break;
                }
            }
        }
        Ok(())
    }
}
