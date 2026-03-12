use super::{TreeId, Tree};
use derive_more::{Deref, DerefMut};
use std::iter::FusedIterator;

// ----------
// ITERATORS
// ----------

#[derive(Clone, Debug)]
struct Iter<'a, K: TreeId, V> {
    pub(super) tree: &'a Tree<K, V>,
    pub(super) node: K,
}

impl<'a, K: TreeId, V> Iter<'a, K, V> {
    pub fn new(tree: &'a Tree<K, V>, node: K) -> Self {
        Self { tree, node }
    }
}

#[derive(Clone, Debug)]
pub struct Ancestors<'a, K: TreeId, V>(Iter<'a, K, V>);

impl<'a, K: TreeId, V> Ancestors<'a, K, V> {
    pub fn new(tree: &'a Tree<K, V>, node: K) -> Self {
        Self(Iter {
            tree,
            node: tree[node].parent,
        })
    }
}

impl<'a, K: TreeId, V> Iterator for Ancestors<'a, K, V> {
    type Item = K;

    fn next(&mut self) -> Option<K> {
        match self.0.node.as_option() {
            Some(node) => {
                self.0.node = self.0.tree[node].parent;
                Some(node)
            }
            None => None,
        }
    }
}

impl<'a, K: TreeId, V> FusedIterator for Ancestors<'a, K, V> {}

#[derive(Clone, Debug)]
pub struct Predecessors<'a, K: TreeId, V>(Iter<'a, K, V>);

impl<'a, K: TreeId, V> Predecessors<'a, K, V> {
    pub fn new(tree: &'a Tree<K, V>, key: K) -> Self {
        Self(Iter { tree, node: key })
    }
}

impl<'a, K: TreeId, V> Iterator for Predecessors<'a, K, V> {
    type Item = K;

    fn next(&mut self) -> Option<K> {
        let key = self.0.node;
        if key.is_null() {
            return None;
        }

        self.0.node = self.0.tree[key]
            .previous_sibling
            .or_else(|| self.0.tree[key].parent);

        Some(key)
    }
}

impl<'a, K: TreeId, V> FusedIterator for Predecessors<'a, K, V> {}

// ----------
// DOUBLE ENDED ITERATORS
// ----------

#[derive(Clone, Debug)]
struct DoubleEndedIter<'a, K: TreeId, V> {
    pub(super) tree: &'a Tree<K, V>,
    pub(super) head: K,
    pub(super) tail: K,
}

impl<'a, K: TreeId, V> DoubleEndedIter<'a, K, V> {
    pub fn new(tree: &'a Tree<K, V>, head: K, tail: K) -> Self {
        Self { tree, head, tail }
    }
}
impl<'a, K: TreeId, V> DoubleEndedIter<'a, K, V> {
    fn advance_head(&mut self, advance: impl FnOnce(&Tree<K, V>, K) -> K) -> Option<K> {
        match (self.head.as_option(), self.tail.as_option()) {
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

    fn advance_tail(&mut self, advance: impl FnOnce(&Tree<K, V>, K) -> K) -> Option<K> {
        match (self.head.as_option(), self.tail.as_option()) {
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

#[derive(Clone, Debug)]
pub struct Children<'a, K: TreeId, V>(DoubleEndedIter<'a, K, V>);

impl<'a, K: TreeId, V> Children<'a, K, V> {
    pub fn new(tree: &'a Tree<K, V>, node: K) -> Self {
        Self(DoubleEndedIter {
            tree,
            head: tree[node].first_child,
            tail: tree[node].last_child,
        })
    }
}

impl<'a, K: TreeId, V> Iterator for Children<'a, K, V> {
    type Item = K;

    fn next(&mut self) -> Option<K> {
        self.0.advance_head(|tree, node| tree[node].next_sibling)
    }
}

impl<'a, K: TreeId, V> DoubleEndedIterator for Children<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        match (self.0.head.as_option(), self.0.tail.as_option()) {
            (Some(head), Some(tail)) if head == tail => {
                let result = head;
                self.0.head = K::null();
                self.0.tail = K::null();
                Some(result)
            }
            (None, Some(tail)) | (Some(_), Some(tail)) => {
                self.0.tail = self.0.tree[tail].previous_sibling;
                Some(tail)
            }
            (Some(_), None) | (None, None) => None,
        }
    }
}

pub struct PrecedingSiblings<'a, K: TreeId, V>(DoubleEndedIter<'a, K, V>);

impl<'a, K: TreeId, V> PrecedingSiblings<'a, K, V> {
    pub fn new(tree: &'a Tree<K, V>, key: K) -> Self {
        Self(DoubleEndedIter {
            tree,
            head: key,
            tail: tree[key]
                .parent
                .and_then(|parent_id| tree[parent_id].first_child),
        })
    }
}

impl<'a, K: TreeId, V> Iterator for PrecedingSiblings<'a, K, V> {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .advance_head(|tree, node| tree[node].previous_sibling)
    }
}

impl<'a, K: TreeId, V> DoubleEndedIterator for PrecedingSiblings<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.advance_tail(|tree, node| tree[node].next_sibling)
    }
}

pub struct FollowingSiblings<'a, K: TreeId, V>(DoubleEndedIter<'a, K, V>);

impl<'a, K: TreeId, V> FollowingSiblings<'a, K, V> {
    pub fn new(tree: &'a Tree<K, V>, key: K) -> Self {
        Self(DoubleEndedIter {
            tree,
            head: key,
            tail: tree[key]
                .parent
                .and_then(|parent_id| tree[parent_id].last_child),
        })
    }
}

impl<'a, K: TreeId, V> Iterator for FollowingSiblings<'a, K, V> {
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.advance_head(|tree, node| tree[node].next_sibling)
    }
}

impl<'a, K: TreeId, V> DoubleEndedIterator for FollowingSiblings<'a, K, V> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0
            .advance_tail(|tree, node| tree[node].previous_sibling)
    }
}

// ----------
// TRAVERSAL
// ----------

#[derive(Clone, Debug)]
pub struct Traverse<'a, K: TreeId, V> {
    tree: &'a Tree<K, V>,
    root: K,
    next: Option<NodeEdge<K>>,
}

impl<'a, K: TreeId, V> Traverse<'a, K, V> {
    pub(crate) fn new(tree: &'a Tree<K, V>, current: K) -> Self {
        Self {
            tree,
            root: current,
            next: Some(NodeEdge::Start(current)),
        }
    }
}

impl<K: TreeId, V> Iterator for Traverse<'_, K, V> {
    type Item = NodeEdge<K>;

    fn next(&mut self) -> Option<NodeEdge<K>> {
        let next = self.next.take()?;

        // Next of next
        self.next = match next {
            NodeEdge::End(key) if key == self.root => None,

            NodeEdge::Start(node) => match self.tree[node].first_child() {
                Some(first_child) => Some(NodeEdge::Start(first_child)),
                None => Some(NodeEdge::End(node)),
            },
            NodeEdge::End(node) => {
                let node = &self.tree[node];
                match node.next_sibling() {
                    Some(next_sibling) => Some(NodeEdge::Start(next_sibling)),
                    // `node.parent()` here can only be `None` if the tree has
                    // been modified during iteration, but silently stoping
                    // iteration seems a more sensible behavior than panicking.
                    None => node.parent().map(NodeEdge::End),
                }
            }
        };

        Some(next)
    }
}

impl<K: TreeId, V> FusedIterator for Traverse<'_, K, V> {}

#[derive(Clone, Debug)]
pub struct ReverseTraverse<'a, K: TreeId, V> {
    tree: &'a Tree<K, V>,
    root: K,
    next: Option<NodeEdge<K>>,
}

impl<'a, K: TreeId, V> ReverseTraverse<'a, K, V> {
    pub(crate) fn new(tree: &'a Tree<K, V>, current: K) -> Self {
        Self {
            tree,
            root: current,
            next: Some(NodeEdge::End(current)),
        }
    }
}

impl<K: TreeId, V> Iterator for ReverseTraverse<'_, K, V> {
    type Item = NodeEdge<K>;

    fn next(&mut self) -> Option<NodeEdge<K>> {
        let next = self.next.take()?;

        // Next of next
        self.next = match next {
            NodeEdge::Start(key) if key == self.root => None,
            NodeEdge::End(node) => match self.tree[node].last_child() {
                Some(last_child) => Some(NodeEdge::End(last_child)),
                None => Some(NodeEdge::Start(node)),
            },
            NodeEdge::Start(node) => {
                let node = &self.tree[node];
                match node.previous_sibling() {
                    Some(previous_sibling) => Some(NodeEdge::End(previous_sibling)),
                    // `node.parent()` here can only be `None` if the tree has
                    // been modified during iteration, but silently stopping
                    // iteration seems a more sensible behavior than panicking.
                    None => node.parent().map(NodeEdge::Start),
                }
            }
        };

        Some(next)
    }
}

impl<K: TreeId, V> FusedIterator for ReverseTraverse<'_, K, V> {}

/// Indicator if the node is at a start or endpoint of the tree
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NodeEdge<K> {
    /// Indicates that start of a node that has children.
    ///
    /// Yielded by `Traverse::next()` before the node’s descendants. In HTML or
    /// XML, this corresponds to an opening tag like `<div>`.
    Start(K),

    /// Indicates that end of a node that has children.
    ///
    /// Yielded by `Traverse::next()` after the node’s descendants. In HTML or
    /// XML, this corresponds to a closing tag like `</div>`
    End(K),
}

impl<K: TreeId> NodeEdge<K> {
    pub fn option(&self) -> NodeEdge<Option<K>> {
        match self {
            NodeEdge::Start(key) => NodeEdge::Start(key.as_option()),
            NodeEdge::End(key) => NodeEdge::End(key.as_option()),
        }
    }
    pub fn key(&self) -> Option<K> {
        match self {
            NodeEdge::Start(key) | NodeEdge::End(key) => Some(*key),
        }
    }
}

impl<K: TreeId> PartialEq<K> for NodeEdge<K> {
    fn eq(&self, other: &K) -> bool {
        match self {
            NodeEdge::Start(key) | NodeEdge::End(key) => key == other,
        }
    }
}

#[derive(Clone, Deref, DerefMut)]
pub struct Descendants<'a, K: TreeId, V>(Traverse<'a, K, V>);

impl<'a, K: TreeId, V> Descendants<'a, K, V> {
    pub fn new(tree: &'a Tree<K, V>, node: K) -> Self {
        Self(Traverse::new(tree, node))
    }
}

impl<'a, K: TreeId, V> Iterator for Descendants<'a, K, V> {
    type Item = K;

    fn next(&mut self) -> Option<K> {
        self.0.find_map(|edge| match edge {
            NodeEdge::Start(node) => Some(node),
            NodeEdge::End(_) => None,
        })
    }
}

impl<'a, K: TreeId, V> FusedIterator for Descendants<'a, K, V> {}
