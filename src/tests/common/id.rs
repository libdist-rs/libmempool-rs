use network::{Identifier, Message};
use serde::{Deserialize, Serialize};

#[derive(Debug, Hash, PartialEq, PartialOrd, Eq, Serialize, Deserialize, Clone)]
pub struct Id(usize);

impl Message for Id {}
impl Identifier for Id {}

impl From<usize> for Id {
    fn from(id: usize) -> Self {
        Self { 0: id }
    }
}
