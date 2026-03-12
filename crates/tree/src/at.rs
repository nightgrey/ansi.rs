

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum At<K> {
    /// Insert the node as a detached node.
    Detached,

    /// Insert the node as the first child of the given ID.
    Prepend(K),
    /// Insert the node as the last child of the given ID.
    Append(K),
    
    /// Alias for `Self::Append`.
    Child(K),

    /// Insert the node as a sibling before the node with the given ID.
    Before(K),
    /// Insert the node as a sibling after the node with the given ID.
    After(K),
}

