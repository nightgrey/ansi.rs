use std::ops::{Add, Div, Mul, Rem, Sub};
use derive_more::{Deref, DerefMut};
use synonym::Synonym;

/// A row in index coordinates.
#[derive_const(Synonym, Deref, DerefMut)]
#[synonym(skip(Value))]
#[repr(transparent)]
pub struct Row(pub usize);

impl const Row {
    pub fn into_inner(self) -> usize {
        self.0
    }
}
