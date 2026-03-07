mod arena;
mod graph;

pub use arena::*;
pub use graph::*;

use std::fmt;
use std::ops::Deref;
use bilge::prelude::*;

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
///   ZWJ, skin-tone modifiers, etc.) are stored in a [`GraphemeArena`] and
///   referenced by a 24-bit offset. The high byte is set to `0x01` as a
///   marker, giving 16 MiB of addressable arena space.
///
/// # Encoding (little-endian byte interpretation of the `u32`)
///
/// ```text
/// bytes[0..=3] with bytes[3] != 0x01  →  inline UTF-8 (up to 4 bytes)
/// bytes[3] == 0x01                     →  extended; bytes[0..=2] = arena offset
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
///
/// # Bitfield layout (via `bilge`)
///
/// ```text
/// ┌─────────────────────────┬──────────┐
/// │ payload (u24)           │ tag (u8) │
/// │ bits 0..23              │ bits 24..31 │
/// └─────────────────────────┴──────────┘
/// ```
///
/// - **Inline**: all 32 bits hold UTF-8 data (tag is the 4th byte).
/// - **Extended**: tag = `0x01`, payload = 24-bit arena offset.
#[bitsize(32)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, FromBits)]
pub struct Grapheme {
    /// Low 24 bits: arena offset (extended) or lower 3 bytes of inline UTF-8.
    payload: u24,
    /// High byte: `0x01` = extended marker; otherwise the 4th byte of inline UTF-8.
    tag: u8,
}

impl Grapheme {
    /// A grapheme representing [`char::REPLACEMENT_CHARACTER`] (�).
    pub const REPLACEMENT_CHARACTER: Self = Self {
        value: u32::from_le_bytes([0xEF, 0xBF, 0xBD, 0x00])
    };


    /// An empty grapheme (no character). This is the default for blank cells.
    pub const EMPTY: Self = Self { value: 0 };
    /// The sentinel tag value marking an extended (arena-stored) grapheme.
    pub const EXTENDED_TAG: u8 = 0x01;

    /// Create an extended grapheme from a arena offset.
    ///
    /// # Safety
    ///
    /// The caller must ensure `offset` refers to a valid, live entry in a
    /// [`GraphemeArena`]. This is now memory-safe (no UB), but a bogus offset
    /// will produce garbage when resolved.
    pub fn from_offset(offset: usize) -> Self {
        debug_assert!(offset <= <u24 as Bitsized>::MAX.value() as usize);
        Self::new(u24::new(offset as u32), Self::EXTENDED_TAG)
    }

    /// Create a grapheme from a string slice.
    ///
    /// If the string fits in 4 UTF-8 bytes, it is stored inline (no arena
    /// interaction). Otherwise, it is stashed in the given [`GraphemeArena`].
    ///
    /// # Panics
    ///
    /// Panics if the string exceeds 4 bytes and the arena is full (16 MiB).
    /// Use [`try_encode`](Self::try_encode) for the fallible variant.
    pub fn encode(s: &str, arena: &mut GraphemeArena) -> Self {
        Self::try_encode(s, arena).expect("grapheme arena is full")
    }

    /// Fallible version of [`encode`](Self::encode).
    ///
    /// Returns `Err` only if the string exceeds 4 bytes and the arena cannot
    /// allocate space for it.
    pub fn try_encode(s: &str, arena: &mut GraphemeArena) -> Result<Self, GraphemePoolError> {
        let bytes = s.as_bytes();

        if bytes.is_empty() {
            return Ok(Self::EMPTY);
        }

        if bytes.len() <= 4 {
            Ok(Self::from_inline_bytes(bytes))
        } else {
            Ok(arena.stash(s)?)
        }
    }

    /// Try to create an inline grapheme without a arena.
    ///
    /// Returns `None` if the string exceeds 4 UTF-8 bytes.
    pub fn inline(s: &str) -> Self {
        Self::try_inline(s).expect("grapheme too long")
    }

    /// Try to create an inline grapheme without a arena.
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
    /// always succeeds and never needs a arena.
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

    /// Returns `true` if this grapheme is stored in a [`GraphemeArena`].
    #[inline]
    pub const fn is_extended(self) -> bool {
        (self.value >> 24) as u8 == Self::EXTENDED_TAG
    }

    /// Resolve this grapheme to a `&str`.
    ///
    /// This is the **primary** way to read a grapheme. On little-endian
    /// targets, inline graphemes are read zero-copy directly from `&self`.
    /// On big-endian targets (or when the compiler can prove equivalence),
    /// inline graphemes are read from a stack copy.
    ///
    /// For extended graphemes, the returned reference borrows from the arena.
    ///
    /// # Example
    ///
    /// ```
    /// # use sigil::{Grapheme, GraphemeArena};
    /// let arena = GraphemeArena::new();
    /// let g = Grapheme::from_char('A');
    /// assert_eq!(g.as_str(&arena), "A");
    /// ```
    #[cfg(target_endian = "little")]
    pub fn as_str<'a>(&'a self, arena: &'a GraphemeArena) -> &'a str {
        if self.is_empty() {
            ""
        } else if self.is_inline() {
            let bytes = self.to_le_bytes();
            let len = Self::inline_len(&bytes) as usize;
            // SAFETY: bilge's #[bitsize(32)] wraps a u32. On LE the memory
            // layout matches the logical byte order. We only store valid
            // UTF-8 via from_inline_bytes. Lifetime is tied to &self.
            unsafe {
                let ptr = (self as *const Self).cast::<u8>();
                std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, len))
            }
        } else {
            arena.resolve(self)
        }
    }

    /// Resolve this grapheme to a `&str` (big-endian fallback).
    ///
    /// Inline graphemes are copied to `buf` and the returned reference borrows
    /// from it. Extended graphemes borrow from the arena. The caller must keep
    /// both `buf` and `arena` alive for the duration.
    ///
    /// Prefer the little-endian [`as_str`](Self::as_str) when available.
    #[cfg(target_endian = "big")]
    pub fn as_str_with_buf<'a>(
        &self,
        arena: &'a GraphemeArena,
        buf: &'a mut [u8; 4],
    ) -> &'a str {
        if self.is_empty() {
            ""
        } else if self.is_inline() {
            *buf = self.to_le_bytes();
            let len = Self::inline_len(buf) as usize;
            // SAFETY: We only store valid UTF-8 via from_inline_bytes.
            unsafe { std::str::from_utf8_unchecked(&buf[..len]) }
        } else {
            arena.resolve(self)
        }
    }

    /// Resolve this grapheme to a [`Graph`] for pattern matching.
    ///
    /// For simple `&str` access, prefer [`as_str`](Self::as_str).
    ///
    /// # Example
    ///
    /// ```
    /// # use sigil::{Grapheme, GraphemeArena};
    /// let arena = GraphemeArena::new();
    /// let g = Grapheme::from_char('A');
    /// let resolved = g.as_graph(&arena);
    /// assert_eq!(resolved.as_str(), "A");
    /// ```
    pub fn as_graph<'a>(&self, arena: &'a GraphemeArena) -> Graph<'a> {
        if self.is_empty() {
            Graph::None
        } else if self.is_inline() {
            let bytes = self.to_le_bytes();
            let len = Self::inline_len(&bytes);
            Graph::Inline { bytes, len }
        } else {
            Graph::Extended(arena.resolve(self))
        }
    }

    /// Release any arena storage held by this grapheme.
    ///
    /// Must be called before overwriting a cell's grapheme with a new value,
    /// otherwise the arena entry leaks. No-op for inline and empty graphemes.
    pub fn release(self, arena: &mut GraphemeArena) {
        if self.is_extended() {
            arena.release(&self);
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
        Self::from(u32::from_le_bytes(buf))
    }

    // ── Internal helpers ───────────────────────────────────────────────

    /// Get the raw little-endian bytes of this grapheme's `u32`.
    #[inline]
    pub fn to_le_bytes(self) -> [u8; 4] {
        self.value.to_le_bytes()
    }

    /// The 24-bit arena offset for an extended grapheme.
    #[inline]
    pub(crate) fn offset(self) -> usize {
        self.payload().as_usize()
    }

    /// Determine the byte length of an inline UTF-8 grapheme stored as `[u8; 4]`.
    ///
    /// Scans for the first zero byte. Since UTF-8 continuation bytes are always
    /// `0x80..=0xBF`, a zero byte can only appear as NUL (treated as empty)
    /// or as padding after the grapheme data.
    #[inline]
    pub(crate) fn inline_len(bytes: &[u8; 4]) -> u8 {
        memchr::memchr(0, bytes).unwrap_or(4) as u8
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
            return f.write_str("Grapheme::EMPTY")
        }

        if self.is_inline() {
            let bytes = self.to_le_bytes();
            let len = Self::inline_len(&bytes) as usize;
            let s = unsafe { std::str::from_utf8_unchecked(&bytes[..len]) };

            f.debug_tuple("Grapheme::Inline").field(&s).finish()
        } else {
            f.debug_tuple("Grapheme::Extended").field(&self.offset()).finish()
        }
    }
}

// ── Graph ────────────────────────────────────────────────────────────


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

        let arena = GraphemeArena::new();
        assert_eq!(g.as_str(&arena), "A");
        assert_eq!(g.as_graph(&arena), "A");
    }

    #[test]
    fn inline_multibyte() {
        let arena = GraphemeArena::new();

        // 2-byte: Latin é (U+00E9)
        let g = Grapheme::try_inline("é").unwrap();
        assert!(g.is_inline());
        assert_eq!(g.as_str(&arena), "é");

        // 3-byte: CJK 中 (U+4E2D)
        let g = Grapheme::try_inline("中").unwrap();
        assert!(g.is_inline());
        assert_eq!(g.as_str(&arena), "中");
    }

    #[test]
    fn inline_four_byte_emoji() {
        let arena = GraphemeArena::new();
        // 4-byte: party popper 🎉 (U+1F389) = F0 9F 8E 89
        let g = Grapheme::from_char('🎉');
        assert!(g.is_inline());
        assert_eq!(g.as_str(&arena), "🎉");
    }

    #[test]
    fn inline_combining_marks() {
        let arena = GraphemeArena::new();
        // e + combining acute accent = 3 bytes, fits inline
        let s = "e\u{0301}"; // é as decomposed
        assert_eq!(s.len(), 3);
        let g = Grapheme::try_inline(s).unwrap();
        assert!(g.is_inline());
        assert_eq!(g.as_str(&arena), s);
    }

    #[test]
    fn extended_emoji_sequence() {
        let mut arena = GraphemeArena::new();
        // Family emoji: 👨‍👩‍👧‍👦 = 25 bytes
        let family = "👨\u{200D}👩\u{200D}👧\u{200D}👦";
        assert!(family.len() > 4);
        let offset = arena.stash(family).unwrap();

        let g = Grapheme::encode(family, &mut arena);
        assert!(!g.is_empty());
        assert!(g.is_extended());
        assert_eq!(g.as_graph(&arena), family);
    }

    #[test]
    fn try_inline_rejects_long() {
        let family = "👨\u{200D}👩\u{200D}👧\u{200D}👦";
        assert!(Grapheme::try_inline(family).is_none());
    }

    #[test]
    fn release_frees_arena_space() {
        let mut arena = GraphemeArena::new();
        let family = "👨\u{200D}👩\u{200D}👧\u{200D}👦";

        let used_before = arena.used();
        let g = Grapheme::encode(family, &mut arena);
        let used_after_insert = arena.used();
        assert!(used_after_insert > used_before);

        g.release(&mut arena);
        assert_eq!(arena.used(), used_before);
    }

    #[test]
    fn as_str_inline_and_extended() {
        let mut arena = GraphemeArena::new();

        let g1 = Grapheme::from_char('X');
        assert_eq!(g1.as_str(&arena), "X");

        let family = "👨\u{200D}👩\u{200D}👧\u{200D}👦";
        let g2 = Grapheme::encode(family, &mut arena);
        assert_eq!(g2.as_str(&arena), family);

        assert_eq!(Grapheme::EMPTY.as_str(&arena), "");
    }

    #[test]
    fn from_char_trait() {
        let g: Grapheme = 'A'.into();
        assert!(g.is_inline());
        let arena = GraphemeArena::new();
        assert_eq!(g.as_str(&arena), "A");
    }

    #[test]
    fn graph_deref_to_str() {
        let arena = GraphemeArena::new();
        let g = Grapheme::from_char('H');
        let resolved = g.as_graph(&arena);

        // Deref gives us &str methods for free.
        assert!(resolved.starts_with('H'));
        assert_eq!(resolved.to_uppercase(), "H");
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

        let arena = GraphemeArena::new();
        assert_eq!(g.as_str(&arena), "\x01");
    }

    #[test]
    fn offset_round_trips() {
        // Verify the offset round-trips correctly for extended graphemes.
        let mut arena = GraphemeArena::new();
        let family = "👨\u{200D}👩\u{200D}👧\u{200D}👦";
        let g = Grapheme::encode(family, &mut arena);
        assert!(g.is_extended());
        // Offset should be 0 since it's the first entry.
        assert_eq!(g.offset(), 0);
    }

    #[test]
    fn bilge_field_accessors() {
        // Verify bilge's generated accessors match the encoding.
        let g = Grapheme::from_char('A');
        // 'A' = 0x41, stored in LE byte 0 → raw u32 = 0x00000041
        // tag (high byte) should be 0, payload (low 24) should be 0x41
        assert_eq!(g.tag(), 0x00);
        assert_eq!(u32::from(g.payload()), 0x41);

        // Extended grapheme at offset 42
        let ext = Grapheme::from_offset(42);
        assert_eq!(ext.tag(), Grapheme::EXTENDED_TAG);
        assert_eq!(u32::from(ext.payload()), 42);
    }
}
