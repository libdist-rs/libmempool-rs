use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tx(pub bool);

impl net_common::Message for Tx {
    type DeserializationError = Box<bincode::ErrorKind>;

    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::DeserializationError> {
        bincode::deserialize(bytes)
    }
}

pub fn dummy_tx() -> Tx {
    Tx(true)
}
