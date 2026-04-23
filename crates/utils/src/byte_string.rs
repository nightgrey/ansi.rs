use core::borrow::{Borrow, BorrowMut};
use core::cmp::Ordering;
use core::ops::{
    Deref, DerefMut, Index, IndexMut, Range, RangeFrom, RangeFull, RangeInclusive,
    RangeTo, RangeToInclusive,
};
use core::str::FromStr;
use core::{fmt, hash};
use std::borrow::{Cow, ToOwned};
use std::mem::MaybeUninit;
use smallvec::SmallVec;


/// A wrapper for `SmallVec<u8, N>` representing a human-readable string that's conventionally, but not always, UTF-8.
///
/// The underlying storage is heap-allocated as long as it does not exceed [`N`].
///
/// A `ByteString` owns its contents and can grow and shrink, like a `Vec` or `String`. For a
/// borrowed intermediates string, see [`ByteStr`].
///
/// `ByteString` implements `Deref` to `&SmallVec<u8>`, so all methods available on `&SmallVec<u8>` are
/// available on `ByteString`. Similarly, `ByteString` implements `DerefMut` to `&mut SmallVec<u8>`,
/// so you can modify a `ByteString` using any method available on `&mut SmallVec<u8>`.
///
/// The `Debug` and `Display` implementations for `ByteString` are the same as those for `ByteStr<N>`,
/// showing invalid UTF-8 as hex escapes or the Unicode replacement character, respectively.
#[repr(transparent)]
#[derive(Clone)]
#[doc(alias = "ByteStr")]
pub struct ByteString<const N: usize>(pub SmallVec<u8, N>);

impl<const N: usize> ByteString<N> {
    pub const ZERO: ByteString<0> = ByteString(SmallVec::new());
    pub const EMPTY: Self = Self(SmallVec::new());

    /// Returns a `ByteString` with zero capacity, no matter the `N`.
    pub const fn zero() -> ByteString<0> {
        Self::ZERO
    }

    /// Returns a `ByteString` with `N` capacity, but no elements.
    pub const fn empty() -> Self {
        Self::EMPTY
    }

    #[inline]
    pub const fn new<const M: usize>(bytes: [u8; M]) -> Self {
        Self(SmallVec::from_buf(bytes))
    }

    #[inline]
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    #[inline]
    pub(crate) fn as_byte_str(&self) -> &ByteStr<N> {
        ByteStr::new(&self.0)
    }

    #[inline]
    pub(crate) fn as_mut_byte_str(&mut self) -> &mut ByteStr<N> {
        ByteStr::from_bytes_mut(&mut self.0)
    }
}

impl<const N: usize> Deref for ByteString<N> {
    type Target = SmallVec<u8, N>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> DerefMut for ByteString<N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const N: usize> fmt::Debug for ByteString<N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_byte_str(), f)
    }
}

impl<const N: usize> fmt::Display for ByteString<N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_byte_str(), f)
    }
}
impl<const N: usize> AsRef<[u8]> for ByteString<N> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
impl<const N: usize> AsRef<[u8; N]> for ByteString<N> {
    #[inline]
    fn as_ref(&self) -> &[u8; N] {
        unsafe { &*(self.0.as_ptr() as *const [u8; N]) }
    }
}

impl<const N: usize> AsRef<ByteStr<N>> for ByteString<N> {
    #[inline]
    fn as_ref(&self) -> &ByteStr<N> {
        self.as_byte_str()
    }
}

impl<const N: usize> AsMut<[u8]> for ByteString<N> {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl<const N: usize> AsMut<ByteStr<N>> for ByteString<N> {
    #[inline]
    fn as_mut(&mut self) -> &mut ByteStr<N> {
        self.as_mut_byte_str()
    }
}

impl<const N: usize> Borrow<[u8]> for ByteString<N> {
    #[inline]
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}


impl<const N: usize> Borrow<[u8; N]> for ByteString<N> {
    #[inline]
    fn borrow(&self) -> &[u8; N] {
        unsafe { &*(self.0.as_ptr() as *const [u8; N]) }
    }
}
impl<const N: usize> Borrow<ByteStr<N>> for ByteString<N> {
    #[inline]
    fn borrow(&self) -> &ByteStr<N> {
        self.as_byte_str()
    }
}
impl<const N: usize> Borrow<ByteStr<N>> for Vec<u8> {
    #[inline]
    fn borrow(&self) -> &ByteStr<N> {
        ByteStr::new(self)
    }
}
impl<const N: usize> Borrow<ByteStr<N>> for String {
    #[inline]
    fn borrow(&self) -> &ByteStr<N> {
        ByteStr::new(self.as_bytes())
    }
}

impl<const N: usize> BorrowMut<[u8]> for ByteString<N> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}
impl<const N: usize> BorrowMut<[u8; N]> for ByteString<N> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut [u8; N] {
        unsafe { &mut *(self.0.as_mut_ptr() as *mut [u8; N]) }
    }
}
impl<const N: usize> BorrowMut<ByteStr<N>> for ByteString<N> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut ByteStr<N> {
        self.as_mut_byte_str()
    }
}
impl<const N: usize> BorrowMut<ByteStr<N>> for String {
    #[inline]
    fn borrow_mut(&mut self) -> &mut ByteStr<N> {
        ByteStr::from_bytes_mut(unsafe { self.as_bytes_mut() })
    }
}

impl<const N: usize> Default for ByteString<N> {
    fn default() -> Self {
        ByteString(SmallVec::new())
    }
}

impl<const N: usize> From<&[u8]> for ByteString<N> {
    #[inline]
    fn from(s: &[u8]) -> Self {
        Self(SmallVec::from_slice_copy(s))
    }
}

impl<const N: usize, const M: usize> From<[u8; M]> for ByteString<N> {
    #[inline]
    fn from(s: [u8; M]) -> Self {
        Self::new(s)
    }
}

impl<const N: usize, const M: usize> From<&[u8; M]> for ByteString<N> {
    #[inline]
    fn from(s: &[u8; M]) -> Self {
        Self::new(*s)
    }
}
impl<const N: usize> From<ByteString<N>> for SmallVec<u8, N> {
    #[inline]
    fn from(s: ByteString<N>) -> Self {
        s.0
    }
}

impl<'a, const N: usize> From<&'a ByteStr<N>> for ByteString<N> {
    #[inline]
    fn from(s: &'a ByteStr<N>) -> Self {
        ByteString(SmallVec::from_slice_copy(&s.0))
    }
}

impl<'a, const N: usize> From<ByteString<N>> for Cow<'a, ByteStr<N>> {
    #[inline]
    fn from(s: ByteString<N>) -> Self {
        Cow::Owned(s)
    }
}

impl<'a, const N: usize> From<&'a ByteString<N>> for Cow<'a, ByteStr<N>> {
    #[inline]
    fn from(s: &'a ByteString<N>) -> Self {
        Cow::Borrowed(s.as_byte_str())
    }
}

impl<const N: usize> FromIterator<char> for ByteString<N> {
    #[inline]
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        ByteString(SmallVec::from(iter.into_iter().collect::<String>().into_bytes()))
    }
}

impl<const N: usize> FromIterator<u8> for ByteString<N> {
    #[inline]
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
        ByteString(iter.into_iter().collect())
    }
}

impl<'a, const N: usize> FromIterator<&'a u8> for ByteString<N> {
    #[inline]
    fn from_iter<T: IntoIterator<Item = &'a u8>>(iter: T) -> Self {
        let mut buf = SmallVec::new();
        for b in iter {
            buf.push(*b);
        }
        ByteString(buf)
    }
}

impl<'a, const N: usize> FromIterator<&'a str> for ByteString<N> {
    #[inline]
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        ByteString(SmallVec::from(iter.into_iter().collect::<String>().into_bytes()))
    }
}

impl<'a, const N: usize> FromIterator<&'a [u8]> for ByteString<N> {
    #[inline]
    fn from_iter<T: IntoIterator<Item = &'a [u8]>>(iter: T) -> Self {
        let mut buf = SmallVec::new();
        for b in iter {
            buf.extend_from_slice(b);
        }
        ByteString(buf)
    }
}

impl<'a, const N: usize> FromIterator<&'a ByteStr<N>> for ByteString<N> {
    fn from_iter<T: IntoIterator<Item = &'a ByteStr<N>>>(iter: T) -> Self {
        let mut buf = SmallVec::new();
        for b in iter {
            buf.extend_from_slice(&b.0);
        }
        ByteString(buf)
    }
}

impl<const N: usize> FromIterator<ByteString<N>> for ByteString<N> {
    #[inline]
    fn from_iter<T: IntoIterator<Item = ByteString<N>>>(iter: T) -> Self {
        let mut buf = SmallVec::new();
        for mut b in iter {
            buf.extend_from_slice(&b.0);
        }
        ByteString(buf)
    }
}

impl<const N: usize> FromStr for ByteString<N> {
    type Err = core::convert::Infallible;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ByteString(SmallVec::from(s.as_bytes())))
    }
}

impl<const N: usize> Index<usize> for ByteString<N> {
    type Output = u8;

    #[inline]
    fn index(&self, idx: usize) -> &u8 {
        &self.0[idx]
    }
}

impl<const N: usize> Index<RangeFull> for ByteString<N> {
    type Output = ByteStr<N>;

    #[inline]
    fn index(&self, _: RangeFull) -> &ByteStr<N> {
        self.as_byte_str()
    }
}

impl<const N: usize> Index<Range<usize>> for ByteString<N> {
    type Output = ByteStr<N>;

    #[inline]
    fn index(&self, r: Range<usize>) -> &ByteStr<N> {
        ByteStr::from_bytes(&self.0[r])
    }
}

impl<const N: usize> Index<RangeInclusive<usize>> for ByteString<N> {
    type Output = ByteStr<N>;

    #[inline]
    fn index(&self, r: RangeInclusive<usize>) -> &ByteStr<N> {
        ByteStr::from_bytes(&self.0[r])
    }
}

impl<const N: usize> Index<RangeFrom<usize>> for ByteString<N> {
    type Output = ByteStr<N>;

    #[inline]
    fn index(&self, r: RangeFrom<usize>) -> &ByteStr<N> {
        ByteStr::from_bytes(&self.0[r])
    }
}

impl<const N: usize> Index<RangeTo<usize>> for ByteString<N> {
    type Output = ByteStr<N>;

    #[inline]
    fn index(&self, r: RangeTo<usize>) -> &ByteStr<N> {
        ByteStr::from_bytes(&self.0[r])
    }
}

impl<const N: usize> Index<RangeToInclusive<usize>> for ByteString<N> {
    type Output = ByteStr<N>;

    #[inline]
    fn index(&self, r: RangeToInclusive<usize>) -> &ByteStr<N> {
        ByteStr::from_bytes(&self.0[r])
    }
}

impl<const N: usize> IndexMut<usize> for ByteString<N> {
    #[inline]
    fn index_mut(&mut self, idx: usize) -> &mut u8 {
        &mut self.0[idx]
    }
}

impl<const N: usize> IndexMut<RangeFull> for ByteString<N> {
    #[inline]
    fn index_mut(&mut self, _: RangeFull) -> &mut ByteStr<N> {
        self.as_mut_byte_str()
    }
}

impl<const N: usize> IndexMut<Range<usize>> for ByteString<N> {
    #[inline]
    fn index_mut(&mut self, r: Range<usize>) -> &mut ByteStr<N> {
        ByteStr::from_bytes_mut(&mut self.0[r])
    }
}

impl<const N: usize> IndexMut<RangeInclusive<usize>> for ByteString<N> {
    #[inline]
    fn index_mut(&mut self, r: RangeInclusive<usize>) -> &mut ByteStr<N> {
        ByteStr::from_bytes_mut(&mut self.0[r])
    }
}

impl<const N: usize> IndexMut<RangeFrom<usize>> for ByteString<N> {
    #[inline]
    fn index_mut(&mut self, r: RangeFrom<usize>) -> &mut ByteStr<N> {
        ByteStr::from_bytes_mut(&mut self.0[r])
    }
}

impl<const N: usize> IndexMut<RangeTo<usize>> for ByteString<N> {
    #[inline]
    fn index_mut(&mut self, r: RangeTo<usize>) -> &mut ByteStr<N> {
        ByteStr::from_bytes_mut(&mut self.0[r])
    }
}

impl<const N: usize> IndexMut<RangeToInclusive<usize>> for ByteString<N> {
    #[inline]
    fn index_mut(&mut self, r: RangeToInclusive<usize>) -> &mut ByteStr<N> {
        ByteStr::from_bytes_mut(&mut self.0[r])
    }
}

impl<const N: usize> hash::Hash for ByteString<N> {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<const N: usize> Eq for ByteString<N> {}

impl<const N: usize> PartialEq for ByteString<N> {
    #[inline]
    fn eq(&self, other: &ByteString<N>) -> bool {
        self.0 == other.0
    }
}

#[doc(hidden)]
macro_rules! impl_partial_eq_ord_cow {
    (<const N: usize> $lhs:ty, $rhs:ty) => {
        impl<const N: usize> PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                let other: &[u8] = (&**other).as_ref();
                PartialEq::eq(self.as_bytes(), other)
            }
        }

        impl<const N: usize> PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool {
                let this: &[u8] = (&**self).as_ref();
                PartialEq::eq(this, other.as_bytes())
            }
        }

        impl<const N: usize> PartialOrd<$rhs> for $lhs {
            #[inline]
            fn partial_cmp(&self, other: &$rhs) -> Option<Ordering> {
                let other: &[u8] = (&**other).as_ref();
                PartialOrd::partial_cmp(self.as_bytes(), other)
            }
        }

        impl<const N: usize> PartialOrd<$lhs> for $rhs {
            #[inline]
            fn partial_cmp(&self, other: &$lhs) -> Option<Ordering> {
                let this: &[u8] = (&**self).as_ref();
                PartialOrd::partial_cmp(this, other.as_bytes())
            }
        }
    };

    ($lhs:ty, $rhs:ty) => {
        impl PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                let other: &[u8] = (&**other).as_ref();
                PartialEq::eq(self.as_bytes(), other)
            }
        }

        impl PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool {
                let this: &[u8] = (&**self).as_ref();
                PartialEq::eq(this, other.as_bytes())
            }
        }

        impl PartialOrd<$rhs> for $lhs {
            #[inline]
            fn partial_cmp(&self, other: &$rhs) -> Option<Ordering> {
                let other: &[u8] = (&**other).as_ref();
                PartialOrd::partial_cmp(self.as_bytes(), other)
            }
        }

        impl PartialOrd<$lhs> for $rhs {
            #[inline]
            fn partial_cmp(&self, other: &$lhs) -> Option<Ordering> {
                let this: &[u8] = (&**self).as_ref();
                PartialOrd::partial_cmp(this, other.as_bytes())
            }
        }
    };
}

#[doc(hidden)]
macro_rules! impl_partial_eq_ord {
    (<const N: usize> $lhs:ty, $rhs:ty) => {
        impl_partial_eq!(<const N: usize> $lhs, $rhs);

        impl<const N: usize> PartialOrd<$rhs> for $lhs {
            #[inline]
            fn partial_cmp(&self, other: &$rhs) -> Option<Ordering> {
                let other: &[u8] = other.as_ref();
                PartialOrd::partial_cmp(self.as_bytes(), other)
            }
        }

        impl<const N: usize> PartialOrd<$lhs> for $rhs {
            #[inline]
            fn partial_cmp(&self, other: &$lhs) -> Option<Ordering> {
                let this: &[u8] = self.as_ref();
                PartialOrd::partial_cmp(this, other.as_bytes())
            }
        }
    };

    ($lhs:ty, $rhs:ty) => {
        impl_partial_eq!($lhs, $rhs);

        impl PartialOrd<$rhs> for $lhs {
            #[inline]
            fn partial_cmp(&self, other: &$rhs) -> Option<Ordering> {
                let other: &[u8] = other.as_ref();
                PartialOrd::partial_cmp(self.as_bytes(), other)
            }
        }

        impl PartialOrd<$lhs> for $rhs {
            #[inline]
            fn partial_cmp(&self, other: &$lhs) -> Option<Ordering> {
                let this: &[u8] = self.as_ref();
                PartialOrd::partial_cmp(this, other.as_bytes())
            }
        }
    };

}

#[doc(hidden)]
#[doc(hidden)]
macro_rules! impl_partial_eq {
    (<const N: usize> $lhs:ty, $rhs:ty) => {
        impl<const N: usize> PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                let other: &[u8] = other.as_ref();
                PartialEq::eq(self.as_bytes(), other)
            }
        }

        impl<const N: usize> PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool {
                let this: &[u8] = self.as_ref();
                PartialEq::eq(this, other.as_bytes())
            }
        }
    };
    ($lhs:ty, $rhs:ty) => {
        impl PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                let other: &[u8] = other.as_ref();
                PartialEq::eq(self.as_bytes(), other)
            }
        }

        impl PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool {
                let this: &[u8] = self.as_ref();
                PartialEq::eq(this, other.as_bytes())
            }
        }
    };
}

#[doc(hidden)]
macro_rules! impl_partial_eq_n {
    (<const N: usize> $lhs:ty, $rhs:ty) => {
        impl<const N: usize> PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                let other: &[u8] = other.as_ref();
                PartialEq::eq(self.as_bytes(), other)
            }
        }

        impl<const N: usize> PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool {
                let this: &[u8] = self.as_ref();
                PartialEq::eq(this, other.as_bytes())
            }
        }
    };

    ($lhs:ty, $rhs:ty) => {
        impl PartialEq<$rhs> for $lhs {
            #[inline]
            fn eq(&self, other: &$rhs) -> bool {
                let other: &[u8] = other.as_ref();
                PartialEq::eq(self.as_bytes(), other)
            }
        }

        impl PartialEq<$lhs> for $rhs {
            #[inline]
            fn eq(&self, other: &$lhs) -> bool {
                let this: &[u8] = self.as_ref();
                PartialEq::eq(this, other.as_bytes())
            }
        }
    };

}

// PartialOrd with `Vec<u8>` omitted to avoid inference failures
impl_partial_eq!(<const N: usize> ByteString<N>, Vec<u8>);
// PartialOrd with `[u8]` omitted to avoid inference failures
impl_partial_eq!(<const N: usize> ByteString<N>, [u8]);
// PartialOrd with `&[u8]` omitted to avoid inference failures
impl_partial_eq!(<const N: usize> ByteString<N>, &[u8]);
// PartialOrd with `String` omitted to avoid inference failures
impl_partial_eq!(<const N: usize> ByteString<N>, String);
// PartialOrd with `str` omitted to avoid inference failures
impl_partial_eq!(<const N: usize> ByteString<N>, str);
// PartialOrd with `&str` omitted to avoid inference failures
impl_partial_eq!(<const N: usize> ByteString<N>, &str);
impl_partial_eq_ord!(<const N: usize> ByteString<N>, ByteStr<N>);
impl_partial_eq_ord!(<const N: usize> ByteString<N>, &ByteStr<N>);
// PartialOrd with `[u8; N]` omitted to avoid inference failures
impl_partial_eq_n!(<const N: usize> ByteString<N>, [u8; N]);
// PartialOrd with `&[u8; N]` omitted to avoid inference failures
impl_partial_eq_n!(<const N: usize> ByteString<N>, &[u8; N]);
impl_partial_eq_ord_cow!(<const N: usize> ByteString<N>, Cow<'_, ByteStr<N>>);
impl_partial_eq_ord_cow!(<const N: usize> ByteString<N>, Cow<'_, str>);
impl_partial_eq_ord_cow!(<const N: usize> ByteString<N>, Cow<'_, [u8]>);

impl<const N: usize> Ord for ByteString<N> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&self.0, &other.0)
    }
}

impl<const N: usize> PartialOrd for ByteString<N> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(&self.0, &other.0)
    }
}

impl<const N: usize> ToOwned for ByteStr<N> {
    type Owned = ByteString<N>;

    #[inline]
    fn to_owned(&self) -> ByteString<N> {
        ByteString(SmallVec::from(self.0.to_vec()))
    }
}

impl<const N: usize> TryFrom<ByteString<N>> for String {
    type Error = std::string::FromUtf8Error;

    #[inline]
    fn try_from(s: ByteString<N>) -> Result<Self, Self::Error> {
        String::from_utf8(s.0.to_vec())
    }
}

impl<'a, const N: usize> TryFrom<&'a ByteString<N>> for &'a str {
    type Error = std::str::Utf8Error;

    #[inline]
    fn try_from(s: &'a ByteString<N>) -> Result<Self, Self::Error> {
        std::str::from_utf8(s.0.as_slice())
    }
}

// Additional impls for `ByteStr<N>` that require types from `alloc`:
impl<const N: usize> Clone for Box<ByteStr<N>> {
    #[inline]
    fn clone(&self) -> Self {
        Self::from(Box::<[u8]>::from(&self.0 as &[u8]))
    }
}

impl<'a, const N: usize> From<&'a ByteStr<N>> for Cow<'a, ByteStr<N>> {
    #[inline]
    fn from(s: &'a ByteStr<N>) -> Self {
        Cow::Borrowed(s)
    }
}

impl<const N: usize> From<Box<[u8]>> for Box<ByteStr<N>> {
    #[inline]
    fn from(s: Box<[u8]>) -> Box<ByteStr<N>> {
        // SAFETY: `ByteStr<N>` is a transparent wrapper around `[u8]`.
        unsafe { Box::from_raw(Box::into_raw(s) as _) }
    }
}

impl<const N: usize> From<Box<ByteStr<N>>> for Box<[u8; N]> {
    #[inline]
    fn from(s: Box<ByteStr<N>>) -> Box<[u8; N]> {
        // SAFETY: `ByteStr<N>` is a transparent wrapper around `[u8; N]`.
        unsafe { Box::from_raw(Box::into_raw(s) as _) }
    }
}

// PartialOrd with `Vec<u8>` omitted to avoid inference failures
impl_partial_eq!(<const N: usize> ByteStr<N>, Vec<u8>);
// PartialOrd with `String` omitted to avoid inference failures
impl_partial_eq!(<const N: usize> ByteStr<N>, String);
impl_partial_eq_ord_cow!(<const N: usize> &ByteStr<N>, Cow<'_, ByteStr<N>>);
impl_partial_eq_ord_cow!(<const N: usize> &ByteStr<N>, Cow<'_, str>);
impl_partial_eq_ord_cow!(<const N: usize> &ByteStr<N>, Cow<'_, [u8]>);

impl<'a, const N: usize> TryFrom<&'a ByteStr<N>> for String {
    type Error = core::str::Utf8Error;

    #[inline]
    fn try_from(s: &'a ByteStr<N>) -> Result<Self, Self::Error> {
        Ok(core::str::from_utf8(&s.0)?.into())
    }
}


/// A wrapper for `&[u8; N]` representing a human-readable string that's conventionally, but not always, UTF-8.
///
/// For an owned, growable string buffer, use
/// [`ByteString`].
///
/// `ByteStr` implements `Deref` to `[u8]`, so all methods available on `[u8]` are available on
/// `ByteStr`.
///
/// # Representation
///
/// A `&ByteStr` has the same representation as a `&str`. That is, a `&ByteStr` is a wide pointer
/// which includes a pointer to some bytes and a length.
///
/// # Trait implementations
///
/// The `ByteStr` type has a number of trait implementations, and in particular, defines equality
/// and comparisons between `&ByteStr`, `&str`, and `&[u8]`, for convenience.
///
/// The `Debug` implementation for `ByteStr` shows its bytes as a normal string, with invalid UTF-8
/// presented as hex escape sequences.
///
/// The `Display` implementation behaves as if the `ByteStr` were first lossily converted to a
/// `str`, with invalid UTF-8 presented as the Unicode replacement character (�).
#[derive(
    PartialEq, Eq, PartialOrd, Ord, Hash
)]
#[repr(transparent)]
pub struct ByteStr<const N: usize = 2>(pub [u8; N]);

impl<const N: usize> ByteStr<N> {
    /// Creates a `ByteStr<N>` slice from anything that can be converted to a byte slice.
    ///
    /// This is a zero-cost conversion.
    ///
    /// # Example
    ///
    /// You can create a `ByteStr<N>` from a byte array, a byte slice or a string slice:
    ///
    /// ```
    /// # #![feature(bstr)]
    /// # use std::bstr::ByteStr<N>;
    /// let a = ByteStr::new(b"abc");
    /// let b = ByteStr::new(&b"abc"[..]);
    /// let c = ByteStr::new("abc");
    ///
    /// assert_eq!(a, b);
    /// assert_eq!(a, c);
    /// ```
    #[inline]
    pub const fn new<B: ?Sized + [const] AsRef<[u8]>>(bytes: &B) -> &Self {
        Self::from_bytes(bytes.as_ref())
    }

    /// Returns the same string as `&ByteStr<N>`.
    ///
    /// This method is redundant when used directly on `&ByteStr<N>`, but
    /// it helps dereferencing other "container" types,
    /// for example `Box<ByteStr<N>>` or `Arc<ByteStr<N>>`.
    #[inline]
    pub const fn as_byte_str(&self) -> &Self {
        self
    }

    /// Returns the same string as `&mut ByteStr<N>`.
    ///
    /// This method is redundant when used directly on `&mut ByteStr<N>`, but
    /// it helps dereferencing other "container" types,
    /// for example `Box<ByteStr<N>>` or `MutexGuard<ByteStr<N>>`.
    #[inline]
    pub const fn as_mut_byte_str(&mut self) -> &mut Self {
        self
    }

    #[inline]
    pub const fn from_bytes(slice: &[u8]) -> &Self {
        // SAFETY: `Self` is a transparent wrapper around `[u8]`, so we can turn a reference to
        // the wrapped type into a reference to the wrapper type.
        unsafe { &*(slice as *const [u8] as *const Self) }
    }

    #[inline]
    pub const fn from_bytes_mut(slice: &mut [u8]) -> &mut Self {
        // SAFETY: `ByteStr<N>` is a transparent wrapper around `[u8]`, so we can turn a reference to
        // the wrapped type into a reference to the wrapper type.
        unsafe { &mut *(slice as *mut [u8] as *mut Self) }
    }

    #[inline]
    pub const fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    #[inline]
    pub const fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}


impl<const N: usize> const Deref for ByteStr<N> {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        &self.0
    }
}

impl<const N: usize> const DerefMut for ByteStr<N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl<const N: usize> fmt::Debug for ByteStr<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"")?;
        for chunk in self.utf8_chunks() {
            for c in chunk.valid().chars() {
                match c {
                    '\0' => write!(f, "\\0")?,
                    '\x01'..='\x7f' => write!(f, "{}", (c as u8).escape_ascii())?,
                    _ => write!(f, "{}", c.escape_debug())?,
                }
            }
            write!(f, "{}", chunk.invalid().escape_ascii())?;
        }
        write!(f, "\"")?;
        Ok(())
    }
}

impl<const N: usize> fmt::Display for ByteStr<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_byte_str())
    }
}

impl<const N: usize> AsRef<[u8]> for ByteStr<N> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<const N: usize> AsRef<ByteStr<N>> for ByteStr<N> {
    #[inline]
    fn as_ref(&self) -> &ByteStr<N> {
        self
    }
}

// `impl AsRef<ByteStr<N>> for [u8]` omitted to avoid widespread inference failures

impl<const N: usize> AsRef<ByteStr<N>> for str {
    #[inline]
    fn as_ref(&self) -> &ByteStr<N> {
        ByteStr::new(self)
    }
}

impl<const N: usize> AsMut<[u8]> for ByteStr<N> {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

// `impl AsMut<ByteStr<N>> for [u8]` omitted to avoid widespread inference failures

// `impl Borrow<ByteStr<N>> for [u8]` omitted to avoid widespread inference failures

// `impl Borrow<ByteStr<N>> for str` omitted to avoid widespread inference failures

impl<const N: usize> Borrow<[u8]> for ByteStr<N> {
    #[inline]
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}

// `impl BorrowMut<ByteStr<N>> for [u8]` omitted to avoid widespread inference failures

impl<const N: usize> BorrowMut<[u8]> for ByteStr<N> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl<'a, const N: usize> Default for &'a ByteStr<N> {
    fn default() -> Self {
        ByteStr::from_bytes(b"")
    }
}

impl<'a, const N: usize> Default for &'a mut ByteStr<N> {
    fn default() -> Self {
        ByteStr::from_bytes_mut(&mut [])
    }
}

impl<'a, const N: usize> const TryFrom<&'a ByteStr<N>> for &'a str {
    type Error = std::str::Utf8Error;

    #[inline]
    fn try_from(s: &'a ByteStr<N>) -> Result<Self, Self::Error> {
        std::str::from_utf8(&s.0)
    }
}

impl<'a, const N: usize> const TryFrom<&'a mut ByteStr<N>> for &'a mut str {
    type Error = std::str::Utf8Error;

    #[inline]
    fn try_from(s: &'a mut ByteStr<N>) -> Result<Self, Self::Error> {
        std::str::from_utf8_mut(&mut s.0)
    }
}
