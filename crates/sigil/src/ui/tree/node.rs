use super::iter::*;
use super::{Key, Tree};
use derive_more::{Deref, DerefMut, Index, IndexMut};
use std::ops::Deref;

/// A tree node with embedded structural links.
///
/// All keys use `K::null()` to represent "no link" instead of `Option<K>`.
#[derive(Debug, Deref, DerefMut)]
pub struct Node<K: Key, V> {
    pub(super) parent: K,
    pub(super) first_child: K,
    pub(super) last_child: K,
    pub(super) previous_sibling: K,
    pub(super) next_sibling: K,
    #[deref]
    #[deref_mut]
    pub(super) value: V,
}

impl<K: Key, V> Node<K, V> {
    pub(super) fn new(value: V) -> Self {
        Self {
            value,
            parent: K::null(),
            first_child: K::null(),
            last_child: K::null(),
            previous_sibling: K::null(),
            next_sibling: K::null(),
        }
    }

    pub fn parent(&self) -> Option<K> {
        (self.parent.option())
    }

    pub fn first_child(&self) -> Option<K> {
        self.first_child.option()
    }

    pub fn last_child(&self) -> Option<K> {
        self.last_child.option()
    }

    pub fn next_sibling(&self) -> Option<K> {
        self.next_sibling.option()
    }

    pub fn previous_sibling(&self) -> Option<K> {
        self.previous_sibling.option()
    }
}

pub struct NodeRef<'a, K: Key, V> {
    pub id: K,
    tree: &'a Tree<K, V>,
}

impl<'a, K: Key, V> NodeRef<'a, K, V> {
    pub fn new(id: K, tree: &'a Tree<K, V>) -> Self {
        Self { id, tree }
    }

    pub fn node(&self) -> &'a Node<K, V> {
        self.tree.get_node(self.id).unwrap()
    }

    pub fn parent(&self) -> Option<K> {
        self.node().parent()
    }

    pub fn first_child(&self) -> Option<K> {
        self.node().first_child()
    }

    pub fn last_child(&self) -> Option<K> {
        self.node().last_child()
    }

    pub fn next_sibling(&self) -> Option<K> {
        self.node().next_sibling()
    }

    pub fn previous_sibling(&self) -> Option<K> {
        self.node().previous_sibling()
    }

    pub fn children(&self, key: K) -> Children<K, V> {
        self.tree.children(key)
    }

    pub fn descendants(&self, key: K) -> Descendants<K, V> {
        self.tree.descendants(key)
    }

    pub fn ancestors(&self, key: K) -> Ancestors<K, V> {
        self.tree.ancestors(key)
    }

    pub fn predecessors(&self, key: K) -> Predecessors<K, V> {
        self.tree.predecessors(key)
    }

    pub fn following_siblings(&self, key: K) -> FollowingSiblings<K, V> {
        self.tree.following_siblings(key)
    }

    pub fn preceding_siblings(&self, key: K) -> PrecedingSiblings<K, V> {
        self.tree.preceding_siblings(key)
    }

    pub fn traverse(&self, key: K) -> Traverse<K, V> {
        self.tree.traverse(key)
    }

    pub fn reverse_traverse(&self, key: K) -> ReverseTraverse<K, V> {
        self.tree.reverse_traverse(key)
    }
}

impl<'a, K: Key, V> Deref for NodeRef<'a, K, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        self.node()
    }
}

pub struct NodeRefMut<'a, K: Key, V> {
    pub id: K,
    tree: &'a mut Tree<K, V>,
}

impl<'a, K: Key, V> NodeRefMut<'a, K, V> {
    pub fn new(id: K, tree: &'a mut Tree<K, V>) -> Self {
        Self { id, tree }
    }

    pub fn node(&self) -> &Node<K, V> {
        self.tree.get_node(self.id).unwrap()
    }

    pub fn node_mut(&mut self) -> &mut Node<K, V> {
        self.tree.get_node_mut(self.id).unwrap()
    }

    pub fn parent(&self) -> Option<K> {
        self.node().parent()
    }

    pub fn first_child(&self) -> Option<K> {
        self.node().first_child()
    }

    pub fn last_child(&self) -> Option<K> {
        self.node().last_child()
    }

    pub fn next_sibling(&self) -> Option<K> {
        self.node().next_sibling()
    }

    pub fn previous_sibling(&self) -> Option<K> {
        self.node().previous_sibling()
    }

    pub fn children(&self, key: K) -> Children<K, V> {
        self.tree.children(key)
    }

    pub fn descendants(&self, key: K) -> Descendants<K, V> {
        self.tree.descendants(key)
    }

    pub fn ancestors(&self, key: K) -> Ancestors<K, V> {
        self.tree.ancestors(key)
    }

    pub fn predecessors(&self, key: K) -> Predecessors<K, V> {
        self.tree.predecessors(key)
    }

    pub fn following_siblings(&self, key: K) -> FollowingSiblings<K, V> {
        self.tree.following_siblings(key)
    }

    pub fn preceding_siblings(&self, key: K) -> PrecedingSiblings<K, V> {
        self.tree.preceding_siblings(key)
    }

    pub fn traverse(&self, key: K) -> Traverse<K, V> {
        self.tree.traverse(key)
    }

    pub fn reverse_traverse(&self, key: K) -> ReverseTraverse<K, V> {
        self.tree.reverse_traverse(key)
    }
}

impl<'a, K: Key, V> Deref for NodeRefMut<'a, K, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        self.node()
    }
}
#[test]
fn qwe() {
    // Setup
    crate::key! {
        pub struct Id;
    }

    #[derive(Debug, Clone, PartialEq)]
    pub struct Node {
        pub value: &'static str,
    }

    let mut tree = Tree::<Id, Node>::new();

    let id = tree.insert(Node { value: "root" });

    let a = tree.insert(Node { value: "a" });
    tree.insert_children(id, &[a]);

    let reference = NodeRef::new(id, &tree);

    println!("{:?}", reference.children(id).collect::<Vec<_>>());
}
