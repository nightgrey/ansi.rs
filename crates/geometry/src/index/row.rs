use crate::Position;
use derive_more::{AsRef, From, Into};
use std::ops::{Deref, DerefMut};

/// A row in buffer coordinates.
#[derive(Copy, Debug)]
#[derive_const(Clone, Default, PartialEq, Eq, PartialOrd, Ord, AsRef, From, Into)]
pub struct Row(pub usize);

impl Row {
    /// Create a new row at the given index.
    pub const fn new(row: usize) -> Self {
        Self(row)
    }
}

impl const Deref for Row {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl const DerefMut for Row {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Position> for Row {
    fn from(value: Position) -> Self {
        Self::new(value.row)
    }
}
