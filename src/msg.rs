use crate::Transaction;
use libcrypto::hash::Hash;
use network::Message;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Batch<Tx> {
    payload: Vec<Tx>,
}

impl<Tx> Message for Batch<Tx>
where
    Tx: Transaction,
{
    fn from_bytes(data: &[u8]) -> Self {
        bincode::deserialize(data).unwrap()
    }

    fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MempoolMsg<Tx> {
    RequestBatch(Vec<Hash>),
    /// This is sent by the primary or by a helper
    Batch(Batch<Tx>),
}

impl<Tx> Message for MempoolMsg<Tx>
where
    Tx: Transaction,
{
    fn from_bytes(data: &[u8]) -> Self {
        bincode::deserialize(data).unwrap()
    }

    fn to_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap()
    }
}

pub enum ConsensusMempoolMsg<Id, Round> {
    End(Round),
    UnknownBatch(Id, Vec<Hash>),
}
