use crate::Buffer;

/// A double-buffered [`Buffer`].
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
}