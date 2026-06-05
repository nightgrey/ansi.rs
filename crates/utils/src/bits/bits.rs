//! A compact, const-friendly bitflag/set toolkit.
//!
//! Three concepts, three names:
//!
//! * [`Bit`]        — a single flag (one enum variant).
//! * [`Bits<B>`]    — a *set* of flags; reads as the plural of `Bit`.
//! * [`Bit::Repr`]  — the unsigned integer that physically stores the bits.
//!
//! The set is a single generic newtype defined once here, so the [`bits!`]
//! macro only emits the per-enum parts that genuinely cannot be generic:
//! the variants, their bit layout, and the `enum -> Bits` conversion.
//!
//! Almost every method accepts `impl Into<Bits<B>>`, so a bare `Bit`, a
//! `Bits` set, and a borrowed/owned mix are all interchangeable arguments.
//!
//! ----------------------------------------------------------------------
//! NOTE: this targets the same nightly you're already on (`const_trait_impl`,
//! `[const]` bounds, `derive_const`). It was written against your `number`
//! crate's API by name only and has not been compiled here — expect to nudge
//! a `[const]` bound or two, and to confirm `number::Unsigned: Copy` plus the
//! const bit-operators. The structure is the point.
//! ----------------------------------------------------------------------

use core::fmt::Debug;
use std::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Rem, RemAssign, Sub,
    SubAssign,
};
use std::marker::Destruct;

/// A single flag: one variant of a bit enum.
///
/// Deliberately small. A flag does not need to be a whole boolean algebra —
/// it only needs to know its representation, the universe of valid bits, and
/// how to become a [`Bits`] set. All set behaviour lives on [`Bits`].
pub const trait Bit: Sized
+ [ const ] Destruct
+ Copy
+ [ const ] Default
+ [ const ] PartialEq
+ [ const ] Eq
+ [ const ] PartialOrd
+ Debug
+ [ const ] Ord
+ [ const ] BitAnd<Self, Output=Bits<Self>>
+ [ const ] BitOr<Self, Output=Bits<Self>>
+ [ const ] BitXor<Self, Output=Bits<Self>>
+ [ const ] Not<Output=Bits<Self>>
+ [ const ] Into<Self::Repr>
+ 'static
{
    /// The unsigned integer that stores the bits.
    type Repr: Sized
    + [ const ] Destruct
    + Copy
    + [ const ] Default
    + [ const ] PartialEq
    + [ const ] Eq
    + [ const ] PartialOrd
    + Debug
    + [ const ] Ord
    + [ const ] BitAnd<Self::Repr, Output=Self::Repr>
    + [ const ] BitOr<Self::Repr, Output=Self::Repr>
    + [ const ] BitXor<Self::Repr, Output=Self::Repr>
    + [ const ] Not<Output=Self::Repr>;

    /// Every flag, in declaration order. Drives iteration and counting.
    const LIST: &'static [Self];
    /// Number of declared flags.
    const COUNT: usize = Self::LIST.len();

    /// Bitwise OR of every flag in [`LIST`](Self::LIST): the mask of all valid bits.
    const ALL: Self::Repr;
    /// Empty.
    const EMPTY: Self::Repr;
}

#[repr(transparent)]
#[derive(Copy)]
#[derive_const(Clone)]
pub struct Bits<B: Bit>(B::Repr);

impl<B: [ const ] Bit> const Bits<B> {
    /// Every flag, in declaration order. Drives iteration and counting.
    pub const LIST: &'static [B] = B::LIST;
    /// Number of declared flags.
    pub const COUNT: usize = B::COUNT;

    /// The empty set.
    pub fn empty() -> Self {
        Self::new(B::EMPTY)
    }

    /// Every valid flag.
    pub fn all() -> Self {
        Self::new(B::ALL)
    }

    /// Construct new bits.
    pub fn new(bits: impl [ const ] Into<B::Repr>) -> Self {
        Self(bits.into())
    }

    /// Wrap raw bits as-is, keeping any unknown bits. Cheapest constructor.
    #[inline]
    pub fn from_bits_retained(bits: impl [ const ] Into<B::Repr>) -> Self {
        Self(bits.into())
    }

    /// Wrap raw bits, masking away anything outside [`Bit::ALL`].
    #[inline]
    pub fn from_bits_truncated(bits: impl [ const ] Into<B::Repr>) -> Self {
        Self(bits.into() & B::ALL)
    }

    /// Like [`from_bits`](Self::from_bits) but returns a typed error.
    #[inline]
    pub fn try_from_bits(bits: impl [ const ] Into<B::Repr>) -> Result<Self, BitsError> {
        let bits = bits.into();

        if bits & !B::ALL == B::EMPTY {
            Ok(Self(bits))
        } else {
            Err(BitsError::Unknown)
        }
    }

    /// Wrap raw bits, panicking if any bit outside [`Bit::ALL`] is set.
    ///
    /// Use [`try_from_bits`](Self::try_from_bits) for a fallible version, or
    /// [`from_bits_truncated`](Self::from_bits_truncated) to silently drop
    /// unknown bits.
    #[inline]
    pub fn from_bits(bits: impl [ const ] Into<B::Repr>) -> Self {
        match Self::try_from_bits(bits) {
            Ok(b) => b,
            Err(_) => panic!("Bits::from_bits: unknown bits set"),
        }
    }

    /// The raw integer behind this set.
    #[inline]
    pub fn bits(self) -> B::Repr {
        self.0
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self == Self::empty()
    }

    #[inline]
    pub fn is_all(self) -> bool {
        self == Self::all()
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
        self & other.into() != Self::empty()
    }

    /// `true` if no flag is shared.
    #[inline]
    pub fn is_disjoint(self, other: impl [ const ] Into<Self>) -> bool {
        self & other.into() == Self::empty()
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
        Self((self.0 ^ other.into().0) & B::ALL)
    }

    /// All valid flags not in `self`.
    #[inline]
    #[must_use]
    pub fn complement(self) -> Self {
        Self(!self.0 & B::ALL)
    }

    #[inline]
    pub fn insert(&mut self, other: impl [ const ] Into<Self>) {
        self.0 = self.0 | other.into().0;
    }

    #[inline]
    pub fn remove(&mut self, other: impl [ const ] Into<Self>) {
        self.0 = self.0 & !other.into().0;
    }

    #[inline]
    pub fn toggle(&mut self, other: impl [ const ] Into<Self>) {
        self.0 = (self.0 ^ other.into().0) & B::ALL;
    }

    /// Insert when `on`, remove otherwise.
    #[inline]
    pub fn set(&mut self, other: impl [ const ] Into<Self>, on: bool) {
        if on {
            self.insert(other)
        } else {
            self.remove(other)
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        *self = Self::empty();
    }

    /// Iterate the individual flags present in this set.
    #[inline]
    pub fn iter(self) -> BitsIter<B> {
        BitsIter::new(self)
    }

    /// Number of flags yielded by iteration.
    #[inline]
    pub fn count(self) -> usize {
        self.iter().count()
    }
}

impl<B: [ const ] Bit> const IntoIterator for Bits<B> {
    type Item = B;
    type IntoIter = BitsIter<B>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        BitsIter::new(self)
    }
}

impl<B: Bit, I: Into<Bits<B>>> Extend<I> for Bits<B> {
    fn extend<T: IntoIterator<Item=I>>(&mut self, iter: T) {
        for item in iter {
            self.insert(item);
        }
    }
}

impl<B: Bit, I: Into<Bits<B>>> FromIterator<I> for Bits<B> {
    fn from_iter<T: IntoIterator<Item=I>>(iter: T) -> Self {
        let mut set = Self::empty();
        set.extend(iter);
        set
    }
}

impl<B: Bit> core::fmt::Debug for Bits<B> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Bits(")?;
        for (i, flag) in self.iter().enumerate() {
            if i > 0 {
                write!(f, " | ")?;
            }
            write!(f, "{flag:?}")?;
        }
        write!(f, ")")
    }
}

impl<B: [ const ] Bit, I: core::marker::Copy + [ const ] Into<Bits<B>>> const core::cmp::PartialEq<I> for Bits<B> {
    #[inline]
    fn eq(&self, other: &I) -> bool {
        // Compare the underlying repr directly. Going through `==` on `Bits`
        // would re-enter this same impl (`Bits<B>: Into<Bits<B>>`) and recurse
        // forever.
        self.0 == (*other).into().0
    }
}

impl<B: Bit> Eq for Bits<B> {}

impl<B: [ const ] Bit> const core::default::Default for Bits<B> {
    #[inline]
    fn default() -> Self {
        Self::empty()
    }
}

impl<B: [ const ] Bit, I: [ const ] Into<Bits<B>>> const BitOr<I> for Bits<B> {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: I) -> Self {
        self.union(rhs)
    }
}
impl<B: [ const ] Bit, I: [ const ] Into<Bits<B>>> const BitOrAssign<I> for Bits<B> {
    #[inline]
    fn bitor_assign(&mut self, rhs: I) {
        *self = self.union(rhs);
    }
}

impl<B: [ const ] Bit, I: [ const ] Into<Bits<B>>> const BitAnd<I> for Bits<B> {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: I) -> Self {
        self.intersection(rhs)
    }
}
impl<B: [ const ] Bit, I: [ const ] Into<Bits<B>>> const BitAndAssign<I> for Bits<B> {
    #[inline]
    fn bitand_assign(&mut self, rhs: I) {
        *self = self.intersection(rhs);
    }
}

impl<B: [ const ] Bit, I: [ const ] Into<Bits<B>>> const Sub<I> for Bits<B> {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: I) -> Self {
        self.difference(rhs)
    }
}
impl<B: [ const ] Bit, I: [ const ] Into<Bits<B>>> const SubAssign<I> for Bits<B> {
    #[inline]
    fn sub_assign(&mut self, rhs: I) {
        *self = self.difference(rhs);
    }
}

impl<B: [ const ] Bit, I: [ const ] Into<Bits<B>>> const BitXor<I> for Bits<B> {
    type Output = Self;
    #[inline]
    fn bitxor(self, rhs: I) -> Self {
        self.symmetric_difference(rhs)
    }
}
impl<B: [ const ] Bit, I: [ const ] Into<Bits<B>>> const BitXorAssign<I> for Bits<B> {
    #[inline]
    fn bitxor_assign(&mut self, rhs: I) {
        *self = self.symmetric_difference(rhs);
    }
}

impl<B: [ const ] Bit, I: [ const ] Into<Bits<B>>> const Rem<I> for Bits<B> {
    type Output = Self;
    #[inline]
    fn rem(self, rhs: I) -> Self {
        self.symmetric_difference(rhs)
    }
}

impl<B: [ const ] Bit, I: [ const ] Into<Bits<B>>> const RemAssign<I> for Bits<B> {
    #[inline]
    fn rem_assign(&mut self, rhs: I) {
        *self = self.symmetric_difference(rhs);
    }
}

impl<B: [ const ] Bit> const Not for Bits<B> {
    type Output = Self;
    #[inline]
    fn not(self) -> Self {
        self.complement()
    }
}

/// Yields the individual flags present in a [`Bits<B>`] set.
///
/// Walks [`Bit::LIST`] in order. A flag is yielded when all of its bits are in
/// the source set *and* it still covers bits no earlier flag has claimed — so
/// overlapping flags both appear, while a convenience alias whose bits are
/// fully covered by already-yielded flags does not.
#[derive(Copy, Clone)]
pub struct BitsIter<B: Bit> {
    source: Bits<B>,
    remaining: Bits<B>,
    idx: usize,
}

impl<B: [ const ] Bit> const BitsIter<B> {
    #[inline]
    pub fn new(source: impl [ const ] Into<Bits<B>>) -> Self {
        let bits = source.into();
        Self { source: bits, remaining: bits, idx: 0 }
    }

    #[inline]
    pub fn with_remaining(mut self, remaining: impl [ const ] Into<Bits<B>>) -> Self {
        self.remaining = remaining.into();
        self
    }
}

impl<B: [ const ] Bit> const Iterator for BitsIter<B> {
    type Item = B;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < B::COUNT {
            let flag = B::LIST[self.idx];
            self.idx += 1;

            let bits = Bits::new(flag);
            if self.source.contains(bits) && self.remaining.intersects(bits) {
                self.remaining.remove(bits);
                return Some(flag);
            }
            if self.remaining.is_empty() {
                self.idx = B::COUNT;
                return None;
            }
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // Upper bound only: the exact count depends on the overlap rules above.
        (0, Some(B::COUNT - self.idx))
    }
}

impl<B: Bit> ExactSizeIterator for BitsIter<B> {
    #[inline]
    fn len(&self) -> usize {
        // Drain a copy — correct under overlapping/alias flags, and COUNT is tiny.
        let mut probe = *self;
        let mut n = 0;
        while probe.next().is_some() {
            n += 1;
        }
        n
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum BitsError {
    #[error("unknown bits set")]
    Unknown,
}

#[cfg(test)]
mod test {
    use super::*;


    bits! {
        Attributes,
        pub enum Attribute: u16 {
            #[default]
            None = 0,
            Bold = (1 << 1),
            Faint = (1 << 2),
            Italic = (1 << 3),
            Underline = (1 << 4),
            UnderlineDouble = (1 << 5),
            UnderlineCurly = (1 << 6),
            Blink = (1 << 7),
            RapidBlink = (1 << 8),
            Inverse = (1 << 9),
            Invisible = (1 << 10),
            Strikethrough = (1 << 11),
            Frame = (1 << 12),
            Encircle = (1 << 13),
            Overline = (1 << 14),
        },
    }

    #[test]
    fn combine() {
        let a = Attribute::Bold | Attribute::Italic | Attribute::Underline;
        assert!(a.contains(Attribute::Bold));
        assert!(a.contains(Attribute::Italic));
        assert!(a.contains(Attribute::Underline));
    }

    #[test]
    fn empty_semantics() {
        let e = Attributes::empty();
        assert!(e.is_empty());
        assert_eq!(e.count(), 0, "empty set must yield no flags");
        assert_eq!(e.bits(), 0, "empty set must be the zero integer");

        let bold: Attributes = Attribute::Bold.into();
        assert!(e.is_disjoint(bold));
        assert!(!e.intersects(bold));
        assert!(bold.contains(e), "every set contains the empty set");
    }

    #[test]
    fn iter_excludes_zero_and_counts() {
        let a = Attribute::Bold | Attribute::Underline;
        let v: Vec<_> = a.iter().collect();
        assert_eq!(v, [Attribute::Bold, Attribute::Underline]);
        assert_eq!(a.count(), 2);
    }
}
