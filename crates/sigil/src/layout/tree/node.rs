use super::iter::*;
use super::{TreeId, Tree};
use derive_more::{Deref, DerefMut};
use std::ops::{Deref, DerefMut};

/// A tree node with embedded structural links.
#[derive(Debug, Deref, DerefMut)]
pub struct TreeNode<K: TreeId, V> {
    pub(super) parent: K,
    pub(super) first_child: K,
    pub(super) last_child: K,
    pub(super) previous_sibling: K,
    pub(super) next_sibling: K,
    #[deref]
    #[deref_mut]
    pub(super) value: V,
}

impl<K: TreeId, V> TreeNode<K, V> {
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

    pub fn parent(&self) -> K {
        (self.parent)
    }

    pub fn first_child(&self) -> K {
        self.first_child
    }

    pub fn last_child(&self) -> K {
        self.last_child
    }

    pub fn next_sibling(&self) -> K {
        self.next_sibling
    }

    pub fn previous_sibling(&self) -> K {
        self.previous_sibling
    }
}

#[derive(Debug)]
pub struct TreeNodeRef<'a, K: TreeId, V> {
    pub id: K,
    tree: &'a Tree<K, V>,
}

impl<'a, K: TreeId, V> TreeNodeRef<'a, K, V> {
    pub fn new(id: K, tree: &'a Tree<K, V>) -> Self {
        Self { id, tree }
    }

    pub fn node(&self) -> &TreeNode<K, V> {
        &self.tree[self.id]
    }

    pub fn parent(&self) -> K {
        self.node().parent()
    }

    pub fn first_child(&self) -> K {
        self.node().first_child()
    }

    pub fn last_child(&self) -> K {
        self.node().last_child()
    }

    pub fn next_sibling(&self) -> K {
        self.node().next_sibling()
    }

    pub fn previous_sibling(&self) -> K {
        self.node().previous_sibling()
    }

    pub fn children(&self) -> Children<K, V> {
        self.tree.children(self.id)
    }

    pub fn descendants(&self) -> Descendants<K, V> {
        self.tree.descendants(self.id)
    }

    pub fn ancestors(&self) -> Ancestors<K, V> {
        self.tree.ancestors(self.id)
    }

    pub fn predecessors(&self) -> Predecessors<K, V> {
        self.tree.predecessors(self.id)
    }

    pub fn following_siblings(&self) -> FollowingSiblings<K, V> {
        self.tree.following_siblings(self.id)
    }

    pub fn preceding_siblings(&self) -> PrecedingSiblings<K, V> {
        self.tree.preceding_siblings(self.id)
    }

    pub fn traverse(&self) -> Traverse<K, V> {
        self.tree.traverse(self.id)
    }

    pub fn reverse_traverse(&self) -> ReverseTraverse<K, V> {
        self.tree.reverse_traverse(self.id)
    }
}

impl<'a, K: TreeId, V: PartialEq> PartialEq<V> for TreeNodeRef<'a, K, V> {
    fn eq(&self, other: &V) -> bool {
        *self == *other
    }
}

impl<'a, K: TreeId, V> Deref for TreeNodeRef<'a, K, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        self.node()
    }
}

pub struct TreeNodeRefMut<'a, K: TreeId, V> {
    pub id: K,
    tree: &'a mut Tree<K, V>,
}

impl<'a, K: TreeId, V> TreeNodeRefMut<'a, K, V> {
    pub fn new(id: K, tree: &'a mut Tree<K, V>) -> Self {
        Self { id, tree }
    }

    pub fn node(&self) -> &TreeNode<K, V> {
        &self.tree[self.id]
    }

    pub fn node_mut(&mut self) -> &mut TreeNode<K, V> {
        &mut self.tree[self.id]
    }

    pub fn append_child(&mut self, child: K) {
        self.tree.append_child(self.id, child);
    }

    pub fn append_children(&mut self, children: &[K]) {
        self.tree.append_children(self.id, children);
    }

    pub fn prepend_child(&mut self, child: K) {
        self.tree.prepend_child(self.id, child);
    }

    pub fn prepend_children(&mut self, children: &[K]) {
        self.tree.prepend_children(self.id, children);
    }

    pub fn parent(&self) -> K {
        self.node().parent()
    }

    pub fn first_child(&self) -> K {
        self.node().first_child()
    }

    pub fn last_child(&self) -> K {
        self.node().last_child()
    }

    pub fn next_sibling(&self) -> K {
        self.node().next_sibling()
    }

    pub fn previous_sibling(&self) -> K {
        self.node().previous_sibling()
    }

    pub fn children(&self) -> Children<K, V> {
        self.tree.children(self.id)
    }

    pub fn descendants(&self) -> Descendants<K, V> {
        self.tree.descendants(self.id)
    }

    pub fn ancestors(&self) -> Ancestors<K, V> {
        self.tree.ancestors(self.id)
    }

    pub fn predecessors(&self) -> Predecessors<K, V> {
        self.tree.predecessors(self.id)
    }

    pub fn following_siblings(&self) -> FollowingSiblings<K, V> {
        self.tree.following_siblings(self.id)
    }

    pub fn preceding_siblings(&self) -> PrecedingSiblings<K, V> {
        self.tree.preceding_siblings(self.id)
    }

    pub fn traverse(&self) -> Traverse<K, V> {
        self.tree.traverse(self.id)
    }

    pub fn reverse_traverse(&self) -> ReverseTraverse<K, V> {
        self.tree.reverse_traverse(self.id)
    }
}

impl<'a, K: TreeId, V> Deref for TreeNodeRefMut<'a, K, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.node()
    }
}

impl<'a, K: TreeId, V> DerefMut for TreeNodeRefMut<'a, K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.node_mut()
    }
}
