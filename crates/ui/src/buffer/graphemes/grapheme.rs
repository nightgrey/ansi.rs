use super::Slot;
use crate::{Graphemes, GraphemesError};
use std::fmt;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// A compact grapheme-cluster handle stored in 4 bytes.
///
/// This is the core unit of text storage in the framebuffer. It is a hand-rolled
/// niche-packed sum type — morally
/// `enum { Empty, Inline([u8; ≤4]), Extended(u24) }` — squeezed
/// into `[u8; 4]`. The packing exploits a property the compiler can't see: the
/// **4th byte** of any valid UTF-8 sequence of ≤4 bytes is either `0x00`
/// (padding) or a UTF-8 continuation byte (`0x80..=0xBF`), so it can never
/// collide with the sentinel tag `0x01`.
///
/// The representation is a *byte array*, not an integer, so the layout is
/// identical on every target — there is no endianness to reason about.
///
/// # Encoding (`repr[3]` is always the tag)
///
/// ```text
/// repr == [0, 0, 0, 0x00]   → empty (no grapheme)
/// repr[3] == 0x01           → extended; repr[0..3] = 24-bit arena slot (LE)
/// otherwise                 → inline UTF-8 (1..=4 bytes, zero-padded)
/// ```
///
/// - **Inline** clusters of ≤4 UTF-8 bytes are stored directly — zero heap
///   allocation for ASCII, Latin, Cyrillic, CJK, and most single-codepoint
///   emoji. Reads are zero-copy on every target.
/// - **Extended** clusters (ZWJ sequences, skin-tone modifiers, …) live in an
///   [`Graphemes`] and are referenced by a 24-bit slot, giving 16 MiB of space.
///
/// # NUL handling
///
/// NUL (`0x00`) is the inline padding byte, so a standalone NUL is
/// indistinguishable from empty: [`from_char('\0')`](Self::from_char) yields
/// [`EMPTY`](Self::EMPTY). This is intentional — NUL has no visual cell.
#[derive(Copy, Hash)]
#[derive_const(Clone, PartialEq, Eq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Grapheme {
    /// UTF-8 / tag bytes in storage order. `repr[3]` is the tag.
    inner: [u8; Grapheme::MAX_LEN],
}

const impl Grapheme {
    const MAX_LEN: usize = char::MAX_LEN_UTF8;

    /// Sentinel tag for an extended (arena-stored) grapheme.
    pub(super) const EXTENDED_TAG: u8 = 0x01;

    /// Empty cell content (no character).
    pub const EMPTY: Self = Self {
        inner: [0, 0, 0, 0],
    };

    #[inline]
    pub fn try_new(value: impl [const] IntoGrapheme) -> Result<Self, GraphemesError> {
        value.try_into_grapheme()
    }

    #[inline]
    pub fn new(value: impl [const] IntoGrapheme) -> Self {
        match Self::try_new(value) {
            Ok(g) => g,
            Err(_err) => panic!("failed to encode grapheme"),
        }
    }

    pub fn try_from_char(char: char) -> Result<Self, GraphemesError> {
        let mut bytes = [0; Grapheme::MAX_LEN];
        char.encode_utf8(&mut bytes);
        Ok(Self::from_bytes_unchecked(bytes))
    }

    pub fn from_char(char: char) -> Self {
        match Self::try_from_char(char) {
            Ok(g) => g,
            Err(_err) => panic!("failed to encode char"),
        }
    }

    pub fn try_from_str(s: &str) -> Result<Self, GraphemesError> {
        match s.len() {
            0 => Ok(Self::EMPTY),
            1..=Grapheme::MAX_LEN => Ok(Self::pack_inline(s.as_bytes())),
            len => Err(GraphemesError::RequiresStorage { len }),
        }
    }

    pub fn from_str(s: &str) -> Self {
        match Self::try_from_str(s) {
            Ok(g) => g,
            Err(_err) => panic!("failed to encode string"),
        }
    }

    /// Try to encode raw UTF-8 bytes as an inline grapheme.
    ///
    /// Returns `Ok` for 0..=4 bytes of valid UTF-8 (empty input produces
    /// [`EMPTY`](Self::EMPTY)). Returns [`TooLong`](GraphemesError::TooLong)
    /// for input exceeding 4 bytes, and [`Invalid`](GraphemesError::Invalid)
    /// for non-UTF-8 byte sequences.
    pub fn try_from_bytes(bytes: &[u8]) -> Result<Self, GraphemesError> {
        if bytes.is_empty() {
            return Ok(Self::EMPTY);
        }

        if bytes.len() > Self::MAX_LEN {
            return Err(GraphemesError::TooLong {
                len: bytes.len(),
                max: Self::MAX_LEN,
            });
        }

        if str::from_utf8(bytes).is_err() {
            return Err(GraphemesError::Invalid);
        }

        let mut inner = [0u8; Self::MAX_LEN];
        inner[..bytes.len()].copy_from_slice(bytes);
        Ok(Self { inner })
    }

    /// Encode raw UTF-8 bytes as an inline grapheme.
    ///
    /// # Panics
    ///
    /// Panics if `bytes` exceeds 4 bytes or contains invalid UTF-8.
    /// Use [`try_from_bytes`](Self::try_from_bytes) for fallible conversion.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        match Self::try_from_bytes(bytes) {
            Ok(g) => g,
            Err(_err) => panic!("failed to encode bytes"),
        }
    }

    /// Encode raw UTF-8 bytes without validation.
    ///
    /// The caller guarantees that `bytes` contains 0..=4 UTF-8 bytes
    /// (zero-padded if shorter than 4). No checks are performed — invalid
    /// input produces an unresolvable or garbled grapheme.
    pub fn from_bytes_unchecked(bytes: [u8; Grapheme::MAX_LEN]) -> Self {
        Self { inner: bytes }
    }

    /// `true` if empty (no character).
    #[inline]
    pub fn is_empty(self) -> bool {
        self == Self::EMPTY
    }

    /// `true` if stored inline (≤4 UTF-8 bytes). [`EMPTY`](Self::EMPTY) counts
    /// as inline; the extended tag does not.
    #[inline]
    pub fn is_inline(self) -> bool {
        self.inner[3] != Self::EXTENDED_TAG
    }

    /// `true` if stored in an [`Graphemes`].
    #[inline]
    pub fn is_extended(self) -> bool {
        self.inner[3] == Self::EXTENDED_TAG
    }

    /// The raw 4-byte representation.
    ///
    /// Byte `[3]` is the tag: `0x01` means extended (arena slot in bytes
    /// `[0..3]`), anything else means inline UTF-8 (zero-padded).
    #[inline]
    pub fn as_bytes(&self) -> &[u8; Grapheme::MAX_LEN] {
        &self.inner
    }

    /// Pack 1..=4 valid UTF-8 bytes into the inline representation.
    ///
    /// # Invariant
    ///
    /// `bytes` must be a non-empty, valid UTF-8 sequence of at most 4 bytes.
    /// Under that invariant `repr[3]` is never a sentinel tag, so the result is
    /// unambiguously inline.
    fn pack_inline(bytes: &[u8]) -> Self {
        debug_assert!(!bytes.is_empty() && bytes.len() <= char::MAX_LEN_UTF8);
        let mut repr = [0u8; Grapheme::MAX_LEN];
        repr[..bytes.len()].copy_from_slice(bytes);
        debug_assert!(
            repr[3] != Self::EXTENDED_TAG,
            "inline UTF-8 collided with a sentinel tag (invariant violated)"
        );
        Self { inner: repr }
    }
}

impl Grapheme {
    /// Resolve to a `&str`. Inline graphemes read zero-copy from `self`;
    /// extended graphemes borrow from `arena`. Empty and continuation cells
    /// resolve to `""`.
    pub fn as_str<'a>(&'a self, arena: &'a Graphemes) -> &'a str {
        arena.get(self)
    }

    /// Resolve to a `&str`. Inline graphemes read zero-copy from `self`;
    /// extended graphemes borrow from `arena`. Empty and continuation cells
    /// resolve to `default`.
    pub fn as_str_or<'a>(&'a self, arena: &'a Graphemes, default: &'a str) -> &'a str {
        arena.get_or(self, default)
    }

    /// Resolve to a `char` iff this grapheme is exactly one scalar value.
    ///
    /// Returns `None` for empty, continuation, extended, or multi-scalar
    /// (e.g. combining-mark) graphemes. Unlike resolving via the arena, this
    /// needs no arena because only inline single-scalar graphemes qualify.
    pub fn try_as_char(self) -> Option<char> {
        let s = self.try_as_inline_str()?;
        let mut chars = s.chars();
        let first = chars.next()?;
        // Exactly one scalar value: reject empty and multi-scalar clusters.
        chars.next().is_none().then_some(first)
    }

    /// Resolve to a `char`, falling back to `U+FFFD` when this is not a single
    /// scalar value.
    pub fn as_char(self) -> char {
        self.try_as_char().unwrap_or(char::REPLACEMENT_CHARACTER)
    }

    /// Resolve to a 24-bit arena slot.
    ///
    /// Meaningful only when [`is_extended`](Self::is_extended); for other
    /// kinds the low 3 bytes are UTF-8 data, not an slot.
    #[inline]
    pub fn slot(self) -> Option<Slot> {
        if !self.is_extended() {
            return None;
        }

        Some(Slot::new(u32::from_le_bytes([
            self.inner[0],
            self.inner[1],
            self.inner[2],
            0,
        ])))
    }

    // ── Inline internals ────────────────────────────────────────────────

    /// Byte length of the inline UTF-8 data: the position of the first zero
    /// padding byte, or 4 if there is none.
    #[inline]
    pub fn len_utf8(self) -> usize {
        const MAX_1: u8 = 0x7F;
        const MIN_2: u8 = 0xC2;
        const MAX_2: u8 = 0xDF;
        const MIN_3: u8 = 0xE0;
        const MAX_3: u8 = 0xEF;
        const MIN_4: u8 = 0xF0;
        const MAX_4: u8 = 0xF4;

        if self.is_extended() {
            return 0;
        }

        let byte = self.inner[0];

        if byte <= MAX_1 {
            1
        } else if byte <= MAX_2 {
            2
        } else if byte <= MAX_3 {
            3
        } else {
            4
        }
    }

    #[inline]
    pub fn try_as_inline_bytes(&self) -> Option<&[u8]> {
        if !self.is_inline() {
            return None;
        }
        Some(&self.inner[..self.len_utf8()])
    }

    pub fn as_inline_bytes(&self) -> &[u8] {
        self.try_as_inline_bytes().expect("inline grapheme")
    }

    #[inline]
    pub fn try_as_inline_str(&self) -> Option<&str> {
        if !self.is_inline() {
            return None;
        }
        // SAFETY: every inline grapheme is constructed from valid UTF-8 via
        // `pack_inline`, and `len` stops at the first padding byte, so the
        // slice is a complete, valid UTF-8 sequence.
        Some(unsafe { std::str::from_utf8_unchecked(self.as_inline_bytes()) })
    }

    pub fn as_inline_str(&self) -> &str {
        self.try_as_inline_str().expect("inline grapheme")
    }
}

impl From<char> for Grapheme {
    fn from(value: char) -> Self {
        Self::from_char(value)
    }
}

impl fmt::Debug for Grapheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            f.debug_tuple("Grapheme::Empty").finish()
        } else if self.is_extended() {
            f.debug_tuple("Grapheme::Extended")
                .field(&self.slot())
                .finish()
        } else if self.is_inline() {
            f.debug_tuple("Grapheme::Inline")
                .field(&self.as_inline_str())
                .finish()
        } else {
            f.debug_tuple("Grapheme").field(&self.inner).finish()
        }
    }
}

/// Source values that can be encoded into a [`Grapheme`].
///
/// `arena` is `Some` to permit spilling clusters over 4 bytes; `None` means
/// inline-only, in which case oversized input yields
/// [`ArenaRequired`](GraphemesError::RequiresStorage).
pub const trait IntoGrapheme {
    fn try_into_grapheme(self) -> Result<Grapheme, GraphemesError>;
    fn into_grapheme(self) -> Grapheme
    where
        Self: Sized,
    {
        match self.try_into_grapheme() {
            Ok(g) => g,
            Err(_err) => panic!("failed to convert into grapheme"),
        }
    }
}

pub trait IntoGraphemeWidth {
    fn width(&self) -> usize;
}

impl IntoGraphemeWidth for Grapheme {
    fn width(&self) -> usize {
        if self.is_extended() {
            panic!("extended grapheme width cannot be determined");
        }

        UnicodeWidthStr::width(self.as_inline_str())
    }
}

const impl IntoGrapheme for Grapheme {
    fn try_into_grapheme(self) -> Result<Grapheme, GraphemesError> {
        Ok(self)
    }
}

const impl IntoGrapheme for char {
    fn try_into_grapheme(self) -> Result<Grapheme, GraphemesError> {
        Ok(Grapheme::from_char(self)) // a scalar always fits inline
    }
}

impl IntoGraphemeWidth for char {
    fn width(&self) -> usize {
        UnicodeWidthChar::width(*self).unwrap_or(0)
    }
}

impl IntoGrapheme for (&str, &mut Graphemes) {
    fn try_into_grapheme(self) -> Result<Grapheme, GraphemesError> {
        let (str, arena) = self;
        match str.len() {
            0 => Ok(Grapheme::EMPTY),
            1..=Grapheme::MAX_LEN => Ok(Grapheme::from_bytes(str.as_bytes())),
            _len => arena.try_insert(str),
        }
    }
}

impl IntoGraphemeWidth for (&str, &mut Graphemes) {
    fn width(&self) -> usize {
        UnicodeWidthStr::width(self.0)
    }
}

const impl IntoGrapheme for &str {
    fn try_into_grapheme(self) -> Result<Grapheme, GraphemesError> {
        Grapheme::try_from_str(self)
    }
}

impl IntoGraphemeWidth for &str {
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
        assert!(g.is_empty());
    }

    #[test]
    fn space_is_not_empty() {
        assert!(!Grapheme::from_char(' ').is_empty());
    }

    #[test]
    fn nul_is_empty() {
        assert!(Grapheme::from_char('\0').is_empty());
    }

    #[test]
    fn inline_ascii() {
        let arena = Graphemes::new();
        let g = Grapheme::from_char('A');
        assert!(g.is_inline() && !g.is_empty());
        assert_eq!(g.as_str(&arena), "A");
        assert_eq!(g.as_char(), 'A');
    }

    #[test]
    fn inline_multibyte() {
        let arena = Graphemes::new();
        let g = Grapheme::try_new("é").unwrap(); // 2 bytes
        assert!(g.is_inline());
        assert_eq!(g.as_str(&arena), "é");

        let g = Grapheme::try_new("中").unwrap(); // 3 bytes
        assert!(g.is_inline());
        assert_eq!(g.as_str(&arena), "中");
    }

    #[test]
    fn inline_four_byte_emoji() {
        let arena = Graphemes::new();
        let g = Grapheme::from_char('🎉'); // F0 9F 8E 89
        assert!(g.is_inline());
        assert_eq!(g.as_str(&arena), "🎉");
        assert_eq!(g.try_as_char(), Some('🎉'));
    }

    #[test]
    fn inline_combining_is_not_a_single_char() {
        let arena = Graphemes::new();
        let s = "e\u{0301}"; // 3 bytes, two scalars, fits inline
        let g = Grapheme::try_new(s).unwrap();
        assert!(g.is_inline());
        assert_eq!(g.as_str(&arena), s);
        assert_eq!(g.try_as_char(), None, "multi-scalar is not a single char");
    }

    #[test]
    fn try_inline_rejects_long() {
        assert!(matches!(
            Grapheme::try_new(FAMILY),
            Err(GraphemesError::RequiresStorage { .. })
        ));
    }

    #[test]
    fn extended_round_trip() {
        let mut arena = Graphemes::new();
        let g = Grapheme::new((FAMILY, &mut arena));
        assert!(g.is_extended() && !g.is_empty());
        assert_eq!(g.as_str(&arena), FAMILY);
        assert_eq!(g.try_as_char(), None);
    }

    #[test]
    fn remove_frees_arena_space() {
        let mut arena = Graphemes::new();
        let before = arena.count_occupied();
        let g = arena.insert(FAMILY);
        assert!(arena.count_occupied() > before);
        arena.remove(g);
        assert_eq!(arena.count_occupied(), before);
    }

    #[test]
    fn from_char_trait() {
        let arena = Graphemes::new();
        let g: Grapheme = 'A'.into();
        assert_eq!(g.as_str(&arena), "A");
    }

    #[test]
    fn debug_output() {
        assert_eq!(format!("{:?}", Grapheme::EMPTY), "Grapheme::Empty");
        assert!(format!("{:?}", Grapheme::from_char('Z')).contains('Z'));
    }

    #[test]
    fn soh_byte_is_stored_inline() {
        // Unlike notcurses, SOH (0x01) can be stored inline: as a single-byte
        // UTF-8 sequence its tag byte (repr[3]) is 0, not the extended marker.
        let arena = Graphemes::new();
        let g = Grapheme::from_char('\x01');
        assert!(g.is_inline() && !g.is_extended() && !g.is_empty());
        assert_eq!(g.as_str(&arena), "\x01");
    }

    #[test]
    fn slot_round_trips() {
        let mut arena = Graphemes::new();
        let g = Grapheme::new((FAMILY, &mut arena));
        assert!(g.is_extended());
        assert_eq!(
            Slot::try_from_grapheme(g).unwrap().as_usize(),
            0,
            "first arena entry sits at slot 0"
        );
    }

    #[test]
    fn from_slot_round_trips() {
        let g = Slot::new(0x00AB_CDEF).into_grapheme();
        assert!(g.is_extended());
        assert_eq!(Slot::try_from_grapheme(g).unwrap().as_usize(), 0x00AB_CDEF);
    }
}
