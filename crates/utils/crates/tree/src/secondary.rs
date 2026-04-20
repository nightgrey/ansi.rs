use crate::{Error, Id};
use derive_more::{Index, IndexMut};
use std::fmt::Debug;

/// A secondary map for associating auxiliary data with tree nodes.
///
/// `Secondary<K, V>` is a thin wrapper around [`slotmap::SecondaryMap`] that
/// uses the same key type as a [`Tree`](crate::Tree). This lets you attach
/// extra per-node data (e.g. computed styles, user state) without modifying
/// the tree's value type.
///
/// Keys are only valid if they also exist in the primary tree — it is the
/// caller's responsibility to keep the two in sync.
#[derive(Index, IndexMut)]
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

    pub fn set(&mut self, key: K, value: V) {
        self.try_set(key, value).unwrap()
    }

    pub fn try_set(&mut self, key: K, value: V) -> Result<(), Error<K>> {
        if !self.contains(key) {
            return Err(Error::Missing(key));
        }

        self.inner[key] = value;

        Ok(())
    }

    /// Inserts a value at the given id, returning the previous value if any.
    pub fn insert(&mut self, id: K, value: V) -> Option<V> {
        self.inner.insert(id, value)
    }

    /// Removes the entry at `id`, returning the removed value.
    pub fn remove(&mut self, id: K) -> Option<V> {
        self.inner.remove(id)
    }

    /// Returns the number of entries in the map.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if the map contains no entries.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Removes all entries from the map.
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    /// Iterates over all `(key, &value)` pairs.
    pub fn iter(&self) -> Iter<'_, K, V> {
        self.inner.iter()
    }

    /// Mutably iterates over all `(key, &mut value)` pairs.
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        self.inner.iter_mut()
    }

    /// Iterates over all keys.
    pub fn keys(&self) -> Keys<'_, K, V> {
        self.inner.keys()
    }

    /// Iterates over all values.
    pub fn values(&self) -> Values<'_, K, V> {
        self.inner.values()
    }

    /// Mutably iterates over all values.
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        self.inner.values_mut()
    }

    /// Removes all entries, yielding them as `(key, value)` pairs.
    pub fn drain(&mut self) -> Drain<'_, K, V> {
        self.inner.drain()
    }
}

impl<K: Id, V> Clone for Secondary<K, V>
where
    V: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.inner.clone_from(&source.inner);
    }
}
impl<K: Id, V> Default for Secondary<K, V> {
    fn default() -> Self {
        Self::new()
    }
}
impl<K: Id, V: Debug> Debug for Secondary<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<K: Id, V> Extend<(K, V)> for Secondary<K, V> {
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        self.inner.extend(iter);
    }
}

impl<'a, K: Id, V: 'a + Copy> Extend<(K, &'a V)> for Secondary<K, V> {
    fn extend<I: IntoIterator<Item = (K, &'a V)>>(&mut self, iter: I) {
        self.inner.extend(iter);
    }
}
pub type Iter<'a, K, V> = slotmap::secondary::Iter<'a, K, V>;
pub type IterMut<'a, K, V> = slotmap::secondary::IterMut<'a, K, V>;
pub type Drain<'a, K, V> = slotmap::secondary::Drain<'a, K, V>;
pub type Keys<'a, K, V> = slotmap::secondary::Keys<'a, K, V>;
pub type Values<'a, K, V> = slotmap::secondary::Values<'a, K, V>;
pub type ValuesMut<'a, K, V> = slotmap::secondary::ValuesMut<'a, K, V>;
