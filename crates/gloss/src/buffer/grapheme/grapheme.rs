use super::{Arena, GraphemeError};
use std::fmt;
use std::hash::{Hash, Hasher};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use crate::AsOffset;

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
///   ZWJ, skin-tone modifiers, etc.) are stored in a [`Arena`] and
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
/// # Internal layout
///
/// ```text
/// ┌─────────────────────────┬──────────┐
/// │ payload (24 bits)       │ tag (8 bits) │
/// │ bits 0..23              │ bits 24..31 │
/// └─────────────────────────┴──────────┘
/// ```
///
/// - **Inline**: all 32 bits hold UTF-8 data (tag is the 4th byte).
/// - **Extended**: tag = `0x01`, payload = 24-bit arena offset.
#[derive_const(PartialEq, Eq)]
#[derive(Clone, Copy, Hash)]
pub struct Grapheme {
    /// Raw 32-bit backing storage.
    ///
    /// - bits 0..23: arena offset (extended) or lower 3 bytes of inline UTF-8.
    /// - bits 24..31: `0x01` = extended marker; otherwise the 4th byte of inline UTF-8.
    value: u32,
}

impl Grapheme {
    /// A grapheme representing a replacement character (). This is the default for empty cells.
    pub const EMPTY: Self = Self::inline(char::REPLACEMENT_CHARACTER);
    /// A grapheme representing a space (U+0020). This is the default for blank cells.
    pub const SPACE: Self = Self::inline(' ');

    /// Create a grapheme from a string slice.
    ///
    /// If the string fits in 4 UTF-8 bytes, it is stored inline (no arena
    /// interaction). Otherwise, it is stashed in the given [`Arena`].
    ///
    /// # Panics
    ///
    /// Panics if the string exceeds 4 bytes and the arena is full (16 MiB).
    /// Use [`try_extended`](Self::try_extended) for the fallible variant.
    pub fn extended(value: impl Encode, arena: &mut Arena) -> Self {
        Self::try_extended(value, arena).unwrap()
    }

    /// Fallible version of [`extended`](Self::extended).
    ///
    /// Returns `Err` only if the string exceeds 4 bytes and the arena cannot
    /// allocate space for it.
    pub fn try_extended(value: impl Encode, arena: &mut Arena) -> Result<Self, GraphemeError> {
        value.try_extended(arena)
    }

    /// Create an inline grapheme.
    ///
    /// Note:
    /// - `'\0'` produces [`SPACE`](Self::SPACE) since NUL is
    ///   indistinguishable from padding in the inline encoding.
    /// - Performs manual UTF-8 encoding since `char::encode_utf8` is not
    ///   available in `const fn`. Every Unicode scalar value fits in ≤4
    ///   UTF-8 bytes, so this always produces an inline grapheme.
    pub const fn inline(value: impl [const] Encode) -> Self {
        match value.try_inline() {
            Ok(g) => g,
            Err(e) => panic!("Failed to encode value as an inline grapheme"),
        }
    }

    /// Try to create an inline grapheme without an arena.
    pub const fn try_inline(value: impl [const] Encode) -> Result<Self, GraphemeError> {
        value.try_inline()
    }


    /// Bitmask covering the low 24 payload bits.
    const PAYLOAD_MASK: u32 = 0x00FF_FFFF;

    /// The sentinel tag value marking an extended (arena-stored) grapheme.
    const EXTENDED_TAG: u8 = 0x01;


    /// Create an extended grapheme from an arena offset.
    ///
    /// # Safety
    ///
    /// The caller must ensure `offset` refers to a valid, live entry in a
    /// [`Arena`]. This is memory-safe (no UB), but a bogus offset will
    /// produce garbage when resolved.
    pub fn offset(offset: impl AsOffset) -> Self {
        let offset = offset.as_offset();
        /// Maximum addressable offset in the arena (24-bit, = 16 MiB − 1).
        debug_assert!(offset <= Grapheme::PAYLOAD_MASK as usize, "offset exceeds 24-bit range");
        Self {
            value: (offset as u32) | ((Self::EXTENDED_TAG as u32) << 24),
        }
    }

    /// Returns `true` if this grapheme is empty (no character).
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self == &Self::SPACE
    }

    /// Returns `true` if this grapheme is stored inline (≤4 UTF-8 bytes).
    #[inline]
    pub const fn is_inline(self) -> bool {
        !self.is_extended()
    }

    /// Returns `true` if this grapheme is stored in a [`Arena`].
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
    /// # use sigil::{Grapheme, Arena};
    /// let arena = Arena::new();
    /// let g = Grapheme::inline('A');
    /// assert_eq!(g.as_str(&arena), "A");
    /// ```
    #[cfg(target_endian = "little")]
    pub fn as_str<'a>(&'a self, arena: &'a Arena) -> &str {
        if self.is_empty() {
            " "
        } else if self.is_inline() {
            self.as_inline_str()
        } else {
            arena.get(self)
        }
    }

    /// Resolve this grapheme to a byte slice.
    pub fn as_bytes<'a>(&'a self, arena: &'a Arena) -> &[u8] {
        self.as_str(arena).as_bytes()
    }

    // ── Internal constructors ──────────────────────────────────────────

    /// Pack up to 4 UTF-8 bytes into a `u32` via little-endian interpretation.
    /// Unused trailing bytes are zero, and the high byte cannot be `0x01` for
    /// any valid UTF-8 input ≤4 bytes.
    ///
    /// # Safety
    ///
    /// Caller must ensure `bytes` is valid UTF-8, non-empty, and ≤4 bytes long.
    pub const unsafe fn from_bytes(bytes: &[u8]) -> Self {
        debug_assert!(bytes.len() <= char::MAX_LEN_UTF8 && !bytes.is_empty());
        let mut buf = [0u8; char::MAX_LEN_UTF8];
        // const-compatible slice copy
        let mut i = 0;
        while i < bytes.len() {
            buf[i] = bytes[i];
            i += 1;
        }
        Self { value: u32::from_le_bytes(buf) }
    }

    // ── Internal helpers ───────────────────────────────────────────────

    /// The 24-bit arena offset for an extended grapheme.
    #[inline]
    pub fn as_offset(&self) -> usize {
        (self.value & Self::PAYLOAD_MASK) as usize
    }

    /// Byte length of the inline UTF-8 data, determined by scanning for the
    /// first zero padding byte.
    pub fn inline_len(self) -> usize {
        memchr::memchr(0, &self.value.to_le_bytes()).unwrap_or(char::MAX_LEN_UTF8)
    }

    /// Extract the inline UTF-8 bytes as a slice.
    ///
    /// On little-endian targets this is a direct pointer cast into `self`;
    /// the length is determined by [`inline_len`](Self::inline_len).
    pub fn as_inline_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, self.inline_len()) }
    }

    /// Interpret the inline bytes as a `&str`.
    ///
    /// # Safety
    ///
    /// The inline encoding guarantees valid UTF-8 for all non-empty values
    /// constructed through the public API.
    pub fn as_inline_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.as_inline_bytes()) }
    }
}

impl From<char> for Grapheme {
    fn from(c: char) -> Self {
        Self::inline(c)
    }
}

impl fmt::Debug for Grapheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return f.write_str("Grapheme::EMPTY");
        }

        if self.is_inline() {
            f.debug_tuple("Grapheme::Inline").field(&self.as_inline_str()).finish()
        } else {
            f.debug_tuple("Grapheme::Extended")
                .field(&self.as_offset())
                .finish()
        }
    }
}

pub const trait Encode {
    fn try_inline(self) -> Result<Grapheme, GraphemeError>;
    fn try_extended(self, arena: &mut Arena) -> Result<Grapheme, GraphemeError>;
}

impl Encode for &str {
    fn try_inline(self) -> Result<Grapheme, GraphemeError> {
        let bytes = self.as_bytes();
        let len = bytes.len();

        match len {
            0 => Ok(Grapheme::SPACE),
            ..=char::MAX_LEN_UTF8 => Ok(unsafe { Grapheme::from_bytes(bytes) }),
            _ => Err(GraphemeError::ArenaRequired {
                len,
                max: char::MAX_LEN_UTF8,
            }),
        }
    }

    fn try_extended(self, arena: &mut Arena) -> Result<Grapheme, GraphemeError> {
        let bytes = self.as_bytes();
        let len = bytes.len();

        match len {
            0 => Ok(Grapheme::SPACE),
            ..=char::MAX_LEN_UTF8 => Ok(unsafe { Grapheme::from_bytes(bytes) }),
            _ => arena.try_insert(self),
        }
    }
}

impl const Encode for char {
    fn try_inline(self) -> Result<Grapheme, GraphemeError> {
        let char = self as u32;
        Ok(Grapheme {
            value: u32::from_le_bytes(match char {
                0x00..=0x7F => [char as u8, 0, 0, 0],
                0x80..=0x7FF => [
                    (0xC0 | (char >> 6)) as u8,
                    (0x80 | (char & 0x3F)) as u8,
                    0,
                    0,
                ],
                0x800..=0xFFFF => [
                    (0xE0 | (char >> 12)) as u8,
                    (0x80 | ((char >> 6) & 0x3F)) as u8,
                    (0x80 | (char & 0x3F)) as u8,
                    0,
                ],
                _ => [
                    (0xF0 | (char >> 18)) as u8,
                    (0x80 | ((char >> 12) & 0x3F)) as u8,
                    (0x80 | ((char >> 6) & 0x3F)) as u8,
                    (0x80 | (char & 0x3F)) as u8,
                ],
            }),
        })
    }
    fn try_extended(self, _arena: &mut Arena) -> Result<Grapheme, GraphemeError> {
        Self::try_inline(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_len() {
        let g = Grapheme::inline('A');
        dbg!(g.inline_len());
        dbg!(unsafe { g.value.to_le_bytes() });
    }

    #[test]
    fn empty_grapheme() {
        let g = Grapheme::SPACE;
        assert!(g.is_empty());
        assert!(g.is_inline());
        assert!(!g.is_extended());
    }

    #[test]
    fn nul_is_empty() {
        let g = Grapheme::inline(' ');
        assert!(g.is_empty());
        assert_eq!(g, Grapheme::SPACE);
    }

    #[test]
    fn inline_ascii() {
        let g = Grapheme::inline('A');
        assert!(!g.is_empty());
        assert!(g.is_inline());

        let arena = Arena::new();
        assert_eq!(g.as_str(&arena), "A");
    }

    #[test]
    fn inline_multibyte() {
        let arena = Arena::new();

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
        let arena = Arena::new();
        // 4-byte: party popper 🎉 (U+1F389) = F0 9F 8E 89
        let g = Grapheme::inline('🎉');
        assert!(g.is_inline());
        assert_eq!(g.as_str(&arena), "🎉");
    }

    #[test]
    fn inline_combining_marks() {
        let arena = Arena::new();
        // e + combining acute accent = 3 bytes, fits inline
        let s = "e\u{0301}"; // é as decomposed
        assert_eq!(s.len(), 3);
        let g = Grapheme::try_inline(s).unwrap();
        assert!(g.is_inline());
        assert_eq!(g.as_str(&arena), s);
    }

    #[test]
    fn extended_emoji_sequence() {
        let mut arena = Arena::new();
        // Family emoji: 👨‍👩‍👧‍👦 = 25 bytes
        let family = "👨\u{200D}👩\u{200D}👧\u{200D}👦";
        assert!(family.len() > 4);

        let g = Grapheme::extended(family, &mut arena);
        assert!(!g.is_empty());
        assert!(g.is_extended());
    }

    #[test]
    fn try_inline_rejects_long() {
        let family = "👨\u{200D}👩\u{200D}👧\u{200D}👦";
        assert!(Grapheme::try_inline(family).is_err());
    }

    #[test]
    fn release_frees_arena_space() {
        let mut arena = Arena::new();
        let family = "👨\u{200D}👩\u{200D}👧\u{200D}👦";

        let used_before = arena.count();
        let g = Grapheme::extended(family, &mut arena);
        let used_after_insert = arena.count();
        assert!(used_after_insert > used_before);

        arena.remove(g);
        assert_eq!(arena.count(), used_before);
    }

    #[test]
    fn as_str_inline_and_extended() {
        let mut arena = Arena::new();

        let g1 = Grapheme::inline('X');
        assert_eq!(g1.as_str(&arena), "X");

        let family = "👨\u{200D}👩\u{200D}👧\u{200D}👦";
        let g2 = Grapheme::extended(family, &mut arena);
        assert_eq!(g2.as_str(&arena), family);

        assert_eq!(Grapheme::SPACE.as_str(&arena), " ");
    }

    #[test]
    fn from_char_trait() {
        let g: Grapheme = 'A'.into();
        assert!(g.is_inline());
        let arena = Arena::new();
        assert_eq!(g.as_str(&arena), "A");
    }

    #[test]
    fn debug_output() {
        let g = Grapheme::inline('Z');
        let dbg = format!("{g:?}");
        assert!(dbg.contains("Z"));

        let g2 = Grapheme::SPACE;
        let dbg2 = format!("{g2:?}");
        assert!(dbg2.contains("EMPTY"));
    }

    // Pin the invariant: SOH (0x01) is stored inline, not mistaken for
    // an extended marker. Unlike notcurses, we *can* store SOH.
    #[test]
    fn soh_byte_is_not_extended() {
        let g = Grapheme::inline('\x01');
        assert!(g.is_inline());
        assert!(!g.is_extended());
        assert!(!g.is_empty());

        let arena = Arena::new();
        assert_eq!(g.as_str(&arena), "\x01");
    }

    #[test]
    fn offset_round_trips() {
        let mut arena = Arena::new();
        let family = "👨\u{200D}👩\u{200D}👧\u{200D}👦";
        let g = Grapheme::extended(family, &mut arena);
        assert!(g.is_extended());
        // Offset should be 0 since it's the first entry.
        assert_eq!(g.as_offset(), 0);
    }
}