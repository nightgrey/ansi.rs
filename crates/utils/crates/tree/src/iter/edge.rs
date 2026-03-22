use crate::Id;

/// Represents the entry or exit of a node during tree traversal.
///
/// Used by [`Traverse`](super::Traverse) and
/// [`ReverseTraverse`](super::ReverseTraverse) to signal whether the iterator
/// is *entering* a subtree or *leaving* it — analogous to opening and closing
/// tags in HTML/XML.
///
/// ```text
/// Start(root)        <!-- <root> -->
///   Start(child)     <!--   <child> -->
///   End(child)       <!--   </child> -->
/// End(root)          <!-- </root> -->
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NodeEdge<K> {
    /// The traversal is entering the node (pre-order position).
    ///
    /// Yielded *before* the node's descendants. Corresponds to an opening
    /// tag like `<div>`.
    Start(K),

    /// The traversal is leaving the node (post-order position).
    ///
    /// Yielded *after* all of the node's descendants. Corresponds to a
    /// closing tag like `</div>`.
    End(K),
}

impl<K: Id> NodeEdge<K> {
    /// Returns the inner key regardless of the edge variant.
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
