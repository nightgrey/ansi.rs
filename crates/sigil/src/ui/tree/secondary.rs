use super::{Key, Node, iter::*};
use derive_more::{Deref, DerefMut, Index, IndexMut};
type Inner<K, V> = slotmap::SecondaryMap<K, V>;

#[derive(Debug, Deref, DerefMut, Index, IndexMut)]
#[repr(transparent)]
pub struct Secondary<K: Key, V> {
    inner: Inner<K, V>,
}

impl<K: Key, V> Secondary<K, V> {
    pub fn new() -> Self {
        Self {
            inner: Inner::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Inner::with_capacity(capacity),
        }
    }
}
