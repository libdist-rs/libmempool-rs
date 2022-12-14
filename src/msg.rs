use std::fmt::{Debug, Formatter, self};

use libcrypto::hash::Hash;
use serde::{Deserialize, Serialize};

/// A short-hand to represent Hash<Batch<Tx>>
pub type BatchHash<Tx> = Hash<Batch<Tx>>;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Batch<Tx> {
    pub payload: Vec<Tx>,
}

impl<Tx> Debug for Batch<Tx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Batch")
            .field("payload length", &self.payload.len())
            .finish()
    }
}

impl<Tx> From<Vec<Tx>> for Batch<Tx> {
    fn from(tx_batch: Vec<Tx>) -> Self {
        Self { payload: tx_batch }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MempoolMsg<Id, Tx> {
    RequestBatch(Id, Vec<BatchHash<Tx>>),
    /// This is sent by the primary or by a helper
    Batch(Batch<Tx>),
}

pub enum ConsensusMempoolMsg<Id, Round, Tx> {
    End(Round),
    UnknownBatch(Id, Vec<BatchHash<Tx>>),
}
