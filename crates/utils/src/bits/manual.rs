// A  MANUAL IMPL OF THE MACRO RESULT.

use std::fmt;
use std::str::FromStr;
use utils::{BitsError, Bits, Bit, BitsIter, Base};

#[repr(u16)]
#[derive(Copy, Debug)]
#[derive_const(Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum Attribute {
    #[doc = r" Increases the text intensity."]   Bold = (1 << 1),
    #[doc = r" Decreases the text intensity."]   Faint = (1 << 2),
    #[doc = r" Emphasises the text."]   Italic = (1 << 3),
    #[doc = r" Underlines the text with a single line."]   Underline = (1 << 4),
    #[doc = r" Underlines the text with a double line."]   UnderlineDouble = (1 << 13),
    #[doc = r" Underlines the text with a curly line."]   UnderlineCurly = (1 << 14),
    #[doc = r" Makes the text blink."]   Blink = (1 << 5),
    #[doc = r" Makes the text blink rapidly."]   RapidBlink = (1 << 6),
    #[doc = r" Swaps the foreground and background colors."]   Inverse = (1 << 7),
    #[doc = r" Hides the text."]   Invisible = (1 << 8),
    #[doc = r" Crosses the text out."]   Strikethrough = (1 << 9),
    #[doc = r" Frames the text."]   Frame = (1 << 10),
    #[doc = r" Encircles the text."]   Encircle = (1 << 11),
    #[doc = r" Draws a line at the top of the text."]   Overline = (1 << 12),
}

impl const Base for Attribute {
    type Repr = u16;

    #[inline]
    fn from_repr(repr: u16) -> Self {
        unsafe { std::mem::transmute(repr) }
    }

    #[inline]
    fn into_repr(self) -> u16 {
        self as u16
    }
}
impl const Bit for Attribute {
    const LIST: &'static [(Self, &'static str)] = &[
        (Attribute::Bold, stringify!( Bold )), (Attribute::Faint, stringify!( Faint )), (Attribute::Italic, stringify!( Italic )), (Attribute::Underline, stringify!( Underline )), (Attribute::UnderlineDouble, stringify!( UnderlineDouble )), (Attribute::UnderlineCurly, stringify!( UnderlineCurly )), (Attribute::Blink, stringify!( Blink )), (Attribute::RapidBlink, stringify!( RapidBlink )), (Attribute::Inverse, stringify!( Inverse )), (Attribute::Invisible, stringify!( Invisible )), (Attribute::Strikethrough, stringify!( Strikethrough )), (Attribute::Frame, stringify!( Frame )), (Attribute::Encircle, stringify!( Encircle )), (Attribute::Overline, stringify!( Overline )),
    ];
}

impl const From<Attribute> for u16 {
    #[inline]
    fn from(value: Attribute) -> Self {
        value as u16
    }
}

impl const Attribute {
    const LIST: &'static [(Self, &'static str)] = &[
        (Attribute::Bold, "Bold"), (Attribute::Faint, "Faint"), (Attribute::Italic, "Italic"), (Attribute::Underline, "Underline"), (Attribute::UnderlineDouble, "UnderlineDouble"), (Attribute::UnderlineCurly, "UnderlineCurly"), (Attribute::Blink, "Blink"), (Attribute::RapidBlink, "RapidBlink"), (Attribute::Inverse, "Inverse"), (Attribute::Invisible, "Invisible"), (Attribute::Strikethrough, "Strikethrough"), (Attribute::Frame, "Frame"), (Attribute::Encircle, "Encircle"), (Attribute::Overline, "Overline"),
    ];

    #[allow(non_upper_case_globals)]
    const None: u16 = 0;

    #[inline]
    fn from_repr(repr: u16) -> Self {
        unsafe { std::mem::transmute(repr) }
    }

    #[inline]
    fn into_repr(self) -> u16 {
        self as u16
    }

    #[inline]
    fn bits(self) -> u16 {
        unsafe { std::mem::transmute(self) }
    }
}
#[doc = r" Attributes"]
#[doc = r" A compact representation of ANSI SGR attributes."]
///   A set of [`
#[doc = "Attribute"]
///   `] bits.
#[repr(transparent)]
#[derive(Copy)]
#[derive_const(Clone)]
pub struct Attributes(u16);

#[allow(non_upper_case_globals)]
impl const Attributes {
    /// Every flag, in declaration order. Drives iteration and counting.
    const LIST: &'static [(Attribute, &'static str)] = Attribute::LIST;
    /// Number of declared flags.
    const COUNT: usize = Self::LIST.len();

    pub const None: Self = Self::new(Attribute::None);
    pub const Bold: Self = Self((1 << 1));
    pub const Faint: Self = Self((1 << 2));
    pub const Italic: Self = Self((1 << 3));
    pub const Underline: Self = Self((1 << 4));
    pub const UnderlineDouble: Self = Self((1 << 13));
    pub const UnderlineCurly: Self = Self((1 << 14));
    pub const Blink: Self = Self((1 << 5));
    pub const RapidBlink: Self = Self((1 << 6));
    pub const Inverse: Self = Self((1 << 7));
    pub const Invisible: Self = Self((1 << 8));
    pub const Strikethrough: Self = Self((1 << 9));
    pub const Frame: Self = Self((1 << 10));
    pub const Encircle: Self = Self((1 << 11));
    pub const Overline: Self = Self((1 << 12));
    pub const All: Self = Self::from_repr((Attribute::Bold as u16) | (Attribute::Faint as u16) | (Attribute::Italic as u16) | (Attribute::Underline as u16) | (Attribute::UnderlineDouble as u16) | (Attribute::UnderlineCurly as u16) | (Attribute::Blink as u16) | (Attribute::RapidBlink as u16) | (Attribute::Inverse as u16) | (Attribute::Invisible as u16) | (Attribute::Strikethrough as u16) | (Attribute::Frame as u16) | (Attribute::Encircle as u16) | (Attribute::Overline as u16));

    /// Construct new bits.
    pub fn new(bits: impl [ const ] Into<u16>) -> Self {
        Self(bits.into())
    }

    /// Wrap raw bits as-is, keeping any unknown bits. Cheapest constructor.
    #[inline]
    pub fn from_bits_retained(bits: impl [ const ] Into<u16>) -> Self {
        Self(bits.into())
    }

    /// Wrap raw bits, masking away anything outside [`Bit::ALL`].
    #[inline]
    pub fn from_bits_truncated(bits: impl [ const ] Into<u16>) -> Self {
        Self(bits.into() & Attributes::All.bits())
    }

    /// Like [`from_bits`](Self::from_bits) but returns a typed error.
    #[inline]
    pub fn try_from_bits(bits: impl [ const ] Into<u16>) -> Result<Self, BitsError> {
        let bits = bits.into();

        if bits & !Attributes::All.bits() == Attributes::None.bits() {
            Ok(Self(bits))
        } else {
            Err(BitsError::Unknown)
        }
    }

    /// Wrap raw bits, or `None` if any unknown bit is set.
    #[inline]
    pub fn from_bits(bits: impl [ const ] Into<u16>) -> Self {
        match Self::try_from_bits(bits) {
            Ok(b) => b,
            Err(_) => panic!("invalid bits"),
        }
    }

    /// The raw integer behind this set.
    #[inline]
    pub fn bits(self) -> u16 {
        self.0
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self == Self::None
    }

    #[inline]
    pub fn is_all(self) -> bool {
        self == Self::All
    }

    /// `true` if every flag in `other` is present.
    #[inline]
    pub fn contains(self, other: impl [ const ] Into<Self>) -> bool {
        let o = other.into();
        self.0 & o.0 == o.0
    }

    /// `true` if any flag is shared.
    #[inline]
    pub fn intersects(self, other: impl [ const ] Into<Self>) -> bool {
        self & other.into() != Self::None
    }

    /// `true` if no flag is shared.
    #[inline]
    pub fn is_disjoint(self, other: impl [ const ] Into<Self>) -> bool {
        self & other.into() == Self::None
    }

    /// Set union (`|`).
    #[inline]
    #[must_use]
    pub fn union(self, other: impl [ const ] Into<Self>) -> Self {
        Self(self.0 | other.into().0)
    }

    /// Set intersection (`&`).
    #[inline]
    #[must_use]
    pub fn intersection(self, other: impl [ const ] Into<Self>) -> Self {
        Self(self.0 & other.into().0)
    }

    /// Flags in `self` but not `other` (`self & !other`).
    #[inline]
    #[must_use]
    pub fn difference(self, other: impl [ const ] Into<Self>) -> Self {
        Self(self.0 & !other.into().0)
    }

    /// Flags in exactly one of the two sets (XOR, masked to valid bits).
    #[inline]
    #[must_use]
    pub fn symmetric_difference(self, other: impl [ const ] Into<Self>) -> Self {
        Self((self.0 ^ other.into().0) & Attributes::All.bits())
    }

    /// All valid flags not in `self`.
    #[inline]
    #[must_use]
    pub fn complement(self) -> Self {
        Self(!self.0 & Attributes::All.bits())
    }

    #[inline]
    pub fn insert(&mut self, other: impl [ const ] Into<u16>) {
        self.0 = self.0 | other.into();
    }

    #[inline]
    pub fn remove(&mut self, other: impl [ const ] Into<u16>) {
        self.0 = self.0 & !other.into();
    }

    #[inline]
    pub fn toggle(&mut self, other: impl [ const ] Into<u16>) {
        self.0 = (self.0 ^ other.into()) & Attributes::All.bits();
    }

    /// Insert when `on`, remove otherwise.
    #[inline]
    pub fn set(&mut self, other: impl [ const ] Into<u16>, on: bool) {
        if on {
            self.insert(other)
        } else {
            self.remove(other)
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        *self = Self::None;
    }

    /// Iterate the individual flags present in this set.
    #[inline]
    pub fn iter(self) -> BitsIter<Self> {
        BitsIter::new(self)
    }

    /// Number of flags yielded by iteration.
    #[inline]
    pub fn count(self) -> usize {
        self.iter().count()
    }


    #[inline]
    fn from_repr(repr: u16) -> Self {
        Attributes(repr)
    }

    #[inline]
    fn into_repr(self) -> u16 {
        self.0
    }
}
impl const Base for Attributes {
    type Repr = u16;

    #[inline]
    fn from_repr(repr: u16) -> Self {
        Attributes(repr)
    }

    #[inline]
    fn into_repr(self) -> u16 {
        self.0
    }
}
impl const Bits for Attributes {
    type Bit = Attribute;


    #[allow(non_upper_case_globals)]
    const None: Self = Self::from_repr(0);

    #[allow(non_upper_case_globals)]
    const All: Self = Self::from_repr((Attribute::Bold as u16) | (Attribute::Faint as u16) | (Attribute::Italic as u16) | (Attribute::Underline as u16) | (Attribute::UnderlineDouble as u16) | (Attribute::UnderlineCurly as u16) | (Attribute::Blink as u16) | (Attribute::RapidBlink as u16) | (Attribute::Inverse as u16) | (Attribute::Invisible as u16) | (Attribute::Strikethrough as u16) | (Attribute::Frame as u16) | (Attribute::Encircle as u16) | (Attribute::Overline as u16));
}

impl const From<Attributes> for u16 {
    #[inline]
    fn from(value: Attributes) -> Self {
        value.0
    }
}

impl const From<Attribute> for Attributes {
    #[inline]
    fn from(value: Attribute) -> Self {
        Attributes(value as u16)
    }
}

impl const From<u16> for Attributes {
    #[inline]
    fn from(value: u16) -> Self {
        Attributes(value)
    }
}

impl const Default for Attributes {
    #[inline]
    fn default() -> Self {
        Self::None
    }
}
impl const std::ops::BitAnd for Attribute {
    type Output = Attributes;
    #[inline]
    fn bitand(self, rhs: Self) -> Attributes {
        Attributes((self as u16) & (rhs as u16))
    }
}
impl const std::ops::BitOr for Attribute {
    type Output = Attributes;
    #[inline]
    fn bitor(self, rhs: Self) -> Attributes {
        Attributes((self as u16) | (rhs as u16))
    }
}
impl const std::ops::BitXor for Attribute {
    type Output = Attributes;
    #[inline]
    fn bitxor(self, rhs: Self) -> Attributes {
        Attributes((self as u16) ^ (rhs as u16))
    }
}
impl const std::ops::Not for Attribute {
    type Output = Attributes;
    #[inline]
    fn not(self) -> Attributes {
        Attributes(!(self as u16) & Attributes::All.bits())
    }
}
impl<I: [ const ]   Into<Attributes>> const std::ops::BitOr<I> for Attributes {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: I) -> Self {
        Attributes::union(self, rhs)
    }
}
impl<I: [ const ]   Into<Attributes>> const std::ops::BitOrAssign<I> for Attributes {
    #[inline]
    fn bitor_assign(&mut self, rhs: I) {
        *self = Attributes::union(*self, rhs);
    }
}
impl<I: [ const ]   Into<Attributes>> const std::ops::BitAnd<I> for Attributes {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: I) -> Self {
        Attributes::intersection(self, rhs)
    }
}
impl<I: [ const ]   Into<Attributes>> const std::ops::BitAndAssign<I> for Attributes {
    #[inline]
    fn bitand_assign(&mut self, rhs: I) {
        *self = Attributes::intersection(*self, rhs);
    }
}
impl<I: [ const ]   Into<Attributes>> const std::ops::Sub<I> for Attributes {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: I) -> Self {
        Attributes::difference(self, rhs)
    }
}
impl<I: [ const ]   Into<Attributes>> const std::ops::SubAssign<I> for Attributes {
    #[inline]
    fn sub_assign(&mut self, rhs: I) {
        *self = Attributes::difference(*self, rhs);
    }
}
impl<I: [ const ]   Into<Attributes>> const std::ops::BitXor<I> for Attributes {
    type Output = Self;
    #[inline]
    fn bitxor(self, rhs: I) -> Self {
        Attributes::symmetric_difference(self, rhs)
    }
}
impl<I: [ const ]   Into<Attributes>> const std::ops::BitXorAssign<I> for Attributes {
    #[inline]
    fn bitxor_assign(&mut self, rhs: I) {
        *self = Attributes::symmetric_difference(*self, rhs);
    }
}
impl<I: [ const ]   Into<Attributes>> const std::ops::Rem<I> for Attributes {
    type Output = Self;
    #[inline]
    fn rem(self, rhs: I) -> Self {
        Attributes::symmetric_difference(self, rhs)
    }
}
impl<I: [ const ]   Into<Attributes>> const std::ops::RemAssign<I> for Attributes {
    #[inline]
    fn rem_assign(&mut self, rhs: I) {
        *self = Attributes::symmetric_difference(*self, rhs);
    }
}
impl const std::ops::Not for Attributes {
    type Output = Self;
    #[inline]
    fn not(self) -> Self {
        Attributes::complement(self)
    }
}
impl<I: Copy + [ const ]   Into<Attributes>> const PartialEq<I> for Attributes {
    #[inline]
    fn eq(&self, other: &I) -> bool {
        self.0 == Attributes::into_repr((*other).into())
    }
}
impl Eq for Attributes {}
impl fmt::Debug for Attributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{}(", stringify!( Attributes )))?;
        for (i, flag) in self.iter().enumerate() {
            if i > 0 {
                f.write_str(" | ")?;
            }
            f.write_fmt(format_args!("{flag:?}", flag = flag))?;
        }
        f.write_str(")")
    }
}
impl const IntoIterator for Attributes {
    type Item = Attribute;
    type IntoIter = BitsIter<Attributes>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        BitsIter::new(self)
    }
}
impl<I: Into<u16>> Extend<I> for Attributes {
    fn extend<T: IntoIterator<Item=I>>(&mut self, iter: T) {
        for item in iter {
            Attributes::insert(self, item);
        }
    }
}
impl<I: Into<u16>> FromIterator<I> for Attributes {
    fn from_iter<T: IntoIterator<Item=I>>(iter: T) -> Self {
        let mut set = Attributes::None;
        set.extend(iter);
        set
    }
}


impl FromStr for Attribute {
    type Err = BitsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        <u16>::from_str_radix(s, 16)
            .map_err(|_| BitsError::Invalid)
            .map(Attribute::from_repr)
    }
}

impl FromStr for Attributes {
    type Err = BitsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parsed_flags = Self::None;

        if s.trim().is_empty() {
            return Ok(parsed_flags);
        }

        for flag in s.split('|') {
            let flag = flag.trim();


            if flag.is_empty() {
                return Err(BitsError::Empty);
            }


            let parsed_flag = if let Some(hex) = flag.strip_prefix("0x") {
                Attributes::from_bits_retained(
                    <u16>::from_str_radix(hex, 16)
                        .map_err(|_| BitsError::Invalid)?,
                )
            } else {
                let mut found = None;
                for (bit, name) in Attributes::LIST {
                    if *name == flag {
                        found = Some(Attributes::from_bits_retained(
                            Attribute::into_repr(*bit),
                        ));
                        break;
                    }
                }
                found.ok_or(BitsError::Invalid)?
            };

            Attributes::insert(&mut parsed_flags, parsed_flag);
        }

        Ok(parsed_flags)
    }
}

pub type AttributesIter = BitsError;
pub type AttributesError = BitsError;

#[test]
fn qwe() {
    let attrs = Attributes::Bold | Attribute::Bold;
}