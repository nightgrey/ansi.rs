use std::ops::{Add, Div, Mul, Rem, RemAssign, Sub, SubAssign};
use derive_more::{Deref, DerefMut};
use synonym::Synonym;

/// A column in buffer coordinates.
#[derive_const(Synonym)]
#[synonym(skip(Value))]
#[repr(transparent)]
#[derive(Deref, DerefMut)]
pub struct Column(pub usize);

impl const Column {
    pub fn value(self) -> usize {
        self.0
    }
}
