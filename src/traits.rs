// use network::Message;
use serde::{de::DeserializeOwned, Serialize};

pub trait Transaction:
    Clone + std::fmt::Debug + Send + Sync + Serialize + DeserializeOwned + 'static
{
}

pub trait Round:
    Send
    + Sync
    + 'static
    + Ord
    + Default
    + core::fmt::Debug
    + std::ops::Sub<Output = Self>
    + Copy
    + core::fmt::Display
{
}

pub trait Sealer<Tx> {
    fn seal(&mut self) -> Vec<Tx>;
}
