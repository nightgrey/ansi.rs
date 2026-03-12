use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error<K> {
    #[error("Node {0} does not exist")]
    Missing(K),
    #[error("Reference node {0} has no parent")]
    NoParent(K),
    #[error("Cycle detected: node {node} would be its own ancestor")]
    Cycle { node: K, target: K },
    #[error("Operation forbidden")]
    OperationForbidden,
}

