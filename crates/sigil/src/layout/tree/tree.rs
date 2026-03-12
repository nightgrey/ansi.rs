use super::{TreeId, TreeNode, iter::*};
use super::{TreeNodeRef, TreeNodeRefMut};
use derive_more::{Deref, DerefMut, Index, IndexMut, IntoIterator};
use std::iter::FusedIterator;
use std::ops::Deref;

type Inner<K, V> = slotmap::SlotMap<K, V>;

#[derive(Debug, Index, IndexMut, IntoIterator)]
#[repr(transparent)]
pub struct Tree<K: TreeId, V> {
    #[into_iterator(owned, ref, ref_mut)]
    inner: Inner<K, TreeNode<K, V>>,
}

impl<K: TreeId, V> Tree<K, V> {
    pub fn new() -> Self {
        Self {
            inner: Inner::with_capacity_and_key(16),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: Inner::with_capacity_and_key(capacity),
        }
    }

    pub fn contains(&self, key: K) -> bool {
        self.inner.contains_key(key)
    }

    pub fn get(&self, key: K) -> Option<TreeNodeRef<K, V>> {
        match self.contains(key) {
            true => Some(TreeNodeRef::new(key, self)),
            false => None,
        }
    }

    pub fn get_mut(&mut self, key: K) -> Option<TreeNodeRefMut<K, V>> {
        match self.contains(key) {
            true => Some(TreeNodeRefMut::new(key, self)),
            false => None,
        }
    }

    pub fn get_node(&self, key: K) -> Option<&TreeNode<K, V>> {
        self.inner.get(key)
    }

    pub fn get_node_mut(&mut self, key: K) -> Option<&mut TreeNode<K, V>> {
        self.inner.get_mut(key)
    }

    pub fn insert(&mut self, value: V) -> K {
        self.inner.insert(TreeNode::new(value))
    }

    pub fn insert_with(&mut self, f: impl FnOnce(K) -> V) -> K {
        self.inner.insert_with_key(|k| TreeNode::new(f(k)))
    }

    pub fn try_insert_with<E>(&mut self, f: impl FnOnce(K) -> Result<V, E>) -> Result<K, E> {
        self.inner.try_insert_with_key(|k| f(k).map(TreeNode::new))
    }

    /// Insert a node and immediately append the given children to it.
    pub fn insert_with_children(&mut self, value: V, children: &[K]) -> K {
        let id = self.inner.insert(TreeNode::new(value));

        self.append_children(id, children);

        id
    }

    /// Remove a node with all its descendants.
    pub fn remove(&mut self, key: K) -> Option<V> {
        if !self.inner.contains_key(key) {
            return None;
        }

        self.detach(key);

        // Walk descendants depth-first via the linked child lists.
        // We collect into a vec to avoid aliasing issues.
        for key in self.descendants(key).collect::<Vec<_>>() {
            self.inner.remove(key);
        }

        self.inner.remove(key).map(|n| n.value)
    }

    /// Detach a node (keeps the node and its subtree intact).
    pub fn detach(&mut self, node: K) {
        let parent_key = self.inner[node].parent;

        if parent_key.is_null() {
            return;
        }

        let prev = self.inner[node].previous_sibling;
        let next = self.inner[node].next_sibling;

        // Stitch siblings together
        if prev.is_null() {
            self.inner[parent_key].first_child = next;
        } else {
            self.inner[prev].next_sibling = next;
        }

        if next.is_null() {
            self.inner[parent_key].last_child = prev;
        } else {
            self.inner[next].previous_sibling = prev;
        }

        // Clear own links
        let node = &mut self.inner[node];
        node.parent = K::null();
        node.previous_sibling = K::null();
        node.next_sibling = K::null();
    }

    /// Append a child to the end of the parent's child list.
    pub fn append_child(&mut self, parent: K, child: K) {
        self.detach(child);

        let old_tail = self.inner[parent].last_child;

        // Link after old tail
        if old_tail.is_null() {
            self.inner[parent].first_child = child;
        } else {
            self.inner[old_tail].next_sibling = child;
        }

        self.inner[parent].last_child = child;

        let node = &mut self.inner[child];
        node.parent = parent;
        node.previous_sibling = old_tail;
        node.next_sibling = K::null();
    }

    /// Insert a child into the end of the parent's child list.
    pub fn insert_child(&mut self, parent: K, child: V) -> K {
        let id = self.insert(child);
        self.append_child(parent, id);
        id
    }

    pub fn append_children(&mut self, parent: K, children: &[K]) {
        for &child in children {
            self.append_child(parent, child);
        }
    }

    /// Prepend a child to the beginning of the parent's child list.
    pub fn prepend_child(&mut self, parent: K, child: K) {
        self.detach(child);

        let old_head = self.inner[parent].first_child;

        // Link before old head
        if old_head.is_null() {
            self.inner[parent].last_child = child;
        } else {
            self.inner[old_head].previous_sibling = child;
        }

        self.inner[parent].first_child = child;

        let node = &mut self.inner[child];
        node.parent = parent;
        node.previous_sibling = K::null();
        node.next_sibling = old_head;
    }

    /// Prepend a list of children to the beginning of the parent's child list.
    pub fn prepend_children(&mut self, parent: K, children: &[K]) {
        for &child in children {
            self.prepend_child(parent, child);
        }
    }

    /// Insert a child before a specific sibling.
    pub fn prepend(&mut self, node: K, before: K) {
        let parent = self.inner[before].parent;
        debug_assert!(
            !parent.is_null(),
            "insert_before: reference node has no parent"
        );

        self.detach(node);

        let prev = self.inner[before].previous_sibling;

        // Stitch in
        if prev.is_null() {
            self.inner[parent].first_child = node;
        } else {
            self.inner[prev].next_sibling = node;
        }

        self.inner[before].previous_sibling = node;

        let node = &mut self.inner[node];
        node.parent = parent;
        node.previous_sibling = prev;
        node.next_sibling = before;
    }

    /// Insert a node after a specific node.
    pub fn append(&mut self, node: K, after: K) {
        let parent = self.inner[after].parent;
        debug_assert!(
            !parent.is_null(),
            "insert_after: reference node has no parent"
        );

        self.detach(node);

        let next = self.inner[after].next_sibling;

        // Stitch in
        if next.is_null() {
            self.inner[parent].last_child = node;
        } else {
            self.inner[next].previous_sibling = node;
        }

        self.inner[after].next_sibling = node;

        let node = &mut self.inner[node];
        node.parent = parent;
        node.previous_sibling = after;
        node.next_sibling = next;
    }

    pub fn parent(&self, key: K) -> Option<K> {
        self.inner.get(key)?.parent.as_option()
    }

    pub fn first_child(&self, key: K) -> Option<K> {
        self.inner.get(key)?.first_child.as_option()
    }

    pub fn last_child(&self, key: K) -> Option<K> {
        self.inner.get(key)?.last_child.as_option()
    }

    pub fn next_sibling(&self, key: K) -> Option<K> {
        self.inner.get(key)?.next_sibling.as_option()
    }

    pub fn prev_sibling(&self, key: K) -> Option<K> {
        self.inner.get(key)?.previous_sibling.as_option()
    }

    pub fn is_leaf(&self, key: K) -> bool {
        self.get_node(key).map_or(true, |n| n.first_child.is_null())
    }

    pub fn is_root(&self, key: K) -> bool {
        self.inner.get(key).map_or(false, |n| n.parent.is_null())
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn children(&self, key: K) -> Children<K, V> {
        Children::new(self, key)
    }

    pub fn descendants(&self, key: K) -> Descendants<K, V> {
        Descendants::new(self, key)
    }

    pub fn ancestors(&self, key: K) -> Ancestors<K, V> {
        Ancestors::new(self, key)
    }

    pub fn predecessors(&self, key: K) -> Predecessors<K, V> {
        Predecessors::new(self, key)
    }

    pub fn following_siblings(&self, key: K) -> FollowingSiblings<K, V> {
        FollowingSiblings::new(self, key)
    }

    pub fn preceding_siblings(&self, key: K) -> PrecedingSiblings<K, V> {
        PrecedingSiblings::new(self, key)
    }

    pub fn traverse(&self, key: K) -> Traverse<K, V> {
        Traverse::new(self, key)
    }

    pub fn reverse_traverse(&self, key: K) -> ReverseTraverse<K, V> {
        ReverseTraverse::new(self, key)
    }

    pub fn iter(&self) -> slotmap::basic::Iter<K, TreeNode<K, V>> {
        self.inner.iter()
    }

    pub fn values_mut(&mut self) -> slotmap::basic::ValuesMut<K, TreeNode<K, V>> {
        self.inner.values_mut()
    }

    pub fn keys(&self) -> slotmap::basic::Keys<K, TreeNode<K, V>> {
        self.inner.keys()
    }

    pub fn values(&self) -> slotmap::basic::Values<K, TreeNode<K, V>> {
        self.inner.values()
    }
}

#[derive(Debug, Index, IndexMut, IntoIterator, Deref, DerefMut)]
pub struct RootTree<K: TreeId, V> {
    #[deref]
    #[deref_mut]
    #[index]
    #[index_mut]
    #[into_iterator(owned, ref, ref_mut)]
    pub inner: Tree<K, V>,
    pub root: K,
}

impl<K: TreeId, V> RootTree<K, V> {
    pub fn new(root: V) -> Self {
        let mut tree = Tree::new();
        let root = tree.insert(root);

        Self {
            root,
            inner: tree,
        }
    }

    pub fn with_capacity(root: V, capacity: usize) -> Self {
        let mut tree = Tree::with_capacity(capacity);
        let root = tree.insert(root);

        Self { root, inner: tree }
    }


    pub fn get_root(&self) -> Option<TreeNodeRef<K, V>> {
        self.inner.get(self.root)
    }

    pub fn get_root_mut(&mut self) -> Option<TreeNodeRefMut<K, V>> {
        self.inner.get_mut(self.root)
    }

    pub fn get_root_node(&self) -> Option<&TreeNode<K, V>> {
        self.inner.get_node(self.root)
    }

    pub fn get_root_node_mut(&mut self, key: K) -> Option<&mut TreeNode<K, V>> {
        self.inner.get_node_mut(self.root)
    }

    pub fn root(&self) -> K {
        self.root
    }

    pub fn insert(&mut self, value: V) -> K {
        let root = self.root;
        let id  = self.inner.insert(value);
        self.append_child(root, id);
        id
    }

    pub fn insert_with(&mut self, f: impl FnOnce(K) -> V) -> K {
        let root = self.root;
        let id = self.inner.insert_with(f);
        self.append_child(root, id);
        id
    }

    pub fn try_insert_with<F, E>(&mut self, f: impl FnOnce(K) -> Result<V, E>) -> Result<K, E> {
        let root = self.root;
        let id = self.inner.try_insert_with(f)?;
        self.append_child(root, id);
        Ok(id)
    }

    pub fn insert_with_children(&mut self, value: V, children: &[K]) -> K {
        let root = self.root;
        let id = self.inner.insert_with_children(value, children);
        self.append_child(root, id);
        id
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
            let root = tree.insert(Node("root"));
            let a = tree.insert(Node("a"));
            let b = tree.insert(Node("b"));
            let c = tree.insert(Node("c"));
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

        let z = tree.insert(Node("z"));
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

        let x = tree.insert(Node("x"));
        let y = tree.insert(Node("y"));

        tree.prepend(x, b); // a x b c
        tree.append(y, b); // a x b y c

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
        let d = tree.insert(Node("d"));
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
        let a = tree.insert(Node("a"));
        let b = tree.insert(Node("b"));
        let c = tree.insert(Node("c"));

        let root = tree.insert_with_children(Node("root"), &[a, b, c]);

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
        let other = tree.insert(Node("other"));

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

        let leaf = tree.insert(Node("leaf"));
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

            let d = tree.insert(Node("d"));
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

            let d = tree.insert(Node("d"));
            let e = tree.insert(Node("e"));
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
