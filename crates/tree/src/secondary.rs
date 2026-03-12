use super::{Id};
use derive_more::{Deref, DerefMut, Index, IndexMut};

#[derive(Debug, Deref, DerefMut, Index, IndexMut)]
#[repr(transparent)]
pub struct SecondaryTree<K: Id, V> {
    inner: slotmap::SecondaryMap<K, V>,
}

impl<K: Id, V> SecondaryTree<K, V> {
    pub fn new() -> Self {
        Self::with_capacity(16)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: slotmap::SecondaryMap::with_capacity(capacity),
        }
    }
}
