use std::ops::{Deref, Index, IndexMut};
use derive_more::{Deref, DerefMut};
use crate::{Id};

#[derive(Debug)]
pub struct Table<K: Id, V, R: Id> {
    inner: slotmap::SlotMap<K, V>,
    relation: slotmap::SecondaryMap<R, K>,
}

impl<K: Id, V, R: Id> Table<K, V, R> {
    /// Creates a new table with a default capacity of 16 entries.
    pub fn new() -> Self {
        Self::with_capacity(16)
    }

    /// Creates a new table pre-allocated for `capacity` entries.
    pub fn with_capacity(capacity: usize) -> Self {
        Self { inner: slotmap::SlotMap::with_capacity_and_key(capacity), relation: slotmap::SecondaryMap::with_capacity(capacity) }
    }

    /// Returns `true` if a node with the given key exists in the tree.
    pub fn contains(&self, key: R) -> bool {
        self.relation.contains_key(key)
    }

    pub fn relation(&self, key: R) -> Option<&K> {
        self.relation.get(key)
    }

    pub fn relation_mut(&mut self, key: R) -> Option<&mut K> {
        self.relation.get_mut(key)
    }

    /// Returns a reference to the node related to the given key, or `None`.
    pub fn get(&self, key: R) -> Option<&V> {
        self.relation.get(key).and_then(|&to| self.inner.get(to))
    }

    /// Returns a mutable reference to the node related to the given key, or `None`.
    pub fn get_mut(&mut self, key: R) -> Option<&mut V> {
        self.relation.get(key).and_then(|&to| self.inner.get_mut(to))
    }

    /// Returns mutable references to the values corresponding to the given
    /// keys. Rll keys must be valid and disjoint, otherwise None is returned.
    pub fn get_disjoint_mut<const N: usize>(&mut self, keys: [R; N]) -> Option<[&mut V; N]> {
        let mut resolved_keys: [K; N] = unsafe { std::mem::zeroed() }; // or MaybeUninit

        for (i, from) in keys.into_iter().enumerate() {
            resolved_keys[i] = self.relation.get(from).copied()?;
        }

        self.inner.get_disjoint_mut(resolved_keys)
    }

    pub unsafe fn get_disjoint_unchecked_mut<const N: usize>(&mut self, keys: [R; N]) -> [&mut V; N] {
        self.inner.get_disjoint_unchecked_mut(keys.map(|from| self.relation.get(from).copied().unwrap()))
    }

    pub fn get_direct(&self, key: K) -> Option<&V> {
        self.inner.get(key)
    }

    pub fn get_direct_mut(&mut self, key: K) -> Option<&mut V> {
        self.inner.get_mut(key)
    }

    /// Inserts a new detached node and returns its key.
    pub fn insert(&mut self, key: R, value: V) -> Option<K> {
        let to = self.inner.insert(value);
        self.relation.insert(key, to).and_then(|_| Some(to)).or_else(|| {
            self.inner.remove(to);
            None
        })
    }

    pub fn insert_with_key<F>(&mut self, key: R, f: F) -> Option<K>
    where
        F: FnOnce(K) -> V,
    {
        let to = self.inner.insert_with_key(f);
        self.relation.insert(key, to).and_then(|_| Some(to)).or_else(|| {
            self.inner.remove(to);
            None
        })
    }


    /// Removes a node **and all of its descendants** from the tree.
    ///
    /// Returns the inner value of the removed node, or `None` if it did not
    /// exist. The node is first detached from its parent so sibling links are
    /// kept consistent.
    pub fn remove(&mut self, key: R) -> Option<V> {
       self.relation.remove(key).and_then(|to| self.inner.remove(to))
    }

    /// Returns the number of nodes in the table.
    pub fn len(&self) -> usize { self.inner.len() }

    /// Returns `true` if the table contains no nodes.
    pub fn is_empty(&self) -> bool { self.inner.is_empty() }

    /// Iterates over all `(key, node)` pairs in insertion order.
    pub fn iter(&self) -> Iter<'_, K, V, R> { Iter::new(self) }

    /// Mutably iterates over all `(key, node)` pairs.
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V, R> { IterMut::new(self) }

    /// Iterates over all (from, to) keys in insertion order.
    pub fn keys(&self) -> Keys<'_, K, V, R> { Keys::new(self) }

    pub fn values(&self) -> crate::iter::Values<'_, K, V> {
        self.inner.values()
    }

    /// Removes all nodes from the tree, yielding them as `(key, node)` pairs.
    pub fn drain(&mut self) -> Drain<'_, K, V, R> {
        Drain::new(self)
    }

    /// Removes all nodes from the tree.
    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

impl<K: Id, V, R: Id> Index<K> for Table<K, V, R> {
    type Output = V;

    fn index(&self, index: K) -> &Self::Output {
        &self.inner[index]
    }
}

impl<K: Id, V, R: Id> IndexMut<K> for Table<K, V, R> {
    fn index_mut(&mut self, index: K) -> &mut Self::Output {
        &mut self.inner[index]
    }
}

pub struct TableNode<'a, K: 'a + Id, V: 'a, R: 'a + Id> {
    a: R,
    b: K,
    table: &'a Table<K, V, R>,
}

impl<'a, K: 'a + Id, V: 'a, R: 'a + Id> TableNode<'a, K, V, R> {
    pub fn new(from_id: R, to_id: K, table: &'a Table<K, V, R>) -> Self {
        Self { a: from_id, b: to_id, table }
    }
}

impl<'a, K: 'a + Id, V: 'a, R: 'a + Id> Deref for TableNode<'a, K, V, R> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.table[self.b]
    }
}

#[derive(Debug, Clone)]
pub struct Inner<'a, I, K: 'a + Id, V: 'a> {
    iter: I,
    inner: &'a slotmap::SlotMap<K, V>,
}

impl<'a, I: Iterator, K: 'a + Id, V: 'a> Iterator for Inner<'a, I, K, V> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

#[derive(Debug)]
pub struct InnerMut<'a, I, K: 'a + Id, V: 'a> {
    iter: I,
    inner: &'a mut slotmap::SlotMap<K, V>,
}

impl<'a, I: Iterator, K: 'a + Id, V: 'a> Iterator for InnerMut<'a, I, K, V> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

#[derive(Debug, Clone, Deref, DerefMut)]
#[repr(transparent)]
pub struct Iter<'a, K: 'a + Id, V: 'a, R: 'a + Id>(Inner<'a, slotmap::secondary::Iter<'a, R, K>, K, V>);

impl<'a, K: 'a + Id, V: 'a, R: 'a + Id> Iter<'a, K, V, R> {
    pub fn new(table: &'a Table<K, V, R>) -> Self {
        Self(Inner { iter: table.relation.iter(), inner: &table.inner })
    }
}

impl<'a, K: 'a + Id, V: 'a, R: 'a + Id> Iterator for Iter<'a, K, V, R> {
    type Item = ((R, K), &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(from, &to)| ((from, to), &self.inner[to]))
    }
}

#[derive(Debug, Deref, DerefMut)]
#[repr(transparent)]
pub struct IterMut<'a, K: 'a + Id, V: 'a, R: 'a + Id>(InnerMut<'a, slotmap::secondary::IterMut<'a, R, K>, K, V>);

impl<'a, K: 'a + Id, V: 'a, R: 'a + Id> IterMut<'a, K, V, R> {
    pub fn new(table: &'a mut Table<K, V, R>) -> Self {
        Self(InnerMut { iter: table.relation.iter_mut(), inner: &mut table.inner })
    }
}

impl<'a, K: 'a + Id, V: 'a, R: 'a + Id> Iterator for IterMut<'a, K, V, R> {
    type Item = ((R, K), &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(from, &mut to)| {
            // SRFETY: Each `to` key is unique in the secondary map,
            // so we never yield two `&mut V` to the same slot.
            let map = unsafe { &mut *(self.0.inner as *mut slotmap::SlotMap<K, V>) };
            ((from, to), &mut map[to])
        })
    }
}



#[derive(Debug, Deref, DerefMut)]
#[repr(transparent)]
pub struct Keys<'a, K: 'a + Id, V: 'a, R: 'a + Id>(Inner<'a, slotmap::secondary::Iter<'a, R, K>, K, V>);

impl<'a, K: 'a + Id, V: 'a, R: 'a + Id> Keys<'a, K, V, R> {
    pub fn new(table: &'a Table<K, V, R>) -> Self {
        Self(Inner { iter: table.relation.iter(), inner: &table.inner })
    }
}

impl<'a, K: 'a + Id, V: 'a, R: 'a + Id> Iterator for Keys<'a, K, V, R> {
    type Item = (R, K);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(from, &to)| (from, to))
    }
}


#[derive(Debug, Deref, DerefMut)]
#[repr(transparent)]
pub struct Drain<'a, K: 'a + Id, V: 'a, R: 'a + Id>(InnerMut<'a, slotmap::secondary::Drain<'a, R, K>, K, V>);

impl<'a, K: 'a + Id, V: 'a, R: 'a + Id> Drain<'a, K, V, R> {
    pub fn new(table: &'a mut Table<K, V, R>) -> Self {
        Self(InnerMut { iter: table.relation.drain(), inner: &mut table.inner })
    }
}

impl<'a, K: 'a + Id, V: 'a, R: 'a + Id> Iterator for Drain<'a, K, V, R> {
    type Item = ((R, K), V);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|(from, to)| ((from, to), self.inner.remove(to).unwrap()))
    }
}

impl<'a, R: Id, K: Id, V> Drop for Drain<'a, K, V, R> {
    fn drop(&mut self) {
        self.0.inner.drain().for_each(|_drop| {});
    }
}

#[cfg(test)]
mod tests {
    use crate::id;
    use super::*;

    id!(pub struct LayerId);
    id!(pub struct ElementId);
    struct Layer;

    pub type Layers = Table<ElementId, LayerId, Layer>;

}
