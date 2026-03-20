use std::fmt::Debug;
use crate::{Id, Node, iter::*, Error, At};
use crate::{NodeRef, NodeRefMut};
use derive_more::{Index, IndexMut, IntoIterator};
use std::iter::FusedIterator;
use std::ops::Deref;
use smallvec::SmallVec;

/// An arena-allocated tree with O(1) node access and linked-list child ordering.
///
/// Nodes are stored in a [`slotmap::SlotMap`] keyed by `K` (any type
/// implementing [`Id`]). Each node maintains parent, first/last child,
/// and prev/next sibling pointers so that structural queries and mutations
/// are constant-time.
///
/// # Insertion
///
/// Nodes can be inserted as detached roots, as children (first or last),
/// or as siblings relative to an existing node — see [`At`] for details.
///
/// # Removal
///
/// [`Tree::remove`] removes a node **and all of its descendants**.
/// [`Tree::detach`] unlinks a node from its parent without removing it.
///
/// # Iteration
///
/// Rich iterators are available for children, ancestors, descendants,
/// siblings, and full pre-/post-order traversal.
#[derive( Index, IndexMut, IntoIterator)]
#[into_iterator(owned, ref, ref_mut)]
#[repr(transparent)]
pub struct Tree<K: Id, V> {
    pub(crate) inner: slotmap::SlotMap<K, Node<K, V>>,
}

impl<K: Id, V> Tree<K, V> {
    /// Creates a new tree with a default capacity of 16 nodes.
    pub fn new() -> Self {
       Self::with_capacity(16)
    }

    /// Creates a new tree pre-allocated for `capacity` nodes.
    pub fn with_capacity(capacity: usize) -> Self {
        Self { inner: slotmap::SlotMap::with_capacity_and_key(capacity) }
    }

    /// Returns `true` if a node with the given key exists in the tree.
    pub fn contains(&self, key: K) -> bool {
        self.inner.contains_key(key)
    }

    /// Resolves an [`At`] position relative to `key`, returning the
    /// target node id if one exists.
    pub fn get_at(&self, key: K, at: At<K>) -> Option<K> {
        match at {
            At::Detached => Some(key),
            At::FirstChild(n) => self.next_sibling(n),
            At::Child(n) => self.parent(n),
            At::Before(n) => self.prev_sibling(n),
            At::After(n) => self.next_sibling(n),
        }
    }

    pub fn get(&self, key: K) -> Option<&Node<K, V>> {
        self.inner.get(key)
    }

    pub fn get_mut(&mut self, key: K) -> Option<&mut Node<K, V>> {
        self.inner.get_mut(key)
    }

    /// Inserts a new detached node and returns its key.
    pub fn insert(&mut self, value: V) -> K {
        self.inner.insert(Node::new(value))
    }

    /// Inserts a node at the specified position.
    ///
    /// # Panics
    ///
    /// Panics if the position references a missing node or would create a cycle.
    /// Use [`try_insert_at`](Self::try_insert_at) for a fallible version.
    pub fn insert_at(&mut self, value: V, at: At<K>) -> K {
        self.try_insert_at(value, at).unwrap()
    }

    /// Inserts a node at the specified position, returning an error on failure.
    pub fn try_insert_at(&mut self, value: V, at: At<K>) -> Result<K, Error<K>> {
        let id = self.insert(value);
        if let Err(e) = match at {
            At::Detached => Ok(()),
            At::FirstChild(parent) => {
                self.ensure_exists(parent)?;
                self.ensure_no_cycle(id, parent)?;
                self.link_as_first_child(parent, id);
                Ok(())
            }

            At::Child(parent) => {
                self.ensure_exists(parent)?;
                self.ensure_no_cycle(id, parent)?;
                self.link_as_last_child(parent, id);
                Ok(())
            }

            At::Before(id) => {
                self.ensure_exists(id)?;
                if id == id {
                   Ok(())
                }  else {
                    let parent = self.inner[id].parent();
                    if parent.is_null() {
                        Err(Error::NoParent(id))
                    } else {
                        self.ensure_no_cycle(id, parent)?;
                        self.link_before(id, id, parent);
                        Ok(())
                    }
                }
            }

            At::After(after) => {
                self.ensure_exists(after)?;
                if id == after {
                    Ok(())
                } else {
                    let parent = self.inner[after].parent();
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

    /// Inserts a node at `at` and reparents the given `children` under it.
    ///
    /// # Panics
    ///
    /// Panics on missing nodes or cycles. See
    /// [`try_insert_at_with_children`](Self::try_insert_at_with_children).
    pub fn insert_at_with_children(
        &mut self,
        value: V,
        children: &[K],
        at: At<K>,
    ) -> K {
        self.try_insert_at_with_children(value, children, at).unwrap()
    }

    /// Fallible version of [`insert_at_with_children`](Self::insert_at_with_children).
    pub fn try_insert_at_with_children(
        &mut self,
        value: V,
        children: &[K],
        at: At<K>,
    ) -> Result<K, Error<K>> {
        let id = self.try_insert_at(value, at)?;
        for &child in children {
            self.try_move_to(child, At::Child(id))?;
        }
        Ok(id)
    }

    // --- Mutation ----------------------------------------------------------

    /// Moves an existing node to a new position in the tree.
    ///
    /// The node is first detached from its current parent (if any), then
    /// re-linked at `to`. Panics on error — see [`try_move_to`](Self::try_move_to).
    pub fn move_to(&mut self, id: K, to: At<K>) {
        self.try_move_to(id, to).unwrap()
    }

    /// Fallible version of [`move_to`](Self::move_to).
    ///
    /// Returns an error if the source or target node is missing, the target
    /// sibling has no parent, or the move would create a cycle.
    pub fn try_move_to(&mut self, id: K, to: At<K>) -> Result<(), Error<K>> {
        self.ensure_exists(id)?;

        match to {
            At::Detached => self.try_detach(id),

            At::FirstChild(parent) => {
                self.ensure_exists(parent)?;
                self.ensure_no_cycle(id, parent)?;
                self.try_detach(id)?;
                self.link_as_first_child(parent, id);
                Ok(())
            }

            At::Child(parent) => {
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
                let parent = self.inner[before].parent();
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
                let parent = self.inner[after].parent();
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

    /// Detaches a node from its parent, making it a root.
    ///
    /// The node and its subtree remain in the tree but are no longer reachable
    /// from the former parent's child list. Panics if the node is missing —
    /// see [`try_detach`](Self::try_detach).
    pub fn detach(&mut self, id: K) {
        self.try_detach(id).unwrap()
    }

    /// Fallible version of [`detach`](Self::detach).
    pub fn try_detach(&mut self, id: K) -> Result<(), Error<K>> {
        self.ensure_exists(id)?;

        let parent = self.inner[id].parent();
        if parent.is_null() {
            return Ok(());
        }

        let prev = self.inner[id].previous_sibling();
        let next = self.inner[id].next_sibling();

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

    pub fn replace_children(&mut self, id: K, children: &[K]) {
        self.try_replace_children(id, children).unwrap()
    }

    pub fn try_replace_children(&mut self, id: K, children: &[K]) -> Result<(), Error<K>> {
        let current: Vec<_> = self.children(id).collect();

        for &child in &current {
            self.try_detach(child)?;
        }

        for &child in children {
            self.link_as_last_child(id, child);
        }

        Ok(())
    }

    /// Removes a node **and all of its descendants** from the tree.
    ///
    /// Returns the removed keys of the node and its descendants.
    /// The node is first detached from its parent so sibling links are
    /// kept consistent.
    pub fn remove(&mut self, id: K) -> Option<SmallVec<K, 4>> {
        if !self.contains(id) {
            return None;
        }
        

        // Detach root of subtree from parent first.
        let _ = self.detach(id);

        // Remove itself and all descendants
        let mut elements = self
            .descendants(id).collect::<SmallVec<_, _>>();
        
        elements.push(id);
            
        for &k in &elements {
            self.inner.remove(k);
        }

        Some(elements)
    }

    // --- Navigation --------------------------------------------------------

    /// Returns the parent of the given node, or `None` if it is a root.
    pub fn parent(&self, id: K) -> Option<K> {
        self.inner.get(id)?.parent().maybe()
    }

    /// Returns the first child of the given node, or `None` if it is a leaf.
    pub fn first_child(&self, id: K) -> Option<K> {
        self.inner.get(id)?.first_child().maybe()
    }

    /// Returns the last child of the given node, or `None` if it is a leaf.
    pub fn last_child(&self, id: K) -> Option<K> {
        self.inner.get(id)?.last_child().maybe()
    }

    /// Returns the next sibling, or `None` if this is the last child.
    pub fn next_sibling(&self, id: K) -> Option<K> {
        self.inner.get(id)?.next_sibling().maybe()
    }

    /// Returns the previous sibling, or `None` if this is the first child.
    pub fn prev_sibling(&self, id: K) -> Option<K> {
        self.inner.get(id)?.previous_sibling().maybe()
    }

    /// Returns `true` if the node has no children.
    pub fn is_leaf(&self, id: K) -> bool {
        self.inner.get(id).map_or(true, |n| n.first_child().is_none())
    }

    /// Returns `true` if the node has no parent.
    pub fn is_root(&self, id: K) -> bool {
        self.inner.get(id).map_or(false, |n| n.parent().is_none())
    }

    // --- Capacity & bulk operations ----------------------------------------

    /// Returns the number of nodes in the tree.
    pub fn len(&self) -> usize { self.inner.len() }

    /// Returns `true` if the tree contains no nodes.
    pub fn is_empty(&self) -> bool { self.inner.is_empty() }

    // --- Iteration ---------------------------------------------------------

    /// Iterates over all `(key, node)` pairs in insertion order.
    pub fn iter(&self) -> Iter<'_, K, Node<K, V>> { self.inner.iter() }

    /// Mutably iterates over all `(key, node)` pairs.
    pub fn iter_mut(&mut self) -> IterMut<'_, K, Node<K, V>> { self.inner.iter_mut() }

    /// Iterates over all keys in insertion order.
    pub fn keys(&self) -> Keys<'_, K, Node<K, V>> { self.inner.keys() }

    /// Iterates over all nodes (values) in insertion order.
    pub fn nodes(&self) -> Nodes<'_, K, Node<K, V>> { self.inner.values() }

    /// Mutably iterates over all nodes.
    pub fn nodes_mut(&mut self) -> NodesMut<'_, K, Node<K, V>> { self.inner.values_mut() }

    /// Returns a double-ended iterator over the direct children of a node.
    pub fn children(&self, id: K) -> Children<'_, K, V> { Children::new(self, id) }

    /// Returns an iterator over all descendants in pre-order (depth-first).
    pub fn descendants(&self, id: K) -> Descendants<'_, K, V> { Descendants::new(self, id) }

    /// Returns an iterator that walks upward through the node's ancestors.
    pub fn ancestors(&self, id: K) -> Ancestors<'_, K, V> { Ancestors::new(self, id) }

    /// Returns an iterator over this node, its preceding siblings, then ancestors.
    pub fn predecessors(&self, id: K) -> Predecessors<'_, K, V> { Predecessors::new(self, id) }

    /// Returns a double-ended iterator over the node and its following siblings.
    pub fn following_siblings(&self, id: K) -> FollowingSiblings<'_, K, V> { FollowingSiblings::new(self, id) }

    /// Returns a double-ended iterator over the node and its preceding siblings.
    pub fn preceding_siblings(&self, id: K) -> PrecedingSiblings<'_, K, V> { PrecedingSiblings::new(self, id) }

    /// Returns a pre-order traversal iterator yielding [`NodeEdge`] events.
    pub fn traverse(&self, id: K) -> Traverse<'_, K, V> { Traverse::new(self, id) }

    /// Returns a reverse (post-order) traversal iterator yielding [`NodeEdge`] events.
    pub fn reverse_traverse(&self, id: K) -> ReverseTraverse<'_, K, V> { ReverseTraverse::new(self, id) }

    /// Removes all nodes from the tree, yielding them as `(key, node)` pairs.
    pub fn drain(&mut self) -> Drain<K, Node<K, V>> {
        self.inner.drain()
    }

    /// Removes all nodes from the tree.
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


impl<K: Id, V: Debug> Debug for Tree<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
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
        pub tree: Tree<Id, &'static str>,
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
            let a = tree.insert_at("a", At::Child(root));
            let b = tree.insert_at("b", At::Child(root));
            let c = tree.insert_at("c", At::Child(root));

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
        tree.move_to(z, At::FirstChild(root));

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

        let kids: Vec<_> = tree.children(root).map(|id| tree[id].inner().clone()).collect();
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

        assert!(tree.get(b).is_some());
        assert_eq!(tree.get(b).unwrap(), &"b");
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
        tree.move_to(d, At::Child(b));

        tree.remove(b).unwrap();

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

        let inserted = tree.insert_at_with_children("root", &[a, b, c], At::Child(root));
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
        tree.move_to(b, At::Child(other));

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
            tree.move_to(d, At::Child(b));

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
            tree.move_to(d, At::Child(b));
            tree.move_to(e, At::Child(b));

            let names: Vec<_> = tree
                .descendants(root)
                .map(|id| tree.get(id).unwrap())
                .collect();

            assert_eq!(
                names,
                vec![
                    &"root",
                    &"a",
                    &"b",
                    &"d",
                    &"e",
                    &"c"
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
