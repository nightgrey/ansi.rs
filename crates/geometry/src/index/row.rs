use derive_more::{Deref, DerefMut};
use synonym::Synonym;

/// A row in index coordinates.
#[derive_const(Synonym, Deref, DerefMut)]
#[synonym(skip(Value))]
#[repr(transparent)]
pub struct Row(pub u16);

const impl Row {
    pub fn into_inner(self) -> u16 {
        self.0
    }
}
