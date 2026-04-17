use geometry::{Bounds, Size};
use crate::Buffer;

/// A double-buffered [`Buffer`]. The front holds the last rendered frame
/// (what the terminal is assumed to be showing); the back is where the next
/// frame is painted. Call [`swap`](Self::swap) after the frame has been
/// applied to the terminal.
#[derive(Debug)]
pub struct DoubleBuffer {
    pub inner: [Buffer; 2],
    pub index: usize,
}

impl DoubleBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            inner: [Buffer::new(width, height), Buffer::new(width, height)],
            index: 0,
        }
    }

    pub fn front(&self) -> &Buffer {
        &self.inner[self.index]
    }

    pub fn front_mut(&mut self) -> &mut Buffer {
        &mut self.inner[self.index]
    }

    pub fn back(&self) -> &Buffer {
        &self.inner[1 - self.index]
    }

    pub fn back_mut(&mut self) -> &mut Buffer {
        &mut self.inner[1 - self.index]
    }

    pub fn swap(&mut self) {
        self.index = 1 - self.index;
    }

    pub fn size(&self) -> Size {
        self.front().size()
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.inner[0].resize(width, height);
        self.inner[1].resize(width, height);
    }
}