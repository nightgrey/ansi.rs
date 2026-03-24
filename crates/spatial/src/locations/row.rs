use std::ops::{Add, AddAssign, Div, Mul, Rem, Sub};
use synonym::Synonym;
use crate::Position;

/// A row in buffer coordinates.
#[derive_const(Synonym)]
#[synonym(skip(Value))]
#[repr(transparent)]
pub struct Row(pub usize);

impl const Row {
    pub fn value(self) -> usize {
        self.0
    }
}

impl From<Position> for Row {
    fn from(value: Position) -> Self {
        Self(value.row)
    }
}

impl Add<usize> for Row {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl AddAssign<usize> for Row {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs;
    }
}