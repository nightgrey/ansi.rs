use number::{Integer, Unsigned};
use std::fmt::{Debug, Formatter};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Rem, RemAssign, Sub, SubAssign};
use thiserror::Error;

pub const trait Bit: Copy
+ [ const ] Clone
+ [ const ] PartialEq
+ [ const ] Eq
+ [ const ] BitAnd<Self, Output=Bits<Self>>
+ [ const ] BitOr<Self, Output=Bits<Self>>
+ [ const ] BitXor<Self, Output=Bits<Self>>
+ [ const ] Sub<Self, Output=Bits<Self>>
+ [ const ] Rem<Self, Output=Bits<Self>>
+ [ const ] Not<Output=Bits<Self>>
+ [ const ] Into<Bits<Self>>
+ [ const ] Into<<Self as Bit>::Bits>
+ 'static
+ Debug
{
    type Bits: [ const ] Unsigned
    + [ const ] BitAnd<Self::Bits, Output=Self::Bits>
    + [ const ] BitAndAssign<Self::Bits>
    + [ const ] BitOr<Self::Bits, Output=Self::Bits>
    + [ const ] BitOrAssign<Self::Bits>
    + [ const ] BitXor<Self::Bits, Output=Self::Bits>
    + [ const ] BitXorAssign<Self::Bits>
    + [ const ] Not<Output=Self::Bits>;

    const NONE: Self::Bits;
    const ALL: Self::Bits;

    const LIST: &'static [Self];
    const COUNT: usize = Self::LIST.len();
}


#[repr(C)]
#[derive(Copy, Hash)]
#[derive_const(PartialEq, Eq, Clone)]
pub struct Bits<B: Bit>(B::Bits);

impl<B: [ const ] Bit> const Bits<B> {
    pub const EMPTY: Self = Self(B::NONE);
    pub const ALL: Self = Self(B::ALL);

    pub fn empty() -> Self {
        Self::EMPTY
    }

    pub fn all() -> Self {
        Self::ALL
    }

    #[inline]
    pub fn new(bits: impl [ const ] Into<B::Bits>) -> Self {
        Self::from_bits_truncated(bits)
    }


    #[inline]
    pub fn insert(&mut self, rhs: impl [ const ] Into<Self>)
    {
        *self = Self::union(*self, rhs);
    }


    #[inline]
    pub fn remove(&mut self, rhs: impl [ const ] Into<Self>)
    {
        *self = Self::difference(*self, rhs);
    }

    #[inline]
    pub fn toggle(&mut self, rhs: impl [ const ] Into<Self>)
    {
        *self = Self::symmetric_difference(*self, rhs);
    }

    #[inline]
    pub fn contains(&self, rhs: impl [ const ] Into<Self>) -> bool
    {
        let rhs = rhs.into();
        self.0 & rhs.0 == rhs.0
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self == Self::EMPTY
    }

    #[inline]
    pub fn is_all(self) -> bool {
        self == Self::ALL
    }

    #[inline]
    pub fn is_disjoint(self, rhs: impl [ const ] Into<Self>) -> bool {
        self & rhs.into() == Self::EMPTY
    }

    #[inline]
    pub fn intersects(&self, rhs: impl [ const ] Into<Self>) -> bool
    {
        self.0 & rhs.into().0 != B::NONE
    }

    ///   The bitwise and ( `&` ) of the bits in  `self`  and  `rhs` .
    #[inline]
    #[must_use]
    pub fn intersection(self, rhs: impl [ const ] Into<Self>) -> Self
    {
        Self::bitand(self, rhs)
    }


    ///   The bitwise or ( `|` ) of the bits in  `self`  and  `rhs` .
    #[inline]
    #[must_use]
    pub fn union(self, rhs: impl [ const ] Into<Self>) -> Self
    {
        Self::bitor(self, rhs)
    }


    #[inline]
    #[must_use]
    pub fn difference(self, rhs: impl [ const ] Into<Self>) -> Self
    {
        Self::sub(self, rhs)
    }


    #[inline]
    #[must_use]
    pub fn symmetric_difference(self, rhs: impl [ const ] Into<Self>) -> Self
    {
        Self::rem(self, rhs)
    }

    #[inline]
    #[must_use]
    pub fn complement(self) -> Self
    {
        Self::not(self)
    }

    #[inline]
    pub fn truncated(mut self) -> Self {
        self.0 &= B::ALL;
        self
    }

    /// Removes all flags from the Bits.
    #[inline]
    pub fn clear(&mut self) {
        *self = Self::EMPTY;
    }

    #[inline]
    fn bits(self) -> B::Bits {
        self.0
    }

    #[inline]
    pub fn iter(&self) -> BitsIter<B> {
        BitsIter::new(*self)
    }

    pub fn try_from_bits(bits: impl [ const ] Into<B::Bits>) -> Result<Self, BitError> {
        let bits = bits.into();
        let truncated = Self::from_bits_truncated(bits);

        if truncated.bits() == bits {
            Ok(truncated)
        } else {
            Err(BitError::Unknown)
        }
    }

    pub fn from_bits(bits: impl [ const ] Into<B::Bits>) -> Self {
        Self(bits.into() & B::ALL)
    }

    pub fn from_bits_retained(bits: impl [ const ] Into<B::Bits>) -> Self {
        Self(bits.into())
    }

    pub fn from_bits_truncated(bits: impl [ const ] Into<B::Bits>) -> Self {
        Self(bits.into() & B::ALL)
    }
}


impl<B: [ const ] Bit> const AsRef<B::Bits> for Bits<B> {
    #[inline]
    fn as_ref(&self) -> &B::Bits {
        &self.0
    }
}

impl<B: [ const ] Bit> const From<Option<Bits<B>>> for Bits<B> {
    /// Converts from `Option<Bits<B>>` to `Bits<B>`.
    ///
    /// Most notably, this allows for the use of `None` in many places to
    /// substitute for manually creating an empty `Bits<B>`. See below.
    ///
    /// ```
    /// use flagset::{Bits, flags};
    ///
    /// flags! {
    ///     enum Flag: u8 {
    ///         Foo = 0b001,
    ///         Bar = 0b010,
    ///         Baz = 0b100
    ///     }
    /// }
    ///
    /// fn convert(v: impl Into<Bits<Flag>>) -> u8 {
    ///     v.into().bits()
    /// }
    ///
    /// assert_eq!(convert(Flag::Foo | Flag::Bar), 0b011);
    /// assert_eq!(convert(Flag::Foo), 0b001);
    /// assert_eq!(convert(None), 0b000);
    /// ```
    #[inline]
    fn from(value: Option<Bits<B>>) -> Bits<B> {
        value.unwrap_or_default()
    }
}

impl<B: [ const ] Bit> const Default for Bits<B> {
    /// Creates a new, empty Bits.
    ///
    /// ```
    /// use flagset::{Bits, flags};
    ///
    /// flags! {
    ///     enum Flag: u8 {
    ///         Foo = 0b001,
    ///         Bar = 0b010,
    ///         Baz = 0b100
    ///     }
    /// }
    ///
    /// let set = Bits::<Flag>::default();
    /// assert!(set.is_empty());
    /// assert!(!set.is_full());
    /// assert!(!set.contains(Flag::Foo));
    /// assert!(!set.contains(Flag::Bar));
    /// assert!(!set.contains(Flag::Baz));
    /// ```
    #[inline]
    fn default() -> Self {
        Self::empty()
    }
}

impl<B: [ const ] Bit> const IntoIterator for Bits<B> {
    type Item = Bits<B>;
    type IntoIter = BitsIter<B>;

    /// Iterate over the flags in the set.
    ///
    /// **NOTE**: The order in which the flags are iterated is undefined.

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        BitsIter::new(self)
    }
}

impl<B: Bit> Debug for Bits<B> {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Bits(")?;
        for (i, flag) in self.into_iter().enumerate() {
            write!(f, "{}{:?}", if i > 0 { " | " } else { "" }, flag)?;
        }
        write!(f, ")")
    }
}

impl<B: [ const ] Bit> const Not for Bits<B> {
    type Output = Self;

    /// Calculates the complement of the current set.
    ///
    /// In common parlance, this returns the set of all possible flags that are
    /// not in the current set.
    ///
    /// ```
    /// use bits::{Bits, bits};
    ///
    /// bits! {
    ///     #[derive(PartialOrd, Ord)]
    ///     enum Flag: u8 {
    ///         Foo = 1 << 0,
    ///         Bar = 1 << 1,
    ///         Baz = 1 << 2
    ///     }
    /// }
    ///
    /// let set = !Bits::from(Flag::Foo);
    /// assert!(!set.is_empty());
    /// assert!(!set.is_full());
    /// assert!(!set.contains(Flag::Foo));
    /// assert!(set.contains(Flag::Bar));
    /// assert!(set.contains(Flag::Baz));
    /// ```
    #[inline]
    fn not(self) -> Self {
        Bits(!self.0)
    }
}

impl<B: [ const ] Bit, I: [ const ] Into<Bits<B>>> const BitAnd<I> for Bits<B> {
    type Output = Self;

    /// Calculates the intersection of the current set and the specified flags.
    ///
    /// ```
    /// use bits::{Bits, bits};
    ///
    /// bits! {
    ///     #[derive(PartialOrd, Ord)]
    ///     pub enum Flag: u8 {
    ///         Foo = 0b001,
    ///         Bar = 0b010,
    ///         Baz = 0b100
    ///     }
    /// }
    ///
    /// let set0 = Flag::Foo | Flag::Bar;
    /// let set1 = Flag::Baz | Flag::Bar;
    /// assert_eq!(set0 & set1, Flag::Bar);
    /// assert_eq!(set0 & Flag::Foo, Flag::Foo);
    /// assert_eq!(set1 & Flag::Baz, Flag::Baz);
    /// ```
    #[inline]
    fn bitand(self, rhs: I) -> Self {
        Bits(self.0 & rhs.into().0)
    }
}

impl<B: [ const ] Bit, I: [ const ] Into<Bits<B>>> const BitAndAssign<I> for Bits<B> {
    /// Assigns the intersection of the current set and the specified flags.
    ///
    /// ```
    /// use bits::{Bits, bits};
    ///
    /// bits! {
    ///     enum Flag: u64 {
    ///         Foo = 0b001,
    ///         Bar = 0b010,
    ///         Baz = 0b100
    ///     }
    /// }
    ///
    /// let mut set0 = Flag::Foo | Flag::Bar;
    /// let mut set1 = Flag::Baz | Flag::Bar;
    ///
    /// set0 &= set1;
    /// assert_eq!(set0, Flag::Bar);
    ///
    /// set1 &= Flag::Baz;
    /// assert_eq!(set0, Flag::Bar);
    /// ```
    #[inline]
    fn bitand_assign(&mut self, rhs: I) {
        self.0 &= rhs.into().0
    }
}

impl<B: [ const ] Bit, I: [ const ] Into<Bits<B>>> const BitOr<I> for Bits<B> {
    type Output = Self;

    /// Calculates the union of the current set with the specified flags.
    ///
    /// ```
    /// use bits::{Bits, bits};
    ///
    /// bits! {
    ///     #[derive(PartialOrd, Ord)]
    ///     pub enum Flag: u8 {
    ///         Foo = 0b001,
    ///         Bar = 0b010,
    ///         Baz = 0b100
    ///     }
    /// }
    ///
    /// let set0 = Flag::Foo | Flag::Bar;
    /// let set1 = Flag::Baz | Flag::Bar;
    /// assert_eq!(set0 | set1, Bits::full());
    /// ```
    #[inline]
    fn bitor(self, rhs: I) -> Self {
        Bits(self.0 | rhs.into().0)
    }
}

impl<B: [ const ] Bit, I: [ const ] Into<Bits<B>>> const BitOrAssign<I> for Bits<B> {
    /// Assigns the union of the current set with the specified flags.
    ///
    /// ```
    /// use bits::{Bits, bits};
    ///
    /// bits! {
    ///     enum Flag: u64 {
    ///         Foo = 0b001,
    ///         Bar = 0b010,
    ///         Baz = 0b100
    ///     }
    /// }
    ///
    /// let mut set0 = Flag::Foo | Flag::Bar;
    /// let mut set1 = Flag::Bar | Flag::Baz;
    ///
    /// set0 |= set1;
    /// assert_eq!(set0, Bits::full());
    ///
    /// set1 |= Flag::Baz;
    /// assert_eq!(set1, Flag::Bar | Flag::Baz);
    /// ```
    #[inline]
    fn bitor_assign(&mut self, rhs: I) {
        self.0 |= rhs.into().0
    }
}

impl<B: [ const ] Bit, I: [ const ] Into<Bits<B>>> const BitXor<I> for Bits<B> {
    type Output = Self;

    /// Calculates the current set with the specified flags toggled.
    ///
    /// This is commonly known as toggling the presence
    ///
    /// ```
    /// use bits::{Bits, bits};
    ///
    /// bits! {
    ///     enum Flag: u32 {
    ///         Foo = 0b001,
    ///         Bar = 0b010,
    ///         Baz = 0b100
    ///     }
    /// }
    ///
    /// let set0 = Flag::Foo | Flag::Bar;
    /// let set1 = Flag::Baz | Flag::Bar;
    /// assert_eq!(set0 ^ set1, Flag::Foo | Flag::Baz);
    /// assert_eq!(set0 ^ Flag::Foo, Flag::Bar);
    /// ```
    #[inline]
    fn bitxor(self, rhs: I) -> Self {
        Bits(self.0 ^ rhs.into().0)
    }
}

impl<B: [ const ] Bit, I: [ const ] Into<Bits<B>>> const BitXorAssign<I> for Bits<B> {
    /// Assigns the current set with the specified flags toggled.
    ///
    /// ```
    /// use bits::{Bits, bits};
    ///
    /// bits! {
    ///     enum Flag: u16 {
    ///         Foo = 0b001,
    ///         Bar = 0b010,
    ///         Baz = 0b100
    ///     }
    /// }
    ///
    /// let mut set0 = Flag::Foo | Flag::Bar;
    /// let mut set1 = Flag::Baz | Flag::Bar;
    ///
    /// set0 ^= set1;
    /// assert_eq!(set0, Flag::Foo | Flag::Baz);
    ///
    /// set1 ^= Flag::Baz;
    /// assert_eq!(set1, Flag::Bar);
    /// ```
    #[inline]
    fn bitxor_assign(&mut self, rhs: I) {
        self.0 ^= rhs.into().0
    }
}

impl<B: [ const ] Bit, I: [ const ] Into<Bits<B>>> const Sub<I> for Bits<B> {
    type Output = Self;

    /// Calculates set difference (the current set without the specified flags).
    ///
    /// ```
    /// use bits::{Bits, bits};
    ///
    /// bits! {
    ///     pub enum Flag: u8 {
    ///         Foo = 1,
    ///         Bar = 2,
    ///         Baz = 4
    ///     }
    /// }
    ///
    /// let set0 = Flag::Foo | Flag::Bar;
    /// let set1 = Flag::Baz | Flag::Bar;
    /// assert_eq!(set0 - set1, Flag::Foo);
    /// ```
    #[inline]
    fn sub(self, rhs: I) -> Self {
        self & !rhs.into()
    }
}

impl<B: [ const ] Bit, I: [ const ] Into<Bits<B>>> const SubAssign<I> for Bits<B> {
    /// Assigns set difference (the current set without the specified flags).
    ///
    /// ```
    /// use bits::{Bits, bits};
    ///
    /// bits! {
    ///     pub enum Flag: u8 {
    ///         Foo = 1,
    ///         Bar = 2,
    ///         Baz = 4
    ///     }
    /// }
    ///
    /// let mut set0 = Flag::Foo | Flag::Bar;
    /// set0 -= Flag::Baz | Flag::Bar;
    /// assert_eq!(set0, Flag::Foo);
    /// ```
    #[inline]
    fn sub_assign(&mut self, rhs: I) {
        *self &= !rhs.into();
    }
}

impl<B: [ const ] Bit, I: [ const ] Into<Bits<B>>> const Rem<I> for Bits<B> {
    type Output = Self;

    /// Calculates the symmetric difference between two sets.
    ///
    /// The symmetric difference between two sets is the set of all flags
    /// that appear in one set or the other, but not both.
    ///
    /// ```
    /// use bits::{Bits, bits};
    ///
    /// bits! {
    ///     pub enum Flag: u8 {
    ///         Foo = 1,
    ///         Bar = 2,
    ///         Baz = 4
    ///     }
    /// }
    ///
    /// let set0 = Flag::Foo | Flag::Bar;
    /// let set1 = Flag::Baz | Flag::Bar;
    /// assert_eq!(set0 % set1, Flag::Foo | Flag::Baz);
    /// ```
    #[inline]
    fn rem(self, rhs: I) -> Self {
        let rhs = rhs.into();
        (self - rhs) | (rhs - self)
    }
}

impl<B: [ const ] Bit, I: [ const ] Into<Bits<B>>> const RemAssign<I> for Bits<B> {
    /// Assigns the symmetric difference between two sets.
    ///
    /// The symmetric difference between two sets is the set of all flags
    /// that appear in one set or the other, but not both.
    ///
    /// ```
    /// use bits::{Bits, bits};
    ///
    /// bits! {
    ///     pub enum Flag: u8 {
    ///         Foo = 1,
    ///         Bar = 2,
    ///         Baz = 4
    ///     }
    /// }
    ///
    /// let mut set0 = Flag::Foo | Flag::Bar;
    /// let set1 = Flag::Baz | Flag::Bar;
    /// set0 %= set1;
    /// assert_eq!(set0, Flag::Foo | Flag::Baz);
    /// ```
    #[inline]
    fn rem_assign(&mut self, rhs: I) {
        *self = *self % rhs
    }
}

impl<B: Bit, I: Into<Bits<B>>> Extend<I> for Bits<B> {
    /// Add values by iterating over some collection.
    ///
    /// ```
    /// use bits::{Bits, bits};
    ///
    /// bits! {
    ///     #[derive(PartialOrd, Ord)]
    ///     pub enum Flag: u8 {
    ///         Foo = 1,
    ///         Bar = 2,
    ///         Baz = 4
    ///     }
    /// }
    ///
    /// let flag_vec = vec![Flag::Bar, Flag::Baz];
    /// let mut some_extended_flags = Bits::from(Flag::Foo);
    /// some_extended_flags.extend(flag_vec);
    /// assert_eq!(some_extended_flags, Flag::Foo | Flag::Bar | Flag::Baz);
    /// ```
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item=I>,
    {
        for item in iter {
            *self |= item;
        }
    }
}


#[doc(hidden)]
#[derive(Copy, Clone)]
pub struct BitsIter<B: Bit> {
    bits: Bits<B>,
    remaining: Bits<B>,
    idx: usize,
}

impl<B: [ const ] Bit> const BitsIter<B> {
    #[inline]
    pub fn new(bits: impl [ const ] Into<Bits<B>>) -> Self {
        let bits = bits.into();
        Self { bits, remaining: bits, idx: 0 }
    }

    pub fn with_remaining(mut self, remaining: impl [ const ] Into<Bits<B>>) -> Self {
        self.remaining = remaining.into();
        self
    }
}

impl<B: [ const ] Bit> const Iterator for BitsIter<B> {
    type Item = Bits<B>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while self.idx < B::COUNT {
            // Short-circuit if our state is empty
            if self.remaining.is_empty() {
                self.idx = B::COUNT;
                return None;
            }

            let next = Bits::from_bits_retained(B::LIST[self.idx]);
            // If the flag is set in the original source _and_ it has bits that haven't
            // been covered by a previous flag yet then yield it. These conditions cover
            // two cases for multi-bit flags:
            //
            // 1. When flags partially overlap, such as `0b00000001` and `0b00000101`, we'll
            // yield both flags.
            // 2. When flags fully overlap, such as in convenience flags that are a shorthand for others,
            // we won't yield both flags.
            if self.bits.contains(next)
                && self.remaining.intersects(next)
            {
                self.remaining.remove(next);

                return Some(next);
            }
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let x = self.remaining.bits().count_ones() as usize;
        (x, Some(x))
    }
}

impl<B: Bit> ExactSizeIterator for BitsIter<B> {
    #[inline]
    fn len(&self) -> usize {
        self.remaining.bits().count_ones() as usize
    }
}

#[derive(Debug, PartialEq, Error)]
pub enum BitError {
    #[error("invalid bits")]
    Invalid,
    #[error("unknown bits")]
    Unknown,
}

// #[repr(u16)]
// #[derive(Copy, Debug)]
// #[derive_const(Clone, PartialEq, Eq)]
// pub enum Attribute {
//     None = 0,
//     Bold = 1 << 1,
//     Faint = 1 << 2,
//     Italic = 1 << 3,
//     Underline = 1 << 4,
//     UnderlineDouble = 1 << 13,
//     UnderlineCurly = 1 << 14,
//     Blink = 1 << 5,
//     RapidBlink = 1 << 6,
//     Inverse = 1 << 7,
//     Invisible = 1 << 8,
//     Strikethrough = 1 << 9,
//     Frame = 1 << 10,
//     Encircle = 1 << 11,
//     Overline = 1 << 12,
// }
//
// impl const Bit for Attribute {
//     type Bits = u16;
//
//     const NONE: Self::Bits = Self::None as Self::Bits;
//     const ALL: Self::Bits = Self::Bold as Self::Bits | Self::Faint as Self::Bits | Self::Italic as Self::Bits | Self::Underline as Self::Bits | Self::Blink as Self::Bits | Self::RapidBlink as Self::Bits | Self::Inverse as Self::Bits | Self::Invisible as Self::Bits | Self::Strikethrough as Self::Bits | Self::Frame as Self::Bits | Self::Encircle as Self::Bits | Self::Overline as Self::Bits | Self::UnderlineDouble as Self::Bits | Self::UnderlineCurly as Self::Bits;
//     const LIST: &'static [Self] = &[Self::None, Self::Bold, Self::Faint, Self::Italic, Self::Underline, Self::UnderlineDouble, Self::UnderlineCurly, Self::Blink, Self::RapidBlink, Self::Inverse, Self::Invisible, Self::Strikethrough, Self::Frame, Self::Encircle, Self::Overline];
//
//     fn bits(self) -> Self::Bits {
//         self as u16
//     }
// }

#[macro_export]
macro_rules! bits {
    () => {};

    // Entry point for enumerations without values.
    ($bits:ident => $(#[$m:meta])* $vis:vis enum $n:ident: $t:ty { $($(#[$a:meta])* $k:ident),+ $(,)* } $($next:tt)*) => {
        $crate::bits! { $(#[$m])* $vis enum $n: $t { $($(#[$a])* $k = ((1 as $t).shl($n::$k as $t))),+ } $($next)* }
    };

    // Entrypoint for enumerations with values.
    ($bits:ident => $(#[$m:meta])* $vis:vis enum $n:ident: $t:ty { $($(#[$a:meta])*$k:ident = $v:expr),* $(,)* } $($next:tt)*) => {
        $(#[$m])*
        #[derive(Copy,  Debug)]
        #[derive_const(Clone, PartialEq, Eq)]
        $vis enum $n { $($(#[$a])* $k),* }

        impl const $crate::Bit for $n {
            type Bits = $t;

            const NONE: Self::Bits = 0;
            const ALL: Self::Bits = $($n::$k as Self::Bits)|*;

            const LIST: &'static [Self] = &[$($n::$k),*];
        }
        $vis type $bits = $crate::Bits<$n>;

        impl const $bits {
            $(
             #[allow(non_upper_case_globals)]
             pub const $k: Self = Self($v);
            )*
        }


        impl const ::core::convert::Into<$t> for $n {
            #[inline]
            fn into(self) -> $t {
                self as $t
            }
        }

        impl const ::core::convert::From<$n> for $crate::Bits<$n> {
            #[inline]
            fn from(value: $n) -> Self {
                match value {
                    $($n::$k => Self($v)),*
                }
            }
        }

        impl const ::core::ops::Not for $n {
            type Output = $crate::Bits<$n>;

            #[inline]
            fn not(self) -> Self::Output {
                !$crate::Bits::from(self)
            }
        }

        impl<I: [const] ::core::convert::Into<$crate::Bits<$n>>> const ::core::ops::BitAnd<I> for $n {
            type Output = $crate::Bits<$n>;

            #[inline]
            fn bitand(self, rhs: I) -> Self::Output {
                $crate::Bits::from(self) & rhs
            }
        }

        impl<I: [const] ::core::convert::Into<$crate::Bits<$n>>> const  ::core::ops::BitOr<I> for $n {
            type Output = $crate::Bits<$n>;

            #[inline]
            fn bitor(self, rhs: I) -> Self::Output {
                $crate::Bits::from(self) | rhs
            }
        }

        impl<I: [const] ::core::convert::Into<$crate::Bits<$n>>> const  ::core::ops::BitXor<I> for $n {
            type Output = $crate::Bits<$n>;

            #[inline]
            fn bitxor(self, rhs: I) -> Self::Output {
                $crate::Bits::from(self) ^ rhs
            }
        }

        impl<I: [const] ::core::convert::Into<$crate::Bits<$n>>> const  ::core::ops::Sub<I> for $n {
            type Output = $crate::Bits<$n>;

            #[inline]
            fn sub(self, rhs: I) -> Self::Output {
                $crate::Bits::from(self) - rhs
            }
        }

        impl<I: [const] ::core::convert::Into<$crate::Bits<$n>>> const  ::core::ops::Rem<I> for $n {
            type Output = $crate::Bits<$n>;

            #[inline]
            fn rem(self, rhs: I) -> Self::Output {
                $crate::Bits::from(self) % rhs
            }
        }

        $crate::bits! { $($next)* }
    };
}
