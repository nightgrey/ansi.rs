use super::position::Position;
use std::ops::{Add, Div, Mul, Rem, RemAssign, Sub, SubAssign};
use synonym::Synonym;

/// A column in buffer coordinates.
#[derive_const(Synonym)]
#[synonym(skip(Value))]
#[repr(transparent)]
pub struct Column(pub usize);

impl const Column {
    pub fn value(self) -> usize {
        self.0
    }
}

impl From<Position> for Column {
    fn from(value: Position) -> Self {
        Self(value.col)
    }
}
