use std::ops::Deref;
use derive_more::{AsMut, AsRef, Deref};
use unicode_width::UnicodeWidthChar;

#[derive(Debug, Clone, Copy, Deref, AsRef, AsMut, Hash)]
#[derive_const(PartialEq, Eq)]
pub struct Symbol<T = char> {
    #[deref]
    #[as_ref]
    #[as_mut]
    pub inner: T,
    pub(super) width: usize,
}

impl<T> Symbol<T> {
    pub const fn new(inner: T) -> Self {
        Self { inner, width: 1 }
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

impl Symbol<char> {
    pub const MIN: Self = Self { inner: char::MIN, width: 0 };
    pub const SPACE: Self = Self { inner: ' ', width: 1 };
    pub fn measured(inner: char) -> Self {
        Self { inner, width: inner.width().unwrap_or(1) }
    }
}
impl From<Symbol<char>> for char {
    fn from(symbol: Symbol<char>) -> Self {
        symbol.inner
    }
}
