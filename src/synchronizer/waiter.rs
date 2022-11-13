use anyhow::Result;
use libcrypto::hash::Hash;
use tokio::sync::mpsc::UnboundedReceiver;

/// This struct waits for a key to be available in a Store
pub async fn wait<Storage, T>(
    mut store: Storage,
    key: Hash<T>,
    mut cancel_handler: UnboundedReceiver<()>,
) -> Result<Option<Vec<u8>>>
where
    Storage: libstorage::Store,
{
    tokio::select! {
        result = store.notify_read(key.to_vec()) => {
            result.map(|_| Some(key.to_vec()))
        },
        _ = cancel_handler.recv() => Ok(None),
    }
}
