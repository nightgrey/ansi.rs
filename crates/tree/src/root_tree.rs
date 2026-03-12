use super::{At, Id, Node, Tree, Error};
use super::{NodeRef, NodeRefMut};
use derive_more::{Deref, DerefMut, Index, IndexMut, IntoIterator};
use std::ops::Deref;


#[derive(Debug, Deref, DerefMut, Index, IndexMut, IntoIterator)]
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
    pub fn new(root: V) -> Self {
        let mut tree = Tree::new();
        let root = tree.insert(root);
        Self { inner: tree, root_id: root }
    }

    pub fn with_capacity(root: V, capacity: usize) -> Self {
        let mut tree = Tree::with_capacity(capacity);
        let root = tree.insert(root);
        Self { inner: tree, root_id: root }
    }

    pub fn root_id(&self) -> K { self.root_id }

    pub fn root(&self) -> &Node<K, V> {
        &self.inner[self.root_id]
    }

    pub fn root_mut(&mut self) -> &mut Node<K, V> {
        &mut self.inner[self.root_id]
    }

    pub fn root_ref(&self) -> NodeRef<K, V> {
        NodeRef::new(self.root_id, self)
    }

    pub fn root_ref_mut(&mut self) -> NodeRefMut<K, V> {
        NodeRefMut::new(self.root_id, self)
    }

    pub fn insert(&mut self, value: V) -> K {
        self.inner.insert_at(value, At::Child(self.root_id))
    }

    pub fn remove(&mut self, id: K) -> Result<Option<V>, Error<K>> {
        if id == self.root_id {
            return Err(Error::OperationForbidden);
        }
        Ok(self.inner.remove(id))
    }

    /// remove everything except root
    pub fn clear(&mut self) {
        let kids: Vec<_> = self.inner.children(self.root_id).collect();
        for k in kids {
            let _ = self.inner.remove(k);
        }
    }
}