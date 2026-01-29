use crate::Position;
use crate::{Point, Rect, Size};
use derive_more::{AsRef, Deref, DerefMut, From, Into, Mul};
use std::iter::FusedIterator;
use std::ops::Range;
use std::ops::{Add, AddAssign, Sub};

/// A row in buffer coordinates.
#[derive(
    Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, AsRef, Deref, DerefMut, From, Into,
)]
pub struct Row(pub usize);

pub type RowLike = [usize; 1];

impl Row {
    /// Create a new row at the given index.
    pub const fn new(row: usize) -> Self {
        Self(row)
    }
}

impl From<RowLike> for Row {
    fn from(value: RowLike) -> Self {
        Self::new(value[0])
    }
}

impl From<Position> for Row {
    fn from(value: Position) -> Self {
        Self::new(value.row)
    }
}
