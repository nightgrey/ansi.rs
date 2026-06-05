#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum BitsError {
    #[error("empty")]
    Empty,
    #[error("unknown")]
    Unknown,
    #[error("invalid")]
    Invalid,
}
