use network::Message;
use serde::{Deserialize, Serialize};

use crate::Transaction;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Hash, Eq)]
pub struct Tx(bool);

impl Message for Tx {}
impl Transaction for Tx {}

impl From<bool> for Tx {
    fn from(b: bool) -> Self {
        Self { 0: b }
    }
}

impl PartialEq for Tx {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.0 == other.0
    }
}

pub fn transform(old_vec: Vec<bool>) -> Vec<Tx> {
    old_vec.into_iter().map(|b| b.into()).collect()
}

impl Tx {
    pub fn dummy() -> Self {
        true.into()
    }
}