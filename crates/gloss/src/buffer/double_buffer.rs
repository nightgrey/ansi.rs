use crate::Buffer;
use geometry::{Bound, Size};

/// A double-buffered [`Buffer`]. The front holds the last rendered frame
/// (what the terminal is assumed to be showing); the back is where the next
/// frame is painted. Call [`swap`](Self::swap) after the frame has been
/// applied to the terminal.
#[derive(Debug)]
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

    pub fn front(&self) -> &Buffer {
        &self.front
    }

    pub fn front_mut(&mut self) -> &mut Buffer {
        &mut self.front
    }

    pub fn back(&self) -> &Buffer {
        &self.back
    }

    pub fn back_mut(&mut self) -> &mut Buffer {
        &mut self.back
    }

    pub fn both(&self) -> (&Buffer, &Buffer) {
        (&self.front(), &self.back())
    }

    pub fn swap(&mut self) {
        std::mem::swap(&mut self.front, &mut self.back);
    }

    pub fn size(&self) -> Size {
        self.front().size()
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.front.resize(width, height);
        self.back.resize(width, height);
    }
}
