use super::{Arena, Buffer};
use derive_more::{Deref, DerefMut, Index, IndexMut, IntoIterator};
use std::fmt::{self, Display};

/// A shared view over a [`Buffer`] paired with its [`Arena`].
///
/// The pairing exists because rendering a cell to a string requires both the
/// buffer (to reach the cell) and the arena (to resolve its grapheme). A `Buf`
/// bundles them.
#[derive(Debug, Deref, DerefMut, Index, IndexMut, IntoIterator)]
pub struct Buf<'a> {
    #[deref]
    #[deref_mut]
    #[index]
    #[index_mut]
    #[into_iterator(ref)]
    pub buffer: &'a Buffer,
    pub arena: &'a Arena,
}

impl<'a> Buf<'a> {
    pub fn new(buffer: &'a Buffer, arena: &'a Arena) -> Self {
        Self { buffer, arena }
    }
}

impl<'a> Display for Buf<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for row in self.buffer.iter_rows() {
            if !first {
                f.write_str("\n")?;
            }
            first = false;
            for cell in row {
                f.write_str(cell.as_str(self.arena))?;
            }
        }
        Ok(())
    }
}

/// A mutable view over a [`Buffer`] paired with its [`Arena`]. See [`Buf`].
#[derive(Debug, Deref, DerefMut, Index, IndexMut, IntoIterator)]
pub struct BufMut<'a> {
    #[deref]
    #[deref_mut]
    #[index]
    #[index_mut]
    #[into_iterator(owned, ref, ref_mut)]
    pub buffer: &'a mut Buffer,
    pub arena: &'a mut Arena,
}

impl<'a> BufMut<'a> {
    pub fn new(buffer: &'a mut Buffer, arena: &'a mut Arena) -> Self {
        Self { buffer, arena }
    }

    /// Re-borrow as an immutable [`Buf`] for read-only operations.
    pub fn as_buf(&self) -> Buf<'_> {
        Buf {
            buffer: self.buffer,
            arena: self.arena,
        }
    }
}

impl<'a> Display for BufMut<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.as_buf(), f)
    }
}
