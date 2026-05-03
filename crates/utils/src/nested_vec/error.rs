use thiserror::Error;

/// Error type for fallible nested structure operations.
#[derive(Error, Debug)]
pub enum NestedError {
    #[error("Overflow")]
    Overflow,
}
