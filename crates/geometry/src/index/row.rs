use derive_more::{Deref, DerefMut};
use std::ops::{Add, Div, Mul, Rem, Sub};
use synonym::Synonym;

/// A row in index coordinates.
#[derive_const(Synonym, Deref, DerefMut)]
#[synonym(skip(Value))]
#[repr(transparent)]
pub struct Row(pub usize);

const impl Row {
    pub fn into_inner(self) -> usize {
        self.0
    }
}
