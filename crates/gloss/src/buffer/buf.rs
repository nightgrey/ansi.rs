use super::{Buffer, BufferIndex, Arena};
use derive_more::{Deref, DerefMut, Index, IndexMut, IntoIterator};
use geometry::{Intersect};
use std::fmt::Debug;
use std::ops::{Index, IndexMut};
use std::slice::SliceIndex;

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

    pub fn to_string(&self) -> String {
        self.iter_rows()
            .map(|row| row.map(|cell| cell.as_str(self.arena)).collect::<String>())
            .intersperse(String::from("\n"))
            .collect()
    }
}

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

    pub fn to_string(&self) -> String {
        self.iter_rows()
            .map(|row| row.map(|cell| cell.as_str(self.arena)).collect::<String>())
            .intersperse(String::from("\n"))
            .collect()
    }
}