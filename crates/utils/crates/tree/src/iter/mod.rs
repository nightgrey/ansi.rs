mod edge;
mod iter;

pub use edge::*;
pub use iter::*;

use crate::{Id, Node};

/// An iterator over `(key, &node)` pairs in a [`Tree`](crate::Tree).
pub type Iter<'a, K: 'a + Id, V: 'a> = slotmap::basic::Iter<'a, K, V>;

/// A mutable iterator over `(key, &mut node)` pairs in a [`Tree`](crate::Tree).
pub type IterMut<'a, K: 'a + Id, V: 'a> = slotmap::basic::IterMut<'a, K, V>;

/// An iterator over inner values `&V` in a [`Tree`](crate::Tree).
pub type Values<'a, K: 'a + Id, V: 'a> = slotmap::basic::Values<'a, K, V>;

/// A mutable iterator over inner values `&mut V` in a [`Tree`](crate::Tree).
pub type ValuesMut<'a, K: 'a + Id, V: 'a> = slotmap::basic::ValuesMut<'a, K, V>;

/// An iterator over the keys in a [`Tree`](crate::Tree).
pub type Keys<'a, K: 'a + Id, V: 'a> = slotmap::basic::Keys<'a, K, V>;

/// A draining iterator that removes and yields all `(key, node)` pairs.
pub type Drain<'a, K: 'a + Id, V: 'a> = slotmap::basic::Drain<'a, K, V>;

/// An iterator over `&Node` references in a [`Tree`](crate::Tree).
pub type Nodes<'a, K: 'a + Id, V: 'a> = Values<'a, K, V>;

/// A mutable iterator over `&mut Node` references in a [`Tree`](crate::Tree).
pub type NodesMut<'a, K: 'a + Id, V: 'a> = ValuesMut<'a, K, V>;

/// An owning iterator that moves `(key, node)` pairs out of a [`Tree`](crate::Tree).
pub type IntoIter<K: Id, V> = slotmap::basic::IntoIter<K, V>;
