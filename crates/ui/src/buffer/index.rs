use crate::{Buffer, Cell};
use geometry::{Point, PointLike, Resolve, Row};
use std::ops;
use std::slice::SliceIndex;

pub trait BufferIndex: Clone {
    type Output: ?Sized;
    type SliceIndex: SliceIndex<[Cell], Output = Self::Output>;

    /// Returns the [`SliceIndex`][`Self::SliceIndex`] for this location.
    ///
    /// This method does not perform any bounds checking.
    fn into_slice_index(self, context: &Buffer) -> Self::SliceIndex;

    /// Returns `self` as a [`SliceIndex`][`Self::SliceIndex`].
    ///
    /// This method does not perform any bounds checking.
    fn as_slice_index(&self, context: &Buffer) -> Self::SliceIndex {
        self.clone().into_slice_index(context)
    }

    /// Returns a shared reference to the output at this location, if in
    /// bounds.
    #[inline]
    fn get(self, context: &Buffer) -> Option<&Self::Output> {
        SliceIndex::get(self.into_slice_index(context), context)
    }

    /// Returns a mutable reference to the output at this location, if in
    /// bounds.
    #[inline]
    fn get_mut(self, context: &mut Buffer) -> Option<&mut Self::Output> {
        SliceIndex::get_mut(self.into_slice_index(context), context)
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
        unsafe { SliceIndex::get_unchecked(self.into_slice_index(context), context.as_ref()) }
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
        unsafe { SliceIndex::get_unchecked_mut(self.into_slice_index(context), context.as_mut()) }
    }

    /// Returns a shared reference to the output at this location, panicking
    /// if out of bounds.
    #[track_caller]
    #[inline]
    fn index(self, context: &Buffer) -> &Self::Output {
        SliceIndex::index(self.into_slice_index(context), context)
    }

    /// Returns a mutable reference to the output at this location, panicking
    /// if out of bounds.
    #[track_caller]
    #[inline]
    fn index_mut(self, context: &mut Buffer) -> &mut Self::Output {
        SliceIndex::index_mut(self.into_slice_index(context), context)
    }
}

impl BufferIndex for usize {
    type Output = Cell;
    type SliceIndex = usize;

    #[inline]
    fn into_slice_index(self, _: &Buffer) -> usize {
        self
    }

    #[inline]
    fn as_slice_index(&self, context: &Buffer) -> usize {
        BufferIndex::into_slice_index(*self, context)
    }
}

impl BufferIndex for Point {
    type Output = Cell;
    type SliceIndex = usize;

    #[inline]
    fn into_slice_index(self, context: &Buffer) -> usize {
        context.resolve(self)
    }

    #[inline]
    fn as_slice_index(&self, context: &Buffer) -> usize {
        BufferIndex::into_slice_index(*self, context)
    }
}

impl BufferIndex for PointLike {
    type Output = Cell;
    type SliceIndex = usize;

    #[inline]
    fn into_slice_index(self, context: &Buffer) -> usize {
        context.resolve(self)
    }

    #[inline]
    fn as_slice_index(&self, context: &Buffer) -> usize {
        BufferIndex::into_slice_index(*self, context)
    }
}

impl BufferIndex for PointLike<usize> {
    type Output = Cell;
    type SliceIndex = usize;

    #[inline]
    fn into_slice_index(self, context: &Buffer) -> usize {
        context.resolve(self)
    }

    #[inline]
    fn as_slice_index(&self, context: &Buffer) -> usize {
        BufferIndex::into_slice_index(*self, context)
    }
}

impl BufferIndex for Row {
    type Output = [Cell];
    type SliceIndex = ops::Range<usize>;

    #[inline]
    fn into_slice_index(self, context: &Buffer) -> ops::Range<usize> {
        context.resolve(self)
    }

    #[inline]
    fn as_slice_index(&self, context: &Buffer) -> ops::Range<usize> {
        BufferIndex::into_slice_index(*self, context)
    }
}

impl<T: BufferIndex<SliceIndex= usize>> BufferIndex for ops::Range<T> {
    type Output = [Cell];
    type SliceIndex = ops::Range<usize>;

    #[inline]
    fn into_slice_index(self, context: &Buffer) -> ops::Range<usize> {
        let start = self.start.into_slice_index(context);
        let end = self.end.into_slice_index(context);
        start..end
    }
}


impl<T: BufferIndex<SliceIndex= usize>> BufferIndex for ops::RangeInclusive<T> {
    type Output = [Cell];
    type SliceIndex = ops::RangeInclusive<usize>;

    #[inline]
    fn into_slice_index(self, context: &Buffer) -> ops::RangeInclusive<usize> {
        let start = self.start().as_slice_index(context);
        let end = self.end().as_slice_index(context);
        start..=end
    }
}


impl<T: BufferIndex<SliceIndex= usize>> BufferIndex for ops::RangeTo<T> {
    type Output = [Cell];
    type SliceIndex = ops::RangeTo<usize>;

    #[inline]
    fn into_slice_index(self, context: &Buffer) -> ops::RangeTo<usize> {
        let end = self.end.into_slice_index(context);
        ..end
    }
}

impl<T: BufferIndex<SliceIndex= usize>> BufferIndex for ops::RangeToInclusive<T> {
    type Output = [Cell];
    type SliceIndex = ops::RangeToInclusive<usize>;

    #[inline]
    fn into_slice_index(self, context: &Buffer) -> ops::RangeToInclusive<usize> {
        let end = self.end.into_slice_index(context);
        ..=end
    }
}
impl<T: BufferIndex<SliceIndex= usize>> BufferIndex for ops::RangeFrom<T> {
    type Output = [Cell];
    type SliceIndex = ops::RangeFrom<usize>;

    #[inline]
    fn into_slice_index(self, context: &Buffer) -> ops::RangeFrom<usize> {
        let start = self.start.into_slice_index(context);
        start..
    }
}

impl BufferIndex for ops::RangeFull {
    type Output = [Cell];
    type SliceIndex = ops::RangeFull;

    #[inline]
    fn into_slice_index(self, _: &Buffer) -> ops::RangeFull {
        ..
    }

    #[inline]
    fn as_slice_index(&self, context: &Buffer) -> ops::RangeFull {
        BufferIndex::into_slice_index(*self, &context)
    }
}

impl<I: BufferIndex> ops::Index<I> for Buffer {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        BufferIndex::index(index, self)
    }
}

impl<I: BufferIndex> ops::IndexMut<I> for Buffer {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        BufferIndex::index_mut(index, self)
    }
}
