use std::fmt::Debug;
use crate::{At, Id, Node, Tree, Error};
use derive_more::{Deref, DerefMut, Index, IndexMut, IntoIterator};
use std::ops::Deref;


/// A tree that always contains a root node.
///
/// `RootTree` wraps a [`Tree`] and guarantees that one distinguished root node
/// exists for the tree's entire lifetime. The root cannot be removed — calling
/// [`remove`](Self::remove) with the root id returns
/// [`Error::OperationForbidden`].
///
/// All other [`Tree`] methods are available through [`Deref`] / [`DerefMut`].
#[derive(Deref, DerefMut, Index, IndexMut, IntoIterator)]
pub struct RootTree<K: Id, V> {
    #[deref]
    #[deref_mut]
    #[index]
    #[index_mut]
    #[into_iterator(owned, ref, ref_mut)]
    inner: Tree<K, V>,
    root_id: K,
}

impl<K: Id, V> RootTree<K, V> {
    /// Creates a new tree with the given value as the root node.
    pub fn new(root: V) -> Self {
        let mut tree = Tree::new();
        let root = tree.insert(root);
        Self { inner: tree, root_id: root }
    }

    /// Creates a new tree with the given root value and pre-allocated capacity.
    pub fn with_capacity(root: V, capacity: usize) -> Self {
        let mut tree = Tree::with_capacity(capacity);
        let root = tree.insert(root);
        Self { inner: tree, root_id: root }
    }

    /// Returns the id of the root node.
    pub fn root_id(&self) -> K { self.root_id }

    /// Returns a reference to the root value.
    pub fn root(&self) -> &Node<K, V> {
        &self.inner[self.root_id]
    }

    /// Returns a mutable reference to the root value.
    pub fn root_mut(&mut self) -> &mut Node<K, V> {
        &mut self.inner[self.root_id]
    }

    /// Inserts a new node as a child of the root and returns its key.
    pub fn insert(&mut self, value: V) -> K {
        self.inner.insert_at(value, At::Child(self.root_id))
    }

    /// Removes a node and its descendants, returning the inner value.
    ///
    /// Returns [`Error::OperationForbidden`] if `id` is the root node.
    pub fn remove(&mut self, id: K) -> Result<Option<V>, Error<K>> {
        if id == self.root_id {
            return Err(Error::OperationForbidden);
        }
        Ok(self.inner.remove(id))
    }

    /// Removes all nodes except the root, leaving the tree with a single
    /// childless root node.
    pub fn clear(&mut self) {
        for k in self.inner.children(self.root_id).collect::<Vec<_>>() {
            self.inner.remove(k);
        }
    }
}

impl<K: Id, V> AsRef<Tree<K, V>> for RootTree<K, V> {
    fn as_ref(&self) -> &Tree<K, V> {
        &self.inner
    }
}

impl<K: Id, V> AsMut<Tree<K, V>> for RootTree<K, V> {
    fn as_mut(&mut self) -> &mut Tree<K, V> {
        &mut self.inner
    }
}

impl<K: Id, V: Debug> Debug for RootTree<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}
