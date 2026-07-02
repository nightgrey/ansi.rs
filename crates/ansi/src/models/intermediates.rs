use core::ascii;
use derive_more::{AsMut, Deref, DerefMut, Index, IndexMut, IntoIterator};
use smallvec::SmallVec;
use std::borrow::{Borrow, BorrowMut};
use std::fmt;
use std::mem::MaybeUninit;
use arrayvec::ArrayVec;

pub type Intermediate = ascii::Char;

#[derive(
    Eq, Hash, Clone, PartialOrd, Ord, Deref, DerefMut, Index, IndexMut, AsMut, IntoIterator,
)]
#[as_mut(forward)]
#[into_iterator(owned, ref, ref_mut)]
pub struct Intermediates<const N: usize = 2>(ArrayVec<Intermediate, N>);

impl<const N: usize> Intermediates<N> {
    #[inline]
    pub const fn empty() -> Self {
        Self(ArrayVec::new_const())
    }

    #[inline]
    pub fn new(intermediates: impl IntoIterator<Item = Intermediate>) -> Self {
        Self(ArrayVec::from_iter(intermediates))
    }

    /// Constructs a new instance on the stack from an array without copying elements."
    #[inline]
    pub fn from_array(value: [Intermediate; N]) -> Self {
        Self(ArrayVec::from(value))
    }

    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self::from_iter(bytes.iter().copied())
    }

    #[inline]
    pub fn from_str(str: &str) -> Self {
        Self::from_iter(str.bytes().filter_map(Intermediate::from_u8))
    }

    #[inline]
    pub fn as_slice(&self) -> &[Intermediate] {
        self.0.as_slice()
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [Intermediate] {
        self.0.as_mut_slice()
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { &*(self.0.as_slice() as *const [Intermediate] as *const [u8]) }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.as_bytes()) }
    }
}

const impl<const N: usize> Default for Intermediates<N> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<const N: usize> AsRef<[u8]> for Intermediates<N> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl<const N: usize> AsRef<str> for Intermediates<N> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> AsRef<[Intermediate]> for Intermediates<N> {
    #[inline]
    fn as_ref(&self) -> &[Intermediate] {
        self.0.as_slice()
    }
}

impl<const N: usize> Borrow<[Intermediate]> for Intermediates<N> {
    fn borrow(&self) -> &[Intermediate] {
        self.as_slice()
    }
}

impl<const N: usize> BorrowMut<[Intermediate]> for Intermediates<N> {
    fn borrow_mut(&mut self) -> &mut [Intermediate] {
        self.as_mut_slice()
    }
}

impl<const N: usize> Borrow<str> for Intermediates<N> {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<const N: usize> FromIterator<Intermediate> for Intermediates<N> {
    fn from_iter<__T: IntoIterator<Item = Intermediate>>(iter: __T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<const N: usize> FromIterator<u8> for Intermediates<N> {
    fn from_iter<__T: IntoIterator<Item = u8>>(iter: __T) -> Self {
        Self(ArrayVec::from_iter(iter.into_iter().filter_map(|b| {
            Intermediate::from_u8(b)
        })))
    }
}
impl<const N: usize> Extend<Intermediate> for Intermediates<N> {
    #[inline]
    fn extend<I: IntoIterator<Item = Intermediate>>(&mut self, iter: I) {
        self.0.extend(iter)
    }
}

impl<const N: usize> fmt::Display for Intermediates<N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl<const N: usize> fmt::Debug for Intermediates<N> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl<const N: usize> PartialEq<Intermediates<N>> for [u8] {
    #[inline]
    fn eq(&self, other: &Intermediates<N>) -> bool {
        self == other.as_bytes()
    }
}

 impl<const N: usize> PartialEq<&[u8]> for Intermediates<N> {
    #[inline]
    fn eq(&self, other: &&[u8]) -> bool {
        self.as_bytes() == *other
    }
}

 impl<const N: usize> PartialEq<Intermediates<N>> for &[u8] {
    #[inline]
    fn eq(&self, other: &Intermediates<N>) -> bool {
        *self == other.as_bytes()
    }
}

impl<const N: usize, const M: usize> PartialEq<[u8; M]> for Intermediates<N> {
    #[inline]
    fn eq(&self, other: &[u8; M]) -> bool {
        self.as_bytes() == other
    }
}

impl<const N: usize, const M: usize> PartialEq<Intermediates<N>> for [u8; M] {
    #[inline]
    fn eq(&self, other: &Intermediates<N>) -> bool {
        self.as_slice() == other.as_bytes()
    }
}

impl<const N: usize> PartialEq<str> for Intermediates<N> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}
impl<const N: usize> PartialEq<Intermediates<N>> for str {
    #[inline]
    fn eq(&self, other: &Intermediates<N>) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl<const N: usize> PartialEq<&str> for Intermediates<N> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl<const N: usize> PartialEq<Intermediates<N>> for &str {
    #[inline]
    fn eq(&self, other: &Intermediates<N>) -> bool {
        *self == other.as_str()
    }
}

impl<const N: usize> PartialEq<String> for Intermediates<N> {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<const N: usize> PartialEq<Intermediates<N>> for String {
    #[inline]
    fn eq(&self, other: &Intermediates<N>) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<const N: usize> PartialEq<Vec<u8>> for Intermediates<N> {
    #[inline]
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.as_bytes() == other.as_slice()
    }
}

impl<const N: usize> PartialEq<Intermediates<N>> for Vec<u8> {
    #[inline]
    fn eq(&self, other: &Intermediates<N>) -> bool {
        self.as_slice() == other.as_bytes()
    }
}

impl<const N: usize, const M: usize> PartialEq<Intermediates<M>> for Intermediates<N> {
    #[inline]
    fn eq(&self, other: &Intermediates<M>) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl<const N: usize> From<Intermediate> for Intermediates<N> {
    fn from(value: Intermediate) -> Self {
        Self(ArrayVec::from_iter([value; 1]))
    }
}

impl<const N: usize> From<&[u8]> for Intermediates<N> {
    #[inline]
    fn from(value: &[u8]) -> Self {
        Self::from_bytes(value)
    }
}

impl<const N: usize, const M: usize> From<&[u8; M]> for Intermediates<N> {
    #[inline]
    fn from(value: &[u8; M]) -> Self {
        Self::from_bytes(&value[..])
    }
}

impl<const N: usize> From<&str> for Intermediates<N> {
    #[inline]
    fn from(value: &str) -> Self {
        Self::from_str(value)
    }
}

impl<const N: usize> From<String> for Intermediates<N> {
    #[inline]
    fn from(value: String) -> Self {
        Self::from_str(&value)
    }
}

impl<const N: usize> From<Vec<u8>> for Intermediates<N> {
    #[inline]
    fn from(value: Vec<u8>) -> Self {
        Self::from_bytes(&value)
    }
}
