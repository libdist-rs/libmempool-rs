use futures::Future;
use network::Message;

pub trait Transaction: Message {}

impl<T> Transaction for T
where
    T: Message,
{}

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
    const MIN: Self;
}

macro_rules! implement_round {
    ($tp: ty, $default: literal) => {
        impl crate::Round for $tp {
            const MIN: $tp = $default;
        }
    };
}

implement_round!(u8, 0);
implement_round!(u16, 0);
implement_round!(u32, 0);
implement_round!(u64, 0);
implement_round!(u128, 0);
implement_round!(i8, 0);
implement_round!(i16, 0);
implement_round!(i32, 0);
implement_round!(i64, 0);
implement_round!(i128, 0);

pub trait Sealer<Tx>: Send + Sync + 'static + Future<Output = Vec<Tx>> + Unpin {
    /// Cleans the sealer and returns all transactions
    fn seal(&mut self) -> Vec<Tx>;

    /// Updates the sealer with a new transaction
    fn update(
        &mut self,
        _tx: Tx,
        _tx_size: usize,
    ) {
    }
}
