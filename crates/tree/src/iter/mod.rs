mod iter;
mod edge;

pub use iter::*;
pub use edge::*;

use crate::{Id, Node};

/// An iterator over the key-value pairs in a [`Tree`].
pub type Iter<'a, K: 'a + Id, V: 'a> = slotmap::basic::Iter<'a, K, Node<K, V>>;

/// A mutable iterator over the key-value pairs in a [`Tree`].
pub type IterMut<'a, K: 'a + Id, V: 'a> = slotmap::basic::IterMut<'a, K, Node<K, V>>;

/// An iterator over the values in a [`Tree`].
pub type Values<'a, K: 'a + Id, V: 'a> = slotmap::basic::Values<'a, K, V>;
/// A mutable iterator over the values in a [`Tree`].
pub type ValuesMut<'a, K: 'a + Id, V: 'a> = slotmap::basic::ValuesMut<'a, K, V>;

/// An iterator over the keys in a [`Tree`].
pub type Keys<'a, K: 'a + Id, V: 'a> = slotmap::basic::Keys<'a, K, Node<K, V>>;
/// A draining iterator over the key-value pairs in a [`Tree`].
pub type Drain<'a, K: 'a + Id, V: 'a> = slotmap::basic::Drain<'a, K, Node<K, V>>;

/// An iterator over the nodes in a [`Tree`].
pub type Nodes<'a, K: 'a + Id, V: 'a> = Values<'a, K, Node<K, V>>;
/// A mutable iterator over the nodes in a [`Tree`].
pub type NodesMut<'a, K: 'a + Id, V: 'a> = ValuesMut<'a, K, Node<K, V>>;

/// An iterator that moves key-value pairs out of a [`Tree`].
pub type IntoIter<K: Id, V> = slotmap::basic::IntoIter<K, Node<K, V>>;