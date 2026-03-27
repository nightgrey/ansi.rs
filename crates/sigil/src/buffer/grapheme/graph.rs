use crate::Grapheme;
use std::fmt;
use std::ops::Deref;

/// A resolved grapheme cluster — the result of reading a [`Grapheme`] handle.
///
/// For inline graphemes the data lives on the stack; for extended graphemes
/// it borrows from the [`GraphemePool`]. Use `.as_str()` or the [`Deref`]
/// impl for uniform `&str` access.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Graph<'a> {
    /// No grapheme.
    None,

    /// Inline UTF-8 data (≤4 bytes).
    Inline { bytes: [u8; 4], len: u8 },

    /// A reference into the [`GraphemePool`].
    Extended(&'a str),
}

impl Graph<'_> {
    pub const EMPTY: Self = Self::None;

    pub fn empty() -> Self {
        Self::None
    }

    /// View this resolved grapheme as a string slice.
    pub fn as_str(&self) -> &str {
        match self {
            Self::None => "",
            Self::Inline { bytes, len } => {
                // SAFETY: We only store valid UTF-8 via Grapheme::from_inline_bytes.
                unsafe { std::str::from_utf8_unchecked(&bytes[..*len as usize]) }
            }
            Self::Extended(s) => s,
        }
    }

    /// The byte length of the grapheme cluster.
    pub fn len(&self) -> usize {
        match self {
            Self::None => 0,
            Self::Inline { len, .. } => *len as usize,
            Self::Extended(s) => s.len(),
        }
    }

    /// Returns `true` if this is the empty grapheme (no content).
    pub fn is_empty(&self) -> bool {
        match self {
            Self::None => true,
            Self::Inline { len, .. } => *len == 0,
            Self::Extended(s) => s.is_empty(),
        }
    }
}

impl Deref for Graph<'_> {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for Graph<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for Graph<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Graph({:?})", self.as_str())
    }
}

impl<T: AsRef<str>> PartialEq<T> for Graph<'_> {
    fn eq(&self, other: &T) -> bool {
        self.as_str() == other.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GraphemeArena;

    #[test]
    fn graph_copy() {
        let pool = GraphemeArena::new();
        let g = Grapheme::char('A').as_graph(&pool);
        let g2 = g; // Copy
        assert_eq!(g, g2);
    }

    #[test]
    fn graph_eq() {
        let pool = GraphemeArena::new();
        let g = Grapheme::char('A').as_graph(&pool);

        // Graph == str
        assert_eq!(g, "A");
        // Graph == &str
        assert_eq!(g, "A");
    }

    #[test]
    fn graph_is_empty_covers_all_variants() {
        let pool = GraphemeArena::new();
        assert!(Graph::None.is_empty());
        assert!(
            Graph::Inline {
                bytes: [0; 4],
                len: 0
            }
            .is_empty()
        );
        assert!(!Grapheme::char('X').as_graph(&pool).is_empty());
    }
}
