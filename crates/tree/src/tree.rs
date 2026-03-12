use super::{Id, Node, iter::*, Error, At};
use super::{NodeRef, NodeRefMut};
use derive_more::{Deref, DerefMut, Index, IndexMut, IntoIterator};
use std::iter::FusedIterator;
use std::ops::Deref;

#[derive(Debug, Index, IndexMut, IntoIterator)]
#[into_iterator(owned, ref, ref_mut)]
#[repr(transparent)]
pub struct Tree<K: Id, V> {
    inner: slotmap::SlotMap<K, Node<K, V>>,
}

impl<K: Id, V> Tree<K, V> {
    pub fn new() -> Self {
       Self::with_capacity(16)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { inner: slotmap::SlotMap::with_capacity_and_key(capacity) }
    }

    pub fn contains(&self, key: K) -> bool {
        self.inner.contains_key(key)
    }

    pub fn get(&self, key: K) -> Option<&Node<K, V>> {
        self.inner.get(key)
    }

    pub fn get_mut(&mut self, key: K) -> Option<&mut Node<K, V>> {
        self.inner.get_mut(key)
    }

    pub fn get_ref(&self, key: K) -> Option<NodeRef<K, V>> {
        self.inner.get(key).map(|_| NodeRef::new(key, self))
    }

    pub fn get_ref_mut(&mut self, key: K) -> Option<NodeRefMut<K, V>> {
        if self.inner.get(key).is_none() {
            return None;
        }
        Some(NodeRefMut::new(key, self))
    }

    pub fn insert(&mut self, value: V) -> K {
        self.inner.insert(Node::new(value))
    }

    pub fn insert_at(&mut self, value: V, at: At<K>) -> K {
        self.try_insert_at(value, at).unwrap()
    }

    pub fn try_insert_at(&mut self, value: V, at: At<K>) -> Result<K, Error<K>> {
        let id = self.insert(value);
        if let Err(e) = match at {
            At::Detached => Ok(()),
            At::Prepend(parent) => {
                self.ensure_exists(parent)?;
                self.ensure_no_cycle(id, parent)?;
                self.link_as_first_child(parent, id);
                Ok(())
            }

            At::Append(parent) | At::Child(parent) => {
                self.ensure_exists(parent)?;
                self.ensure_no_cycle(id, parent)?;
                self.link_as_last_child(parent, id);
                Ok(())
            }

            At::Before(before) => {
                self.ensure_exists(before)?;
                if id == before {
                   Ok(())
                }  else {
                    let parent = self.inner[before].parent;
                    if parent.is_null() {
                        Err(Error::NoParent(before))
                    } else {
                        self.ensure_no_cycle(id, parent)?;
                        self.link_before(id, before, parent);
                        Ok(())
                    }
                }
            }

            At::After(after) => {
                self.ensure_exists(after)?;
                if id == after {
                    Ok(())
                } else {
                    let parent = self.inner[after].parent;
                    if parent.is_null() {
                        Err(Error::NoParent(after))
                    } else {
                        self.ensure_no_cycle(id, parent)?;
                        self.link_after(id, after, parent);
                        Ok(())
                    }
                }
            }
        } {
            let _ = self.inner.remove(id);
            return Err(e);
        }

        Ok(id)
    }

    pub fn insert_at_with_children(
        &mut self,
        value: V,
        children: &[K],
        at: At<K>,
    ) -> K {
        self.try_insert_at_with_children(value, children, at).unwrap()
    }

    pub fn try_insert_at_with_children(
        &mut self,
        value: V,
        children: &[K],
        at: At<K>,
    ) -> Result<K, Error<K>> {
        let id = self.try_insert_at(value, at)?;
        for &child in children {
            self.try_move_to(child, At::Append(id))?;
        }
        Ok(id)
    }

    // --- Mutation ----------------------------------------------------------

    pub fn move_to(&mut self, id: K, to: At<K>) {
        self.try_move_to(id, to).unwrap()
    }

    pub fn try_move_to(&mut self, id: K, to: At<K>) -> Result<(), Error<K>> {
        self.ensure_exists(id)?;

        match to {
            At::Detached => self.try_detach(id),

            At::Prepend(parent) => {
                self.ensure_exists(parent)?;
                self.ensure_no_cycle(id, parent)?;
                self.try_detach(id)?;
                self.link_as_first_child(parent, id);
                Ok(())
            }

            At::Append(parent) | At::Child(parent) => {
                self.ensure_exists(parent)?;
                self.ensure_no_cycle(id, parent)?;
                self.try_detach(id)?;
                self.link_as_last_child(parent, id);
                Ok(())
            }

            At::Before(before) => {
                self.ensure_exists(before)?;
                if id == before {
                    return Ok(());
                }
                let parent = self.inner[before].parent;
                if parent.is_null() {
                    return Err(Error::NoParent(before));
                }
                self.ensure_no_cycle(id, parent)?;
                self.try_detach(id)?;
                self.link_before(id, before, parent);
                Ok(())
            }

            At::After(after) => {
                self.ensure_exists(after)?;
                if id == after {
                    return Ok(());
                }
                let parent = self.inner[after].parent;
                if parent.is_null() {
                    return Err(Error::NoParent(after));
                }
                self.ensure_no_cycle(id, parent)?;
                self.try_detach(id)?;
                self.link_after(id, after, parent);
                Ok(())
            }
        }
    }

    pub fn detach(&mut self, id: K) {
        self.try_detach(id).unwrap()
    }

    pub fn try_detach(&mut self, id: K) -> Result<(), Error<K>> {
        self.ensure_exists(id)?;

        let parent = self.inner[id].parent;
        if parent.is_null() {
            return Ok(());
        }

        let prev = self.inner[id].previous_sibling;
        let next = self.inner[id].next_sibling;

        if prev.is_null() {
            self.inner[parent].first_child = next;
        } else {
            self.inner[prev].next_sibling = next;
        }

        if next.is_null() {
            self.inner[parent].last_child = prev;
        } else {
            self.inner[next].previous_sibling = prev;
        }

        let n = &mut self.inner[id];
        n.parent = K::null();
        n.previous_sibling = K::null();
        n.next_sibling = K::null();

        Ok(())
    }

    /// Remove node and its descendants.
    pub fn remove(&mut self, id: K) -> Option<V> {
        if !self.contains(id) {
            return None;
        }

        // Detach root of subtree from parent first.
        let _ = self.detach(id);

        // Remove descendants excluding `id`.
        let to_remove: Vec<_> = self
            .descendants(id)
            .filter(|&k| k != id)
            .collect();

        for k in to_remove {
            let _ = self.inner.remove(k);
        }

        self.inner.remove(id).map(|n| n.inner)
    }

    // --- read-only helpers -------------------------------------------------

    pub fn parent(&self, id: K) -> Option<K> {
        self.inner.get(id)?.parent.maybe()
    }

    pub fn first_child(&self, id: K) -> Option<K> {
        self.inner.get(id)?.first_child.maybe()
    }

    pub fn last_child(&self, id: K) -> Option<K> {
        self.inner.get(id)?.last_child.maybe()
    }

    pub fn next_sibling(&self, id: K) -> Option<K> {
        self.inner.get(id)?.next_sibling.maybe()
    }

    pub fn prev_sibling(&self, id: K) -> Option<K> {
        self.inner.get(id)?.previous_sibling.maybe()
    }

    pub fn is_leaf(&self, id: K) -> bool {
        self.inner.get(id).map_or(true, |n| n.first_child.is_none())
    }

    pub fn is_root(&self, id: K) -> bool {
        self.inner.get(id).map_or(false, |n| n.parent.is_none())
    }

    pub fn len(&self) -> usize { self.inner.len() }
    pub fn is_empty(&self) -> bool { self.inner.is_empty() }

    pub fn iter(&self) -> Iter<'_, K, V> { self.inner.iter() }
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> { self.inner.iter_mut() }
    pub fn keys(&self) -> Keys<'_, K, V> { self.inner.keys() }
    pub fn nodes(&self) -> Nodes<'_, K, V> { self.inner.values() }
    pub fn nodes_mut(&mut self) -> NodesMut<'_, K, V> { self.inner.values_mut() }
    pub fn children(&self, id: K) -> Children<'_, K, V> { Children::new(self, id) }
    pub fn descendants(&self, id: K) -> Descendants<'_, K, V> { Descendants::new(self, id) }
    pub fn ancestors(&self, id: K) -> Ancestors<'_, K, V> { Ancestors::new(self, id) }
    pub fn predecessors(&self, id: K) -> Predecessors<'_, K, V> { Predecessors::new(self, id) }
    pub fn following_siblings(&self, id: K) -> FollowingSiblings<'_, K, V> { FollowingSiblings::new(self, id) }
    pub fn preceding_siblings(&self, id: K) -> PrecedingSiblings<'_, K, V> { PrecedingSiblings::new(self, id) }
    pub fn traverse(&self, id: K) -> Traverse<'_, K, V> { Traverse::new(self, id) }
    pub fn reverse_traverse(&self, id: K) -> ReverseTraverse<'_, K, V> { ReverseTraverse::new(self, id) }
    pub fn drain(&mut self) -> Drain<K, V> {
        self.inner.drain()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    // --- internals ---------------------------------------------------------

    fn ensure_exists(&self, k: K) -> Result<(), Error<K>> {
        self.contains(k).ok_or_else(|| Error::Missing(k))
    }

    fn ensure_no_cycle(&self, node: K, target_parent: K) -> Result<(), Error<K>> {
        if node == target_parent || self.is_ancestor(node, target_parent) {
            Err(Error::Cycle { node, target: target_parent })
        } else {
            Ok(())
        }
    }

    fn is_ancestor(&self, maybe_ancestor: K, mut node: K) -> bool {
        while let Some(p) = self.parent(node) {
            if p == maybe_ancestor {
                return true;
            }
            node = p;
        }
        false
    }

    fn link_as_last_child(&mut self, parent: K, child: K) {
        let old_tail = self.inner[parent].last_child;

        if old_tail.is_null() {
            self.inner[parent].first_child = child;
        } else {
            self.inner[old_tail].next_sibling = child;
        }

        self.inner[parent].last_child = child;

        let n = &mut self.inner[child];
        n.parent = parent;
        n.previous_sibling = old_tail;
        n.next_sibling = K::null();
    }

    fn link_as_first_child(&mut self, parent: K, child: K) {
        let old_head = self.inner[parent].first_child;

        if old_head.is_null() {
            self.inner[parent].last_child = child;
        } else {
            self.inner[old_head].previous_sibling = child;
        }

        self.inner[parent].first_child = child;

        let n = &mut self.inner[child];
        n.parent = parent;
        n.previous_sibling = K::null();
        n.next_sibling = old_head;
    }

    fn link_before(&mut self, node: K, before: K, parent: K) {
        let prev = self.inner[before].previous_sibling;

        if prev.is_null() {
            self.inner[parent].first_child = node;
        } else {
            self.inner[prev].next_sibling = node;
        }

        self.inner[before].previous_sibling = node;

        let n = &mut self.inner[node];
        n.parent = parent;
        n.previous_sibling = prev;
        n.next_sibling = before;
    }

    fn link_after(&mut self, node: K, after: K, parent: K) {
        let next = self.inner[after].next_sibling;

        if next.is_null() {
            self.inner[parent].last_child = node;
        } else {
            self.inner[next].previous_sibling = node;
        }

        self.inner[after].next_sibling = node;

        let n = &mut self.inner[node];
        n.parent = parent;
        n.previous_sibling = after;
        n.next_sibling = next;
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    // Setup
    crate::id!(pub struct Id);
    #[derive(Debug, derive_more::Deref, derive_more::DerefMut, IndexMut, Index)]
    pub struct Test {
        pub root: Id,
        pub a: Id,
        pub b: Id,
        pub c: Id,
        #[deref]
        #[deref_mut]
        #[index]
        #[index_mut]
        pub tree: super::Tree<Id, &'static str>,
    }

    impl Test {
        pub fn empty() -> Self {
            Self {
                root: Id::null(),
                a: Id::null(),
                b: Id::null(),
                c: Id::null(),
                tree: Tree::new(),
            }
        }
        pub fn default() -> Self {
            let mut tree = Tree::new();
            let root = tree.insert("root");
            let a = tree.insert_at("a", At::Append(root));
            let b = tree.insert_at("b", At::Append(root));
            let c = tree.insert_at("c", At::Append(root));

            Self {
                root,
                a,
                b,
                c,
                tree,
            }
        }
    }

    #[test]
    fn append_child_relationships() {
        let Test {
            root,
            a,
            b,
            c,
            tree,
        } = Test::default();

        assert_eq!(tree.parent(a), Some(root));
        assert_eq!(tree.parent(b), Some(root));
        assert_eq!(tree.parent(c), Some(root));
        assert_eq!(tree.parent(root), None);

        assert_eq!(tree.first_child(root), Some(a));
        assert_eq!(tree.last_child(root), Some(c));

        assert_eq!(tree.next_sibling(a), Some(b));
        assert_eq!(tree.next_sibling(b), Some(c));
        assert_eq!(tree.next_sibling(c), None);

        assert_eq!(tree.prev_sibling(c), Some(b));
        assert_eq!(tree.prev_sibling(b), Some(a));
        assert_eq!(tree.prev_sibling(a), None);
    }

    #[test]
    fn prepend_child() {
        let Test {
            root,
            a,
            b,
            c,
            mut tree,
        } = Test::default();

        let z = tree.insert("z");
        tree.move_to(z, At::Prepend(root));

        assert_eq!(tree.first_child(root), Some(z));
        assert_eq!(tree.next_sibling(z), Some(a));
        assert_eq!(tree.prev_sibling(a), Some(z));
    }

    #[test]
    fn insert_before_after() {
        let Test {
            root,
            a,
            b,
            c,
            mut tree,
        } = Test::default();

        let x = tree.insert_at("x", At::Before(b));
        let y = tree.insert_at("y", At::After(b));

        let kids: Vec<_> = tree.children(root).map(|id| tree[id].inner).collect();
        assert_eq!(kids, vec!["a", "x", "b", "y", "c"]);
    }

    #[test]
    fn detach_middle() {
        let Test {
            root,
            a,
            b,
            c,
            mut tree,
        } = Test::default();

        tree.try_detach(b).unwrap();

        assert_eq!(tree.parent(b), None);
        assert_eq!(tree.next_sibling(a), Some(c));
        assert_eq!(tree.prev_sibling(c), Some(a));

        let kids: Vec<_> = tree.children(root).collect();
        assert_eq!(kids, vec![a, c]);

        // b still exists
        assert_eq!(tree.contains(b), true);

        assert!(tree.get_ref(b).is_some());
        assert_eq!(tree.get_ref(b).unwrap(), "b");
    }

    #[test]
    fn detach_first_and_last() {
        let Test {
            root,
            a,
            b,
            c,
            mut tree,
        } = Test::default();

        tree.detach(a);
        assert_eq!(tree.first_child(root), Some(b));
        assert_eq!(tree.prev_sibling(b), None);

        tree.detach(c);
        assert_eq!(tree.last_child(root), Some(b));
        assert_eq!(tree.next_sibling(b), None);
    }

    #[test]
    fn remove_subtree() {
        //       root
        //      / | \
        //    a   b   c
        //        |
        //        d
        let Test {
            root,
            a,
            b,
            c,
            mut tree,
        } = Test::default();
        let d = tree.insert("d");
        tree.move_to(d, At::Append(b));

        tree.remove(b);

        assert!(!tree.contains(b));
        assert!(!tree.contains(d));

        let kids: Vec<_> = tree.children(root).collect();
        assert_eq!(kids, vec![a, c]);
        assert_eq!(tree.next_sibling(a), Some(c));
    }

    #[test]
    fn insert_with_children() {
        let Test {
            root,
            a,
            b,
            c,
            mut tree,
        } = Test::default();

        let inserted = tree.insert_at_with_children("root", &[a, b, c], At::Append(root));
        let kids: Vec<_> = tree.children(inserted).collect();
        assert_eq!(kids, vec![a, b, c]);
        assert_eq!(tree.parent(b), Some(inserted));
    }

    #[test]
    fn reparent_child() {
        let Test {
            root,
            a,
            b,
            c,
            mut tree,
        } = Test::default();
        let other = tree.insert("other");

        // Move b under `other`
        tree.move_to(b, At::Append(other));

        assert_eq!(tree.parent(b), Some(other));
        let root_kids: Vec<_> = tree.children(root).collect();
        assert_eq!(root_kids, vec![a, c]);
        assert_eq!(tree.next_sibling(a), Some(c));
    }

    #[test]
    fn empty_tree_and_leaf() {
        let mut tree = Test::empty();
        assert!(tree.is_empty());

        let leaf = tree.insert("leaf");
        assert!(tree.is_leaf(leaf));
        assert_eq!(tree.first_child(leaf), None);
        assert_eq!(tree.last_child(leaf), None);
        assert_eq!(tree.children(leaf).next(), None);
    }

    mod iter {
        use super::*;

        #[test]
        fn children_iter() {
            let Test {
                root,
                a,
                b,
                c,
                mut tree,
            } = Test::default();

            let mut children = tree.children(root);
            assert_eq!(children.next(), Some(a));
            assert_eq!(children.next_back(), Some(c));

            let reversed: Vec<_> = tree.children(root).rev().collect();
            assert_eq!(reversed, vec![c, b, a]);
        }

        #[test]
        fn ancestors_iter() {
            let Test {
                root,
                a,
                b,
                c,
                mut tree,
            } = Test::default();

            let d = tree.insert("d");
            tree.move_to(d, At::Append(b));

            let ancs: Vec<_> = tree.ancestors(d).collect();
            assert_eq!(ancs, vec![b, root]);
        }

        #[test]
        fn descendants() {
            //       root
            //      / | \
            //    a   b   c
            //       / \
            //      d   e
            let Test {
                root,
                a,
                b,
                c,
                mut tree,
            } = Test::default();

            let d = tree.insert("d");
            let e = tree.insert("e");
            tree.move_to(d, At::Append(b));
            tree.move_to(e, At::Append(b));

            let names: Vec<_> = tree
                .descendants(root)
                .map(|id| tree.get_ref(id).unwrap())
                .collect();

            assert_eq!(
                names,
                vec![
                    "root",
                    "a",
                    "b",
                    "d",
                    "e",
                    "c"
                ]
            );
        }

        #[test]
        fn following_siblings() {
            let Test {
                root,
                a,
                b,
                c,
                mut tree,
            } = Test::default();

            let fwd: Vec<_> = tree.following_siblings(b).collect();
            assert_eq!(fwd, vec![b, c]);
        }

        #[test]
        fn preceding_siblings() {
            let Test {
                root,
                a,
                b,
                c,
                mut tree,
            } = Test::default();

            let bwd: Vec<_> = tree.preceding_siblings(b).collect();
            assert_eq!(bwd, vec![b, a]);
        }

        #[test]
        fn traverse() {
            let Test {
                root,
                a,
                b,
                c,
                mut tree,
            } = Test::default();
            let mut iter = tree.traverse(root);

            assert_eq!(iter.next().unwrap(), NodeEdge::Start(root));
            assert_eq!(iter.next().unwrap(), NodeEdge::Start(a));
            assert_eq!(iter.next().unwrap(), NodeEdge::End(a));
            assert_eq!(iter.next().unwrap(), NodeEdge::Start(b));
            assert_eq!(iter.next().unwrap(), NodeEdge::End(b));
            assert_eq!(iter.next().unwrap(), NodeEdge::Start(c));
            assert_eq!(iter.next().unwrap(), NodeEdge::End(c));
            assert_eq!(iter.next().unwrap(), NodeEdge::End(root));
            assert_eq!(iter.next(), None);
        }

        #[test]
        fn reverse_traverse() {
            let Test {
                root,
                a,
                b,
                c,
                mut tree,
            } = Test::default();
            let mut iter = tree.reverse_traverse(root);

            assert_eq!(iter.next().unwrap(), NodeEdge::End(root));
            assert_eq!(iter.next().unwrap(), NodeEdge::End(c));
            assert_eq!(iter.next().unwrap(), NodeEdge::Start(c));
            assert_eq!(iter.next().unwrap(), NodeEdge::End(b));
            assert_eq!(iter.next().unwrap(), NodeEdge::Start(b));
            assert_eq!(iter.next().unwrap(), NodeEdge::End(a));
            assert_eq!(iter.next().unwrap(), NodeEdge::Start(a));
            assert_eq!(iter.next().unwrap(), NodeEdge::Start(root));
            assert_eq!(iter.next(), None);
        }
    }
}
