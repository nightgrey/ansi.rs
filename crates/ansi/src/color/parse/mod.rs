use std::io;
use thiserror::Error;

/// Parse error with byte offset information.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ParseError {
    #[error("The input is empty")]
    Empty,
    #[error("Unknown color format prefix")]
    UnknownFormat,
    #[error("invalid hex value at offset {offset}")]
    InvalidHex { offset: usize },
    #[error("invalid # color length: {len} (expected 3, 6, 9, or 12)")]
    InvalidSharpLength { len: usize },
    #[error("Missing color component")]
    MissingComponent,
    #[error("Too many color components")]
    TooManyComponents,
    #[error("invalid floating-point value at offset {offset}")]
    InvalidFloat { offset: usize },
    #[error("Floating-point value out of range")]
    OutOfRange,
}

/// Error type for encoding X11 color specifications.
#[derive(Debug, Error)]
pub enum EncodeError {
    #[error("Buffer overflow - not enough space to write encoded data.")]
    BufferOverflow(usize),
    #[error("I/O error during encoding.")]
    IoError(io::Error),
}
