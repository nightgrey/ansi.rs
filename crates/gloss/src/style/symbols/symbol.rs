use std::ops::Deref;
use derive_more::Deref;

#[derive(Debug, Clone, Copy, Deref)]
#[derive_const(PartialEq, Eq)]
pub struct Symbol<T = char> {
    #[deref]
    pub inner: T,
    pub(super) width: usize,
}

impl<T> Symbol<T> {
    pub const fn new(inner: T, width: usize) -> Self {
        Self { inner, width }
    }

    #[inline]
    pub const fn symbol(&self) -> T where T: Copy {
        self.inner
    }
    #[inline]
    pub const fn width(&self) -> usize {
        self.width
    }
}

impl Into<char> for Symbol<char> {
    fn into(self) -> char {
        self.inner
    }
}

impl<T> const AsRef<T> for Symbol<T> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}
