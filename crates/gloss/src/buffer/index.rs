use geometry::{Point, PointLike, Resolve, Row};
use std::ops;
use std::ops::{Index, IndexMut};
use std::slice::SliceIndex;
use crate::{Buffer, Cell};

pub trait BufferIndex: Clone {
    type Output: ?Sized;
    type Index: SliceIndex<[Cell], Output = Self::Output>;

    /// Returns the [`Self::Index`] for this location.
    ///
    /// This method does not perform any bounds checking.
    #[inline]
    fn slice_index(self, context: &Buffer) -> Self::Index;

    /// Returns a shared reference to the output at this location, if in
    /// bounds.
    #[inline]
    fn get(self, context: &Buffer) -> Option<&Self::Output> {
        SliceIndex::get(self.slice_index(context), context)
    }

    /// Returns a mutable reference to the output at this location, if in
    /// bounds.
    #[inline]
    fn get_mut(self, context: &mut Buffer) -> Option<&mut Self::Output> {
        SliceIndex::get_mut(self.slice_index(context), context)
    }

    /// Returns a pointer to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    #[inline]
    unsafe fn get_unchecked(self, context: &Buffer) -> *const Self::Output {
        SliceIndex::get_unchecked(self.slice_index(context), context.as_ref())
    }
    /// Returns a mutable pointer to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    #[inline]
    unsafe fn get_unchecked_mut(self, context: &mut Buffer) -> *mut Self::Output {
        SliceIndex::get_unchecked_mut(self.slice_index(context), context.as_mut())
    }

    /// Returns a shared reference to the output at this location, panicking
    /// if out of bounds.
    #[track_caller]
    #[inline]
    fn index(self, context: &Buffer) -> &Self::Output {
        SliceIndex::index(self.slice_index(context), context)
    }

    /// Returns a mutable reference to the output at this location, panicking
    /// if out of bounds.
    #[track_caller] 
    #[inline]
    fn index_mut(self, context: &mut Buffer) -> &mut Self::Output {
        SliceIndex::index_mut(self.slice_index(context), context)
    }
}

impl BufferIndex for Point {
    type Output = Cell;
    type Index = usize;

    #[inline]
    fn slice_index(self, buffer: &Buffer) -> usize {
        buffer.resolve(self)
    }
}

impl BufferIndex for PointLike {
    type Output = Cell;
    type Index = usize;

    #[inline]
    fn slice_index(self, buffer: &Buffer) -> usize {
        buffer.resolve(self)
    }
}

impl BufferIndex for Row {
    type Output = [Cell];
    type Index = ops::Range<usize>;

    #[inline]
    fn slice_index(self, buffer: &Buffer) -> ops::Range<usize> {
        buffer.resolve(self)
    }
}

impl BufferIndex for ops::Range<Row> {
    type Output = [Cell];
    type Index = ops::Range<usize>;

    #[inline]
    fn slice_index(self, buffer: &Buffer) -> ops::Range<usize> {
        buffer.resolve(self)
    }
}

impl BufferIndex for ops::RangeInclusive<Row> {
    type Output = [Cell];
    type Index = ops::RangeInclusive<usize>;

    #[inline]
    fn slice_index(self, buffer: &Buffer) -> ops::RangeInclusive<usize> {
        buffer.resolve(self)
    }
}


impl BufferIndex for ops::RangeTo<Row> {
    type Output = [Cell];
    type Index = ops::RangeTo<usize>;

    #[inline]
    fn slice_index(self, buffer: &Buffer) -> ops::RangeTo<usize> {
        buffer.resolve(self)
    }
}

impl BufferIndex for ops::RangeToInclusive<Row> {
    type Output = [Cell];
    type Index = ops::RangeToInclusive<usize>;

    #[inline]
    fn slice_index(self, buffer: &Buffer) -> ops::RangeToInclusive<usize> {
        buffer.resolve(self)
    }
}
impl BufferIndex for ops::RangeFrom<Row> {
    type Output = [Cell];
    type Index = ops::RangeFrom<usize>;

    #[inline]
    fn slice_index(self, buffer: &Buffer) -> ops::RangeFrom<usize> {
        buffer.resolve(self)

    }
}

impl BufferIndex for ops::RangeFull {
    type Output = [Cell];
    type Index = ops::RangeFull;

    #[inline]
    fn slice_index(self, _: &Buffer) -> ops::RangeFull {
        ..
    }
}

// Convenience for `Index` and `Position`
impl BufferIndex for usize {
    type Output = Cell;
    type Index = usize;

    #[inline]
    fn slice_index(self, _: &Buffer) -> usize {
        self
    }
}

impl<I: BufferIndex<Index = usize>> BufferIndex for ops::Range<I> {
    type Output = [Cell];
    type Index = ops::Range<usize>;

    #[inline]
    fn slice_index(self, buffer: &Buffer) -> ops::Range<usize> {
        let start = self.start.slice_index(buffer);
        let end = self.end.slice_index(buffer);
        start..end
    }
}

impl<I: BufferIndex<Index = usize>> BufferIndex for ops::RangeTo<I> {
    type Output = [Cell];
    type Index = ops::RangeTo<usize>;

    #[inline]
    fn slice_index(self, buffer: &Buffer) -> ops::RangeTo<usize> {
        let end = self.end.slice_index(buffer);
        ..end
    }
}


impl<I: BufferIndex<Index = usize>> BufferIndex for ops::RangeFrom<I> {
    type Output = [Cell];
    type Index = ops::RangeFrom<usize>;

    #[inline]
    fn slice_index(self, buffer: &Buffer) -> ops::RangeFrom<usize> {
        let start = self.start.slice_index(buffer);
        start..
    }
}

impl<I: BufferIndex<Index = usize>> BufferIndex for ops::RangeInclusive<I> {
    type Output = [Cell];
    type Index = ops::RangeInclusive<usize>;

    #[inline]
    fn slice_index(self, buffer: &Buffer) -> ops::RangeInclusive<usize> {
        let start = self.start().clone().slice_index(buffer);
        let end = self.end().clone().slice_index(buffer);
        start..=end
    }
}

impl<I: BufferIndex<Index = usize>> BufferIndex for ops::RangeToInclusive<I> {
    type Output = [Cell];
    type Index = ops::RangeToInclusive<usize>;

    #[inline]
    fn slice_index(self, buffer: &Buffer) -> ops::RangeToInclusive<usize> {
        let end = self.end.slice_index(buffer);
        ..=end
    }
}


impl<I: BufferIndex> Index<I> for Buffer {
    type Output = I::Output;
    fn index(&self, index: I) -> &Self::Output {
        BufferIndex::index(index, self)
    }
}

impl<I: BufferIndex> IndexMut<I> for Buffer {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        BufferIndex::index_mut(index, self)
    }
}
