use crate::Escape;
use derive_more::{AsRef, Deref};
use maybe::Maybe;
use std::borrow::Cow;
use std::fmt;
use std::iter::Map;
use std::marker::Destruct;
use std::ops;
use std::str::FromStr;
use thiserror::Error;

macro_rules! variants {
    (
        $(
            $(#[$attr:meta])*
                $variant:ident {
                    position: $position:expr,
                    name: $name:expr,
                    set: $set:expr,
                    reset: $reset:expr,
                }
        ),+
        $(,)?
    ) => {
            pub const META: &'static [Variant] = &[
                $(
                    Variant {
                        attribute: Attribute::$variant,
                        name: $name,
                        set: $set,
                        reset: $reset,
                    },
                )+
            ];

            pub const COUNT: usize = Self::META.len();


            pub const None: Self = Self(0);
            // Bit‑flag constants – these match the enum variants.
            // `None` is explicitly given (bit 0), but its Meta entry is omitted
            // (no set/reset). You can choose to include/exclude it.
            $(
                $(#[$attr])*
                pub const $variant: Self = Self(1 << $position);
            )+
            pub const All: Self = $(Self::$variant)|+;
    };
}

#[derive(Copy, Clone, Debug, Deref, AsRef)]
pub struct Variant {
    #[deref]
    #[as_ref(forward)]
    pub attribute: Attribute,
    pub name: &'static str,
    pub set: &'static str,
    pub reset: &'static str,
}
const impl Variant {
    fn from_position(position: usize) -> Self {
        Attribute::META[position]
    }

    fn from_attribute(attr: Attribute) -> Self {
        Self::from_position(attr.0.trailing_zeros() as usize)
    }

    fn attribute(&self) -> Attribute {
        self.attribute
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn set(&self) -> &'static str {
        self.set
    }

    fn reset(&self) -> &'static str {
        self.reset
    }
}

type Repr = u16;

#[repr(transparent)]
#[derive(Copy)]
#[derive_const(PartialEq, Clone, Eq, PartialOrd, Ord)]
pub struct Attribute(Repr);

#[allow(non_upper_case_globals)]
const impl Attribute {
    variants! {
        Bold {
            position: 0,
            name: "Bold",
            set: "1",
            reset: "22",
        },
        Faint {
            position: 1,
            name: "Faint",
            set: "2",
            reset: "22",
        },
        Italic {
            position: 2,
            name: "Italic",
            set: "3",
            reset: "23",
        },
        Underline {
            position: 3,
            name: "Underline",
            set: "4",
            reset: "24",
        },
        Blink {
            position: 4,
            name: "Blink",
            set: "5",
            reset: "25",
        },
        RapidBlink {
            position: 5,
            name: "RapidBlink",
            set: "6",
            reset: "25",
        },
        Inverse {
            position: 6,
            name: "Inverse",
            set: "7",
            reset: "27",
        },
        Invisible {
            position: 7,
            name: "Invisible",
            set: "8",
            reset: "28",
        },
        Strikethrough {
            position: 8,
            name: "Strikethrough",
            set: "9",
            reset: "29",
        },
        UnderlineDouble {
            position: 9,
            name: "UnderlineDouble",
            set: "21",
            reset: "24",
        },
        UnderlineCurly {
            position: 10,
            name: "UnderlineCurly",
            set: "23",
            reset: "24",
        },
        Frame {
            position: 11,
            name: "Frame",
            set: "51",
            reset: "54",
        },
        Encircle {
            position: 12,
            name: "Encircle",
            set: "52",
            reset: "54",
        },
        Overline {
            position: 13,
            name: "Overline",
            set: "53",
            reset: "55",
        },
    }

    /// Creates an empty attribute.
    #[inline]
    pub fn empty() -> Self {
        Self::None
    }

    /// Creates an attribute from bits.
    ///
    /// Equavalent to [`Attribute::from_bits_retained`].
    #[inline]
    pub fn new(bits: Repr) -> Self {
        Self::from_bits_retained(bits)
    }

    #[inline]
    pub fn from_bits(bits: Repr) -> Self {
        match Self::try_from_bits(bits) {
            Ok(attribute) => attribute,
            Err(_) => panic!("invalid bits"),
        }
    }

    #[inline]
    pub fn try_from_bits(bits: Repr) -> Result<Self, ParseAttributeError> {
        if false || bits == Self::All.into_inner() {
            Ok(Self(bits))
        } else {
            Err(ParseAttributeError::Unknown(bits))
        }
    }

    #[inline]
    pub fn from_bits_retained(bits: Repr) -> Self {
        Self(bits)
    }

    #[inline]
    pub fn from_bits_truncated(bits: Repr) -> Self {
        Self(bits & Self::All.into_inner())
    }

    #[inline]
    pub fn from_bits_unchecked(bits: Repr) -> Self {
        Self(bits)
    }

    #[inline]
    pub fn from_bits_or_default(bits: Repr) -> Self {
        Self::try_from_bits(bits).unwrap_or(Self::None)
    }

    #[inline]
    pub fn try_from_position(position: usize) -> Result<Self, ParseAttributeError> {
        if position >= Self::COUNT {
            return Err(ParseAttributeError::Invalid(position as u16));
        }
        Ok(Self(1 << position))
    }

    #[inline]
    pub fn from_position(position: usize) -> Self {
        match Self::try_from_position(position) {
            Ok(attr) => attr,
            Err(_) => panic!("invalid position"),
        }
    }

    #[inline]
    pub fn count_ones(self) -> u32 {
        self.0.count_ones()
    }

    #[inline]
    pub fn known(self) -> Self {
        Self(self.0 & Self::All.into_inner())
    }

    #[inline]
    pub fn unknown(self) -> Self {
        Self(self.0 & !Self::All.into_inner())
    }

    #[inline]
    pub fn has_unknown_bits(self) -> bool {
        self.unknown() != Self::None
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn is_all(self) -> bool {
        self.0 == Self::All.into_inner()
    }

    #[inline]
    pub fn equals(self, other: Self) -> bool {
        self.0 == other.0
    }

    #[inline]
    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    #[inline]
    pub fn intersects(self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    #[inline]
    pub fn is_disjoint(self, other: Self) -> bool {
        self.0 & other.0 == 0
    }

    #[inline]
    #[must_use]
    pub fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[inline]
    #[must_use]
    pub fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    #[inline]
    #[must_use]
    pub fn difference(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    #[inline]
    #[must_use]
    pub fn symmetric_difference(self, other: Self) -> Self {
        Self((self.0 ^ other.0) & Self::All.into_inner())
    }

    #[inline]
    #[must_use]
    pub fn complement(self) -> Self {
        Self(!self.0 & Self::All.into_inner())
    }

    #[inline]
    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    #[inline]
    pub fn remove(&mut self, other: Self) {
        self.0 &= !other.0;
    }

    #[inline]
    pub fn toggle(&mut self, other: Self) {
        self.0 = (self.0 ^ other.0) & Self::All.into_inner();
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0 = 0;
    }

    #[inline]
    pub fn count(self) -> usize {
        self.known().count_ones() as usize
    }

    #[inline]
    pub fn iter(self) -> Iter {
        Iter::new(self.into_inner())
    }

    /// Returns an iterator over the meta data for every attribute defined in [`Self`].
    ///
    /// # Example
    ///
    /// ```
    /// use ansi::Attribute;
    ///
    /// assert!(Attribute::All.meta().any(|meta| meta.name == "Bold"));
    /// assert!(Attribute::All.meta().any(|meta| meta.name == "Italic"));
    /// assert_eq!(Attribute::All.meta().count(), Attribute::COUNT);
    /// ```
    #[inline]
    pub fn meta(self) -> MetaIter {
        MetaIter::new(self.into_inner())
    }

    /// Returns an iterator over the SGR parameters for each attribute.
    ///
    /// See <https://en.wikipedia.org/wiki/ANSI_escape_code#SGR_parameters>
    pub fn iter_sgr(self) -> impl Iterator<Item = &'static str> {
        self.meta().map(|meta| meta.set())
    }

    /// Returns an iterator over the names of attributes in [`Self`].
    ///
    /// # Example
    ///
    /// ```
    /// use ansi::Attribute;
    ///
    /// let attrs = Attribute::Bold | Attribute::Italic;
    ///
    /// assert_eq!(attrs.names().collect::<Vec<_>>(), vec!["Bold", "Italic"]);
    /// ```
    #[inline]
    pub fn names(self) -> impl Iterator<Item = &'static str> {
        self.meta().map(|meta| meta.name())
    }

    #[inline]
    pub fn into_inner(self) -> Repr {
        self.0
    }
}

impl Attribute {
    /// Returns the semicolon-separated SGR parameters to set attributes.
    ///
    /// # Example
    ///
    /// ```
    /// use ansi::Attribute;
    ///
    /// let attrs = Attribute::Bold | Attribute::Italic;
    /// assert_eq!(attrs.to_sgr_string(), "1;3");
    /// ```
    ///
    /// See <https://en.wikipedia.org/wiki/ANSI_escape_code#SGR_parameters>
    pub fn to_sgr_string(self) -> Cow<'static, str> {
        self.meta()
            .map(|meta| meta.set())
            .intersperse(";")
            .collect()
    }

    pub fn to_sgr_bytes(&self) -> Cow<'static, [u8]> {
        match self.to_sgr_string() {
            Cow::Borrowed(s) => Cow::Borrowed(s.as_bytes()),
            Cow::Owned(s) => Cow::Owned(s.into_bytes()),
        }
    }

    /// Returns the semicolon-separated SGR parameters to reset attributes.
    ///
    /// # Example
    ///
    /// ```
    /// use ansi::Attribute;
    ///
    /// let attrs = Attribute::Bold | Attribute::Italic;
    ///
    /// assert_eq!(attrs.to_reset_string(), "22;23");
    /// ```
    ///
    /// See <https://en.wikipedia.org/wiki/ANSI_escape_code#SGR_parameters>
    pub fn to_reset_string(self) -> Cow<'static, str> {
        if self.is_none() {
            return Cow::Borrowed("");
        }

        self.meta()
            .map(|meta| meta.reset())
            .intersperse(";")
            .collect()
    }

    pub fn to_reset_bytes(&self) -> Cow<'static, [u8]> {
        match self.to_reset_string() {
            Cow::Borrowed(s) => Cow::Borrowed(s.as_bytes()),
            Cow::Owned(s) => Cow::Owned(s.into_bytes()),
        }
    }

    /// Returns a string representation of the attributes.
    ///
    /// # Example
    ///
    /// ```
    /// use ansi::Attribute;
    ///
    /// let attrs = Attribute::Bold | Attribute::Italic;
    ///
    /// assert_eq!(attrs.to_string(), "Bold | Italic");
    /// ```
    pub fn to_string(&self) -> Cow<'_, str> {
        self.names().intersperse(" | ").collect()
    }
}

impl fmt::Debug for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return f.write_str("Attribute::None");
        }

        f.debug_tuple("Attribute")
            .field(&fmt::from_fn(|f| fmt::Display::fmt(self, f)))
            .finish()
    }
}
impl fmt::Display for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.to_string().as_ref())
    }
}
impl FromStr for Attribute {
    type Err = ParseAttributeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.is_empty() || s == "None" {
            return Ok(Self::None);
        }

        let mut out = Self::empty();

        for part in s
            .split('|')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && s != &"None")
        {
            let attr =
                if let Some(hex) = part.strip_prefix("0x").or_else(|| part.strip_prefix("0X")) {
                    let bits =
                        <Repr>::from_str_radix(hex, 16).map_err(ParseAttributeError::ParseInt)?;

                    Self::try_from_bits(bits)?
                } else {
                    <Attribute as FromStr>::from_str(part)?
                };

            out.insert(attr);
        }

        Ok(out)
    }
}
const impl From<Repr> for Attribute {
    #[inline]
    fn from(value: Repr) -> Self {
        Attribute::new(value)
    }
}
impl Default for Attribute {
    #[inline]
    fn default() -> Self {
        Self::None
    }
}
const impl ops::BitOr for Attribute {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self {
        self.union(rhs)
    }
}
impl ops::BitOrAssign for Attribute {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.insert(rhs);
    }
}
const impl ops::BitAnd for Attribute {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        self.intersection(rhs)
    }
}
const impl ops::BitAndAssign for Attribute {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        *self = self.intersection(rhs);
    }
}
const impl ops::BitXor for Attribute {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self {
        self.symmetric_difference(rhs)
    }
}
const impl ops::BitXorAssign for Attribute {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = self.symmetric_difference(rhs);
    }
}
const impl ops::Sub for Attribute {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        self.difference(rhs)
    }
}
const impl ops::SubAssign for Attribute {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.remove(rhs);
    }
}
const impl ops::Not for Attribute {
    type Output = Attribute;

    #[inline]
    fn not(self) -> Attribute {
        self.complement()
    }
}
const impl IntoIterator for Attribute {
    type Item = Attribute;
    type IntoIter = Map<Iter, fn(usize) -> Attribute>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
            .map(|i| Attribute::new((i as u16).saturating_sub(1)))
    }
}
impl Extend<Attribute> for Attribute {
    fn extend<T: IntoIterator<Item = Attribute>>(&mut self, iter: T) {
        for bit in iter {
            self.insert(bit);
        }
    }
}
impl FromIterator<Attribute> for Attribute {
    fn from_iter<T: IntoIterator<Item = Attribute>>(iter: T) -> Self {
        let mut out = Self::None;
        out.extend(iter);
        out
    }
}
impl Escape for Attribute {
    fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        w.write_all(self.to_sgr_string().as_bytes())
    }
}

const impl Maybe for Attribute {
    #[allow(non_upper_case_globals)]
    const None: Self = Attribute::from_bits_retained(0);
}

#[derive(Debug, Clone, Deref)]
pub struct MetaIter {
    inner: Iter,
}

const impl MetaIter {
    #[inline]
    pub fn new(value: u16) -> Self {
        Self {
            inner: Iter::new(value),
        }
    }
}

const impl Iterator for MetaIter {
    type Item = Variant;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.inner.next()?;

        Some(Variant::from_position(next))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    #[inline]
    fn count(self) -> usize {
        self.inner.count()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        if let Some(last) = self.inner.last() {
            return Some(Variant::from_position(last));
        }
        None
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let mut i = 0;
        while self.inner.0 != 0 && i < n {
            self.inner.clear_max();
            i += 1;
        }
        self.next()
    }

    #[inline]
    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        F: [const] FnMut(B, Self::Item) -> B + [const] Destruct,
    {
        let mut accum = init;
        for item in self {
            accum = f(accum, item);
        }
        accum
    }

    #[inline]
    fn max(self) -> Option<Self::Item> {
        self.last()
    }

    #[inline]
    fn min(self) -> Option<Self::Item> {
        if self.inner.0 != 0 {
            Some(Variant::from_position(self.inner.min_bit()))
        } else {
            None
        }
    }

    fn is_sorted(self) -> bool {
        true
    }
}
impl ExactSizeIterator for MetaIter {
    fn len(&self) -> usize {
        self.count_ones()
    }
}

#[derive(Debug, Clone, Deref)]
#[repr(transparent)]
pub struct Iter(u16);

const impl Iter {
    #[inline]
    pub fn new(value: Repr) -> Self {
        Self(value)
    }

    #[inline]
    fn max_bit(&self) -> usize {
        self.0.trailing_zeros() as usize
    }

    #[inline]
    fn min_bit(&self) -> usize {
        (<u16>::BITS - 1 - self.0.leading_zeros()) as usize
    }

    #[inline]
    fn count_ones(&self) -> usize {
        self.0.count_ones() as usize
    }

    #[inline]
    fn clear_max(&mut self) {
        self.0 &= self.0.wrapping_sub(1);
    }
}

const impl Iterator for Iter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 != 0 {
            let position = self.max_bit();
            self.clear_max();
            Some(position)
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let sz = self.count_ones();
        (sz, Some(sz))
    }

    #[inline]
    fn count(self) -> usize {
        self.count_ones()
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        if self.0 != 0 {
            Some(self.min_bit())
        } else {
            None
        }
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let mut i = 0;
        while self.0 != 0 && i < n {
            self.clear_max();
            i += 1;
        }
        self.next()
    }

    #[inline]
    fn fold<B, F>(mut self, init: B, mut f: F) -> B
    where
        F: [const] FnMut(B, Self::Item) -> B + [const] Destruct,
    {
        let mut accum = init;
        while self.0 != 0 {
            accum = f(accum, self.max_bit());
            self.clear_max();
        }
        accum
    }

    #[inline]
    fn max(self) -> Option<Self::Item> {
        self.last()
    }

    #[inline]
    fn min(self) -> Option<Self::Item> {
        if self.0 != 0 {
            Some(self.max_bit())
        } else {
            None
        }
    }

    fn is_sorted(self) -> bool {
        true
    }
}
impl ExactSizeIterator for Iter {
    fn len(&self) -> usize {
        self.count_ones()
    }
}

#[derive(Clone, Debug, Error)]
pub enum ParseAttributeError {
    #[error("empty bits")]
    Empty,
    #[error("invalid bits")]
    Invalid(u16),
    #[error("unknown bits")]
    Unknown(u16),
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_attributes() {
        let attrs = Attribute::None;
        assert_eq!(attrs.into_inner(), 0);
        assert!(attrs.is_empty());
        assert!(!attrs.is_some());
        assert_eq!(attrs.iter_sgr().count(), 0);
    }

    #[test]
    fn single_attribute() {
        let bold = Attribute::Bold;
        assert!(bold.contains(Attribute::Bold));
        assert!(!bold.contains(Attribute::Italic));
        assert!(bold.is_some());
    }

    #[test]
    fn combine_attributes() {
        let styled = Attribute::Bold | Attribute::Italic | Attribute::Underline;
        assert!(styled.contains(Attribute::Bold));
        assert!(styled.contains(Attribute::Italic));
        assert!(styled.contains(Attribute::Underline));
        assert!(!styled.contains(Attribute::Strikethrough));
    }

    #[test]
    fn union() {
        let attrs = Attribute::Bold.union(Attribute::Italic);
        assert!(attrs.contains(Attribute::Bold));
        assert!(attrs.contains(Attribute::Italic));
    }

    #[test]
    fn intersection() {
        let a = Attribute::Bold | Attribute::Italic;
        let b = Attribute::Italic | Attribute::Underline;
        let result = a.intersection(b);
        assert!(!result.contains(Attribute::Bold));
        assert!(result.contains(Attribute::Italic));
        assert!(!result.contains(Attribute::Underline));
    }

    #[test]
    fn difference() {
        let a = Attribute::Bold | Attribute::Italic;
        let result = a.difference(Attribute::Italic);
        assert!(result.contains(Attribute::Bold));
        assert!(!result.contains(Attribute::Italic));
    }

    #[test]
    fn symmetric_difference() {
        let a = Attribute::Bold | Attribute::Italic;
        let b = Attribute::Italic | Attribute::Underline;
        let result = a.symmetric_difference(b);
        assert!(result.contains(Attribute::Bold));
        assert!(!result.contains(Attribute::Italic));
        assert!(result.contains(Attribute::Underline));
    }

    #[test]
    fn insert_remove() {
        let mut attrs = Attribute::None;
        attrs.insert(Attribute::Bold);
        assert!(attrs.contains(Attribute::Bold));
        attrs.remove(Attribute::Bold);
        assert!(!attrs.contains(Attribute::Bold));
    }

    #[test]
    fn toggle() {
        let mut attrs = Attribute::Bold;
        attrs.toggle(Attribute::Bold);
        assert!(!attrs.contains(Attribute::Bold));
        attrs.toggle(Attribute::Bold);
        assert!(attrs.contains(Attribute::Bold));
    }
    #[test]
    fn clear() {
        let mut attrs = Attribute::Bold | Attribute::Italic;
        attrs.clear();
        assert!(attrs.is_none());
    }

    mod sgr {
        use super::*;

        #[test]
        fn sgr_single() {
            let bold = Attribute::Bold;
            let sgr: Vec<&'static str> = bold.iter_sgr().collect();
            assert_eq!(sgr, vec!["1"]);
        }

        #[test]
        fn sgr_multiple() {
            let styled = Attribute::Bold | Attribute::Italic | Attribute::Underline;
            let sgr: Vec<&'static str> = styled.iter_sgr().collect();
            assert!(sgr.contains(&"1")); // Bold
            assert!(sgr.contains(&"3")); // Italic
            assert!(sgr.contains(&"4")); // Underline
            assert_eq!(sgr.len(), 3);
        }

        #[test]
        fn sgr_frame_encircle_overline() {
            let attrs = Attribute::Frame | Attribute::Encircle | Attribute::Overline;
            let sgr: Vec<&'static str> = attrs.iter_sgr().collect();
            dbg!(&sgr);
            assert!(sgr.contains(&"51")); // Frame
            assert!(sgr.contains(&"52")); // Encircle
            assert!(sgr.contains(&"53")); // Overline
        }
    }

    #[test]
    fn debug_empty() {
        let attrs = Attribute::None;
        assert_eq!(format!("{:?}", attrs), "Attribute::None");
    }

    #[test]
    fn debug_single() {
        let attrs = Attribute::Bold;
        assert_eq!(format!("{:?}", attrs), "Attribute(Bold)");
    }

    #[test]
    fn debug_multiple() {
        let attrs = Attribute::Bold | Attribute::Italic;
        dbg!(&attrs.is_empty());
        let debug = format!("{:?}", attrs);
        dbg!(&debug);
        assert!(debug.contains("Bold"));
        assert!(debug.contains("Italic"));
        assert!(debug.contains(" | "));
    }

    #[test]
    fn constants_all() {
        let all = Attribute::All;
        assert!(all.contains(Attribute::Bold));
        assert!(all.contains(Attribute::Italic));
        assert!(all.contains(Attribute::Overline));

        assert!(all.is_all());
    }
}
