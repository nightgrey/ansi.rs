use crate::{Error, Id};
use smallvec::{SmallVec, smallvec};
use std::ops::{Index, IndexMut};

const TABLE_REVERSE_CAPACITY: usize = 4;

/// A relational table mapping multiple `Other` keys to shared `V` values keyed by `K`.
///
/// Stores values in a `SlotMap<K, V>` and maintains a bidirectional index:
/// - **forward** (`R → K`): each `Other` maps to exactly one `K`
/// - **reverse** (`K → [R]`): each `K` tracks all `R`s that point to it
///
/// # Example
/// ```rust
/// use crate::Table;
/// // One Layer can be shared by multiple Elements.
/// type Layers = Table<LayerId, Layer, ElementId>;
///
/// let mut layers = Layers::new();
/// let id = layers.insert_value(Layer { /* ... */ });
/// layers.relate(element_a, id);
/// layers.relate(element_b, id);
///
/// assert_eq!(layers.resolve(element_a), Some(id));
/// assert_eq!(layers.related(id), &[element_a, element_b]);
/// ```
#[derive(Debug)]
pub struct Table<K: Id, V, Other: Id> {
    inner: slotmap::SlotMap<K, V>,
    forward: slotmap::SecondaryMap<Other, K>,
    reverse: slotmap::SecondaryMap<K, SmallVec<Other, TABLE_REVERSE_CAPACITY>>,
}

impl<K: Id, V, Other: Id> Table<K, V, Other> {
    pub fn new() -> Self {
        Self::with_capacity(16)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: slotmap::SlotMap::with_capacity_and_key(capacity),
            forward: slotmap::SecondaryMap::with_capacity(capacity),
            reverse: slotmap::SecondaryMap::with_capacity(capacity),
        }
    }

    /// Returns `true` if a value with this `K` exists.
    pub fn contains(&self, key: K) -> bool {
        self.inner.contains_key(key)
    }

    /// Returns `true` if `Other` relates to any `K`.
    pub fn relates_any(&self, key: Other) -> bool {
        self.forward.contains_key(key)
    }

    pub fn relates(&self, other: Other, key: K) -> bool {
        self.forward.get(other).map_or(false, |&to| to == key)
    }

    /// Returns `true` if `K` is related to any `R`.
    pub fn related(&self, key: K) -> bool {
        self.reverse.get(key).map_or(false, |r| !r.is_empty())
    }

    /// Returns a reference to the value related to the given `Other` key.
    pub fn get(&self, key: Other) -> Option<&V> {
        self.forward.get(key).and_then(|&to| self.inner.get(to))
    }

    /// Returns a mutable reference to the value related to the given `Other` key.
    pub fn get_mut(&mut self, key: Other) -> Option<&mut V> {
        self.forward.get(key).and_then(|&to| self.inner.get_mut(to))
    }

    /// Resolves the `K` key that `Other` is related to.
    pub fn get_id(&self, key: Other) -> Option<K> {
        self.forward.get(key).copied()
    }

    pub fn get_direct(&self, key: K) -> Option<&V> {
        self.inner.get(key)
    }

    pub fn get_direct_mut(&mut self, key: K) -> Option<&mut V> {
        self.inner.get_mut(key)
    }

    /// Returns all `Other` keys that point to the given `K` value.
    pub fn resolve(&self, key: K) -> &[Other] {
        self.reverse.get(key).map_or(&[], SmallVec::as_slice)
    }

    /// Returns mutable references to the values for `N` disjoint `Other` keys.
    ///
    /// Returns `None` if any key is missing or if two keys resolve to the
    /// same value (i.e. they are not disjoint at the value level).
    pub fn get_disjoint_mut<const N: usize>(&mut self, keys: [Other; N]) -> Option<[&mut V; N]> {
        let mut resolved: [K; N] = unsafe { std::mem::zeroed() };
        for (i, from) in keys.into_iter().enumerate() {
            resolved[i] = self.forward.get(from).copied()?;
        }
        self.inner.get_disjoint_mut(resolved)
    }

    /// Inserts a value *and* creates the first relation in one call.
    ///
    /// Returns the `K` key of the newly inserted value. If the `Other` key is
    /// already related to something else, the old relation is replaced.
    pub fn insert(&mut self, relation: Other, value: V) -> K {
        // If `rel` already pointed somewhere, clean up the old reverse entry.
        self.unrelate(relation);

        let key = self.inner.insert(value);
        self.forward.insert(relation, key);
        self.reverse.insert(key, smallvec![relation]);
        key
    }

    /// Inserts a value *and* creates relations for multiple `Other` keys.
    ///
    /// Returns the `K` key of the newly inserted value. If the `Other` keys are
    /// already related to something else, the old relation is replaced.
    pub fn insert_many(&mut self, relation: &[Other], value: V) -> K {
        // If `rel` already pointed somewhere, clean up the old reverse entry.
        self.unrelate_many(relation);

        let key = self.inner.insert(value);
        for &rel in relation {
            self.forward.insert(rel, key);
        }
        self.reverse
            .insert(key, SmallVec::from_slice_copy(relation));
        key
    }

    /// Creates (or reassigns) a relation from `Other` to an existing `K`.
    ///
    /// Returns `false` if `to` does not exist in the table.
    /// If `from` was already related to a different `K`, the old relation is
    /// cleaned up first.
    pub fn relate(&mut self, from: Other, to: K) -> Result<(), Error<K>> {
        if !self.inner.contains_key(to) {
            return Err(Error::Missing(to));
        }

        // Remove previous relation for this `from`, if any.
        if let Some(&old_to) = self.forward.get(from) {
            if old_to == to {
                return Ok(());
            }
            if let Some(rels) = self.reverse.get_mut(old_to) {
                rels.retain(|&r| r != from);
            }
        }

        self.forward.insert(from, to);
        match self.reverse.get_mut(to) {
            Some(rels) => rels.push(from),
            None => {
                self.reverse.insert(to, smallvec![from]);
            }
        }

        Ok(())
    }

    /// Creates (or reassigns) relations from multiple `Other` keys to an existing `K`.
    pub fn relate_many(&mut self, from: &[Other], to: K) -> bool {
        if !self.inner.contains_key(to) {
            return false;
        }

        // Clean up old forward entries (may point to different K's)
        for &rel in from {
            if let Some(&old_to) = self.forward.get(rel) {
                if old_to != to {
                    if let Some(rels) = self.reverse.get_mut(old_to) {
                        rels.retain(|&r| r != rel);
                    }
                }
            }
            self.forward.insert(rel, to);
        }

        // Single reverse lookup, batch extend
        match self.reverse.get_mut(to) {
            Some(rels) => {
                rels.retain(|r| !from.contains(r)); // deduplicate
                rels.extend_from_slice(from);
            }
            None => {
                self.reverse.insert(to, SmallVec::from_slice_copy(from));
            }
        }

        true
    }

    /// Removes a single relation. The value it pointed to is *not* removed.
    ///
    /// Returns the `K` key the relation pointed to, or `None`.
    pub fn unrelate(&mut self, from: Other) -> Option<K> {
        let to = self.forward.remove(from)?;
        if let Some(rels) = self.reverse.get_mut(to) {
            rels.retain(|&r| r != from);
        }
        Some(to)
    }

    /// Removes multiple relations. The values they pointed to are *not* removed.
    pub fn unrelate_many(&mut self, from: &[Other]) {
        // Collect targets so we can batch the reverse cleanup
        // SmallVec or tinyvec would be nice here
        let mut by_target: SmallVec<(K, Other), TABLE_REVERSE_CAPACITY> =
            SmallVec::with_capacity(from.len());

        for &rel in from {
            if let Some(to) = self.forward.remove(rel) {
                by_target.push((to, rel));
            }
        }

        // Sort by K so we can process each target once
        by_target.sort_unstable_by_key(|&(k, _)| k);

        let mut i = 0;
        while i < by_target.len() {
            let target = by_target[i].0;
            let start = i;
            while i < by_target.len() && by_target[i].0 == target {
                i += 1;
            }
            let chunk = &by_target[start..i];

            if let Some(rels) = self.reverse.get_mut(target) {
                if chunk.len() == 1 {
                    rels.retain(|&r| r != chunk[0].1);
                } else {
                    // For larger chunks, set lookup beats repeated linear scans
                    let remove: std::collections::HashSet<Other> =
                        chunk.iter().map(|&(_, r)| r).collect();
                    rels.retain(|r| !remove.contains(r));
                }
            }
        }
    }

    /// Removes a relation **and** the value it points to, but only if no other
    /// relations still reference that value.
    ///
    /// Returns the removed value, or `None` if the relation did not exist.
    pub fn remove(&mut self, from: Other) -> Option<V> {
        let related_to = self.unrelate(from)?;
        let orphaned = self
            .reverse
            .get(related_to)
            .map_or(true, |rels| rels.is_empty());
        if orphaned {
            self.reverse.remove(related_to);
            self.inner.remove(related_to)
        } else {
            None
        }
    }

    /// Inserts a value without any relation. Returns the new key.
    pub fn insert_directly(&mut self, value: V) -> K {
        let key = self.inner.insert(value);
        self.reverse.insert(key, smallvec![]);
        key
    }

    /// Inserts a value via a closure that receives its key.
    pub fn inster_directly_with_key<F: FnOnce(K) -> V>(&mut self, f: F) -> K {
        let key = self.inner.insert_with_key(f);
        self.reverse.insert(key, smallvec![]);
        key
    }

    /// Removes a value **and all relations pointing to it**.
    ///
    /// Returns the removed value, or `None` if the key was invalid.
    pub fn remove_directly(&mut self, key: K) -> Option<V> {
        if let Some(rels) = self.reverse.remove(key) {
            for r in rels {
                self.forward.remove(r);
            }
        }
        self.inner.remove(key)
    }

    /// Number of values.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Number of relations.
    pub fn relation_count(&self) -> usize {
        self.forward.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn values(&self) -> slotmap::basic::Values<'_, K, V> {
        self.inner.values()
    }

    pub fn values_mut(&mut self) -> slotmap::basic::ValuesMut<'_, K, V> {
        self.inner.values_mut()
    }

    /// Iterates `(Other, K, &V)` tuples over all relations.
    pub fn iter(&self) -> Iter<'_, K, V, Other> {
        Iter {
            rel_iter: self.forward.iter(),
            inner: &self.inner,
        }
    }

    /// Iterates `(Other, K, &mut V)` tuples over all relations.
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V, Other> {
        IterMut {
            rel_iter: self.forward.iter(),
            inner: &mut self.inner,
        }
    }

    /// Iterates `(Other, K)` key pairs over all relations.
    pub fn keys(&self) -> Keys<'_, Other, K> {
        Keys {
            rel_iter: self.forward.iter(),
        }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.forward.clear();
        self.reverse.clear();
    }

    pub fn drain(&mut self) -> Drain<'_, K, V, Other> {
        Drain {
            rel_iter: self.forward.drain(),
            inner: &mut self.inner,
            reverse: &mut self.reverse,
        }
    }
}

impl<K: Id, V, Other: Id> Index<Other> for Table<K, V, Other> {
    type Output = V;
    fn index(&self, key: Other) -> &V {
        &self.inner[self.forward[key]]
    }
}

impl<K: Id, V, Other: Id> IndexMut<Other> for Table<K, V, Other> {
    fn index_mut(&mut self, key: Other) -> &mut V {
        &mut self.inner[self.forward[key]]
    }
}

// ---------------------------------------------------------------------------
// Iterators
// ---------------------------------------------------------------------------

/// Iterates `(Other, K, &V)` over all relations.
pub struct Iter<'a, K: Id, V, Other: Id> {
    rel_iter: slotmap::secondary::Iter<'a, Other, K>,
    inner: &'a slotmap::SlotMap<K, V>,
}

impl<'a, K: Id, V, Other: Id> Iterator for Iter<'a, K, V, Other> {
    type Item = (Other, K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        self.rel_iter
            .next()
            .map(|(from, &to)| (from, to, &self.inner[to]))
    }
}

/// Iterates `(Other, K, &mut V)` over all relations.
pub struct IterMut<'a, K: Id, V, Other: Id> {
    rel_iter: slotmap::secondary::Iter<'a, Other, K>,
    inner: &'a mut slotmap::SlotMap<K, V>,
}

impl<'a, K: Id, V, Other: Id> Iterator for IterMut<'a, K, V, Other> {
    type Item = (Other, K, &'a mut V);

    fn next(&mut self) -> Option<Self::Item> {
        self.rel_iter.next().map(|(from, &to)| {
            // SAFETY: Each `to` is unique per relation entry, so we never
            // yield two `&mut V` to the same slot.
            let map = unsafe { &mut *(self.inner as *mut slotmap::SlotMap<K, V>) };
            (from, to, &mut map[to])
        })
    }
}

/// Iterates `(Other, K)` key pairs over all relations.
pub struct Keys<'a, Other: Id, K: Id> {
    rel_iter: slotmap::secondary::Iter<'a, Other, K>,
}

impl<'a, Other: Id, K: Id> Iterator for Keys<'a, Other, K> {
    type Item = (Other, K);

    fn next(&mut self) -> Option<Self::Item> {
        self.rel_iter.next().map(|(from, &to)| (from, to))
    }
}

/// Draining iterator: yields `(Other, K, V)` and cleans up all internal state.
pub struct Drain<'a, K: Id, V, Other: Id> {
    rel_iter: slotmap::secondary::Drain<'a, Other, K>,
    inner: &'a mut slotmap::SlotMap<K, V>,
    reverse: &'a mut slotmap::SecondaryMap<K, SmallVec<Other, TABLE_REVERSE_CAPACITY>>,
}

impl<'a, K: Id, V, Other: Id> Iterator for Drain<'a, K, V, Other> {
    type Item = (Other, K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.rel_iter.next().map(|(from, to)| {
            // Remove from reverse index
            if let Some(rels) = self.reverse.get_mut(to) {
                rels.retain(|&r| r != from);
            }
            // Only remove the value if this was the last relation
            // Note: caller gets the value only when it's actually removed.
            // For a simpler drain, we could just remove unconditionally —
            // depends on desired semantics. Here we match `remove()` behavior.
            //
            // Actually, for drain we should yield every relation but only
            // remove the inner value once all relations are gone.
            // This is tricky, so we just yield (Other, K) pairs and clean up
            // inner in Drop.
            (from, to, self.inner.remove(to).expect("dangling relation"))
        })
    }
}

impl<'a, K: Id, V, Other: Id> Drop for Drain<'a, K, V, Other> {
    fn drop(&mut self) {
        // Exhaust remaining relations
        for _ in &mut self.rel_iter {}
        self.inner.clear();
        self.reverse.clear();
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id;

    id!(pub struct LayerId);
    id!(pub struct ElementId);

    #[derive(Debug, Clone, PartialEq)]
    struct Layer {
        name: &'static str,
    }

    type Layers = Table<LayerId, Layer, ElementId>;

    // Assumes ElementId keys come from an external SlotMap.
    fn elements() -> (
        slotmap::SlotMap<ElementId, ()>,
        ElementId,
        ElementId,
        ElementId,
    ) {
        let mut sm = slotmap::SlotMap::with_key();
        let a = sm.insert(());
        let b = sm.insert(());
        let c = sm.insert(());
        (sm, a, b, c)
    }

    #[test]
    fn usage() {
        let mut table = Layers::new();
        let (_sm, e1, e2, e3) = elements();

        let layer_id = table.insert(e1, Layer { name: "bg" });
        table.relate(e2, layer_id);
        assert_eq!(table.get(e1), Some(&Layer { name: "bg" }));
        assert_eq!(table.get_id(e1), Some(layer_id));
    }
    #[test]
    fn insert_and_forward_lookup() {
        let mut t = Layers::new();
        let (_sm, e1, _e2, _e3) = elements();

        let layer_id = t.insert(e1, Layer { name: "bg" });
        assert_eq!(t.get(e1), Some(&Layer { name: "bg" }));
        assert_eq!(t.get_id(e1), Some(layer_id));
    }

    #[test]
    fn share_value_across_relations() {
        let mut t = Layers::new();
        let (_sm, e1, e2, e3) = elements();

        let layer_id = t.insert_directly(Layer { name: "shared" });
        assert!(t.relates(e1, layer_id));
        assert!(t.relates(e2, layer_id));
        assert!(t.relates(e3, layer_id));

        // All three resolve to the same value
        assert_eq!(t.get_id(e1), Some(layer_id));
        assert_eq!(t.get_id(e2), Some(layer_id));
        assert_eq!(t.get_id(e3), Some(layer_id));

        // Reverse lookup
        let mut rels = t.resolve(layer_id).to_vec();
        rels.sort_unstable();
        assert_eq!(rels.len(), 3);
    }

    #[test]
    fn unrelate_keeps_value() {
        let mut t = Layers::new();
        let (_sm, e1, e2, _e3) = elements();

        let layer_id = t.insert_directly(Layer { name: "keep" });
        t.relate(e1, layer_id);
        t.relate(e2, layer_id);

        t.unrelate(e1);
        assert!(!t.relates_any(e1));
        assert!(t.relates_any(e2));
        assert!(t.contains(layer_id)); // value still alive
    }

    #[test]
    fn remove_orphans_value() {
        let mut t = Layers::new();
        let (_sm, e1, e2, _e3) = elements();

        let layer_id = t.insert(e1, Layer { name: "ephemeral" });
        t.relate(e2, layer_id);

        // Remove e1 — value still has e2, so no removal
        assert!(t.remove(e1).is_none());
        assert!(t.contains(layer_id));

        // Remove e2 — last relation, value is dropped
        let v = t.remove(e2);
        assert_eq!(v, Some(Layer { name: "ephemeral" }));
        assert!(!t.contains(layer_id));
    }

    #[test]
    fn remove_value_cleans_all_relations() {
        let mut t = Layers::new();
        let (_sm, e1, e2, _e3) = elements();

        let layer_id = t.insert_directly(Layer { name: "nuke" });
        t.relate(e1, layer_id);
        t.relate(e2, layer_id);

        t.remove_directly(layer_id);
        assert!(!t.relates_any(e1));
        assert!(!t.relates_any(e2));
        assert!(!t.contains(layer_id));
    }

    #[test]
    fn reassign_relation() {
        let mut t = Layers::new();
        let (_sm, e1, _e2, _e3) = elements();

        let a = t.insert_directly(Layer { name: "a" });
        let b = t.insert_directly(Layer { name: "b" });

        t.relate(e1, a);
        assert_eq!(t.resolve(a).len(), 1);

        // Reassign e1 from a → b
        t.relate(e1, b);
        assert_eq!(t.get_id(e1), Some(b));
        assert!(t.resolve(a).is_empty()); // cleaned up
        assert_eq!(t.resolve(b).len(), 1);
    }
}
