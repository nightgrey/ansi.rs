use super::NodeEdge;
use crate::{Id, Tree};
use derive_more::{Deref, DerefMut};
use std::iter::FusedIterator;

/// Single-cursor base iterator used internally by unidirectional traversals.
#[derive(Clone, Debug)]
struct Iter<'a, K: 'a + Id, V: 'a> {
    pub(super) tree: &'a Tree<K, V>,
    pub(super) node: K,
}

impl<'a, K: 'a + Id, V: 'a> Iter<'a, K, V> {
    pub fn new(tree: &'a Tree<K, V>, node: K) -> Self {
        Self { tree, node }
    }
}

/// Dual-cursor base iterator used internally by double-ended traversals.
#[derive(Clone, Debug)]
struct DoubleEndedIter<'a, K: 'a + Id, V: 'a> {
    pub(super) tree: &'a Tree<K, V>,
    pub(super) head: K,
    pub(super) tail: K,
}

impl<'a, K: 'a + Id, V: 'a> DoubleEndedIter<'a, K, V> {
    pub fn new(tree: &'a Tree<K, V>, head: K, tail: K) -> Self {
        Self { tree, head, tail }
    }
}
impl<'a, K: 'a + Id, V: 'a> DoubleEndedIter<'a, K, V> {
    fn next_head(&mut self, advance: impl FnOnce(&Tree<K, V>, K) -> K) -> Option<K> {
        match (self.head.maybe(), self.tail.maybe()) {
            (Some(head), Some(tail)) if head == tail => {
                self.head = K::null();
                self.tail = K::null();
                Some(head)
            }
            (Some(h), _) => {
                self.head = advance(self.tree, h);
                Some(h)
            }
            _ => None,
        }
    }

    fn next_tail(&mut self, advance: impl FnOnce(&Tree<K, V>, K) -> K) -> Option<K> {
        match (self.head.maybe(), self.tail.maybe()) {
            (Some(h), Some(t)) if h == t => {
                self.head = K::null();
                self.tail = K::null();
                Some(h)
            }
            (_, Some(t)) => {
                self.tail = advance(self.tree, t);
                Some(t)
            }
            _ => None,
        }
    }
}

/// Iterates upward from a node through its ancestors toward the root.
///
/// Does **not** include the starting node itself — only its parent, grandparent, etc.
///
/// Created by [`Tree::ancestors`](crate::Tree::ancestors).
#[derive(Clone, Debug)]
pub struct Ancestors<'a, K: 'a + Id, V: 'a>(Iter<'a, K, V>);

impl<'a, K: 'a + Id, V: 'a> Ancestors<'a, K, V> {
    pub fn new(tree: &'a Tree<K, V>, node: K) -> Self {
        Self(Iter {
            tree,
            node: tree[node].parent(),
        })
    }
}

impl<'a, K: 'a + Id, V: 'a> Iterator for Ancestors<'a, K, V> {
    type Item = K;

    fn next(&mut self) -> Option<K> {
        match self.0.node.maybe() {
            Some(node) => {
                self.0.node = self.0.tree[node].parent();
                Some(node)
            }
            None => None,
        }
    }
}

impl<'a, K: 'a + Id, V: 'a> FusedIterator for Ancestors<'a, K, V> {}

/// Iterates over a node, then its preceding siblings, then up through ancestors.
///
/// This is the traversal order you would encounter when walking backward
/// through a flattened, pre-order representation of the tree.
///
/// Created by [`Tree::predecessors`](crate::Tree::predecessors).
#[derive(Clone, Debug)]
pub struct Predecessors<'a, K: 'a + Id, V: 'a>(Iter<'a, K, V>);

impl<'a, K: 'a + Id, V: 'a> Predecessors<'a, K, V> {
    pub fn new(tree: &'a Tree<K, V>, key: K) -> Self {
        Self(Iter { tree, node: key })
    }
}

impl<'a, K: 'a + Id, V: 'a> Iterator for Predecessors<'a, K, V> {
    type Item = K;

    fn next(&mut self) -> Option<K> {
        let key = self.0.node;
        if key.is_null() {
            return None;
        }

        self.0.node = self.0.tree[key]
            .prev_sibling()
            .or_else(|| self.0.tree[key].parent());

        Some(key)
    }
}

impl<'a, K: 'a + Id, V: 'a> FusedIterator for Predecessors<'a, K, V> {}

/// A double-ended iterator over the direct children of a node.
///
/// Iterates from first child to last child (`next`), or last to first
/// (`next_back`).
///
/// Created by [`Tree::children`](crate::Tree::children).
#[derive(Clone, Debug)]
pub struct Children<'a, K: 'a + Id, V: 'a>(DoubleEndedIter<'a, K, V>);

impl<'a, K: 'a + Id, V: 'a> Children<'a, K, V> {
    pub fn new(tree: &'a Tree<K, V>, node: K) -> Self {
        Self(DoubleEndedIter {
            tree,
            head: tree[node].first_child(),
            tail: tree[node].last_child(),
        })
    }
}

impl<'a, K: 'a + Id, V: 'a> Iterator for Children<'a, K, V> {
    type Item = K;

    fn next(&mut self) -> Option<K> {
        self.0.next_head(|tree, node| tree[node].next_sibling())
    }
}

impl<'a, K: 'a + Id, V: 'a> DoubleEndedIterator for Children<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match (self.0.head.maybe(), self.0.tail.maybe()) {
            (Some(head), Some(tail)) if head == tail => {
                let result = head;
                self.0.head = K::null();
                self.0.tail = K::null();
                Some(result)
            }
            (None, Some(tail)) | (Some(_), Some(tail)) => {
                self.0.tail = self.0.tree[tail].prev_sibling();
                Some(tail)
            }
            (Some(_), None) | (None, None) => None,
        }
    }
}

/// A double-ended iterator over a node and its preceding (older) siblings.
///
/// Starts at the given node and walks backward toward the first child.
/// `next_back` walks forward from the first child.
///
/// Created by [`Tree::preceding_siblings`](crate::Tree::preceding_siblings).
#[derive(Clone, Debug)]
pub struct PrecedingSiblings<'a, K: 'a + Id, V: 'a>(DoubleEndedIter<'a, K, V>);

impl<'a, K: 'a + Id, V: 'a> PrecedingSiblings<'a, K, V> {
    pub fn new(tree: &'a Tree<K, V>, key: K) -> Self {
        Self(DoubleEndedIter {
            tree,
            head: key,
            tail: tree[key]
                .parent()
                .and_then(|parent_id| tree[parent_id].first_child()),
        })
    }
}

impl<'a, K: 'a + Id, V: 'a> Iterator for PrecedingSiblings<'a, K, V> {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next_head(|tree, node| tree[node].prev_sibling())
    }
}

impl<'a, K: 'a + Id, V: 'a> DoubleEndedIterator for PrecedingSiblings<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_tail(|tree, node| tree[node].next_sibling())
    }
}

/// A double-ended iterator over a node and its following (younger) siblings.
///
/// Starts at the given node and walks forward toward the last child.
/// `next_back` walks backward from the last child.
///
/// Created by [`Tree::following_siblings`](crate::Tree::following_siblings).
#[derive(Clone, Debug)]
pub struct FollowingSiblings<'a, K: 'a + Id, V: 'a>(DoubleEndedIter<'a, K, V>);

impl<'a, K: 'a + Id, V: 'a> FollowingSiblings<'a, K, V> {
    pub fn new(tree: &'a Tree<K, V>, key: K) -> Self {
        Self(DoubleEndedIter {
            tree,
            head: key,
            tail: tree[key]
                .parent()
                .and_then(|parent_id| tree[parent_id].last_child()),
        })
    }
}

impl<'a, K: 'a + Id, V: 'a> Iterator for FollowingSiblings<'a, K, V> {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next_head(|tree, node| tree[node].next_sibling())
    }
}

impl<'a, K: 'a + Id, V: 'a> DoubleEndedIterator for FollowingSiblings<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_tail(|tree, node| tree[node].prev_sibling())
    }
}

/// Pre-order tree traversal yielding [`NodeEdge`] events.
///
/// For each node, yields [`NodeEdge::Start`] before visiting its children and
/// [`NodeEdge::End`] after all descendants have been visited.
///
/// ```text
/// Start(A) → Start(B) → End(B) → Start(C) → End(C) → End(A)
/// ```
///
/// Created by [`Tree::traverse`](crate::Tree::traverse).
#[derive(Clone, Debug)]
pub struct Traverse<'a, K: 'a + Id, V: 'a> {
    tree: &'a Tree<K, V>,
    root: K,
    next: Option<NodeEdge<K>>,
}

impl<'a, K: 'a + Id, V: 'a> Traverse<'a, K, V> {
    pub(crate) fn new(tree: &'a Tree<K, V>, current: K) -> Self {
        Self {
            tree,
            root: current,
            next: Some(NodeEdge::Start(current)),
        }
    }
}

impl<K: Id, V> Iterator for Traverse<'_, K, V> {
    type Item = NodeEdge<K>;

    fn next(&mut self) -> Option<NodeEdge<K>> {
        let next = self.next.take()?;

        // Next of next
        self.next = match next {
            NodeEdge::End(key) if key == self.root => None,

            NodeEdge::Start(node) => match self.tree[node].first_child().maybe() {
                Some(first_child) => Some(NodeEdge::Start(first_child)),
                None => Some(NodeEdge::End(node)),
            },
            NodeEdge::End(node) => {
                let node = &self.tree[node];
                match node.next_sibling().maybe() {
                    Some(next_sibling) => Some(NodeEdge::Start(next_sibling)),
                    // `node.parent()` here can only be `None` if the tree has
                    // been modified during iteration, but silently stopping
                    // iteration seems a more sensible behavior than panicking.
                    None => node.parent().maybe().map(NodeEdge::End),
                }
            }
        };

        Some(next)
    }
}

impl<K: Id, V> FusedIterator for Traverse<'_, K, V> {}

/// Reverse (post-order) tree traversal yielding [`NodeEdge`] events.
///
/// The mirror image of [`Traverse`]: visits nodes from last descendant back
/// to the root.
///
/// Created by [`Tree::reverse_traverse`](crate::Tree::reverse_traverse).
#[derive(Clone, Debug)]
pub struct ReverseTraverse<'a, K: 'a + Id, V: 'a> {
    tree: &'a Tree<K, V>,
    root: K,
    next: Option<NodeEdge<K>>,
}

impl<'a, K: 'a + Id, V: 'a> ReverseTraverse<'a, K, V> {
    pub(crate) fn new(tree: &'a Tree<K, V>, current: K) -> Self {
        Self {
            tree,
            root: current,
            next: Some(NodeEdge::End(current)),
        }
    }
}

impl<K: Id, V> Iterator for ReverseTraverse<'_, K, V> {
    type Item = NodeEdge<K>;

    fn next(&mut self) -> Option<NodeEdge<K>> {
        let next = self.next.take()?;

        // Next of next
        self.next = match next {
            NodeEdge::Start(id) if id == self.root => None,
            NodeEdge::End(id) => match self.tree[id].last_child().maybe() {
                Some(last_child) => Some(NodeEdge::End(last_child)),
                None => Some(NodeEdge::Start(id)),
            },
            NodeEdge::Start(id) => {
                let node = &self.tree[id];
                node.prev_sibling().map_or_else(
                    || node.parent().map(NodeEdge::Start),
                    |id| Some(NodeEdge::End(id)),
                )
            }
        };

        Some(next)
    }
}

impl<K: Id, V> FusedIterator for ReverseTraverse<'_, K, V> {}

/// Iterates over a node and all its descendants in pre-order (depth-first).
///
/// Unlike [`Traverse`], this iterator yields only keys (`K`) rather than
/// [`NodeEdge`] events — it skips the `End` events and returns just the
/// `Start` keys.
///
/// Created by [`Tree::descendants`](crate::Tree::descendants).
#[derive(Clone, Deref, DerefMut)]
pub struct Descendants<'a, K: 'a + Id, V: 'a>(Traverse<'a, K, V>);

impl<'a, K: 'a + Id, V: 'a> Descendants<'a, K, V> {
    pub fn new(tree: &'a Tree<K, V>, node: K) -> Self {
        Self(Traverse::new(tree, node))
    }
}

impl<'a, K: 'a + Id, V: 'a> Iterator for Descendants<'a, K, V> {
    type Item = K;

    fn next(&mut self) -> Option<K> {
        self.0.find_map(|edge| match edge {
            NodeEdge::Start(node) => Some(node),
            NodeEdge::End(_) => None,
        })
    }
}

impl<'a, K: 'a + Id, V: 'a> FusedIterator for Descendants<'a, K, V> {}
