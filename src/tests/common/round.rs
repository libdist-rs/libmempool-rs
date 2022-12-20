use std::fmt::{self, Display, Formatter};
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
        Self(self.0 - rhs.0)
    }
}

impl From<usize> for Round {
    fn from(num: usize) -> Self {
        Self(num)
    }
}

impl crate::Round for Round {
    const MIN: Self = Self(0);
}
