use crate::{Error, Id};
use derive_more::{Deref, DerefMut, Index, IndexMut};

#[derive(Debug, Deref, DerefMut, Index, IndexMut)]
#[repr(transparent)]
pub struct Secondary<K: Id, V> {
    inner: slotmap::SecondaryMap<K, V>,
}

impl<K: Id, V> Secondary<K, V> {
    pub fn new() -> Self {
        Self::with_capacity(16)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: slotmap::SecondaryMap::with_capacity(capacity),
        }
    }


    pub fn contains(&self, id: K) -> bool {
        self.inner.contains_key(id)
    }

    pub fn get(&self, id: K) -> Option<&V> {
        self.inner.get(id)
    }

    pub fn get_mut(&mut self, id: K) -> Option<&mut V> {
        self.inner.get_mut(id)
    }

    /// Inserts a value into the secondary tree at the given `id`.
    ///
    /// Returns the previous value if any.
    pub fn insert(&mut self, id: K, value: V) -> Option<V> {
        self.inner.insert(id, value)
    }

    // --- Mutation ----------------------------------------------------------
    /// Removes a id from the secondary map, returning the value at the id if the id was not previously removed.
    pub fn remove(&mut self, id: K) -> Result<V, Error<K>> {
        self.inner.remove(id).map_or_else(|| Err(Error::Missing(id)), Ok)
    }
}
