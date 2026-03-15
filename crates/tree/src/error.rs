use thiserror::Error;

/// Errors returned by fallible tree operations.
#[derive(Error, Debug)]
pub enum Error<K> {
    /// The referenced node does not exist in the tree.
    #[error("Node {0} does not exist")]
    Missing(K),

    /// A sibling-relative operation (e.g. [`At::Before`](crate::At::Before))
    /// requires the reference node to have a parent, but it is a root.
    #[error("Reference node {0} has no parent")]
    NoParent(K),

    /// Moving `node` to `target` would create a cycle (the target is a
    /// descendant of the node being moved).
    #[error("Cycle detected: node {node} would be its own ancestor")]
    Cycle { node: K, target: K },

    /// The operation is not permitted (e.g. removing the root of a
    /// [`RootTree`](crate::RootTree)).
    #[error("Operation forbidden")]
    OperationForbidden,
}
