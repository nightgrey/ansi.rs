use crate::{Error, Id};
use derive_more::{Deref, DerefMut, Index, IndexMut};

/// A secondary map for associating auxiliary data with tree nodes.
///
/// `Secondary<K, V>` is a thin wrapper around [`slotmap::SecondaryMap`] that
/// uses the same key type as a [`Tree`](crate::Tree). This lets you attach
/// extra per-node data (e.g. computed styles, user state) without modifying
/// the tree's value type.
///
/// Keys are only valid if they also exist in the primary tree — it is the
/// caller's responsibility to keep the two in sync.
#[derive(Debug, Deref, DerefMut, Index, IndexMut)]
#[repr(transparent)]
pub struct Secondary<K: Id, V> {
   inner: slotmap::SecondaryMap<K, V>,
}

impl<K: Id, V> Secondary<K, V> {
    /// Creates a new secondary map with a default capacity of 16.
    pub fn new() -> Self {
        Self::with_capacity(16)
    }

    /// Creates a new secondary map pre-allocated for `capacity` entries.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: slotmap::SecondaryMap::with_capacity(capacity),
        }
    }

    /// Returns `true` if the map contains an entry for the given id.
    pub fn contains(&self, id: K) -> bool {
        self.inner.contains_key(id)
    }

    /// Returns a reference to the value associated with `id`, or `None`.
    pub fn get(&self, id: K) -> Option<&V> {
        self.inner.get(id)
    }

    /// Returns a mutable reference to the value associated with `id`, or `None`.
    pub fn get_mut(&mut self, id: K) -> Option<&mut V> {
        self.inner.get_mut(id)
    }

    /// Inserts a value at the given id, returning the previous value if any.
    pub fn insert(&mut self, id: K, value: V) -> Option<V> {
        self.inner.insert(id, value)
    }

    /// Removes the entry at `id`, returning the value.
    ///
    /// Returns [`Error::Missing`] if no entry exists for the given id.
    pub fn remove(&mut self, id: K) -> Result<V, Error<K>> {
        self.inner.remove(id).map_or_else(|| Err(Error::Missing(id)), Ok)
    }
}

pub type Iter<'a, K: Id + 'a, V: 'a> = slotmap::secondary::Iter<'a, K, V>;
pub type IterMut<'a, K: Id + 'a, V: 'a> = slotmap::secondary::IterMut<'a, K, V>;
pub type Drain<'a, K: Id + 'a, V: 'a> = slotmap::secondary::Drain<'a, K, V>;
pub type Keys<'a, K: Id + 'a, V: 'a> = slotmap::secondary::Keys<'a, K, V>;
pub type Values<'a, K: Id + 'a, V: 'a> = slotmap::secondary::Values<'a, K, V>;
pub type ValuesMut<'a, K: Id + 'a, V: 'a> = slotmap::secondary::ValuesMut<'a, K, V>;