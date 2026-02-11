use crate::{Buffer, BufferIndex};
use std::ops::{Deref, DerefMut, Index, IndexMut};

#[derive(Clone, PartialEq, Debug)]
pub struct DoubleBuffer {
    pub front: Buffer,
    pub back: Buffer,
}

impl DoubleBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            front: Buffer::new(width, height),
            back: Buffer::new(width, height),
        }
    }

    pub fn swap(&mut self) {
        std::mem::swap(&mut self.front, &mut self.back);
    }
}

impl Deref for DoubleBuffer {
    type Target = Buffer;

    fn deref(&self) -> &Self::Target {
        &self.front
    }
}

impl DerefMut for DoubleBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.front
    }
}

impl<I: BufferIndex> Index<I> for DoubleBuffer {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        index.index(&self.front)
    }
}

impl<I: BufferIndex> IndexMut<I> for DoubleBuffer {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        index.index_mut(&mut self.front)
    }
}
