use std::marker::Destruct;
use crate::other::Base;
use super::*;

/// A *set* of [`Bit`] flags.
///
/// The implementing type is a concrete newtype around [`Bit::Repr`] (minted by
/// [`bits!`]). All it has to provide is the `Bit`/`Repr` wiring and the two
/// `from_repr`/`to_repr` conversions; every set operation below has a default
/// body, so bringing `BitSet` into scope gives the concrete type the full set
/// algebra for free.
pub const trait Bits: Sized
+ [ const ] Base
+ [ const ] Destruct
+ Copy
+ [ const ] PartialEq
+ 'static
{
    /// Every flag, in declaration order. Drives iteration and counting.
    const LIST: &'static [(Self::Bit, &'static str)] = <Self::Bit as Bit>::LIST;
    /// Number of declared flags.
    const COUNT: usize = <Self::Bit as Bit>::COUNT;

    /// The flag type this set is built from.
    type Bit: [ const ] Bit + [ const ] Base<Repr = Self::Repr>;

    const None: Self;
    const All: Self;

    #[inline]
    fn none() -> Self {
        Self::None
    }

    /// Every valid flag.
    #[inline]
    fn all() -> Self {
        Self::All
    }

    /// Construct from anything that converts into the raw representation.
    #[inline]
    fn new(bits: impl [ const ] Into<Self::Repr>) -> Self {
        Self::from_repr(bits.into())
    }

    /// Wrap raw bits as-is, keeping any unknown bits. Cheapest constructor.
    #[inline]
    fn from_bits_retained(bits: impl [ const ] Into<Self::Repr>) -> Self {
        Self::from_repr(bits.into())
    }

    /// Wrap raw bits, masking away anything outside [`Bit::All`].
    #[inline]
    fn from_bits_truncated(bits: impl [ const ] Into<Self::Repr>) -> Self {
        Self::from_repr(bits.into() & Self::All.bits())
    }

    /// Like [`from_bits`](Self::from_bits) but returns a typed error.
    #[inline]
    fn try_from_bits(bits: impl [ const ] Into<Self::Repr>) -> Result<Self, BitsError> {
        let bits = bits.into();

        if bits & !Self::All.bits() == Self::None.bits() {
            Ok(Self::from_repr(bits))
        } else {
            Err(BitsError::Unknown)
        }
    }

    /// Wrap raw bits, panicking if any bit outside [`Bit::All`] is set.
    ///
    /// Use [`try_from_bits`](Self::try_from_bits) for a fallible version, or
    /// [`from_bits_truncated`](Self::from_bits_truncated) to silently drop
    /// unknown bits.
    #[inline]
    fn from_bits(bits: impl [ const ] Into<Self::Repr>) -> Self {
        match Self::try_from_bits(bits) {
            Ok(b) => b,
            Err(_) => panic!("BitSet::from_bits: unknown bits set"),
        }
    }

    /// `true` if every flag in `other` is present.
    #[inline]
    fn contains(self, other: impl [ const ] Into<Self>) -> bool {
        let o = other.into().bits();
        self.bits() & o == o
    }

    #[inline]
    fn is_none(self) -> bool {
        self == Self::None
    }

    #[inline]
    fn is_all(self) -> bool {
        self == Self::all()
    }

    /// `true` if any flag is shared.
    #[inline]
    fn intersects(self, other: impl [ const ] Into<Self>) -> bool {
        Self::new(self.bits() & other.into().bits()) != Self::None
    }

    /// `true` if no flag is shared.
    #[inline]
    fn is_disjoint(self, other: impl [ const ] Into<Self>) -> bool {
        self.bits() & other.into().bits() == Self::None.bits()
    }

    /// Set union (`|`).
    #[inline]
    #[must_use]
    fn union(self, other: impl [ const ] Into<Self>) -> Self {
        Self::from_repr(self.into_repr() | other.into().into_repr())
    }

    /// Set intersection (`&`).
    #[inline]
    #[must_use]
    fn intersection(self, other: impl [ const ] Into<Self>) -> Self {
        Self::from_repr(self.into_repr() & other.into().into_repr())
    }

    /// Flags in `self` but not `other` (`self & !other`).
    #[inline]
    #[must_use]
    fn difference(self, other: impl [ const ] Into<Self>) -> Self {
        Self::from_repr(self.into_repr() & !other.into().into_repr())
    }

    /// Flags in exactly one of the two sets (XOR, masked to valid bits).
    #[inline]
    #[must_use]
    fn symmetric_difference(self, other: impl [ const ] Into<Self>) -> Self {
        Self::from_repr((self.into_repr() ^ other.into().into_repr()) & Self::All.bits())
    }

    /// All valid flags not in `self`.
    #[inline]
    #[must_use]
    fn complement(self) -> Self {
        Self::from_repr(!self.into_repr() & Self::All.bits())
    }

    #[inline]
    fn insert(&mut self, other: impl [ const ] Into<Self>) {
        *self = Self::from_repr((*self).into_repr() | other.into().into_repr());
    }

    #[inline]
    fn remove(&mut self, other: impl [ const ] Into<Self>) {
        *self = Self::from_repr((*self).into_repr() & !other.into().into_repr());
    }

    #[inline]
    fn toggle(&mut self, other: impl [ const ] Into<Self>) {
        *self = Self::from_repr(((*self).into_repr() ^ other.into().into_repr()) & Self::All.bits());
    }

    /// Insert when `on`, remove otherwise.
    #[inline]
    fn set(&mut self, other: impl [ const ] Into<Self>, on: bool) {
        if on {
            self.insert(other)
        } else {
            self.remove(other)
        }
    }

    #[inline]
    fn clear(&mut self) {
        *self = Self::None;
    }

    /// Iterate the individual flags present in this set.
    #[inline]
    fn iter(self) -> BitsIter<Self> {
        BitsIter::new(self)
    }

    /// Number of flags yielded by iteration.
    #[inline]
    fn count(self) -> usize {
        let mut probe = self.iter();
        let mut n = 0;
        while probe.next().is_some() {
            n += 1;
        }
        n
    }
}
