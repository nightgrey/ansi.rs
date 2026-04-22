use std::borrow::Borrow;

use derive_more::{AsRef, Deref, DerefMut, Display, From, Into, IntoIterator};
use smallvec::SmallVec;

#[derive(
    PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Default, Debug, Into, IntoIterator, Deref, DerefMut,
)]
#[into_iterator(owned, ref, ref_mut)]
pub struct SmallByteString<const N: usize>(SmallVec<u8, N>);

impl<const N: usize> SmallByteString<N> {
    #[inline]
    pub const fn new() -> Self {
        Self(SmallVec::new())
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self(SmallVec::with_capacity(capacity))
    }

    #[inline]
    pub const fn from_buf<const S: usize>(elements: [u8; S]) -> Self {
        Self(SmallVec::from_buf(elements))
    }

    #[inline]
    pub fn from_buf_and_len(buf: [u8; N], len: usize) -> Self {
        Self(SmallVec::from_buf_and_len(buf, len))
    }

    #[allow(missing_docs)]
    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(&self.0) }
    }
}

impl<const N: usize> AsRef<[u8]> for SmallByteString<N> {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<const N: usize> AsRef<str> for SmallByteString<N> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> Borrow<[u8]> for SmallByteString<N> {
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}

impl<const M: usize, const N: usize> From<&[u8; M]> for SmallByteString<N> {
    #[inline]
    fn from(value: &[u8; M]) -> Self {
        Self(SmallVec::from(value))
    }
}

impl<const N: usize, const M: usize> From<[u8; M]> for SmallByteString<N> {
    fn from(value: [u8; M]) -> Self {
        Self(SmallVec::from(value))
    }
}

impl<const M: usize, const N: usize> From<&[char; M]> for SmallByteString<N> {
    #[inline]
    fn from(value: &[char; M]) -> Self {
        Self(SmallVec::from(unsafe {
            std::mem::transmute::<&[char; M], &[u8; M]>(value)
        }))
    }
}

impl<const N: usize, const M: usize> From<[char; M]> for SmallByteString<N> {
    fn from(value: [char; M]) -> Self {
        Self(SmallVec::from_iter(value.iter().copied().map(|c| c as u8)))
    }
}
// impl<const N: usize> From<Vec<u8>> for Intermediates<N> {
//     fn from(value: Vec<u8>) -> Self {
//         Self(SmallVec::from_vec(value))
//     }
// }

impl<const N: usize> From<&str> for SmallByteString<N> {
    #[inline]
    fn from(value: &str) -> Self {
        Self(SmallVec::from(value.as_bytes()))
    }
}

impl<const N: usize> From<&[char]> for SmallByteString<N> {
    #[inline]
    fn from(value: &[char]) -> Self {
        Self(SmallVec::from(unsafe {
            std::mem::transmute::<&[char], &[u8]>(value)
        }))
    }
}
impl<const N: usize> From<&[u8]> for SmallByteString<N> {
    #[inline]
    fn from(value: &[u8]) -> Self {
        Self(SmallVec::from(value))
    }
}

impl<const N: usize> From<String> for SmallByteString<N> {
    #[inline]
    fn from(value: String) -> Self {
        Self(SmallVec::from(value.as_bytes()))
    }
}
