use core::borrow::{Borrow, BorrowMut};
use core::cmp::Ordering;
use core::ops::{
    Deref, DerefMut, Index, IndexMut, Range, RangeFrom, RangeFull, RangeInclusive,
    RangeTo, RangeToInclusive,
};
use core::str::FromStr;
use core::{fmt, hash};
use std::borrow::{Cow, ToOwned};
use compact_bytes::CompactBytes;

/// A wrapper for `CompactBytes` representing a human-readable string that's conventionally, but not always, UTF-8.
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
/// The `Debug` and `Display` implementations for `ByteString` are the same as those for `ByteStr`,
/// showing invalid UTF-8 as hex escapes or the Unicode replacement character, respectively.
#[repr(transparent)]
#[derive(Clone)]
#[doc(alias = "ByteStr")]
pub struct ByteString(pub CompactBytes);

impl ByteString {
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
    pub(crate) fn as_byte_str(&self) -> &ByteStr {
        ByteStr::new(&self.0)
    }

    #[inline]
    pub(crate) fn as_mut_byte_str(&mut self) -> &mut ByteStr {
        ByteStr::from_bytes_mut(&mut self.0)
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe  { str::from_utf8_unchecked(&self.0) }
    }

    #[inline]
    pub fn as_str_mut(&mut self) -> &mut str {
        unsafe  { str::from_utf8_unchecked_mut(&mut self.0) }
    }
}

impl Deref for ByteString {
    type Target = CompactBytes;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ByteString {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl fmt::Debug for ByteString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(std::bstr::ByteStr::new(&self), f)
    }
}

impl fmt::Display for ByteString {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_byte_str(), f)
    }
}
impl AsRef<[u8]> for ByteString {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<ByteStr> for ByteString {
    #[inline]
    fn as_ref(&self) -> &ByteStr {
        self.as_byte_str()
    }
}

impl AsMut<[u8]> for ByteString {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl AsMut<ByteStr> for ByteString {
    #[inline]
    fn as_mut(&mut self) -> &mut ByteStr {
        self.as_mut_byte_str()
    }
}

impl Borrow<[u8]> for ByteString {
    #[inline]
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}

impl Borrow<ByteStr> for ByteString {
    #[inline]
    fn borrow(&self) -> &ByteStr {
        self.as_byte_str()
    }
}
impl Borrow<ByteStr> for Vec<u8> {
    #[inline]
    fn borrow(&self) -> &ByteStr {
        ByteStr::new(self)
    }
}
impl Borrow<ByteStr> for String {
    #[inline]
    fn borrow(&self) -> &ByteStr {
        ByteStr::new(self.as_bytes())
    }
}

impl BorrowMut<[u8]> for ByteString {
    #[inline]
    fn borrow_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}
impl BorrowMut<ByteStr> for ByteString {
    #[inline]
    fn borrow_mut(&mut self) -> &mut ByteStr {
        self.as_mut_byte_str()
    }
}
impl BorrowMut<ByteStr> for String {
    #[inline]
    fn borrow_mut(&mut self) -> &mut ByteStr {
        ByteStr::from_bytes_mut(unsafe { self.as_bytes_mut() })
    }
}

impl Default for ByteString {
    fn default() -> Self {
        ByteString::empty()
    }
}

impl From<&[u8]> for ByteString {
    #[inline]
    fn from(s: &[u8]) -> Self {
        Self(CompactBytes::new(s))
    }
}

impl<const N: usize> From<[u8; N]> for ByteString {
    #[inline]
    fn from(s: [u8; N]) -> Self {
        Self(CompactBytes::new(&s))
    }
}
impl<const N: usize> From<& [u8; N]> for ByteString {
    #[inline]
    fn from(s: &[u8; N]) -> Self {
        Self(CompactBytes::new(s))
    }
}
impl From<ByteString> for CompactBytes {
    #[inline]
    fn from(s: ByteString) -> Self {
        s.0
    }
}

impl<'a> From<&'a ByteStr> for ByteString {
    #[inline]
    fn from(s: &'a ByteStr) -> Self {
        ByteString(CompactBytes::new(&s.0))
    }
}

impl<'a> From<ByteString> for Cow<'a, ByteStr> {
    #[inline]
    fn from(s: ByteString) -> Self {
        Cow::Owned(s)
    }
}

impl<'a> From<&'a ByteString> for Cow<'a, ByteStr> {
    #[inline]
    fn from(s: &'a ByteString) -> Self {
        Cow::Borrowed(s.as_byte_str())
    }
}

impl FromIterator<char> for ByteString {
    #[inline]
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        ByteString(CompactBytes::new(iter.into_iter().collect::<String>().as_bytes()))
    }
}

impl FromIterator<u8> for ByteString {
    #[inline]
    fn from_iter<T: IntoIterator<Item = u8>>(iter: T) -> Self {
        let mut buf = CompactBytes::empty();
        for b in iter {
            buf.push(b);
        }
        
        ByteString(buf)
    }
}

impl<'a> FromIterator<&'a u8> for ByteString {
    #[inline]
    fn from_iter<T: IntoIterator<Item = &'a u8>>(iter: T) -> Self {
        let mut buf = CompactBytes::empty();
        for b in iter {
            buf.push(*b);
        }
        ByteString(buf)
    }
}

impl<'a> FromIterator<&'a str> for ByteString {
    #[inline]
    fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
        ByteString(CompactBytes::new(iter.into_iter().collect::<String>().as_bytes()))
    }
}

impl<'a> FromIterator<&'a [u8]> for ByteString {
    #[inline]
    fn from_iter<T: IntoIterator<Item = &'a [u8]>>(iter: T) -> Self {
        let mut buf = CompactBytes::empty();
        for b in iter {
            buf.extend_from_slice(b);
        }
        ByteString(buf)
    }
}

impl<'a> FromIterator<&'a ByteStr> for ByteString {
    fn from_iter<T: IntoIterator<Item = &'a ByteStr>>(iter: T) -> Self {
        let mut buf = CompactBytes::empty();
        for b in iter {
            buf.extend_from_slice(&b.0);
        }
        ByteString(buf)
    }
}

impl FromIterator<ByteString> for ByteString {
    #[inline]
    fn from_iter<T: IntoIterator<Item = ByteString>>(iter: T) -> Self {

        ByteString(CompactBytes::new(iter.into_iter().collect::<Vec<_>>().concat().as_slice()))
    }
}

impl FromStr for ByteString {
    type Err = core::convert::Infallible;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ByteString(CompactBytes::new(s.as_bytes())))
    }
}

impl Index<usize> for ByteString {
    type Output = u8;

    #[inline]
    fn index(&self, idx: usize) -> &u8 {
        &self.0[idx]
    }
}

impl Index<RangeFull> for ByteString {
    type Output = ByteStr;

    #[inline]
    fn index(&self, _: RangeFull) -> &ByteStr {
        self.as_byte_str()
    }
}

impl Index<Range<usize>> for ByteString {
    type Output = ByteStr;

    #[inline]
    fn index(&self, r: Range<usize>) -> &ByteStr {
        ByteStr::from_bytes(&self.0[r])
    }
}

impl Index<RangeInclusive<usize>> for ByteString {
    type Output = ByteStr;

    #[inline]
    fn index(&self, r: RangeInclusive<usize>) -> &ByteStr {
        ByteStr::from_bytes(&self.0[r])
    }
}

impl Index<RangeFrom<usize>> for ByteString {
    type Output = ByteStr;

    #[inline]
    fn index(&self, r: RangeFrom<usize>) -> &ByteStr {
        ByteStr::from_bytes(&self.0[r])
    }
}

impl Index<RangeTo<usize>> for ByteString {
    type Output = ByteStr;

    #[inline]
    fn index(&self, r: RangeTo<usize>) -> &ByteStr {
        ByteStr::from_bytes(&self.0[r])
    }
}

impl Index<RangeToInclusive<usize>> for ByteString {
    type Output = ByteStr;

    #[inline]
    fn index(&self, r: RangeToInclusive<usize>) -> &ByteStr {
        ByteStr::from_bytes(&self.0[r])
    }
}

impl IndexMut<usize> for ByteString {
    #[inline]
    fn index_mut(&mut self, idx: usize) -> &mut u8 {
        &mut self.0[idx]
    }
}

impl IndexMut<RangeFull> for ByteString {
    #[inline]
    fn index_mut(&mut self, _: RangeFull) -> &mut ByteStr {
        self.as_mut_byte_str()
    }
}

impl IndexMut<Range<usize>> for ByteString {
    #[inline]
    fn index_mut(&mut self, r: Range<usize>) -> &mut ByteStr {
        ByteStr::from_bytes_mut(&mut self.0[r])
    }
}

impl IndexMut<RangeInclusive<usize>> for ByteString {
    #[inline]
    fn index_mut(&mut self, r: RangeInclusive<usize>) -> &mut ByteStr {
        ByteStr::from_bytes_mut(&mut self.0[r])
    }
}

impl IndexMut<RangeFrom<usize>> for ByteString {
    #[inline]
    fn index_mut(&mut self, r: RangeFrom<usize>) -> &mut ByteStr {
        ByteStr::from_bytes_mut(&mut self.0[r])
    }
}

impl IndexMut<RangeTo<usize>> for ByteString {
    #[inline]
    fn index_mut(&mut self, r: RangeTo<usize>) -> &mut ByteStr {
        ByteStr::from_bytes_mut(&mut self.0[r])
    }
}

impl IndexMut<RangeToInclusive<usize>> for ByteString {
    #[inline]
    fn index_mut(&mut self, r: RangeToInclusive<usize>) -> &mut ByteStr {
        ByteStr::from_bytes_mut(&mut self.0[r])
    }
}

impl hash::Hash for ByteString {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Eq for ByteString {}

impl PartialEq for ByteString {
    #[inline]
    fn eq(&self, other: &ByteString) -> bool {
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
        impl_partial_eq!( $lhs, $rhs);

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
impl_partial_eq!(ByteString, Vec<u8>);
// PartialOrd with `[u8]` omitted to avoid inference failures
impl_partial_eq!(ByteString, [u8]);
// PartialOrd with `&[u8]` omitted to avoid inference failures
impl_partial_eq!(ByteString, &[u8]);
// PartialOrd with `String` omitted to avoid inference failures
impl_partial_eq!(ByteString, String);
// PartialOrd with `str` omitted to avoid inference failures
impl_partial_eq!(ByteString, str);
// PartialOrd with `&str` omitted to avoid inference failures
impl_partial_eq!(ByteString, &str);
impl_partial_eq_ord!(ByteString, ByteStr);
impl_partial_eq_ord!(ByteString, &ByteStr);
// PartialOrd with `[u8]` omitted to avoid inference failures
impl_partial_eq_n!(ByteString, [u8; N]);
// PartialOrd with `&[u8]` omitted to avoid inference failures
impl_partial_eq_n!(ByteString, &[u8; N]);
impl_partial_eq_ord_cow!(ByteString, Cow<'_, ByteStr>);
impl_partial_eq_ord_cow!(ByteString, Cow<'_, str>);
impl_partial_eq_ord_cow!(ByteString, Cow<'_, [u8]>);

impl Ord for ByteString {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&self.0.as_slice(), &other.0.as_slice())
    }
}

impl PartialOrd for ByteString {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(&self.0.as_slice(), &other.0.as_slice())
    }
}

impl ToOwned for ByteStr {
    type Owned = ByteString;

    #[inline]
    fn to_owned(&self) -> ByteString {
        ByteString(CompactBytes::new(&self.0))
    }
}

impl TryFrom<ByteString> for String {
    type Error = std::string::FromUtf8Error;

    #[inline]
    fn try_from(s: ByteString) -> Result<Self, Self::Error> {
        String::from_utf8(s.0.to_vec())
    }
}

impl<'a> TryFrom<&'a ByteString> for &'a str {
    type Error = std::str::Utf8Error;

    #[inline]
    fn try_from(s: &'a ByteString) -> Result<Self, Self::Error> {
        std::str::from_utf8(s.0.as_slice())
    }
}

// Additional impls for `ByteStr` that require types from `alloc`:
impl Clone for Box<ByteStr> {
    #[inline]
    fn clone(&self) -> Self {
        Self::from(Box::<[u8]>::from(&self.0 as &[u8]))
    }
}

impl<'a> From<&'a ByteStr> for Cow<'a, ByteStr> {
    #[inline]
    fn from(s: &'a ByteStr) -> Self {
        Cow::Borrowed(s)
    }
}

impl From<Box<[u8]>> for Box<ByteStr> {
    #[inline]
    fn from(s: Box<[u8]>) -> Box<ByteStr> {
        // SAFETY: `ByteStr` is a transparent wrapper around `[u8]`.
        unsafe { Box::from_raw(Box::into_raw(s) as _) }
    }
}

impl From<Box<ByteStr>> for Box<[u8]> {
    #[inline]
    fn from(s: Box<ByteStr>) -> Box<[u8]> {
        // SAFETY: `ByteStr` is a transparent wrapper around `[u8]`.
        unsafe { Box::from_raw(Box::into_raw(s) as _) }
    }
}

// PartialOrd with `Vec<u8>` omitted to avoid inference failures
impl_partial_eq!( ByteStr, Vec<u8>);
// PartialOrd with `String` omitted to avoid inference failures
impl_partial_eq!( ByteStr, String);
impl_partial_eq_ord_cow!( &ByteStr, Cow<'_, ByteStr>);
impl_partial_eq_ord_cow!( &ByteStr, Cow<'_, str>);
impl_partial_eq_ord_cow!( &ByteStr, Cow<'_, [u8]>);

impl<'a> TryFrom<&'a ByteStr> for String {
    type Error = core::str::Utf8Error;

    #[inline]
    fn try_from(s: &'a ByteStr) -> Result<Self, Self::Error> {
        Ok(core::str::from_utf8(&s.0)?.into())
    }
}


/// A wrapper for `&[u8]` representing a human-readable string that's conventionally, but not always, UTF-8.
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
pub struct ByteStr(pub [u8]);

impl ByteStr {
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
        unsafe  { str::from_utf8_unchecked(&self.0) }
    }

    #[inline]
    pub const fn as_str_mut(&mut self) -> &mut str {
        unsafe  { str::from_utf8_unchecked_mut(&mut self.0) }
    }
}


impl const Deref for ByteStr {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        &self.0
    }
}

impl const DerefMut for ByteStr {
    #[inline]
    fn deref_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}


impl fmt::Display for ByteStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(std::bstr::ByteStr::new(&self), f)
    }
}

impl fmt::Debug for ByteStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(std::bstr::ByteStr::new(&self), f)

    }
}

impl AsRef<[u8]> for ByteStr {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<ByteStr> for ByteStr {
    #[inline]
    fn as_ref(&self) -> &ByteStr {
        self
    }
}

// `impl AsRef<ByteStr> for [u8]` omitted to avoid widespread inference failures

impl AsRef<ByteStr> for str {
    #[inline]
    fn as_ref(&self) -> &ByteStr {
        ByteStr::new(self)
    }
}

impl AsMut<[u8]> for ByteStr {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

// `impl AsMut<ByteStr> for [u8]` omitted to avoid widespread inference failures

// `impl Borrow<ByteStr> for [u8]` omitted to avoid widespread inference failures

// `impl Borrow<ByteStr> for str` omitted to avoid widespread inference failures

impl Borrow<[u8]> for ByteStr {
    #[inline]
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}

// `impl BorrowMut<ByteStr> for [u8]` omitted to avoid widespread inference failures

impl BorrowMut<[u8]> for ByteStr {
    #[inline]
    fn borrow_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl<'a> Default for &'a ByteStr {
    fn default() -> Self {
        ByteStr::from_bytes(b"")
    }
}

impl<'a> Default for &'a mut ByteStr {
    fn default() -> Self {
        ByteStr::from_bytes_mut(&mut [])
    }
}

impl<'a> const TryFrom<&'a ByteStr> for &'a str {
    type Error = std::str::Utf8Error;

    #[inline]
    fn try_from(s: &'a ByteStr) -> Result<Self, Self::Error> {
        std::str::from_utf8(&s.0)
    }
}

impl<'a> const TryFrom<&'a mut ByteStr> for &'a mut str {
    type Error = std::str::Utf8Error;

    #[inline]
    fn try_from(s: &'a mut ByteStr) -> Result<Self, Self::Error> {
        std::str::from_utf8_mut(&mut s.0)
    }
}

#[test]
fn wqe() {
    let i = std::bstr::ByteString(Vec::from([0,0 ]));
    dbg!(i);

    let i = ByteString::empty();
    // SAFETY: all the elements in `..len` are initialized

    dbg!(i);
}