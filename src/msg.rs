use crate::Transaction;
use libcrypto::hash::Hash;
use network::{Identifier, Message};
use serde::{Deserialize, Serialize};

/// A short-hand to represent Hash<Batch<Tx>>
pub type BatchHash<Tx> = Hash<Batch<Tx>>;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Batch<Tx> {
    payload: Vec<Tx>,
}

impl<Tx> From<Vec<Tx>> for Batch<Tx> {
    fn from(tx_batch: Vec<Tx>) -> Self {
        Self { payload: tx_batch }
    }
}

impl<Tx> Message for Batch<Tx> where Tx: Transaction {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MempoolMsg<Id, Tx> {
    RequestBatch(Id, Vec<BatchHash<Tx>>),
    /// This is sent by the primary or by a helper
    Batch(Batch<Tx>),
}

impl<Id, Tx> Message for MempoolMsg<Id, Tx>
where
    Tx: Transaction,
    Id: Identifier,
{
}

pub enum ConsensusMempoolMsg<Id, Round, Tx> {
    End(Round),
    UnknownBatch(Id, Vec<BatchHash<Tx>>),
}
