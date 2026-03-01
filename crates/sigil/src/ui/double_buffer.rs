use crate::{Buffer};
use std::ops::{Deref, DerefMut, Index, IndexMut};

#[derive(Clone, Debug)]
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
