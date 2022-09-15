use std::fmt::{self, Display, Formatter};

use network::Message;
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Hash, PartialEq, PartialOrd, Eq, Serialize, Deserialize, Clone, Copy, Default, Ord,
)]
pub struct Round(usize);

impl Display for Round {
    fn fmt(
        &self,
        f: &mut Formatter<'_>,
    ) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::ops::Sub for Round {
    type Output = Self;
    fn sub(
        self,
        rhs: Self,
    ) -> Self::Output {
        Self { 0: self.0 - rhs.0 }
    }
}

impl From<usize> for Round {
    fn from(num: usize) -> Self {
        Self { 0: num }
    }
}

impl Message for Round {}
impl crate::Round for Round {}
