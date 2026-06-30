use derive_more::{Deref, DerefMut};
use synonym::Synonym;

/// A column in index coordinates.
#[derive_const(Synonym, Deref, DerefMut)]
#[synonym(skip(Value))]
#[repr(transparent)]
pub struct Column(pub u16);

const impl Column {
    pub fn into_inner(self) -> u16 {
        self.0
    }
}
