use crate::{Arena, GraphemeError};
use std::fmt;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Maximum UTF-8 bytes for any single Unicode scalar value — and therefore the
/// inline capacity of a [`Grapheme`].
const INLINE_CAPACITY: usize = 4;

/// A compact grapheme-cluster handle stored in 4 bytes.
///
/// This is the core unit of text storage in the framebuffer. It is a hand-rolled
/// niche-packed sum type — morally
/// `enum { Empty, Continuation, Inline([u8; ≤4]), Extended(u24) }` — squeezed
/// into `[u8; 4]`. The packing exploits a property the compiler can't see: the
/// **4th byte** of any valid UTF-8 sequence of ≤4 bytes is either `0x00`
/// (padding) or a continuation byte (`0x80..=0xBF`), so it can never collide
/// with the sentinel tags `0x01`/`0x02`.
///
/// The representation is a *byte array*, not an integer, so the layout is
/// identical on every target — there is no endianness to reason about.
///
/// # Encoding (`repr[3]` is always the tag)
///
/// ```text
/// repr == [0, 0, 0, 0x00]   → empty (no grapheme)
/// repr[3] == 0x01           → extended; repr[0..3] = 24-bit arena offset (LE)
/// repr == [0, 0, 0, 0x02]   → continuation (wide-char placeholder)
/// otherwise                 → inline UTF-8 (1..=4 bytes, zero-padded)
/// ```
///
/// - **Inline** clusters of ≤4 UTF-8 bytes are stored directly — zero heap
///   allocation for ASCII, Latin, Cyrillic, CJK, and most single-codepoint
///   emoji. Reads are zero-copy on every target.
/// - **Extended** clusters (ZWJ sequences, skin-tone modifiers, …) live in an
///   [`Arena`] and are referenced by a 24-bit offset, giving 16 MiB of space.
///
/// # NUL handling
///
/// NUL (`0x00`) is the inline padding byte, so a standalone NUL is
/// indistinguishable from empty: [`from_char('\0')`](Self::char) yields
/// [`EMPTY`](Self::EMPTY). This is intentional — NUL has no visual cell.
#[derive(Copy, Hash)]
#[derive_const(Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Grapheme {
    /// UTF-8 / tag bytes in storage order. `repr[3]` is the tag.
    repr: [u8; 4],
}

impl const Grapheme {
    /// Sentinel tag for an extended (arena-stored) grapheme.
    const EXTENDED_TAG: u8 = 0x01;
    /// Sentinel tag for a wide-character continuation cell.
    const CONTINUATION_TAG: u8 = 0x02;

    /// Empty cell content (no character).
    pub const EMPTY: Self = Self { repr: [0, 0, 0, 0] };

    /// Placeholder for the trailing cells of a wide grapheme.
    ///
    /// Distinct from [`EMPTY`](Self::EMPTY): a continuation cell carries no
    /// content of its own but marks that the previous cell spans into this
    /// column.
    pub const CONTINUATION: Self = Self {
        repr: [0, 0, 0, Self::CONTINUATION_TAG],
    };

    /// The Unicode replacement character (`U+FFFD`), stored inline.
    pub const REPLACEMENT: Self = Self {
        repr: [0xEF, 0xBF, 0xBD, 0x00],
    };

    /// Encode a `char` inline. Every scalar value fits in ≤4 UTF-8 bytes, so
    /// this never needs an arena. `'\0'` produces [`EMPTY`](Self::EMPTY).
    pub fn char(char: char) -> Self {
        let mut buf = [0u8; INLINE_CAPACITY];
        Self::pack_inline(char.encode_utf8(&mut buf).as_bytes())
    }

    /// Build an extended handle from a raw 24-bit arena offset.
    ///
    /// Called by [`Arena::try_insert`](crate::Arena::try_insert). The offset
    /// must refer to a live entry in *the* arena this handle will be resolved
    /// against; this is memory-safe regardless, but a bogus offset resolves to
    /// garbage or panics.
    pub fn from_offset(offset: usize) -> Self {
        debug_assert!(offset <= 0x00FF_FFFF, "offset exceeds 24-bit range");
        let o = offset as u32;
        Self {
            repr: [o as u8, (o >> 8) as u8, (o >> 16) as u8, Self::EXTENDED_TAG],
        }
    }

    // ── Discriminants ───────────────────────────────────────────────────

    /// `true` if empty (no character).
    #[inline]
    pub fn is_empty(self) -> bool {
        u32::from_le_bytes(self.repr) == 0
    }

    /// `true` if this is a wide-character continuation marker.
    #[inline]
    pub fn is_continuation(self) -> bool {
        self.repr[3] == Self::CONTINUATION_TAG
    }

    /// `true` if stored inline (≤4 UTF-8 bytes). [`EMPTY`](Self::EMPTY) counts
    /// as inline; the sentinel tags do not.
    #[inline]
    pub fn is_inline(self) -> bool {
        self.repr[3] != Self::EXTENDED_TAG && self.repr[3] != Self::CONTINUATION_TAG
    }

    /// `true` if stored in an [`Arena`].
    #[inline]
    pub fn is_extended(self) -> bool {
        self.repr[3] == Self::EXTENDED_TAG
    }

    /// `true` if empty or a continuation — i.e. carries no resolvable text.
    #[inline]
    pub fn is_none(self) -> bool {
        self.is_empty() || self.is_continuation()
    }
    /// Pack 1..=4 valid UTF-8 bytes into the inline representation.
    ///
    /// # Invariant
    ///
    /// `bytes` must be a non-empty, valid UTF-8 sequence of at most 4 bytes.
    /// Under that invariant `repr[3]` is never a sentinel tag, so the result is
    /// unambiguously inline.
    fn pack_inline(bytes: &[u8]) -> Self {
        debug_assert!(!bytes.is_empty() && bytes.len() <= INLINE_CAPACITY);
        let mut repr = [0u8; INLINE_CAPACITY];
        repr[..bytes.len()].copy_from_slice(bytes);
        debug_assert!(
            repr[3] != Self::EXTENDED_TAG && repr[3] != Self::CONTINUATION_TAG,
            "inline UTF-8 collided with a sentinel tag (invariant violated)"
        );
        Self { repr }
    }
}

impl Grapheme {
    /// Encode a value inline, panicking if it exceeds 4 bytes.
    ///
    /// Accepts anything [`Encodeable`]able (`char`, `&str`). Use
    /// [`try_inline`](Self::try_inline) for the fallible form.
    pub fn inline(value: impl Encodeable) -> Self {
        match value.encode(None) {
            Ok(grapheme) => grapheme,
            Err(err) => panic!("value exceeds 4 bytes and cannot be stored inline"),
        }
    }

    /// Fallible [`inline`](Self::inline). Returns
    /// [`ArenaRequired`](GraphemeError::ArenaRequired) for inputs over 4 bytes.
    pub fn try_inline(value: impl Encodeable) -> Result<Self, GraphemeError> {
        value.encode(None)
    }

    /// Encode a value, spilling to `arena` if it exceeds 4 bytes.
    ///
    /// # Panics
    ///
    /// Panics if the arena is full. Use [`try_extended`](Self::try_extended)
    /// for the fallible form.
    pub fn extended(value: impl Encodeable, arena: &mut Arena) -> Self {
        match value.encode(Some(arena)) {
            Ok(grapheme) => grapheme,
            Err(err) => panic!("failed to encode extended grapheme"),
        }
    }

    /// Fallible [`extended`](Self::extended).
    pub fn try_extended(value: impl Encodeable, arena: &mut Arena) -> Result<Self, GraphemeError> {
        value.encode(Some(arena))
    }

    /// Resolve to a `&str`. Inline graphemes read zero-copy from `self`;
    /// extended graphemes borrow from `arena`. Empty and continuation cells
    /// resolve to `""`.
    pub fn as_str<'a>(&'a self, arena: &'a Arena) -> &'a str {
        if self.is_none() {
            ""
        } else if self.is_inline() {
            self.as_inline_str()
        } else {
            arena.get(*self)
        }
    }

    /// Resolve to a byte slice (see [`as_str`](Self::as_str)).
    pub fn as_bytes<'a>(&'a self, arena: &'a Arena) -> &'a [u8] {
        self.as_str(arena).as_bytes()
    }

    /// Resolve to a `char` iff this grapheme is exactly one scalar value.
    ///
    /// Returns `None` for empty, continuation, extended, or multi-scalar
    /// (e.g. combining-mark) graphemes. Unlike resolving via the arena, this
    /// needs no arena because only inline single-scalar graphemes qualify.
    pub fn try_as_char(self) -> Option<char> {
        if !self.is_inline() {
            return None;
        }
        let mut chars = self.as_inline_str().chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => Some(c),
            _ => None,
        }
    }

    /// Resolve to a `char`, falling back to `U+FFFD` when this is not a single
    /// scalar value.
    pub fn as_char(self) -> char {
        self.try_as_char().unwrap_or(char::REPLACEMENT_CHARACTER)
    }

    /// Display width (in cells) of this grapheme, resolved against `arena`.
    pub fn width(&self, arena: &Arena) -> usize {
        UnicodeWidthStr::width(self.as_str(arena))
    }

    /// The 24-bit arena offset of an extended grapheme.
    ///
    /// Meaningful only when [`is_extended`](Self::is_extended); for other
    /// kinds the low 3 bytes are UTF-8 data, not an offset.
    #[inline]
    pub fn as_offset(self) -> usize {
        u32::from_le_bytes([self.repr[0], self.repr[1], self.repr[2], 0]) as usize
    }

    // ── Inline internals ────────────────────────────────────────────────

    /// Byte length of the inline UTF-8 data: the position of the first zero
    /// padding byte, or 4 if there is none.
    #[inline]
    pub fn inline_len(self) -> usize {
        let mut i = 0;
        while i < INLINE_CAPACITY {
            if self.repr[i] == 0 {
                return i;
            }
            i += 1;
        }
        INLINE_CAPACITY
    }

    /// The inline UTF-8 bytes (without padding).
    pub fn as_inline_bytes(&self) -> &[u8] {
        &self.repr[..self.inline_len()]
    }

    /// The inline bytes as a `&str`.
    pub fn as_inline_str(&self) -> &str {
        // SAFETY: every inline grapheme is constructed from valid UTF-8 via
        // `pack_inline`, and `inline_len` stops at the first padding byte, so
        // the slice is a complete, valid UTF-8 sequence.
        unsafe { std::str::from_utf8_unchecked(self.as_inline_bytes()) }
    }
}

impl From<char> for Grapheme {
    fn from(value: char) -> Self {
        Self::char(value)
    }
}

impl fmt::Debug for Grapheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            f.debug_tuple("Grapheme::Empty").finish()
        } else if self.is_continuation() {
            f.debug_tuple("Grapheme::Continuation").finish()
        } else if self.is_extended() {
            f.debug_tuple("Grapheme::Extended")
                .field(&self.as_offset())
                .finish()
        } else {
            f.debug_tuple("Grapheme::Inline")
                .field(&self.as_inline_str())
                .finish()
        }
    }
}

// ── Encoding ────────────────────────────────────────────────────────────────

/// Source values that can be encoded into a [`Grapheme`].
///
/// `arena` is `Some` to permit spilling clusters over 4 bytes; `None` means
/// inline-only, in which case oversized input yields
/// [`ArenaRequired`](GraphemeError::ArenaRequired).
pub const trait Encodeable {
    fn encode(self, arena: Option<&mut Arena>) -> Result<Grapheme, GraphemeError>;
    fn width(&self) -> usize;
}

impl Encodeable for char {
    fn encode(self, _arena: Option<&mut Arena>) -> Result<Grapheme, GraphemeError> {
        Ok(Grapheme::char(self)) // a scalar always fits inline
    }
    fn width(&self) -> usize {
        UnicodeWidthChar::width(*self).unwrap_or(0)
    }
}

impl Encodeable for &str {
    fn encode(self, arena: Option<&mut Arena>) -> Result<Grapheme, GraphemeError> {
        match self.len() {
            0 => Ok(Grapheme::EMPTY),
            1..=INLINE_CAPACITY => Ok(Grapheme::pack_inline(self.as_bytes())),
            len => match arena {
                Some(arena) => arena.try_insert(self),
                None => Err(GraphemeError::ArenaRequired {
                    len,
                    max: INLINE_CAPACITY,
                }),
            },
        }
    }
    fn width(&self) -> usize {
        UnicodeWidthStr::width(*self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FAMILY: &str = "👨\u{200D}👩\u{200D}👧\u{200D}👦"; // 25 bytes, extended

    #[test]
    fn empty_grapheme() {
        let g = Grapheme::EMPTY;
        assert!(g.is_empty());
        assert!(g.is_inline());
        assert!(!g.is_extended());
        assert!(g.is_none());
    }

    #[test]
    fn space_is_not_empty() {
        assert!(!Grapheme::char(' ').is_empty());
    }

    #[test]
    fn nul_is_empty() {
        assert!(Grapheme::char('\0').is_empty());
    }

    #[test]
    fn inline_ascii() {
        let arena = Arena::new();
        let g = Grapheme::char('A');
        assert!(g.is_inline() && !g.is_empty());
        assert_eq!(g.as_str(&arena), "A");
        assert_eq!(g.as_char(), 'A');
    }

    #[test]
    fn inline_multibyte() {
        let arena = Arena::new();
        let g = Grapheme::try_inline("é").unwrap(); // 2 bytes
        assert!(g.is_inline());
        assert_eq!(g.as_str(&arena), "é");

        let g = Grapheme::try_inline("中").unwrap(); // 3 bytes
        assert!(g.is_inline());
        assert_eq!(g.as_str(&arena), "中");
    }

    #[test]
    fn inline_four_byte_emoji() {
        let arena = Arena::new();
        let g = Grapheme::char('🎉'); // F0 9F 8E 89
        assert!(g.is_inline());
        assert_eq!(g.as_str(&arena), "🎉");
        assert_eq!(g.try_as_char(), Some('🎉'));
    }

    #[test]
    fn inline_combining_is_not_a_single_char() {
        let arena = Arena::new();
        let s = "e\u{0301}"; // 3 bytes, two scalars, fits inline
        let g = Grapheme::try_inline(s).unwrap();
        assert!(g.is_inline());
        assert_eq!(g.as_str(&arena), s);
        assert_eq!(g.try_as_char(), None, "multi-scalar is not a single char");
    }

    #[test]
    fn try_inline_rejects_long() {
        assert!(matches!(
            Grapheme::try_inline(FAMILY),
            Err(GraphemeError::ArenaRequired { .. })
        ));
    }

    #[test]
    fn extended_round_trip() {
        let mut arena = Arena::new();
        let g = Grapheme::extended(FAMILY, &mut arena);
        assert!(g.is_extended() && !g.is_empty());
        assert_eq!(g.as_str(&arena), FAMILY);
        assert_eq!(g.try_as_char(), None);
    }

    #[test]
    fn release_frees_arena_space() {
        let mut arena = Arena::new();
        let before = arena.count();
        let g = Grapheme::extended(FAMILY, &mut arena);
        assert!(arena.count() > before);
        arena.remove(g);
        assert_eq!(arena.count(), before);
    }

    #[test]
    fn from_char_trait() {
        let arena = Arena::new();
        let g: Grapheme = 'A'.into();
        assert_eq!(g.as_str(&arena), "A");
    }

    #[test]
    fn debug_output() {
        assert_eq!(format!("{:?}", Grapheme::EMPTY), "Grapheme::Empty");
        assert_eq!(
            format!("{:?}", Grapheme::CONTINUATION),
            "Grapheme::Continuation"
        );
        assert!(format!("{:?}", Grapheme::char('Z')).contains('Z'));
    }

    #[test]
    fn soh_byte_is_stored_inline() {
        // Unlike notcurses, SOH (0x01) can be stored inline: as a single-byte
        // UTF-8 sequence its tag byte (repr[3]) is 0, not the extended marker.
        let arena = Arena::new();
        let g = Grapheme::char('\x01');
        assert!(g.is_inline() && !g.is_extended() && !g.is_empty());
        assert_eq!(g.as_str(&arena), "\x01");
    }

    #[test]
    fn continuation_is_its_own_tag() {
        let arena = Arena::new();
        let g = Grapheme::CONTINUATION;
        assert!(g.is_continuation() && g.is_none());
        assert!(!g.is_empty() && !g.is_inline() && !g.is_extended());
        assert_eq!(g.as_str(&arena), "");
    }

    #[test]
    fn offset_round_trips() {
        let mut arena = Arena::new();
        let g = Grapheme::extended(FAMILY, &mut arena);
        assert!(g.is_extended());
        assert_eq!(g.as_offset(), 0, "first arena entry sits at offset 0");
    }

    #[test]
    fn from_offset_round_trips() {
        let g = Grapheme::from_offset(0x00AB_CDEF);
        assert!(g.is_extended());
        assert_eq!(g.as_offset(), 0x00AB_CDEF);
    }

    #[test]
    fn width_recovers_from_handle() {
        let mut arena = Arena::new();
        assert_eq!(Grapheme::char('A').width(&arena), 1);
        assert_eq!(Grapheme::char('中').width(&arena), 2);
        let g = Grapheme::extended(FAMILY, &mut arena);
        assert_eq!(g.width(&arena), 2); // emoji ZWJ sequence renders double-width
    }
}
