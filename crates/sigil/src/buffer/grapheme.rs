use std::fmt;
use std::ops::Deref;
use derive_more::Deref;
use super::{GraphemePool, GraphemePoolError};

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
/// The marker byte `0x01` (SOH) can never appear as the *high* byte of an
/// inline grapheme's little-endian representation. For multi-byte UTF-8: byte
/// position 3 holds either a continuation byte (`0x80..=0xBF`) or a 4-byte
/// leading byte (`0xF0..=0xF7`). For single-byte values like SOH itself
/// (`0x01`), only byte 0 is non-zero while byte 3 remains `0x00`. The encoding
/// is therefore unambiguous — unlike notcurses, SOH *can* be stored inline.
///
/// # NUL handling
///
/// NUL bytes (`0x00`) are used as padding in inline graphemes, so a standalone
/// NUL character cannot be distinguished from empty. `Grapheme::from_char('\0')`
/// produces `Grapheme::EMPTY`. This is intentional for terminal framebuffers
/// where NUL has no visual representation.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Deref)]
#[repr(transparent)]
pub struct Grapheme {
    value: u32,
}

impl Grapheme {
    /// An empty grapheme (no character). This is the default for blank cells.
    pub const EMPTY: Self = Self { value: 0 };
    /// The sentinel value marking an extended (pool-stored) grapheme.
    /// Occupies the high byte of the little-endian `u32` representation.
    pub const EXTENDED_MARKER: u32 = 0x01_00_00_00;
    /// Mask for the 24-bit pool offset in an extended grapheme.
    pub const OFFSET_MASK: u32 = 0x00_FF_FF_FF;

    /// Create an extended grapheme from a pool offset.
    pub unsafe fn from_offset(offset: usize) -> Self {
        debug_assert!(offset <= Self::OFFSET_MASK as usize);
        Self { value: Self::EXTENDED_MARKER | (offset as u32 & Self::OFFSET_MASK) }
    }

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
            Ok(pool.stash(s)?)
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
    ///
    /// Note: `'\0'` produces [`EMPTY`](Self::EMPTY) since NUL is
    /// indistinguishable from padding in the inline encoding.
    pub fn from_char(c: char) -> Self {
        let mut buf = [0u8; 4];
        let s = c.encode_utf8(&mut buf);
        Self::from_inline_bytes(s.as_bytes())
    }

    /// Returns `true` if this grapheme is empty (no character).
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.value == 0
    }

    /// Returns `true` if this grapheme is stored inline (≤4 UTF-8 bytes).
    #[inline]
    pub const fn is_inline(self) -> bool {
        !self.is_extended()
    }

    /// Returns `true` if this grapheme is stored in a [`GraphemePool`].
    #[inline]
    pub const fn is_extended(self) -> bool {
        (self.value & 0xFF_00_00_00) == Self::EXTENDED_MARKER
    }

    /// Resolve this grapheme to a `&str`.
    ///
    /// This is the **primary** way to read a grapheme. On little-endian
    /// targets, inline graphemes are read zero-copy directly from `&self`.
    /// On big-endian targets (or when the compiler can prove equivalence),
    /// inline graphemes are read from a stack copy.
    ///
    /// For extended graphemes, the returned reference borrows from the pool.
    ///
    /// # Example
    ///
    /// ```
    /// # use sigil::{Grapheme, GraphemePool};
    /// let pool = GraphemePool::new();
    /// let g = Grapheme::from_char('A');
    /// assert_eq!(g.as_str(&pool), "A");
    /// ```
    #[cfg(target_endian = "little")]
    pub fn as_str<'a>(&'a self, pool: &'a GraphemePool) -> &'a str {
        if self.is_empty() {
            ""
        } else if self.is_inline() {
            let len = Self::inline_len(&self.inline_bytes()) as usize;
            // SAFETY: repr(transparent) over u32, and on LE the memory
            // layout matches the logical byte order. We only store valid
            // UTF-8 via from_inline_bytes. Lifetime is tied to &self.
            unsafe {
                let ptr = (self as *const Self).cast::<u8>();
                std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len))
            }
        } else {
            pool.resolve(self)
        }
    }

    /// Resolve this grapheme to a `&str` (big-endian fallback).
    ///
    /// Inline graphemes are copied to `buf` and the returned reference borrows
    /// from it. Extended graphemes borrow from the pool. The caller must keep
    /// both `buf` and `pool` alive for the duration.
    ///
    /// Prefer the little-endian [`as_str`](Self::as_str) when available.
    #[cfg(target_endian = "big")]
    pub fn as_str_with_buf<'a>(
        &self,
        pool: &'a GraphemePool,
        buf: &'a mut [u8; 4],
    ) -> &'a str {
        if self.is_empty() {
            ""
        } else if self.is_inline() {
            *buf = self.inline_bytes();
            let len = Self::inline_len(buf) as usize;
            // SAFETY: We only store valid UTF-8 via from_inline_bytes.
            unsafe { std::str::from_utf8_unchecked(&buf[..len]) }
        } else {
            pool.resolve(self)
        }
    }

    /// Resolve this grapheme to a [`Graph`] for pattern matching.
    ///
    /// For simple `&str` access, prefer [`as_str`](Self::as_str).
    ///
    /// # Example
    ///
    /// ```
    /// # use sigil::{Grapheme, GraphemePool};
    /// let pool = GraphemePool::new();
    /// let g = Grapheme::from_char('A');
    /// let resolved = g.as_graph(&pool);
    /// assert_eq!(resolved.as_str(), "A");
    /// ```
    pub fn as_graph<'a>(&self, pool: &'a GraphemePool) -> Graph<'a> {
        if self.is_empty() {
            Graph::None
        } else if self.is_inline() {
            let bytes = self.inline_bytes();
            let len = Self::inline_len(&bytes);
            Graph::Inline { bytes, len }
        } else {
            Graph::Extended(pool.resolve(self))
        }
    }

    /// Release any pool storage held by this grapheme.
    ///
    /// Must be called before overwriting a cell's grapheme with a new value,
    /// otherwise the pool entry leaks. No-op for inline and empty graphemes.
    pub fn release(self, pool: &mut GraphemePool) {
        if self.is_extended() {
            pool.release(&self);
        }
    }

    // ── Internal constructors ──────────────────────────────────────────

    /// Pack up to 4 UTF-8 bytes into a `u32` via little-endian interpretation.
    /// Unused trailing bytes are zero, and the high byte cannot be `0x01` for
    /// any valid UTF-8 input ≤4 bytes.
    pub(crate) fn from_inline_bytes(bytes: &[u8]) -> Self {
        debug_assert!(bytes.len() <= 4 && !bytes.is_empty());
        let mut buf = [0u8; 4];
        buf[..bytes.len()].copy_from_slice(bytes);
        Self { value: u32::from_le_bytes(buf) }
    }

    // ── Internal helpers ───────────────────────────────────────────────

    /// Get the raw little-endian bytes of an inline grapheme.
    #[inline]
    fn inline_bytes(self) -> [u8; 4] {
        self.value.to_le_bytes()
    }

    /// The 24-bit pool offset for an extended grapheme.
    #[inline]
    pub(crate) fn offset(self) -> usize {
        (self.value & Self::OFFSET_MASK) as usize
    }

    /// Determine the byte length of an inline UTF-8 grapheme stored as `[u8; 4]`.
    ///
    /// Scans for the first zero byte. Since UTF-8 continuation bytes are always
    /// `0x80..=0xBF`, a zero byte can only appear as NUL (treated as empty)
    /// or as padding after the grapheme data.
    #[inline]
    fn inline_len(bytes: &[u8; 4]) -> u8 {
        bytes.iter().position(|&b| b == 0).unwrap_or(4) as u8
    }
}

impl Default for Grapheme {
    #[inline]
    fn default() -> Self {
        Self::EMPTY
    }
}

impl From<char> for Grapheme {
    /// Create an inline grapheme from a `char`.
    ///
    /// Equivalent to [`Grapheme::from_char`].
    fn from(c: char) -> Self {
        Self::from_char(c)
    }
}

impl fmt::Debug for Grapheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            f.write_str("Grapheme(EMPTY)")
        } else if self.is_inline() {
            let bytes = self.inline_bytes();
            let len = Self::inline_len(&bytes);
            let s = std::str::from_utf8(&bytes[..len as usize]).unwrap_or("<invalid>");
            write!(f, "Grapheme({s:?})")
        } else {
            write!(f, "Grapheme(pool@{})", self.offset())
        }
    }
}

// ── Graph ────────────────────────────────────────────────────────────

/// A resolved grapheme cluster — the result of reading a [`Grapheme`] handle.
///
/// For inline graphemes the data lives on the stack; for extended graphemes
/// it borrows from the [`GraphemePool`]. Use `.as_str()` or the [`Deref`]
/// impl for uniform `&str` access.
#[derive(Clone, Copy)]
pub enum Graph<'a> {
    /// No grapheme.
    None,

    /// Inline UTF-8 data (≤4 bytes).
    Inline { bytes: [u8; 4], len: u8 },

    /// A reference into the [`GraphemePool`].
    Extended(&'a str),
}

impl Graph<'_> {
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

impl PartialEq<str> for Graph<'_> {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<Graph<'_>> for str {
    fn eq(&self, other: &Graph<'_>) -> bool {
        self == other.as_str()
    }
}

impl<'a> PartialEq<&'a str> for Graph<'_> {
    fn eq(&self, other: &&'a str) -> bool {
        self.as_str() == *other
    }
}

impl<'a> PartialEq<Graph<'a>> for &str {
    fn eq(&self, other: &Graph<'a>) -> bool {
        *self == other.as_str()
    }
}

impl PartialEq for Graph<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for Graph<'_> {}

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
    fn nul_is_empty() {
        let g = Grapheme::from_char('\0');
        assert!(g.is_empty());
        assert_eq!(g, Grapheme::EMPTY);
    }

    #[test]
    fn inline_ascii() {
        let g = Grapheme::from_char('A');
        assert!(!g.is_empty());
        assert!(g.is_inline());

        let pool = GraphemePool::new();
        assert_eq!(g.as_str(&pool), "A");
        assert_eq!(g.as_graph(&pool), "A");
    }

    #[test]
    fn inline_multibyte() {
        let pool = GraphemePool::new();

        // 2-byte: Latin é (U+00E9)
        let g = Grapheme::try_inline("é").unwrap();
        assert!(g.is_inline());
        assert_eq!(g.as_str(&pool), "é");

        // 3-byte: CJK 中 (U+4E2D)
        let g = Grapheme::try_inline("中").unwrap();
        assert!(g.is_inline());
        assert_eq!(g.as_str(&pool), "中");
    }

    #[test]
    fn inline_four_byte_emoji() {
        let pool = GraphemePool::new();
        // 4-byte: party popper 🎉 (U+1F389) = F0 9F 8E 89
        let g = Grapheme::from_char('🎉');
        assert!(g.is_inline());
        assert_eq!(g.as_str(&pool), "🎉");
    }

    #[test]
    fn inline_combining_marks() {
        let pool = GraphemePool::new();
        // e + combining acute accent = 3 bytes, fits inline
        let s = "e\u{0301}"; // é as decomposed
        assert_eq!(s.len(), 3);
        let g = Grapheme::try_inline(s).unwrap();
        assert!(g.is_inline());
        assert_eq!(g.as_str(&pool), s);
    }

    #[test]
    fn extended_emoji_sequence() {
        let mut pool = GraphemePool::new();
        // Family emoji: 👨‍👩‍👧‍👦 = 25 bytes
        let family = "👨\u{200D}👩\u{200D}👧\u{200D}👦";
        assert!(family.len() > 4);
        let offset = pool.stash(family).unwrap();

        let g = Grapheme::new(family, &mut pool);
        assert!(!g.is_empty());
        assert!(g.is_extended());
        assert_eq!(g.as_graph(&pool), family);
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
    fn as_str_inline_and_extended() {
        let mut pool = GraphemePool::new();

        let g1 = Grapheme::from_char('X');
        assert_eq!(g1.as_str(&pool), "X");

        let family = "👨\u{200D}👩\u{200D}👧\u{200D}👦";
        let g2 = Grapheme::new(family, &mut pool);
        assert_eq!(g2.as_str(&pool), family);

        assert_eq!(Grapheme::EMPTY.as_str(&pool), "");
    }

    #[test]
    fn from_char_trait() {
        let g: Grapheme = 'A'.into();
        assert!(g.is_inline());
        let pool = GraphemePool::new();
        assert_eq!(g.as_str(&pool), "A");
    }

    #[test]
    fn graph_deref_to_str() {
        let pool = GraphemePool::new();
        let g = Grapheme::from_char('H');
        let resolved = g.as_graph(&pool);

        // Deref gives us &str methods for free.
        assert!(resolved.starts_with('H'));
        assert_eq!(resolved.to_uppercase(), "H");
    }

    #[test]
    fn graph_copy() {
        let pool = GraphemePool::new();
        let g = Grapheme::from_char('A').as_graph(&pool);
        let g2 = g; // Copy
        assert_eq!(g, g2);
    }

    #[test]
    fn graph_eq_symmetric() {
        let pool = GraphemePool::new();
        let g = Grapheme::from_char('A').as_graph(&pool);

        // Graph == str
        assert_eq!(g, *"A");
        // str == Graph
        assert_eq!(*"A", g);
        // Graph == &str
        assert_eq!(g, "A");
        // &str == Graph
        assert_eq!("A", g);
    }

    #[test]
    fn graph_is_empty_covers_all_variants() {
        let pool = GraphemePool::new();
        assert!(Graph::None.is_empty());
        assert!(Graph::Inline { bytes: [0; 4], len: 0 }.is_empty());
        assert!(!Grapheme::from_char('X').as_graph(&pool).is_empty());
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

    // Pin the invariant: SOH (0x01) is stored inline, not mistaken for
    // an extended marker. Unlike notcurses, we *can* store SOH.
    #[test]
    fn soh_byte_is_not_extended() {
        let g = Grapheme::from_char('\x01');
        assert!(g.is_inline());
        assert!(!g.is_extended());
        assert!(!g.is_empty());

        let pool = GraphemePool::new();
        assert_eq!(g.as_str(&pool), "\x01");
    }

    #[test]
    fn offset_is_not_public() {
        // This is a compile-time guarantee via pub(crate), but we can verify
        // the offset round-trips correctly for extended graphemes.
        let mut pool = GraphemePool::new();
        let family = "👨\u{200D}👩\u{200D}👧\u{200D}👦";
        let g = Grapheme::new(family, &mut pool);
        assert!(g.is_extended());
        // Offset should be 0 since it's the first entry.
        assert_eq!(g.offset(), 0);
    }
}