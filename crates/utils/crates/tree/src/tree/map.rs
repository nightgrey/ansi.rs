use crate::Id;
use derive_more::{Deref, DerefMut, Index, IndexMut, IntoIterator};

#[derive(Debug, Deref, DerefMut, Index, IndexMut, IntoIterator)]
#[repr(transparent)]
#[into_iterator(owned, ref, ref_mut)]
pub struct Map<K: Id, V> {
    inner: slotmap::SlotMap<K, V>,
}

impl<K: Id, V> Map<K, V> {
    /// Creates a new secondary map with a default capacity of 16.
    pub fn new() -> Self {
        Self::with_capacity(16)
    }

    /// Creates a new secondary map pre-allocated for `capacity` entries.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: slotmap::SlotMap::with_capacity_and_key(capacity),
        }
    }
}
