use thiserror::Error;
use std::fmt;

use super::GraphemePool;

/// A compact grapheme-cluster handle stored in 4 bytes.
///
/// This is the core unit of text storage in the framebuffer. It uses a
/// dual-mode encoding inspired by notcurses' `nccell.gcluster`:
///
/// - **Inline**: grapheme clusters that fit in ≤4 UTF-8 bytes are stored
///   directly in the `u32`, with unused trailing bytes zeroed. This covers
///   ASCII, Latin, Cyrillic, CJK, and most emoji codepoints — zero heap
///   allocation for the common case.
///
/// - **Extended**: grapheme clusters exceeding 4 bytes (emoji sequences with
///   ZWJ, skin-tone modifiers, etc.) are stored in a [`GraphemePool`] and
///   referenced by a 24-bit offset. The high byte is set to `0x01` as a
///   marker, giving 16 MiB of addressable pool space.
///
/// # Encoding (little-endian byte interpretation of the `u32`)
///
/// ```text
/// bytes[0..=3] with bytes[3] != 0x01  →  inline UTF-8 (up to 4 bytes)
/// bytes[3] == 0x01                     →  extended; bytes[0..=2] = pool offset
/// all zeros                            →  empty (no grapheme)
/// ```
///
/// The marker byte `0x01` (SOH) can never appear as the first byte of a valid
/// UTF-8 sequence in byte position 3 — continuation bytes are `0x80..=0xBF`,
/// and 4-byte leading bytes are `0xF0..=0xF7` — so the encoding is unambiguous.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Grapheme(u32);

/// The sentinel value marking an extended (pool-stored) grapheme.
/// Occupies the high byte of the little-endian `u32` representation.
const EXTENDED_MARKER: u32 = 0x01_00_00_00;

/// Mask for the 24-bit pool offset in an extended grapheme.
const OFFSET_MASK: u32 = 0x00_FF_FF_FF;

impl Grapheme {
    /// An empty grapheme (no character). This is the default for blank cells.
    pub const EMPTY: Self = Self(0);

    /// Create a grapheme from a string slice.
    ///
    /// If the string fits in 4 UTF-8 bytes, it is stored inline (no pool
    /// interaction). Otherwise, it is stashed in the given [`GraphemePool`].
    ///
    /// # Panics
    ///
    /// Panics if the string exceeds 4 bytes and the pool is full (16 MiB).
    /// Use [`try_new`](Self::try_new) for the fallible variant.
    pub fn new(s: &str, pool: &mut GraphemePool) -> Self {
        Self::try_new(s, pool).expect("grapheme pool is full")
    }

    /// Fallible version of [`new`](Self::new).
    ///
    /// Returns `Err` only if the string exceeds 4 bytes and the pool cannot
    /// allocate space for it.
    pub fn try_new(s: &str, pool: &mut GraphemePool) -> Result<Self, GraphemePoolError> {
        let bytes = s.as_bytes();

        if bytes.is_empty() {
            return Ok(Self::EMPTY);
        }

        if bytes.len() <= 4 {
            Ok(Self::from_inline_bytes(bytes))
        } else {
            let offset = pool.stash(s)?;
            Ok(Self::from_pool_offset(offset))
        }
    }

    /// Try to create an inline grapheme without a pool.
    ///
    /// Returns `None` if the string exceeds 4 UTF-8 bytes.
    pub fn try_inline(s: &str) -> Option<Self> {
        let bytes = s.as_bytes();
        if bytes.is_empty() {
            Some(Self::EMPTY)
        } else if bytes.len() <= 4 {
            Some(Self::from_inline_bytes(bytes))
        } else {
            None
        }
    }

    /// Create an inline grapheme from a single `char`.
    ///
    /// Every Unicode scalar value encodes to at most 4 UTF-8 bytes, so this
    /// always succeeds and never needs a pool.
    pub fn from_char(c: char) -> Self {
        let mut buf = [0u8; 4];
        let s = c.encode_utf8(&mut buf);
        Self::from_inline_bytes(s.as_bytes())
    }

    /// Returns `true` if this grapheme is empty (no character).
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns `true` if this grapheme is stored inline (≤4 UTF-8 bytes).
    #[inline]
    pub const fn is_inline(self) -> bool {
        !self.is_extended()
    }

    /// Returns `true` if this grapheme is stored in a [`GraphemePool`].
    #[inline]
    pub const fn is_extended(self) -> bool {
        // The high byte (bits 24..=31) equals 0x01 for extended graphemes.
        // This works regardless of native endianness because we construct
        // the u32 value arithmetically, not via byte reinterpretation.
        (self.0 & 0xFF_00_00_00) == EXTENDED_MARKER
    }

    /// Resolve this grapheme to a [`Graph`] for reading.
    ///
    /// The returned value borrows from the pool (for extended graphemes) or
    /// holds inline data (for inline graphemes). Call `.as_str()` to get a `&str`.
    ///
    /// # Example
    ///
    /// ```
    /// # use sigil::{Grapheme, GraphemePool};
    /// let mut pool = GraphemePool::new();
    /// let g = Grapheme::from_char('A');
    /// let resolved = g.resolve(&pool);
    /// assert_eq!(resolved.as_str(), "A");
    /// ```
    pub fn resolve<'a>(&self, pool: &'a GraphemePool) -> Graph<'a> {
        if self.is_empty() {
            Graph::Empty
        } else if self.is_inline() {
            let bytes = self.inline_le_bytes();
            let len = inline_utf8_len(&bytes);
            Graph::Inline { bytes, len }
        } else {
            let s = pool.resolve(self.pool_offset());
            Graph::Extended(s)
        }
    }

    /// Execute a closure with the resolved `&str`, avoiding the intermediate
    /// [`Graph`] allocation.
    ///
    /// This is the most efficient way to read a grapheme when you only need
    /// a brief, non-escaping access to the string data.
    pub fn with_str<R>(&self, pool: &GraphemePool, f: impl FnOnce(&str) -> R) -> R {
        if self.is_empty() {
            f("")
        } else if self.is_inline() {
            let bytes = self.inline_le_bytes();
            let len = inline_utf8_len(&bytes);
            // SAFETY: We only store valid UTF-8 via `from_inline_bytes`.
            let s = unsafe { std::str::from_utf8_unchecked(&bytes[..len as usize]) };
            f(s)
        } else {
            f(pool.resolve(self.pool_offset()))
        }
    }

    /// Release any pool storage held by this grapheme.
    ///
    /// Must be called before overwriting a cell's grapheme with a new value,
    /// otherwise the pool entry leaks. No-op for inline and empty graphemes.
    pub fn release(self, pool: &mut GraphemePool) {
        if self.is_extended() {
            pool.release(self.pool_offset());
        }
    }

    // ── Internal constructors ──────────────────────────────────────────

    /// Pack up to 4 UTF-8 bytes into a `u32` via little-endian interpretation.
    /// Unused trailing bytes are zero, and the high byte cannot be `0x01` for
    /// any valid UTF-8 input ≤4 bytes.
    fn from_inline_bytes(bytes: &[u8]) -> Self {
        debug_assert!(bytes.len() <= 4 && !bytes.is_empty());
        let mut buf = [0u8; 4];
        buf[..bytes.len()].copy_from_slice(bytes);
        Self(u32::from_le_bytes(buf))
    }

    /// Create an extended grapheme from a pool offset.
    fn from_pool_offset(offset: usize) -> Self {
        debug_assert!(offset <= OFFSET_MASK as usize);
        Self(EXTENDED_MARKER | (offset as u32 & OFFSET_MASK))
    }

    /// Get the raw little-endian bytes of an inline grapheme.
    fn inline_le_bytes(self) -> [u8; 4] {
        self.0.to_le_bytes()
    }

    /// Get the pool offset of an extended grapheme.
    fn pool_offset(self) -> usize {
        (self.0 & OFFSET_MASK) as usize
    }
}

impl Default for Grapheme {
    #[inline]
    fn default() -> Self {
        Self::EMPTY
    }
}

impl fmt::Debug for Grapheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            f.write_str("Grapheme(EMPTY)")
        } else if self.is_inline() {
            let bytes = self.inline_le_bytes();
            let len = inline_utf8_len(&bytes);
            let s = std::str::from_utf8(&bytes[..len as usize]).unwrap_or("<invalid>");
            write!(f, "Grapheme({s:?})")
        } else {
            write!(f, "Grapheme(pool@{})", self.pool_offset())
        }
    }
}

// ── Graph ────────────────────────────────────────────────────────────

/// A resolved grapheme cluster — the result of reading a [`Grapheme`] handle.
///
/// For inline graphemes the data lives on the stack; for extended graphemes
/// it borrows from the [`GraphemePool`]. Use `.as_str()` for uniform access.
pub enum Graph<'a> {
    /// No grapheme.
    Empty,

    /// Inline UTF-8 data (≤4 bytes).
    Inline { bytes: [u8; 4], len: u8 },

    /// A reference into the [`GraphemePool`].
    Extended(&'a str),
}

impl Graph<'_> {
    /// View this resolved grapheme as a string slice.
    ///
    /// For [`Inline`](Graph::Inline) variants, the returned reference
    /// borrows from this `Graph` value, so bind it to a variable first:
    ///
    /// ```
    /// # use sigil::{Grapheme, GraphemePool};
    /// # let pool = GraphemePool::new();
    /// # let grapheme = Grapheme::from_char('X');
    /// let resolved = grapheme.resolve(&pool);
    /// let s: &str = resolved.as_str();
    /// ```
    pub fn as_str(&self) -> &str {
        match self {
            Self::Empty => "",
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
            Self::Empty => 0,
            Self::Inline { len, .. } => *len as usize,
            Self::Extended(s) => s.len(),
        }
    }

    /// Returns `true` if this is the empty grapheme.
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
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

impl PartialEq<str> for Graph<'_> {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl<'a> PartialEq<&'a str> for Graph<'_> {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_str() == *other
    }
}

// ── GraphemePoolError::Full ───────────────────────────────────────────────────────

#[derive(Error, Debug)]
pub enum GraphemePoolError {
    /// Error returned when the [`GraphemePool`] cannot allocate space for an
    /// extended grapheme cluster (pool has reached its 16 MiB limit with no
    /// reclaimable gaps).
    #[error("grapheme pool is full (16 MiB limit reached)")]
    Full,
    #[error("unknown error")]
    Unknown,
}

// ── Helpers ────────────────────────────────────────────────────────────────

/// Determine the byte length of an inline UTF-8 grapheme stored as `[u8; 4]`.
///
/// Scans for the first zero byte. Since UTF-8 continuation bytes are always
/// `0x80..=0xBF`, a zero byte can only appear as a NUL character (which we
/// treat as empty) or as padding after the grapheme data.
#[inline]
fn inline_utf8_len(bytes: &[u8; 4]) -> u8 {
    // Use position-of-zero rather than leading-byte analysis so we correctly
    // handle multi-codepoint grapheme clusters (e.g., base + combining mark)
    // that happen to fit in ≤4 bytes.
    bytes.iter().position(|&b| b == 0).unwrap_or(4) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_grapheme() {
        let g = Grapheme::EMPTY;
        assert!(g.is_empty());
        assert!(g.is_inline());
        assert!(!g.is_extended());
    }

    #[test]
    fn inline_ascii() {
        let g = Grapheme::from_char('A');
        assert!(!g.is_empty());
        assert!(g.is_inline());

        let pool = GraphemePool::new();
        assert_eq!(g.resolve(&pool), "A");
    }

    #[test]
    fn inline_multibyte() {
        let pool = GraphemePool::new();

        // 2-byte: Latin é (U+00E9)
        let g = Grapheme::try_inline("é").unwrap();
        assert!(g.is_inline());
        assert_eq!(g.resolve(&pool), "é");

        // 3-byte: CJK 中 (U+4E2D)
        let g = Grapheme::try_inline("中").unwrap();
        assert!(g.is_inline());
        assert_eq!(g.resolve(&pool), "中");
    }

    #[test]
    fn inline_four_byte_emoji() {
        let pool = GraphemePool::new();
        // 4-byte: party popper 🎉 (U+1F389) = F0 9F 8E 89
        let g = Grapheme::from_char('🎉');
        assert!(g.is_inline());
        assert_eq!(g.resolve(&pool), "🎉");
    }

    #[test]
    fn inline_combining_marks() {
        let pool = GraphemePool::new();
        // e + combining acute accent = 3 bytes, fits inline
        let s = "e\u{0301}"; // é as decomposed
        assert_eq!(s.len(), 3);
        let g = Grapheme::try_inline(s).unwrap();
        assert!(g.is_inline());
        assert_eq!(g.resolve(&pool), s);
    }

    #[test]
    fn extended_emoji_sequence() {
        let mut pool = GraphemePool::new();
        // Family emoji: 👨‍👩‍👧‍👦 = 25 bytes
        let family = "👨\u{200D}👩\u{200D}👧\u{200D}👦";
        assert!(family.len() > 4);

        let g = Grapheme::new(family, &mut pool);
        assert!(!g.is_empty());
        assert!(g.is_extended());
        assert_eq!(g.resolve(&pool), family);
    }

    #[test]
    fn try_inline_rejects_long() {
        let family = "👨\u{200D}👩\u{200D}👧\u{200D}👦";
        assert!(Grapheme::try_inline(family).is_none());
    }

    #[test]
    fn release_frees_pool_space() {
        let mut pool = GraphemePool::new();
        let family = "👨\u{200D}👩\u{200D}👧\u{200D}👦";

        let used_before = pool.used();
        let g = Grapheme::new(family, &mut pool);
        let used_after_insert = pool.used();
        assert!(used_after_insert > used_before);

        g.release(&mut pool);
        assert_eq!(pool.used(), used_before);
    }

    #[test]
    fn with_str_callback() {
        let mut pool = GraphemePool::new();
        let g = Grapheme::from_char('X');
        let result = g.with_str(&pool, |s| s.to_uppercase());
        assert_eq!(result, "X");

        let family = "👨\u{200D}👩\u{200D}👧\u{200D}👦";
        let g2 = Grapheme::new(family, &mut pool);
        g2.with_str(&pool, |s| {
            assert_eq!(s, family);
        });
    }

    #[test]
    fn debug_output() {
        let g = Grapheme::from_char('Z');
        let dbg = format!("{g:?}");
        assert!(dbg.contains("Z"));

        let g2 = Grapheme::EMPTY;
        let dbg2 = format!("{g2:?}");
        assert!(dbg2.contains("EMPTY"));
    }
}