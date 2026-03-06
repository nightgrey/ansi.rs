use std::ops::{Add, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign};
use synonym::Synonym;

/// A row in buffer coordinates.
#[derive(Synonym)]
#[synonym(skip(Value))]
pub struct Index(pub usize);
impl const Index {
    pub fn value(self) -> usize {
        self.0
    }
}

pub type IndexLike = usize;