use crate::{Id, Tree};
use derive_more::{AsMut, AsRef, Deref, DerefMut};
use std::ops::{Deref, DerefMut};

/// A tree node that stores a value alongside embedded structural links.
///
/// `Node<K, V>` wraps an inner value of type `V` and maintains parent, child,
/// and sibling pointers (all of type `K`). Null pointers are represented by
/// the key's null sentinel ([`Id::none`]).
///
/// The node dereferences to `V` via [`Deref`] / [`DerefMut`], so you can
/// access the inner value directly through `*node`.
#[derive(Debug, Deref, DerefMut, AsRef, AsMut)]
pub struct Node<K: Id, V> {
    pub(crate) parent: K,
    pub(crate) first_child: K,
    pub(crate) last_child: K,
    pub(crate) prev_sibling: K,
    pub(crate) next_sibling: K,
    #[deref]
    #[deref_mut]
    #[as_ref]
    #[as_mut]
    pub(crate) inner: V,
}

impl<K: Id, V> Node<K, V> {
    pub(super) fn new(value: V) -> Self {
        Self {
            inner: value,
            parent: K::null(),
            first_child: K::null(),
            last_child: K::null(),
            prev_sibling: K::null(),
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
    pub fn prev_sibling(&self) -> K {
        self.prev_sibling
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

