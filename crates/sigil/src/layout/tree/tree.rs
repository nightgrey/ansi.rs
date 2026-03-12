use super::{TreeId, TreeNode, iter::*};
use super::{TreeNodeRef, TreeNodeRefMut};
use derive_more::{Deref, DerefMut, Index, IndexMut, IntoIterator};
use std::iter::FusedIterator;
use std::ops::Deref;

use thiserror::Error;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum At<K> {
    Detached,

    FirstChild(K),
    LastChild(K),

    Prepend(K),
    Append(K),

}

#[derive(Error, Debug)]
pub enum TreeError<K> {
    #[error("Node {0} does not exist")]
    Missing(K),
    #[error("Reference node {0} has no parent")]
    NoParent(K),
    #[error("Cycle detected: node {node} would be its own ancestor")]
    Cycle { node: K, target: K },
    #[error("Root operation forbidden")]
    RootOperationForbidden,

}

type Inner<K, V> = slotmap::SlotMap<K, V>;

#[derive(Debug, Index, IndexMut, IntoIterator)]
#[into_iterator(owned, ref, ref_mut)]
#[repr(transparent)]
pub struct Tree<K: TreeId, V> {
    inner: Inner<K, TreeNode<K, V>>,
}

impl<K: TreeId, V> Tree<K, V> {
    pub fn new() -> Self {
        Self { inner: Inner::with_capacity_and_key(16) }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { inner: Inner::with_capacity_and_key(capacity) }
    }

    pub fn contains(&self, key: K) -> bool {
        self.inner.contains_key(key)
    }

    pub fn get(&self, key: K) -> Option<TreeNodeRef<K, V>> {
        self.inner.get(key).map(|_| TreeNodeRef::new(key, self))
    }

    pub fn get_mut(&mut self, key: K) -> Option<TreeNodeRefMut<K, V>> {
        if self.inner.get(key).is_none() {
            return None;
        }
        Some(TreeNodeRefMut::new(key, self))
    }

    pub fn get_node(&self, key: K) -> Option<&TreeNode<K, V>> {
        self.inner.get(key)
    }

    pub fn get_node_mut(&mut self, key: K) -> Option<&mut TreeNode<K, V>> {
        self.inner.get_mut(key)
    }

    // --- creation ----------------------------------------------------------

    pub fn insert_detached(&mut self, value: V) -> K {
        self.inner.insert(TreeNode::new(value))
    }

    pub fn insert_detached_with_key(&mut self, f: impl FnOnce(K) -> V) -> K {
        self.inner.insert_with_key(|k| TreeNode::new(f(k)))
    }

    pub fn try_insert_detached_with_key<E>(
        &mut self,
        f: impl FnOnce(K) -> Result<V, E>,
    ) -> Result<K, E> {
        self.inner.try_insert_with_key(|k| f(k).map(TreeNode::new))
    }

    pub fn insert_at(&mut self, value: V, at: At<K>) -> Result<K, TreeError<K>> {
        let id = self.insert_detached(value);
        if let Err(e) = self.move_to(id, at) {
            let _ = self.inner.remove(id); // rollback
            return Err(e);
        }
        Ok(id)
    }

    pub fn insert_at_with_key(
        &mut self,
        pos: At<K>,
        f: impl FnOnce(K) -> V,
    ) -> Result<K, TreeError<K>> {
        let id = self.insert_detached_with_key(f);
        if let Err(e) = self.move_to(id, pos) {
            let _ = self.inner.remove(id); // rollback
            return Err(e);
        }
        Ok(id)
    }

    pub fn insert_with_children(
        &mut self,
        value: V,
        children: &[K],
        pos: At<K>,
    ) -> Result<K, TreeError<K>> {
        let id = self.insert_at(value, pos)?;
        self.append_children(id, children)?;
        Ok(id)
    }

    // ergonomic helpers
    pub fn append_new_child(&mut self, parent: K, value: V) -> Result<K, TreeError<K>> {
        self.insert_at(value, At::LastChild(parent))
    }

    pub fn prepend_new_child(&mut self, parent: K, value: V) -> Result<K, TreeError<K>> {
        self.insert_at(value, At::FirstChild(parent))
    }

    pub fn insert_new_before(&mut self, before: K, value: V) -> Result<K, TreeError<K>> {
        self.insert_at(value, At::Prepend(before))
    }

    pub fn insert_new_after(&mut self, after: K, value: V) -> Result<K, TreeError<K>> {
        self.insert_at(value, At::Append(after))
    }

    // --- mutation ----------------------------------------------------------

    pub fn move_to(&mut self, node: K, pos: At<K>) -> Result<(), TreeError<K>> {
        self.ensure_exists(node)?;

        match pos {
            At::Detached => self.detach(node),

            At::FirstChild(parent) => {
                self.ensure_exists(parent)?;
                self.ensure_no_cycle(node, parent)?;
                self.detach(node)?;
                self.link_as_first_child(parent, node);
                Ok(())
            }

            At::LastChild(parent) => {
                self.ensure_exists(parent)?;
                self.ensure_no_cycle(node, parent)?;
                self.detach(node)?;
                self.link_as_last_child(parent, node);
                Ok(())
            }

            At::Prepend(before) => {
                self.ensure_exists(before)?;
                if node == before {
                    return Ok(());
                }
                let parent = self.inner[before].parent;
                if parent.is_null() {
                    return Err(TreeError::NoParent(before));
                }
                self.ensure_no_cycle(node, parent)?;
                self.detach(node)?;
                self.link_before(node, before, parent);
                Ok(())
            }

            At::Append(after) => {
                self.ensure_exists(after)?;
                if node == after {
                    return Ok(());
                }
                let parent = self.inner[after].parent;
                if parent.is_null() {
                    return Err(TreeError::NoParent(after));
                }
                self.ensure_no_cycle(node, parent)?;
                self.detach(node)?;
                self.link_after(node, after, parent);
                Ok(())
            }
        }
    }

    pub fn detach(&mut self, node: K) -> Result<(), TreeError<K>> {
        self.ensure_exists(node)?;

        let parent = self.inner[node].parent;
        if parent.is_null() {
            return Ok(());
        }

        let prev = self.inner[node].previous_sibling;
        let next = self.inner[node].next_sibling;

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

        let n = &mut self.inner[node];
        n.parent = K::null();
        n.previous_sibling = K::null();
        n.next_sibling = K::null();

        Ok(())
    }

    pub fn append_child(&mut self, parent: K, child: K) -> Result<(), TreeError<K>> {
        self.move_to(child, At::LastChild(parent))
    }

    pub fn prepend_child(&mut self, parent: K, child: K) -> Result<(), TreeError<K>> {
        self.move_to(child, At::FirstChild(parent))
    }

    pub fn insert_before(&mut self, node: K, before: K) -> Result<(), TreeError<K>> {
        self.move_to(node, At::Prepend(before))
    }

    pub fn insert_after(&mut self, node: K, after: K) -> Result<(), TreeError<K>> {
        self.move_to(node, At::Append(after))
    }

    pub fn append_children(&mut self, parent: K, children: &[K]) -> Result<(), TreeError<K>> {
        for &child in children {
            self.append_child(parent, child)?;
        }
        Ok(())
    }

    pub fn prepend_children(&mut self, parent: K, children: &[K]) -> Result<(), TreeError<K>> {
        // preserve order
        for &child in children.iter().rev() {
            self.prepend_child(parent, child)?;
        }
        Ok(())
    }

    /// Remove node and its descendants.
    pub fn remove(&mut self, key: K) -> Option<V> {
        if !self.inner.contains_key(key) {
            return None;
        }

        // Detach root of subtree from parent first.
        let _ = self.detach(key);

        // Remove descendants excluding `key`.
        let to_remove: Vec<_> = self
            .descendants(key)
            .filter(|&k| k != key)
            .collect();

        for k in to_remove {
            let _ = self.inner.remove(k);
        }

        self.inner.remove(key).map(|n| n.value)
    }

    // --- read-only helpers -------------------------------------------------

    pub fn parent(&self, key: K) -> Option<K> {
        self.inner.get(key)?.parent.maybe()
    }

    pub fn first_child(&self, key: K) -> Option<K> {
        self.inner.get(key)?.first_child.maybe()
    }

    pub fn last_child(&self, key: K) -> Option<K> {
        self.inner.get(key)?.last_child.maybe()
    }

    pub fn next_sibling(&self, key: K) -> Option<K> {
        self.inner.get(key)?.next_sibling.maybe()
    }

    pub fn prev_sibling(&self, key: K) -> Option<K> {
        self.inner.get(key)?.previous_sibling.maybe()
    }

    pub fn is_leaf(&self, key: K) -> bool {
        self.get_node(key).map_or(true, |n| n.first_child.is_null())
    }

    pub fn is_root(&self, key: K) -> bool {
        self.inner.get(key).map_or(false, |n| n.parent.is_null())
    }

    pub fn len(&self) -> usize { self.inner.len() }
    pub fn is_empty(&self) -> bool { self.inner.is_empty() }

    pub fn iter(&self) -> Iter<'_, K, V> { self.inner.iter() }
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> { self.inner.iter_mut() }
    pub fn children(&self, key: K) -> Children<'_, K, V> { Children::new(self, key) }
    pub fn descendants(&self, key: K) -> Descendants<'_, K, V> { Descendants::new(self, key) }
    pub fn ancestors(&self, key: K) -> Ancestors<'_, K, V> { Ancestors::new(self, key) }
    pub fn predecessors(&self, key: K) -> Predecessors<'_, K, V> { Predecessors::new(self, key) }
    pub fn following_siblings(&self, key: K) -> FollowingSiblings<'_, K, V> { FollowingSiblings::new(self, key) }
    pub fn preceding_siblings(&self, key: K) -> PrecedingSiblings<'_, K, V> { PrecedingSiblings::new(self, key) }
    pub fn traverse(&self, key: K) -> Traverse<'_, K, V> { Traverse::new(self, key) }
    pub fn reverse_traverse(&self, key: K) -> ReverseTraverse<'_, K, V> { ReverseTraverse::new(self, key) }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    // --- internals ---------------------------------------------------------

    fn ensure_exists(&self, k: K) -> Result<(), TreeError<K>> {
        self.contains(k).ok_or_else(|| TreeError::Missing(k))
    }

    fn ensure_no_cycle(&self, node: K, target_parent: K) -> Result<(), TreeError<K>> {
        if node == target_parent || self.is_ancestor(node, target_parent) {
            Err(TreeError::Cycle { node, target: target_parent })
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


// --------------------------------------------------------------------------
// RootTree with invariant: root cannot be removed/invalidated
// --------------------------------------------------------------------------

#[derive(Debug, Deref, DerefMut, Index, IndexMut)]
pub struct RootTree<K: TreeId, V> {
    #[deref]
    #[deref_mut]
    #[index]
    #[index_mut]
    inner: Tree<K, V>,
    root: K,
}

impl<K: TreeId, V> RootTree<K, V> {
    pub fn new(root: V) -> Self {
        let mut tree = Tree::new();
        let root = tree.insert_detached(root);
        Self { inner: tree, root }
    }

    pub fn with_capacity(root: V, capacity: usize) -> Self {
        let mut tree = Tree::with_capacity(capacity);
        let root = tree.insert_detached(root);
        Self { inner: tree, root }
    }

    pub fn root(&self) -> K { self.root }

    pub fn get_root(&self) -> TreeNodeRef<K, V> {
        TreeNodeRef::new(self.root, &self.inner)
    }

    pub fn get_root_mut(&mut self) -> TreeNodeRefMut<K, V> {
        TreeNodeRefMut::new(self.root, &mut self.inner)
    }

    pub fn get_root_node(&self) -> &TreeNode<K, V> {
        &self.inner.inner[self.root]
    }

    pub fn get_root_node_mut(&mut self) -> &mut TreeNode<K, V> {
        &mut self.inner.inner[self.root]
    }

    pub fn append_root(&mut self, value: V) -> Result<K, TreeError<K>> {
        self.inner.append_new_child(self.root, value)
    }

    pub fn append_root_with(&mut self, f: impl FnOnce(K) -> V) -> Result<K, TreeError<K>> {
        self.inner.insert_at_with_key(At::LastChild(self.root), f)
    }

    pub fn try_append_root_with<E>(
        &mut self,
        f: impl FnOnce(K) -> Result<V, E>,
    ) -> Result<K, E> {
        let root = self.root;
        let id = self.inner.try_insert_detached_with_key(f)?;
        // for root child attach this should not fail (root always exists, no cycle for new node)
        self.inner.append_child(root, id).expect("root must be a valid parent");
        Ok(id)
    }

    pub fn remove(&mut self, key: K) -> Result<Option<V>, TreeError<K>> {
        if key == self.root {
            return Err(TreeError::RootOperationForbidden);
        }
        Ok(self.inner.remove(key))
    }

    /// remove everything except root
    pub fn clear_children(&mut self) {
        let kids: Vec<_> = self.inner.children(self.root).collect();
        for k in kids {
            let _ = self.inner.remove(k);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    // Setup
    crate::tree_id! {
        pub struct Id;
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Node(pub &'static str);

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
        pub tree: super::Tree<Id, Node>,
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
            let root = tree.insert_detached(Node("root"));
            let a = tree.insert_detached(Node("a"));
            let b = tree.insert_detached(Node("b"));
            let c = tree.insert_detached(Node("c"));
            tree.append_child(root, a);
            tree.append_child(root, b);
            tree.append_child(root, c);

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

        let z = tree.insert_detached(Node("z"));
        tree.prepend_child(root, z);

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

        let x = tree.insert_detached(Node("x"));
        let y = tree.insert_detached(Node("y"));

        tree.prepend_child(x, b); // a x b c
        tree.append_child(y, b); // a x b y c

        let kids: Vec<_> = tree.children(root).collect();
        assert_eq!(kids, vec![a, x, b, y, c]);
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

        tree.detach(b);

        assert_eq!(tree.parent(b), None);
        assert_eq!(tree.next_sibling(a), Some(c));
        assert_eq!(tree.prev_sibling(c), Some(a));

        let kids: Vec<_> = tree.children(root).collect();
        assert_eq!(kids, vec![a, c]);

        // b still exists
        assert_eq!(tree.contains(b), true);
        assert_eq!(tree.get(b).unwrap(), Node("b"));
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
        let d = tree.insert_detached(Node("d"));
        tree.append_child(b, d);

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
        let a = tree.insert_detached(Node("a"));
        let b = tree.insert_detached(Node("b"));
        let c = tree.insert_detached(Node("c"));

        let root = tree.insert_with_children(Node("root"), &[a, b, c], At::LastChild(root)).unwrap();

        let kids: Vec<_> = tree.children(root).collect();
        assert_eq!(kids, vec![a, b, c]);
        assert_eq!(tree.parent(b), Some(root));
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
        let other = tree.insert_detached(Node("other"));

        // Move b under `other`
        tree.append_child(other, b);

        assert_eq!(tree.parent(b), Some(other));
        let root_kids: Vec<_> = tree.children(root).collect();
        assert_eq!(root_kids, vec![a, c]);
        assert_eq!(tree.next_sibling(a), Some(c));
    }

    #[test]
    fn empty_tree_and_leaf() {
        let mut tree = Test::empty();
        assert!(tree.is_empty());

        let leaf = tree.insert_detached(Node("leaf"));
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

            let d = tree.insert_detached(Node("d"));
            tree.append_child(b, d);

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

            let d = tree.insert_detached(Node("d"));
            let e = tree.insert_detached(Node("e"));
            tree.append_child(b, d);
            tree.append_child(b, e);

            let names: Vec<_> = tree
                .descendants(root)
                .map(|id| tree.get(id).unwrap())
                .collect();

            assert_eq!(
                names,
                vec![
                    Node("root"),
                    Node("a"),
                    Node("b"),
                    Node("d"),
                    Node("e"),
                    Node("c")
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
