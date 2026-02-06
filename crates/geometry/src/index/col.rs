use crate::Position;
use derive_more::{AsRef, Deref, DerefMut, From, Into};

/// A column in buffer coordinates.
#[derive(
    Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, AsRef, Deref, DerefMut, From, Into,
)]
pub struct Col(pub usize);

impl Col {
    /// Create a new row at the given index.
    pub const fn new(row: usize) -> Self {
        Self(row)
    }
}

impl From<Position> for Col {
    fn from(value: Position) -> Self {
        Self::new(value.col)
    }
}
