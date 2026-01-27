use super::{Buffer, Cell};
use std::ops;
use std::ops::{Add, Mul};
use std::slice::SliceIndex;
use crate::position::{Position, PositionLike};

pub trait BufferIndex: Sized {
    type Output: ?Sized;
    type SliceIndex: SliceIndex<[Cell], Output = Self::Output>;

    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex>;

    unsafe fn unchecked_index_of(self, buffer: &Buffer) -> Self::SliceIndex {
        self.index_of(buffer).unwrap_unchecked()
    }

    /// Returns a shared reference to the output at this location, if in
    /// bounds.
    fn get(self, buffer: &Buffer) -> Option<&Self::Output> {
        let index = self.index_of(buffer)?;

        index.get(buffer.as_slice())
    }

    /// Returns a mutable reference to the output at this location, if in
    /// bounds.
    fn get_mut(self, buffer: &mut Buffer) -> Option<&mut Self::Output> {
        let index = self.index_of(buffer)?;

        index.get_mut(buffer.as_mut_slice())
    }

    /// Returns a pointer to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    unsafe fn get_unchecked(self, buffer: *const Buffer) -> *const Self::Output {
        let index = self.unchecked_index_of(&*buffer);

        index.get_unchecked((&*buffer).as_slice() as *const [Cell])
    }

    /// Returns a mutable pointer to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    unsafe fn get_unchecked_mut(self, buffer: *mut Buffer) -> *mut Self::Output {
        let index = self.unchecked_index_of(&*buffer);

        index.get_unchecked_mut((&mut *buffer).as_mut_slice() as *mut [Cell])
    }

    /// Returns a shared reference to the output at this location, panicking
    /// if out of bounds.
    #[track_caller]
    fn index(self, buffer: &Buffer) -> &Self::Output {
        let index = unsafe { self.unchecked_index_of(buffer) };

        index.index(buffer)
    }

    /// Returns a mutable reference to the output at this location, panicking
    /// if out of bounds.
    #[track_caller]
    fn index_mut(self, buffer: &mut Buffer) -> &mut Self::Output {
        let index = unsafe { self.unchecked_index_of(buffer) };

        index.index_mut(buffer)
    }
}

pub type RowLike = [usize; 1];

impl BufferIndex for usize {
    type SliceIndex = usize;
    type Output = Cell;

    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        let start = self;
        let len = buffer.len();

        if start < len { Some(start) } else { None }
    }
}
impl BufferIndex for ops::Range<usize> {
    type SliceIndex = ops::Range<usize>;
    type Output = [Cell];

    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        let start = self.start;
        let end = self.end;
        let len = buffer.len();

        if start < len && end < len {
            Some(start..end)
        } else {
            None
        }
    }
}
impl BufferIndex for ops::RangeInclusive<usize> {
    type SliceIndex = ops::RangeInclusive<usize>;
    type Output = [Cell];

    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        let start = *self.start();
        let end = *self.end();
        let len = buffer.len();

        if start < len && end < len {
            Some(start..=end)
        } else {
            None
        }
    }
}
impl BufferIndex for ops::RangeFrom<usize> {
    type SliceIndex = ops::RangeFrom<usize>;
    type Output = [Cell];

    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        let start = self.start;
        let len = buffer.len();

        if start < len { Some(start..) } else { None }
    }
}
impl BufferIndex for ops::RangeTo<usize> {
    type SliceIndex = ops::RangeTo<usize>;
    type Output = [Cell];

    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        let end = self.end;
        let len = buffer.len();

        if end < len { Some(..end) } else { None }
    }
}
impl BufferIndex for ops::RangeToInclusive<usize> {
    type SliceIndex = ops::RangeToInclusive<usize>;
    type Output = [Cell];

    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        let end = self.end;
        let len = buffer.len();

        if end < len { Some(..=end) } else { None }
    }
}

impl BufferIndex for Position {
    type SliceIndex = usize;
    type Output = Cell;

    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        let width = buffer.width();
        let height = buffer.height();
        
        if self.col >= width || self.row >= height {
            return None;
        }

        Some(self.row * width + self.col)
    }
}
impl BufferIndex for ops::Range<Position> {
    type SliceIndex = ops::Range<usize>;
    type Output = [Cell];

    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        let start = &self.start;
        let end = &self.end;

        if buffer.contains(start) && buffer.contains(end) {
            let width = buffer.width();
            Some(start.row * width + start.col..end.row * width + end.col)
        } else {
            None
        }
    }
}
impl BufferIndex for ops::RangeInclusive<Position> {
    type SliceIndex = ops::RangeInclusive<usize>;
    type Output = [Cell];

    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        let start = self.start();
        let end = self.end();

        if buffer.contains(start) && buffer.contains(end) {
            let width = buffer.width();
            Some(start.row * width + start.col..=end.row * width + end.col)
        } else {
            None
        }
    }
}
impl BufferIndex for ops::RangeFrom<Position> {
    type SliceIndex = ops::RangeFrom<usize>;
    type Output = [Cell];

    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        let start = &self.start;

        if buffer.contains(start) {
            Some(start.row * buffer.width() + start.col..)
        } else {
            None
        }
    }
}
impl BufferIndex for ops::RangeTo<Position> {
    type SliceIndex = ops::RangeTo<usize>;
    type Output = [Cell];

    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        let end = &self.end;

        if buffer.contains(end) {
            let width = buffer.width();
            Some(..end.row * width + end.col)
        } else {
            None
        }
    }
}
impl BufferIndex for ops::RangeToInclusive<Position> {
    type SliceIndex = ops::RangeToInclusive<usize>;
    type Output = [Cell];

    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        let end = &self.end;

        if buffer.contains(end) {
            Some(..=end.row * buffer.width() + end.col)
        } else {
            None
        }
    }
}

impl BufferIndex for RowLike {
    type Output = [Cell];
    type SliceIndex = ops::Range<usize>;

    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        let start = self[0];
        let height = buffer.height();

        if start < height {
            let width = buffer.width();

            let start_of_start = start * width;
            Some(start_of_start..start_of_start + width)
        } else {
            None
        }
    }
}
impl BufferIndex for ops::Range<RowLike> {
    type Output = [Cell];
    type SliceIndex = ops::Range<usize>;

    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        let start = self.start[0];
        let end = self.end[0];

        let height = buffer.height();

        if start < height && end < height {
            let width = buffer.width();

            let start_of_start = start * width;
            let end_of_end = end * width + width;
            Some(start_of_start..end_of_end)
        } else {
            None
        }
    }
}
impl BufferIndex for ops::RangeInclusive<RowLike> {
    type Output = [Cell];
    type SliceIndex = ops::RangeInclusive<usize>;

    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        let start = self.start()[0];
        let end = self.end()[0];

        let height = buffer.height();

        if start < height && end < height {
            let width = buffer.width();

            let start_of_start = start * width;
            let end_of_end = end * width + width;
            Some(start_of_start..=end_of_end)
        } else {
            None
        }
    }
}
impl BufferIndex for ops::RangeFrom<RowLike> {
    type Output = [Cell];
    type SliceIndex = ops::RangeFrom<usize>;

    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        let start = self.start[0];

        let height = buffer.height();

        if start < height {
            let width = buffer.width();

            let start_of_start = start * width;
            Some(start_of_start..)
        } else {
            None
        }
    }
}
impl BufferIndex for ops::RangeTo<RowLike> {
    type SliceIndex = ops::RangeTo<usize>;
    type Output = [Cell];

    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        let end = self.end[0];

        let width = buffer.width();
        let height = buffer.height();

        if end < height {
            let end_of_end = end * width + width;
            Some(..end_of_end)
        } else {
            None
        }
    }
}
impl BufferIndex for ops::RangeToInclusive<RowLike> {
    type SliceIndex = ops::RangeToInclusive<usize>;
    type Output = [Cell];

    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        let end = self.end[0];

        let width = buffer.width();
        let height = buffer.height();

        if end < height {
            let end_of_end = end * width + width;
            Some(..=end_of_end)
        } else {
            None
        }
    }
}

impl BufferIndex for ops::RangeFull {
    type SliceIndex = ops::RangeFull;
    type Output = [Cell];

    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        Some(..)
    }
}

impl<I: BufferIndex> ops::Index<I> for Buffer {
    type Output = <I::SliceIndex as SliceIndex<[Cell]>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        index.index(self)
    }
}

impl<I: BufferIndex> ops::IndexMut<I> for Buffer {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        index.index_mut(self)
    }
}
