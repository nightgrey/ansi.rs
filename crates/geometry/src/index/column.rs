use derive_more::{Deref, DerefMut};
use std::ops::{Add, Div, Mul, Rem, RemAssign, Sub, SubAssign};
use synonym::Synonym;

/// A column in index coordinates.
#[derive_const(Synonym, Deref, DerefMut)]
#[synonym(skip(Value))]
#[repr(transparent)]
pub struct Column(pub usize);

const impl Column {
    pub fn into_inner(self) -> usize {
        self.0
    }
}
