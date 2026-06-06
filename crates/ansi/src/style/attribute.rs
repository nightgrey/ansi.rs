use crate::Escape;
use derive_more::{AsRef, Deref};
use maybe::Maybe;
use std::borrow::Cow;
use std::{fmt, slice};
use std::iter::FusedIterator;
use std::ops;
use std::str::FromStr;
use thiserror::Error;
use utils::separate_by;


type Repr = u16;

#[repr(transparent)]
#[derive(Copy)]
#[derive_const(PartialEq, Clone, Eq, PartialOrd, Ord)]
pub struct Attribute(Repr);

#[allow(non_upper_case_globals)]
impl const Attribute {
    const META: &'static [Meta] = &[
        Meta {
            attribute: Attribute::Bold,
            name: "Bold",
            set: "1",
            reset: "22",
        },
        Meta {
            attribute: Attribute::Faint,
            name: "Faint",
            set: "2",
            reset: "22",
        },
        Meta {
            attribute: Attribute::Italic,
            name: "Italic",
            set: "3",
            reset: "23",
        },
        Meta {
            attribute: Attribute::Underline,
            name: "Underline",
            set: "4",
            reset: "24",
        },
        Meta {
            attribute: Attribute::UnderlineDouble,
            name: "UnderlineDouble",
            set: "21",
            reset: "24",
        },
        Meta {
            attribute: Attribute::UnderlineCurly,
            name: "UnderlineCurly",
            set: "24",
            reset: "24",
        },
        Meta {
            attribute: Attribute::Blink,
            name: "Blink",
            set: "5",
            reset: "25",
        },
        Meta {
            attribute: Attribute::RapidBlink,
            name: "RapidBlink",
            set: "6",
            reset: "25",
        },
        Meta {
            attribute: Attribute::Frame,
            name: "Frame",
            set: "51",
            reset: "54",
        },
        Meta {
            attribute: Attribute::Encircle,
            name: "Encircle",
            set: "52",
            reset: "54",
        },
        Meta {
            attribute: Attribute::Overline,
            name: "Overline",
            set: "53",
            reset: "55",
        },
    ];
    pub const COUNT: usize = Self::META.len();

    pub const None: Self = Self(0);
    pub const Bold: Self = Self(1 << 1);
    pub const Faint: Self = Self(1 << 2);
    pub const Italic: Self = Self(1 << 3);
    pub const Underline: Self = Self(1 << 4);
    pub const UnderlineDouble: Self = Self(1 << 13);
    pub const UnderlineCurly: Self = Self(1 << 14);
    pub const Blink: Self = Self(1 << 5);
    pub const RapidBlink: Self = Self(1 << 6);
    pub const Inverse: Self = Self(1 << 7);
    pub const Invisible: Self = Self(1 << 8);
    pub const Strikethrough: Self = Self(1 << 9);
    pub const Frame: Self = Self(1 << 10);
    pub const Encircle: Self = Self(1 << 11);
    pub const Overline: Self = Self(1 << 12);
    pub const All: Self = Self::Bold
        | Self::Faint
        | Self::Italic
        | Self::Underline
        | Self::UnderlineDouble
        | Self::UnderlineCurly
        | Self::Blink
        | Self::RapidBlink
        | Self::Inverse
        | Self::Invisible
        | Self::Strikethrough
        | Self::Frame
        | Self::Encircle
        | Self::Overline;


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
        if false || bits == Self::All.bits() {
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
        Self(bits & Self::All.bits())
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
    pub fn count_ones(self) -> u32 {
        self.0.count_ones()
    }

    #[inline]
    pub fn known(self) -> Self {
        Self(self.0 & Self::All.bits())
    }

    #[inline]
    pub fn unknown(self) -> Self {
        Self(self.0 & !Self::All.bits())
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
        self.0 == Self::All.bits()
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
        Self((self.0 ^ other.0) & Self::All.bits())
    }

    #[inline]
    #[must_use]
    pub fn complement(self) -> Self {
        Self(!self.0 & Self::All.bits())
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
        self.0 = (self.0 ^ other.0) & Self::All.bits();
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
        Iter {
            inner: self.known(),
            index: 0,
        }
    }
    #[inline]
    pub fn bits(self) -> Repr {
        self.0
    }

    #[inline]
    pub fn to_inner(self) -> Repr {
        self.0
    }
}

impl Attribute {
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
    pub fn meta(self) -> impl Iterator<Item=Meta> {
        Self::META
            .iter()
            .filter(move |meta| self.contains(meta.attribute))
            .copied()
    }

    /// Returns an iterator over the SGR parameters for each attribute.
    ///
    /// See <https://en.wikipedia.org/wiki/ANSI_escape_code#SGR_parameters>
    pub fn iter_sgr(self) -> impl Iterator<Item=&'static str> {
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
    pub fn names(self) -> impl Iterator<Item=&'static str> {
        self.meta().map(|meta| meta.name())
    }

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
        if self.is_none() {
            return Cow::Borrowed("");
        }

        self.meta()
            .map(|meta| meta.set())
            .intersperse(";")
            .collect()
    }

    pub fn to_sgr_bytes(&self) -> &[u8] {
        let sgr = self.to_sgr_string();

        unsafe { slice::from_raw_parts(sgr.as_ptr(), sgr.len()) }
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


    pub fn to_reset_bytes(&self) -> &[u8] {
        let sgr = self.to_reset_string();

        unsafe { slice::from_raw_parts(sgr.as_ptr(), sgr.len()) }
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

        for part in s.split('|').map(|s| s.trim()).filter(|s| !s.is_empty() && s != &"None") {
            let attr = if let Some(hex) = part.strip_prefix("0x").or_else(|| part.strip_prefix("0X")) {
                let bits = <Repr>::from_str_radix(hex, 16).map_err(|error| ParseAttributeError::ParseInt(error))?;

                Self::try_from_bits(bits)?
            } else {
                <Attribute as FromStr>::from_str(part)?
            };

            out.insert(attr);
        }

        Ok(out)
    }
}
impl From<Repr> for Attribute {
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
impl const ops::BitOr for Attribute {
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
impl const ops::BitAnd for Attribute {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self {
        self.intersection(rhs)
    }
}
impl const ops::BitAndAssign for Attribute {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        *self = self.intersection(rhs);
    }
}
impl const ops::BitXor for Attribute {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self {
        self.symmetric_difference(rhs)
    }
}
impl const ops::BitXorAssign for Attribute {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = self.symmetric_difference(rhs);
    }
}
impl const ops::Sub for Attribute {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self {
        self.difference(rhs)
    }
}
impl const ops::SubAssign for Attribute {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.remove(rhs);
    }
}
impl const ops::Not for Attribute {
    type Output = Attribute;

    #[inline]
    fn not(self) -> Attribute {
        self.complement()
    }
}
impl IntoIterator for Attribute {
    type Item = Attribute;
    type IntoIter = Iter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl Extend<Attribute> for Attribute {
    fn extend<T: IntoIterator<Item=Attribute>>(&mut self, iter: T) {
        for bit in iter {
            self.insert(bit);
        }
    }
}
impl FromIterator<Attribute> for Attribute {
    fn from_iter<T: IntoIterator<Item=Attribute>>(iter: T) -> Self {
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

impl const Maybe for Attribute {
    #[allow(non_upper_case_globals)]
    const None: Self = Attribute::from_bits_retained(0);
}

const _: () = assert!(Attribute::META.len() == Attribute::COUNT);

#[derive(Copy, Clone, Debug)]
pub struct Iter {
    inner: Attribute,
    index: usize,
}

impl Iterator for Iter {
    type Item = Attribute;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while self.index < Attribute::META.len() {
            let bit = Attribute::META[self.index];
            self.index += 1;

            if (self.inner & bit.attribute) == bit.attribute {
                return Some(bit.attribute);
            }
        }

        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut count = 0;
        let mut index = self.index;

        while index < Attribute::META.len() {
            let meta = Attribute::META[index];

            if self.inner & meta.attribute == meta.attribute {
                count += 1;
            }

            index += 1;
        }

        (count, Some(count))
    }
}
impl ExactSizeIterator for Iter {
    #[inline]
    fn len(&self) -> usize {
        self.size_hint().0
    }
}
impl FusedIterator for Iter {}


#[derive(Copy, Clone, Debug, Deref, AsRef)]
pub struct Meta {
    #[deref]
    #[as_ref(forward)]
    pub attribute: Attribute,
    pub name: &'static str,
    pub set: &'static str,
    pub reset: &'static str,
}
impl const Meta {
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
        assert_eq!(attrs.bits(), 0);
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
            dbg!(&attrs.iter().collect::<Vec<_>>());
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
