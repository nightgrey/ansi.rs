use super::Tree;
use super::iter::*;
use crate::Id;
use derive_more::{Deref, DerefMut};
use std::ops::{Deref, DerefMut};

/// A tree node that stores a value alongside embedded structural links.
///
/// `Node<K, V>` wraps an inner value of type `V` and maintains parent, child,
/// and sibling pointers (all of type `K`). Null pointers are represented by
/// the key's null sentinel ([`Id::none`]).
///
/// The node dereferences to `V` via [`Deref`] / [`DerefMut`], so you can
/// access the inner value directly through `*node`.
#[derive(Debug, Deref, DerefMut)]
pub struct Node<K: Id, V> {
    pub(crate) parent: K,
    pub(crate) first_child: K,
    pub(crate) last_child: K,
    pub(crate) previous_sibling: K,
    pub(crate) next_sibling: K,
    #[deref]
    #[deref_mut]
    pub(crate) inner: V,
}

impl<K: Id, V> Node<K, V> {
    pub(super) fn new(value: V) -> Self {
        Self {
            inner: value,
            parent: K::null(),
            first_child: K::null(),
            last_child: K::null(),
            previous_sibling: K::null(),
            next_sibling: K::null(),
        }
    }

    /// Returns a reference to the inner value.
    #[inline]
    pub fn inner(&self) -> &V {
        &self.inner
    }

    /// Returns the id of this node's parent, or the null sentinel if it is a root.
    #[inline]
    pub fn parent(&self) -> K {
        self.parent
    }

    /// Returns the id of this node's first child, or null if it is a leaf.
    #[inline]
    pub fn first_child(&self) -> K {
        self.first_child
    }

    /// Returns the id of this node's last child, or null if it is a leaf.
    #[inline]
    pub fn last_child(&self) -> K {
        self.last_child
    }

    /// Returns the id of the next sibling, or null if this is the last child.
    #[inline]
    pub fn next_sibling(&self) -> K {
        self.next_sibling
    }

    /// Returns the id of the previous sibling, or null if this is the first child.
    #[inline]
    pub fn previous_sibling(&self) -> K {
        self.previous_sibling
    }
}

impl<K: Id, V: PartialEq> PartialEq<V> for Node<K, V> {
    fn eq(&self, other: &V) -> bool {
        &self.inner == other
    }
}

impl<K: Id, V: PartialEq> PartialEq<&V> for Node<K, V> {
    fn eq(&self, other: &&V) -> bool {
        &&self.inner == other
    }
}

impl<K: Id, V: PartialEq> AsRef<V> for Node<K, V> {
    fn as_ref(&self) -> &V {
        &self.inner
    }
}

impl<K: Id, V: PartialEq> AsMut<V> for Node<K, V> {
    fn as_mut(&mut self) -> &mut V {
        &mut self.inner
    }
}

/// An immutable reference to a node within a [`Tree`].
///
/// Bundles the node's [`Id`] together with a shared reference to the tree,
/// providing navigation and iteration methods without needing to pass the tree
/// around separately. Dereferences to `V`.
#[derive(Debug)]
pub struct NodeRef<'a, K: Id, V> {
    pub id: K,
    tree: &'a Tree<K, V>,
}

impl<'a, K: Id, V> NodeRef<'a, K, V> {
    pub fn new(id: K, tree: &'a Tree<K, V>) -> Self {
        Self { id, tree }
    }

    /// Returns the underlying [`Node`].
    #[inline]
    pub fn node(&self) -> &Node<K, V> {
        &self.tree[self.id]
    }

    /// Returns the id of this node's parent, or null if it is a root.
    pub fn parent(&self) -> K {
        self.node().parent()
    }

    /// Returns the id of this node's first child, or null if it is a leaf.
    pub fn first_child(&self) -> K {
        self.node().first_child()
    }

    /// Returns the id of this node's last child, or null if it is a leaf.
    pub fn last_child(&self) -> K {
        self.node().last_child()
    }

    /// Returns the id of the next sibling, or null if this is the last child.
    pub fn next_sibling(&self) -> K {
        self.node().next_sibling()
    }

    /// Returns the id of the previous sibling, or null if this is the first child.
    pub fn previous_sibling(&self) -> K {
        self.node().previous_sibling()
    }

    /// Iterates over the direct children of this node.
    pub fn children(&self) -> Children<K, V> {
        self.tree.children(self.id)
    }

    /// Iterates over all descendants in pre-order (depth-first).
    pub fn descendants(&self) -> Descendants<K, V> {
        self.tree.descendants(self.id)
    }

    /// Iterates upward through this node's ancestors toward the root.
    pub fn ancestors(&self) -> Ancestors<K, V> {
        self.tree.ancestors(self.id)
    }

    /// Iterates over this node, its preceding siblings, and then its ancestors.
    pub fn predecessors(&self) -> Predecessors<K, V> {
        self.tree.predecessors(self.id)
    }

    /// Iterates forward through this node and its following siblings.
    pub fn following_siblings(&self) -> FollowingSiblings<K, V> {
        self.tree.following_siblings(self.id)
    }

    /// Iterates backward through this node and its preceding siblings.
    pub fn preceding_siblings(&self) -> PrecedingSiblings<K, V> {
        self.tree.preceding_siblings(self.id)
    }

    /// Pre-order traversal yielding [`NodeEdge::Start`](crate::NodeEdge::Start) /
    /// [`NodeEdge::End`](crate::NodeEdge::End) events.
    pub fn traverse(&self) -> Traverse<K, V> {
        self.tree.traverse(self.id)
    }

    /// Reverse (post-order) traversal yielding [`NodeEdge`](crate::NodeEdge) events.
    pub fn reverse_traverse(&self) -> ReverseTraverse<K, V> {
        self.tree.reverse_traverse(self.id)
    }
}

impl<'a, K: Id, V: PartialEq> PartialEq<V> for NodeRef<'a, K, V> {
    fn eq(&self, other: &V) -> bool {
        &self.node().inner == other
    }
}

impl<'a, K: Id, V> Deref for NodeRef<'a, K, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        self.node()
    }
}

/// A mutable reference to a node within a [`Tree`].
///
/// Like [`NodeRef`], but also allows mutation of the inner value through
/// [`DerefMut`]. Navigation methods still return ids rather than mutable
/// references, preserving borrow safety.
pub struct NodeRefMut<'a, K: Id, V> {
    pub id: K,
    tree: &'a mut Tree<K, V>,
}

impl<'a, K: Id, V> NodeRefMut<'a, K, V> {
    pub fn new(id: K, tree: &'a mut Tree<K, V>) -> Self {
        Self { id, tree }
    }

    /// Returns a shared reference to the underlying [`Node`].
    pub fn node(&self) -> &Node<K, V> {
        &self.tree[self.id]
    }

    /// Returns a mutable reference to the underlying [`Node`].
    pub fn node_mut(&mut self) -> &mut Node<K, V> {
        &mut self.tree[self.id]
    }

    /// Returns the id of this node's parent, or null if it is a root.
    pub fn parent(&self) -> K {
        self.node().parent()
    }

    /// Returns the id of this node's first child, or null if it is a leaf.
    pub fn first_child(&self) -> K {
        self.node().first_child()
    }

    /// Returns the id of this node's last child, or null if it is a leaf.
    pub fn last_child(&self) -> K {
        self.node().last_child()
    }

    /// Returns the id of the next sibling, or null if this is the last child.
    pub fn next_sibling(&self) -> K {
        self.node().next_sibling()
    }

    /// Returns the id of the previous sibling, or null if this is the first child.
    pub fn previous_sibling(&self) -> K {
        self.node().previous_sibling()
    }

    /// Iterates over the direct children of this node.
    pub fn children(&self) -> Children<K, V> {
        self.tree.children(self.id)
    }

    /// Iterates over all descendants in pre-order (depth-first).
    pub fn descendants(&self) -> Descendants<K, V> {
        self.tree.descendants(self.id)
    }

    /// Iterates upward through this node's ancestors toward the root.
    pub fn ancestors(&self) -> Ancestors<K, V> {
        self.tree.ancestors(self.id)
    }

    /// Iterates over this node, its preceding siblings, and then its ancestors.
    pub fn predecessors(&self) -> Predecessors<K, V> {
        self.tree.predecessors(self.id)
    }

    /// Iterates forward through this node and its following siblings.
    pub fn following_siblings(&self) -> FollowingSiblings<K, V> {
        self.tree.following_siblings(self.id)
    }

    /// Iterates backward through this node and its preceding siblings.
    pub fn preceding_siblings(&self) -> PrecedingSiblings<K, V> {
        self.tree.preceding_siblings(self.id)
    }

    /// Pre-order traversal yielding [`NodeEdge`](crate::NodeEdge) events.
    pub fn traverse(&self) -> Traverse<K, V> {
        self.tree.traverse(self.id)
    }

    /// Reverse (post-order) traversal yielding [`NodeEdge`](crate::NodeEdge) events.
    pub fn reverse_traverse(&self) -> ReverseTraverse<K, V> {
        self.tree.reverse_traverse(self.id)
    }
}

impl<'a, K: Id, V> Deref for NodeRefMut<'a, K, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.node()
    }
}

impl<'a, K: Id, V> DerefMut for NodeRefMut<'a, K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.node_mut()
    }
}
