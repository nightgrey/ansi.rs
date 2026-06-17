use compact_bytes::CompactBytes;
use core::borrow::{Borrow, BorrowMut};
use core::cmp::Ordering;
use core::ops::{
    Deref, DerefMut, Index, IndexMut, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo,
    RangeToInclusive,
};
use core::str::FromStr;
use core::{fmt, hash};
use std::borrow::{Cow, ToOwned};

/// A wrapper for `CompactBytes` representing a human-readable string that's conventionally, but not always, UTF-8.
///
/// The underlying storage is heap-allocated as long as it does not exceed [`N`].
///
/// A `ByteString` owns its contents and can grow and shrink, like a `Vec` or `String`. For a
/// borrowed intermediates string, see [`SmallByteStr`].
///
/// `ByteString` implements `Deref` to `&[u8]`, so all methods available on `&[u8]` are
/// available on `ByteString`. Similarly, `ByteString` implements `DerefMut` to `&mut [u8]`,
/// so you can modify a `ByteString` using any method available on `&mut [u8]`.
///
/// The `Debug` and `Display` implementations for `ByteString` are the same as those for `ByteStr`,
/// showing invalid UTF-8 as hex escapes or the Unicode replacement character, respectively.
#[repr(transparent)]
#[derive(Clone)]
#[doc(alias = "ByteStr")]
pub struct SmallByteString(pub CompactBytes);

impl SmallByteString {
    pub const EMPTY: Self = Self(CompactBytes::empty());

    pub const fn empty() -> Self {
        Self::EMPTY
    }

    #[inline]
    pub fn new(bytes: &[u8]) -> Self {
        Self(CompactBytes::new(bytes))
    }

    #[inline]
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    #[inline]
    pub(crate) fn as_byte_str(&self) -> &SmallByteStr {
        SmallByteStr::new(&self.0)
    }

    #[inline]
    pub(crate) fn as_mut_byte_str(&mut self) -> &mut SmallByteStr {
        SmallByteStr::from_bytes_mut(&mut self.0)
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.0) }
    }

    #[inline]
    pub fn as_str_mut(&mut self) -> &mut str {
        unsafe { str::from_utf8_unchecked_mut(&mut self.0) }
    }
}

impl Deref for SmallByteString {
    type Target = CompactBytes;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SmallByteString {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Debug for SmallByteString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(std::bstr::ByteStr::new(&self), f)
    }
}

impl fmt::Display for SmallByteString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_byte_str(), f)
    }
}
impl AsRef<[u8]> for SmallByteString {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<SmallByteStr> for SmallByteString {
    #[inline]
    fn as_ref(&self) -> &SmallByteStr {
        self.as_byte_str()
    }
}

impl AsMut<[u8]> for SmallByteString {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl AsMut<SmallByteStr> for SmallByteString {
    #[inline]
    fn as_mut(&mut self) -> &mut SmallByteStr {
        self.as_mut_byte_str()
    }
}

impl Borrow<[u8]> for SmallByteString {
    #[inline]
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}

impl Borrow<SmallByteStr> for SmallByteString {
    #[inline]
    fn borrow(&self) -> &SmallByteStr {
        self.as_byte_str()
    }
}
impl Borrow<SmallByteStr> for Vec<u8> {
    #[inline]
    fn borrow(&self) -> &SmallByteStr {
        SmallByteStr::new(self)
    }
}
impl Borrow<SmallByteStr> for String {
    #[inline]
    fn borrow(&self) -> &SmallByteStr {
        SmallByteStr::new(self.as_bytes())
    }
}

impl BorrowMut<[u8]> for SmallByteString {
    #[inline]
    fn borrow_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}
impl BorrowMut<SmallByteStr> for SmallByteString {
    #[inline]
    fn borrow_mut(&mut self) -> &mut SmallByteStr {
        self.as_mut_byte_str()
    }
}
impl BorrowMut<SmallByteStr> for String {
    #[inline]
    fn borrow_mut(&mut self) -> &mut SmallByteStr {
        SmallByteStr::from_bytes_mut(unsafe { self.as_bytes_mut() })
    }
}

impl Default for SmallByteString {
    fn default() -> Self {
        SmallByteString::empty()
    }
}

impl From<&[u8]> for SmallByteString {
    #[inline]
    fn from(s: &[u8]) -> Self {
        Self(CompactBytes::new(s))
    }
}

impl<const N: usize> From<[u8; N]> for SmallByteString {
    #[inline]
    fn from(s: [u8; N]) -> Self {
        Self(CompactBytes::new(&s))
    }
}
impl<const N: usize> From<&[u8; N]> for SmallByteString {
    #[inline]
    fn from(s: &[u8; N]) -> Self {
        Self(CompactBytes::new(s))
    }
}
impl From<SmallByteString> for CompactBytes {
    #[inline]
    fn from(s: SmallByteString) -> Self {
        s.0
    }
}

impl<'a> From<&'a SmallByteStr> for SmallByteString {
    #[inline]
    fn from(s: &'a SmallByteStr) -> Self {
        SmallByteString(CompactBytes::new(&s.0))
    }
}

impl<'a> From<SmallByteString> for Cow<'a, SmallByteStr> {
    #[inline]
    fn from(s: SmallByteString) -> Self {
        Cow::Owned(s)
    }
}

impl<'a> From<&'a SmallByteString> for Cow<'a, SmallByteStr> {
    #[inline]
    fn from(s: &'a SmallByteString) -> Self {
        Cow::Borrowed(s.as_byte_str())
    }
}

impl FromIterator<char> for SmallByteString {
    #[inline]
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        SmallByteString(CompactBytes::new(
            iter.into_iter().collect::<String>().as_bytes(),
        ))
    }
}

impl FromIterator<u8> for SmallByteString {
    #[inline]
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
        let mut buf = CompactBytes::empty();
        for b in iter {
            buf.push(b);
        }

        SmallByteString(buf)
    }
}

impl<'a> FromIterator<&'a u8> for SmallByteString {
    #[inline]
    fn from_iter<T: IntoIterator<Item = &'a u8>>(iter: T) -> Self {
        let mut buf = CompactBytes::empty();
        for b in iter {
            buf.push(*b);
        }
        SmallByteString(buf)
    }
}

impl<'a> FromIterator<&'a str> for SmallByteString {
    #[inline]
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        SmallByteString(CompactBytes::new(
            iter.into_iter().collect::<String>().as_bytes(),
        ))
    }
}

impl<'a> FromIterator<&'a [u8]> for SmallByteString {
    #[inline]
    fn from_iter<T: IntoIterator<Item = &'a [u8]>>(iter: T) -> Self {
        let mut buf = CompactBytes::empty();
        for b in iter {
            buf.extend_from_slice(b);
        }
        SmallByteString(buf)
    }
}

impl<'a> FromIterator<&'a SmallByteStr> for SmallByteString {
    fn from_iter<T: IntoIterator<Item = &'a SmallByteStr>>(iter: T) -> Self {
        let mut buf = CompactBytes::empty();
        for b in iter {
            buf.extend_from_slice(&b.0);
        }
        SmallByteString(buf)
    }
}

impl FromIterator<SmallByteString> for SmallByteString {
    #[inline]
    fn from_iter<T: IntoIterator<Item = SmallByteString>>(iter: T) -> Self {
        SmallByteString(CompactBytes::new(
            iter.into_iter().collect::<Vec<_>>().concat().as_slice(),
        ))
    }
}

impl FromStr for SmallByteString {
    type Err = core::convert::Infallible;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(SmallByteString(CompactBytes::new(s.as_bytes())))
    }
}

impl Index<usize> for SmallByteString {
    type Output = u8;

    #[inline]
    fn index(&self, idx: usize) -> &u8 {
        &self.0[idx]
    }
}

impl Index<RangeFull> for SmallByteString {
    type Output = SmallByteStr;

    #[inline]
    fn index(&self, _: RangeFull) -> &SmallByteStr {
        self.as_byte_str()
    }
}

impl Index<Range<usize>> for SmallByteString {
    type Output = SmallByteStr;

    #[inline]
    fn index(&self, r: Range<usize>) -> &SmallByteStr {
        SmallByteStr::from_bytes(&self.0[r])
    }
}

impl Index<RangeInclusive<usize>> for SmallByteString {
    type Output = SmallByteStr;

    #[inline]
    fn index(&self, r: RangeInclusive<usize>) -> &SmallByteStr {
        SmallByteStr::from_bytes(&self.0[r])
    }
}

impl Index<RangeFrom<usize>> for SmallByteString {
    type Output = SmallByteStr;

    #[inline]
    fn index(&self, r: RangeFrom<usize>) -> &SmallByteStr {
        SmallByteStr::from_bytes(&self.0[r])
    }
}

impl Index<RangeTo<usize>> for SmallByteString {
    type Output = SmallByteStr;

    #[inline]
    fn index(&self, r: RangeTo<usize>) -> &SmallByteStr {
        SmallByteStr::from_bytes(&self.0[r])
    }
}

impl Index<RangeToInclusive<usize>> for SmallByteString {
    type Output = SmallByteStr;

    #[inline]
    fn index(&self, r: RangeToInclusive<usize>) -> &SmallByteStr {
        SmallByteStr::from_bytes(&self.0[r])
    }
}

impl IndexMut<usize> for SmallByteString {
    #[inline]
    fn index_mut(&mut self, idx: usize) -> &mut u8 {
        &mut self.0[idx]
    }
}

impl IndexMut<RangeFull> for SmallByteString {
    #[inline]
    fn index_mut(&mut self, _: RangeFull) -> &mut SmallByteStr {
        self.as_mut_byte_str()
    }
}

impl IndexMut<Range<usize>> for SmallByteString {
    #[inline]
    fn index_mut(&mut self, r: Range<usize>) -> &mut SmallByteStr {
        SmallByteStr::from_bytes_mut(&mut self.0[r])
    }
}

impl IndexMut<RangeInclusive<usize>> for SmallByteString {
    #[inline]
    fn index_mut(&mut self, r: RangeInclusive<usize>) -> &mut SmallByteStr {
        SmallByteStr::from_bytes_mut(&mut self.0[r])
    }
}

impl IndexMut<RangeFrom<usize>> for SmallByteString {
    #[inline]
    fn index_mut(&mut self, r: RangeFrom<usize>) -> &mut SmallByteStr {
        SmallByteStr::from_bytes_mut(&mut self.0[r])
    }
}

impl IndexMut<RangeTo<usize>> for SmallByteString {
    #[inline]
    fn index_mut(&mut self, r: RangeTo<usize>) -> &mut SmallByteStr {
        SmallByteStr::from_bytes_mut(&mut self.0[r])
    }
}

impl IndexMut<RangeToInclusive<usize>> for SmallByteString {
    #[inline]
    fn index_mut(&mut self, r: RangeToInclusive<usize>) -> &mut SmallByteStr {
        SmallByteStr::from_bytes_mut(&mut self.0[r])
    }
}

impl hash::Hash for SmallByteString {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Eq for SmallByteString {}

impl PartialEq for SmallByteString {
    #[inline]
    fn eq(&self, other: &SmallByteString) -> bool {
        self.0 == other.0
    }
}

#[doc(hidden)]
macro_rules! impl_partial_eq_ord_cow {
    ( $lhs:ty, $rhs:ty) => {
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
    ( $lhs:ty, $rhs:ty) => {
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
    ( $lhs:ty, $rhs:ty) => {
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
    ($lhs:ty, $rhs:ty) => {
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
}

// PartialOrd with `Vec<u8>` omitted to avoid inference failures
impl_partial_eq!(SmallByteString, Vec<u8>);
// PartialOrd with `[u8]` omitted to avoid inference failures
impl_partial_eq!(SmallByteString, [u8]);
// PartialOrd with `&[u8]` omitted to avoid inference failures
impl_partial_eq!(SmallByteString, &[u8]);
// PartialOrd with `String` omitted to avoid inference failures
impl_partial_eq!(SmallByteString, String);
// PartialOrd with `str` omitted to avoid inference failures
impl_partial_eq!(SmallByteString, str);
// PartialOrd with `&str` omitted to avoid inference failures
impl_partial_eq!(SmallByteString, &str);
impl_partial_eq_ord!(SmallByteString, SmallByteStr);
impl_partial_eq_ord!(SmallByteString, &SmallByteStr);
// PartialOrd with `[u8]` omitted to avoid inference failures
impl_partial_eq_n!(SmallByteString, [u8; N]);
// PartialOrd with `&[u8]` omitted to avoid inference failures
impl_partial_eq_n!(SmallByteString, &[u8; N]);
impl_partial_eq_ord_cow!(SmallByteString, Cow<'_, SmallByteStr>);
impl_partial_eq_ord_cow!(SmallByteString, Cow<'_, str>);
impl_partial_eq_ord_cow!(SmallByteString, Cow<'_, [u8]>);

impl Ord for SmallByteString {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&self.0.as_slice(), &other.0.as_slice())
    }
}

impl PartialOrd for SmallByteString {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(&self.0.as_slice(), &other.0.as_slice())
    }
}

impl ToOwned for SmallByteStr {
    type Owned = SmallByteString;

    #[inline]
    fn to_owned(&self) -> SmallByteString {
        SmallByteString(CompactBytes::new(&self.0))
    }
}

impl TryFrom<SmallByteString> for String {
    type Error = std::string::FromUtf8Error;

    #[inline]
    fn try_from(s: SmallByteString) -> Result<Self, Self::Error> {
        String::from_utf8(s.0.to_vec())
    }
}

impl<'a> TryFrom<&'a SmallByteString> for &'a str {
    type Error = std::str::Utf8Error;

    #[inline]
    fn try_from(s: &'a SmallByteString) -> Result<Self, Self::Error> {
        std::str::from_utf8(s.0.as_slice())
    }
}

// Additional impls for `ByteStr` that require types from `alloc`:
impl Clone for Box<SmallByteStr> {
    #[inline]
    fn clone(&self) -> Self {
        Self::from(Box::<[u8]>::from(&self.0 as &[u8]))
    }
}

impl<'a> From<&'a SmallByteStr> for Cow<'a, SmallByteStr> {
    #[inline]
    fn from(s: &'a SmallByteStr) -> Self {
        Cow::Borrowed(s)
    }
}

impl From<Box<[u8]>> for Box<SmallByteStr> {
    #[inline]
    fn from(s: Box<[u8]>) -> Box<SmallByteStr> {
        // SAFETY: `ByteStr` is a transparent wrapper around `[u8]`.
        unsafe { Box::from_raw(Box::into_raw(s) as _) }
    }
}

impl From<Box<SmallByteStr>> for Box<[u8]> {
    #[inline]
    fn from(s: Box<SmallByteStr>) -> Box<[u8]> {
        // SAFETY: `ByteStr` is a transparent wrapper around `[u8]`.
        unsafe { Box::from_raw(Box::into_raw(s) as _) }
    }
}

// PartialOrd with `Vec<u8>` omitted to avoid inference failures
impl_partial_eq!(SmallByteStr, Vec<u8>);
// PartialOrd with `String` omitted to avoid inference failures
impl_partial_eq!(SmallByteStr, String);
impl_partial_eq_ord_cow!(&SmallByteStr, Cow<'_, SmallByteStr>);
impl_partial_eq_ord_cow!(&SmallByteStr, Cow<'_, str>);
impl_partial_eq_ord_cow!(&SmallByteStr, Cow<'_, [u8]>);

impl<'a> TryFrom<&'a SmallByteStr> for String {
    type Error = core::str::Utf8Error;

    #[inline]
    fn try_from(s: &'a SmallByteStr) -> Result<Self, Self::Error> {
        Ok(core::str::from_utf8(&s.0)?.into())
    }
}

/// A wrapper for `&[u8]` representing a human-readable string that's conventionally, but not always, UTF-8.
///
/// For an owned, growable string buffer, use
/// [`SmallByteString`].
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
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct SmallByteStr(pub [u8]);

impl SmallByteStr {
    /// Creates a `ByteStr` slice from anything that can be converted to a byte slice.
    ///
    /// This is a zero-cost conversion.
    ///
    /// # Example
    ///
    /// You can create a `ByteStr` from a byte array, a byte slice or a string slice:
    ///
    /// ```
    /// # #![feature(bstr)]
    /// # use std::bstr::ByteStr;
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

    /// Returns the same string as `&ByteStr`.
    ///
    /// This method is redundant when used directly on `&ByteStr`, but
    /// it helps dereferencing other "container" types,
    /// for example `Box<ByteStr>` or `Arc<ByteStr>`.
    #[inline]
    pub const fn as_byte_str(&self) -> &Self {
        self
    }

    /// Returns the same string as `&mut ByteStr`.
    ///
    /// This method is redundant when used directly on `&mut ByteStr`, but
    /// it helps dereferencing other "container" types,
    /// for example `Box<ByteStr>` or `MutexGuard<ByteStr>`.
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
        // SAFETY: `ByteStr` is a transparent wrapper around `[u8]`, so we can turn a reference to
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

    #[inline]
    pub const fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.0) }
    }

    #[inline]
    pub const fn as_str_mut(&mut self) -> &mut str {
        unsafe { str::from_utf8_unchecked_mut(&mut self.0) }
    }
}

impl const Deref for SmallByteStr {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        &self.0
    }
}

impl const DerefMut for SmallByteStr {
    #[inline]
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl fmt::Display for SmallByteStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(std::bstr::ByteStr::new(&self), f)
    }
}

impl fmt::Debug for SmallByteStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(std::bstr::ByteStr::new(&self), f)
    }
}

impl AsRef<[u8]> for SmallByteStr {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<SmallByteStr> for SmallByteStr {
    #[inline]
    fn as_ref(&self) -> &SmallByteStr {
        self
    }
}

// `impl AsRef<ByteStr> for [u8]` omitted to avoid widespread inference failures

impl AsRef<SmallByteStr> for str {
    #[inline]
    fn as_ref(&self) -> &SmallByteStr {
        SmallByteStr::new(self)
    }
}

impl AsMut<[u8]> for SmallByteStr {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

// `impl AsMut<ByteStr> for [u8]` omitted to avoid widespread inference failures

// `impl Borrow<ByteStr> for [u8]` omitted to avoid widespread inference failures

// `impl Borrow<ByteStr> for str` omitted to avoid widespread inference failures

impl Borrow<[u8]> for SmallByteStr {
    #[inline]
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}

// `impl BorrowMut<ByteStr> for [u8]` omitted to avoid widespread inference failures

impl BorrowMut<[u8]> for SmallByteStr {
    #[inline]
    fn borrow_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl Default for &SmallByteStr {
    fn default() -> Self {
        SmallByteStr::from_bytes(b"")
    }
}

impl Default for &mut SmallByteStr {
    fn default() -> Self {
        SmallByteStr::from_bytes_mut(&mut [])
    }
}

impl <'a> const TryFrom<&'a SmallByteStr> for &'a str {
    type Error = std::str::Utf8Error;

    #[inline]
    fn try_from(s: &'a SmallByteStr) -> Result<Self, Self::Error> {
        std::str::from_utf8(&s.0)
    }
}

impl <'a> const TryFrom<&'a mut SmallByteStr> for &'a mut str {
    type Error = std::str::Utf8Error;

    #[inline]
    fn try_from(s: &'a mut SmallByteStr) -> Result<Self, Self::Error> {
        std::str::from_utf8_mut(&mut s.0)
    }
}

#[test]
fn wqe() {
    let i = std::bstr::ByteString(Vec::from([0, 0]));
    dbg!(i);

    let i = SmallByteString::empty();
    // SAFETY: all the elements in `..len` are initialized

    dbg!(i);
}
