use crate::Id;

/// Node edge
///
/// Indicates the nodes's position in the tree.
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

impl<K: Id> NodeEdge<K> {
    pub fn into_inner(self) -> K {
        match self {
            NodeEdge::Start(key) | NodeEdge::End(key) => key,
        }
    }
}

impl<K: Id> PartialEq<K> for NodeEdge<K> {
    fn eq(&self, other: &K) -> bool {
        match self {
            NodeEdge::Start(key) | NodeEdge::End(key) => key == other,
        }
    }
}
