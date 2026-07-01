use super::Slot;
use std::fmt;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use super::{ExtendedSource, InlineSource, Graphemes, GraphemesError, Source};

/// A compact grapheme-cluster handle stored in 4 bytes.
///
/// # Representations
///
/// The representation is a *byte array*, not an integer, so the layout is
/// identical on every target — there is no endianness to reason about.
///
/// **Inline `[UTF-8 #1, UTF-8 #2, UTF-8 #3, UTF-8 #4]`**
///
/// An UTF-8 sequence of ≤4 bytes is stored directly. This means zero-copy reads for the most common
/// case.
///
/// **Extended `[Slot #1, Slot #2, Slot #3, 0x01]`**
///
/// An UTF-8 sequence of >4 bytes is stored in an [`Graphemes`] arena. In this case, the bytes represent a [`Slot`].
///
/// **Empty `[0x00, 0x00, 0x00, 0x00]`**
///
/// Represents no grapheme.
///
/// > Note: A standalone `NUL` (`0x00`) is indistinguishable from [`Grapheme::EMPTY`]. This is intentional, as `NUL` has no visual cell.
#[derive(Copy, Hash)]
#[derive_const(Clone, PartialEq, Eq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Grapheme {
    inner: [u8; Grapheme::MAX_LEN],
}

impl Grapheme {
    /// Sentinel tag for an extended (arena-stored) grapheme.
    pub(super) const EXTENDED_TAG: u8 = 0x01;

    /// Length of an inline grapheme in bytes.
    pub const MAX_LEN: usize = char::MAX_LEN_UTF8;

    /// Empty cell content (no character).
    pub const EMPTY: Self = Self {
        inner: [0, 0, 0, 0],
    };

    #[inline]
    pub const fn try_new<T: [const] Source>(value: T) -> Result<Self, GraphemesError> {
        value.try_into()
    }

    #[inline]
    pub const fn new<T: [const] Source>(value: T) -> Self {
        value.into()
    }

    #[inline]
    pub const fn try_inline<T: [const] InlineSource>(value: T) -> Result<Self, GraphemesError> {
        value.try_into()
    }

    #[inline]
    pub const fn inline<T: [const] InlineSource>(value: T) -> Self {
        value.into()
    }

    #[inline]
    pub fn try_extended<T: ExtendedSource>(value: T) -> Result<Self, GraphemesError> {
        value.try_into()
    }

    #[inline]
    pub fn extended<T: ExtendedSource>(value: T) -> Self {
        T::into(value)
    }

    /// Encode raw UTF-8 bytes unchecked.
    ///
    /// The caller guarantees that `bytes` contains 0..=4 UTF-8 bytes
    /// (zero-padded if shorter than 4). No checks are performed — invalid
    /// input produces an unresolvable or garbled grapheme.
    pub const fn from_bytes_unchecked(bytes: [u8; Grapheme::MAX_LEN]) -> Self {
        Self { inner: bytes }
    }

    /// `true` if stored inline (≤4 UTF-8 bytes). [`EMPTY`](Self::EMPTY) counts
    /// as inline; the extended tag does not.
    #[inline]
    pub const fn is_inline(self) -> bool {
        self.inner[3] != Self::EXTENDED_TAG
    }

    /// `true` if stored in an [`Graphemes`].
    #[inline]
    pub const fn is_extended(self) -> bool {
        self.inner[3] == Self::EXTENDED_TAG
    }

    /// `true` if empty (no character).
    #[inline]
    pub const fn is_empty(self) -> bool {
        self == Self::EMPTY
    }

    pub fn try_as_str<'a>(&'a self, arena: &'a Graphemes) -> Option<&'a str> {
        arena.get(self)
    }

    pub fn as_str<'a>(&'a self, arena: &'a Graphemes) -> &'a str {
        arena.get(self).unwrap_or("")
    }

    pub fn as_str_or<'a>(&'a self, arena: &'a Graphemes, default: &'a str) -> &'a str {
        arena.get_or(self, default)
    }

    #[inline]
    pub fn try_as_inline_str(&self) -> Option<&str> {
        // SAFETY: every inline grapheme is constructed from valid UTF-8.
        self.is_inline().then(|| unsafe { str::from_utf8_unchecked(&self.inner[..self.len_utf8()]) })
    }

    #[inline]
    pub fn as_inline_str(&self) -> &str {
        self.try_as_inline_str().unwrap_or("")
    }

    #[inline]
    pub fn as_inline_str_or<'a>(&'a self, default: &'a str) -> &'a str {
        self.try_as_inline_str().unwrap_or(default)
    }

    /// The inner byte representation.
    #[inline]
    pub const fn as_inner(&self) -> &[u8; Grapheme::MAX_LEN] {
        &self.inner
    }

    /// Returns the byte length of the UTF-8 data.
    #[inline]
    fn len_utf8(self) -> usize {
        if self.is_extended() {
            return 0;
        }

        const MAX_1: u8 = 0x7F;
        const MIN_2: u8 = 0xC2;
        const MAX_2: u8 = 0xDF;
        const MIN_3: u8 = 0xE0;
        const MAX_3: u8 = 0xEF;
        const MIN_4: u8 = 0xF0;
        const MAX_4: u8 = 0xF4;


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
}

impl fmt::Debug for Grapheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            f.debug_tuple("Grapheme::Empty").finish()
        } else if self.is_extended() {
            f.debug_tuple("Grapheme::Extended")
                .field(&Slot::try_from(self))
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

#[cfg(test)]
mod tests {
    use crate::ExtendedBy;
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
        assert!(!Grapheme::inline(' ').is_empty());
    }

    #[test]
    fn nul_is_empty() {
        assert!(Grapheme::inline('\0').is_empty());
    }

    #[test]
    fn inline_ascii() {
        let arena = Graphemes::new();
        let g = Grapheme::inline('A');
        assert!(g.is_inline() && !g.is_empty());
        assert_eq!(g.as_str(&arena), "A");
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
        let g = Grapheme::inline('🎉'); // F0 9F 8E 89
        assert!(g.is_inline());
        assert_eq!(g.as_str(&arena), "🎉");
    }

    #[test]
    fn inline_combining_is_not_a_single_char() {
        let arena = Graphemes::new();
        let s = "e\u{0301}"; // 3 bytes, two scalars, fits inline
        let g = Grapheme::try_new(s).unwrap();
        assert!(g.is_inline());
        assert_eq!(g.as_str(&arena), s);
    }

    #[test]
    fn try_inline_rejects_long() {
        assert!(matches!(
            Grapheme::try_new(FAMILY),
            Err(GraphemesError::RequiresArena { .. })
        ));
    }

    #[test]
    fn extended_round_trip() {
        let mut arena = Graphemes::new();
        let g = Grapheme::new(FAMILY.extended(&mut arena));
        assert!(g.is_extended() && !g.is_empty());
        assert_eq!(g.as_str(&arena), FAMILY);
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
        let g: Grapheme = Grapheme::inline('A');
        assert_eq!(g.as_str(&arena), "A");
    }

    #[test]
    fn debug_output() {
        assert_eq!(format!("{:?}", Grapheme::EMPTY), "Grapheme::Empty");
        assert!(format!("{:?}", Grapheme::inline('Z')).contains('Z'));
    }

    #[test]
    fn soh_byte_is_stored_inline() {
        // Unlike notcurses, SOH (0x01) can be stored inline: as a single-byte
        // UTF-8 sequence its tag byte (repr[3]) is 0, not the extended marker.
        let arena = Graphemes::new();
        let g = Grapheme::inline('\x01');
        assert!(g.is_inline() && !g.is_extended() && !g.is_empty());
        assert_eq!(g.as_str(&arena), "\x01");
    }

    #[test]
    fn slot_round_trips() {
        let mut arena = Graphemes::new();
        let g = Grapheme::new((FAMILY, &mut arena));
        assert!(g.is_extended());
        assert_eq!(
            Slot::try_from(g).unwrap().as_usize(),
            0,
            "first arena entry sits at slot 0"
        );
    }

    #[test]
    fn from_slot_round_trips() {
        let g: Grapheme = Grapheme::from(Slot::new(0x00AB_CDEF));
        assert!(g.is_extended());
        assert_eq!(Slot::try_from(g).unwrap().as_usize(), 0x00AB_CDEF);
    }
}
