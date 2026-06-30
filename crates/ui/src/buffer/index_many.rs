//! Multi-cell slice access for buffer indices.
//!
//! [`BufferIndexMany`] extends [`BufferIndex`] with methods that return
//! `&[Cell]` / `&mut [Cell]` slices — even for single-cell indices like `usize`
//! or `Point`, which produce a one-element slice. This normalisation
//! lets text-writing code (`Cells::write`, `Buffer::set_string`) work
//! uniformly with any index type via [`Buffer::get_many_mut`].
//!
//! The trait mirrors [`BufferIndex`]'s access pattern — checked, unchecked,
//! and panicking — but returns `&[Cell]` instead of `&Self::Output`.

use crate::{Buffer, BufferIndex, Cell};
use geometry::{Point, PointLike, Resolve, Row};
use std::ops;
use std::slice::SliceIndex;

/// Normalises any [`BufferIndex`] into `&[Cell]` / `&mut [Cell]` slice access.
///
/// Single-cell indices produce a 1-element slice; range indices pass through
/// their natural slice form. This uniformity is essential for text-drawing
/// primitives that write character-by-character into a mutable cell slice
/// regardless of how the caller chose to address the buffer.
pub trait BufferIndexMany: BufferIndex {
    type SliceIndexMany: SliceIndex<[Cell], Output = [Cell]>;

    /// Returns the [`SliceIndex`][`Self::SliceIndexMany`] for this location.
    ///
    /// This method does not perform any bounds checking.
    ///
    fn into_slice_index_many(self, context: &Buffer) -> Self::SliceIndexMany;

    /// Returns the [`SliceIndex`][`Self::SliceIndexMany`] for this location.
    ///
    /// This method does not perform any bounds checking.
    fn as_slice_index_many(&self, context: &Buffer) -> Self::SliceIndexMany {
        self.clone().into_slice_index_many(context)
    }

    /// Returns a shared reference to the output at this location, if in
    /// bounds.
    #[inline]
    fn get_many(self, context: &Buffer) -> Option<&[Cell]> {
        SliceIndex::get(self.into_slice_index_many(context), context)
    }

    /// Returns a mutable reference to the output at this location, if in
    /// bounds.
    #[inline]
    fn get_many_mut(self, context: &mut Buffer) -> Option<&mut [Cell]> {
        let index = self.into_slice_index_many(context);
        SliceIndex::get_mut(index, context)
    }

    /// Returns a pointer to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    #[inline]
    unsafe fn get_many_unchecked(self, context: &Buffer) -> *const [Cell] {
        unsafe { SliceIndex::get_unchecked(self.into_slice_index_many(context), context.as_ref()) }
    }
    /// Returns a mutable pointer to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    #[inline]
    unsafe fn get_many_unchecked_mut(self, context: &mut Buffer) -> *mut [Cell] {
        unsafe {
            SliceIndex::get_unchecked_mut(self.into_slice_index_many(context), context.as_mut())
        }
    }

    /// Returns a shared reference to the output at this location, panicking
    /// if out of bounds.
    #[track_caller]
    #[inline]
    fn index_many(self, context: &Buffer) -> &[Cell] {
        SliceIndex::index(self.into_slice_index_many(context), context)
    }

    /// Returns a mutable reference to the output at this location, panicking
    /// if out of bounds.
    #[track_caller]
    #[inline]
    fn index_many_mut(self, context: &mut Buffer) -> &mut [Cell] {
        SliceIndex::index_mut(self.into_slice_index_many(context), context)
    }
}

impl BufferIndexMany for usize {
    type SliceIndexMany = ops::Range<usize>;
    #[inline]
    fn into_slice_index_many(self, _: &Buffer) -> Self::SliceIndexMany {
        self..self + 1
    }

    #[inline]
    fn as_slice_index_many(&self, context: &Buffer) -> Self::SliceIndexMany {
        BufferIndexMany::into_slice_index_many(*self, context)
    }
}

impl BufferIndexMany for Point {
    type SliceIndexMany = ops::Range<usize>;
    #[inline]
    fn into_slice_index_many(self, context: &Buffer) -> Self::SliceIndexMany {
        let index = self.into_slice_index(context);
        index..index + 1
    }

    #[inline]
    fn as_slice_index_many(&self, context: &Buffer) -> Self::SliceIndexMany {
        BufferIndexMany::into_slice_index_many(*self, context)
    }
}

impl BufferIndexMany for PointLike {
    type SliceIndexMany = ops::Range<usize>;

    #[inline]
    fn into_slice_index_many(self, context: &Buffer) -> Self::SliceIndexMany {
        let index = self.into_slice_index(context);
        index..index + 1
    }

    #[inline]
    fn as_slice_index_many(&self, context: &Buffer) -> Self::SliceIndexMany {
        BufferIndexMany::into_slice_index_many(*self, context)
    }
}

impl BufferIndexMany for PointLike<usize> {
    type SliceIndexMany = ops::Range<usize>;

    #[inline]
    fn into_slice_index_many(self, context: &Buffer) -> ops::Range<usize> {
        let index = self.into_slice_index(context);
        index..index + 1
    }

    #[inline]
    fn as_slice_index_many(&self, context: &Buffer) -> Self::SliceIndexMany {
        BufferIndexMany::into_slice_index_many(*self, context)
    }
}

impl BufferIndexMany for Row {
    type SliceIndexMany = ops::Range<usize>;

    #[inline]
    fn into_slice_index_many(self, context: &Buffer) -> ops::Range<usize> {
        self.into_slice_index(context)
    }
}

impl<T: BufferIndex<SliceIndex = usize> + Copy> BufferIndexMany for ops::Range<T> {
    type SliceIndexMany = ops::Range<usize>;

    #[inline]
    fn into_slice_index_many(self, context: &Buffer) -> ops::Range<usize> {
        self.into_slice_index(context)
    }
}

impl<T: BufferIndex<SliceIndex = usize> + Copy> BufferIndexMany for ops::RangeInclusive<T> {
    type SliceIndexMany = ops::RangeInclusive<usize>;

    #[inline]
    fn into_slice_index_many(self, context: &Buffer) -> ops::RangeInclusive<usize> {
        self.into_slice_index(context)
    }
}

impl<T: BufferIndex<SliceIndex = usize> + Copy> BufferIndexMany for ops::RangeTo<T> {
    type SliceIndexMany = ops::RangeTo<usize>;

    #[inline]
    fn into_slice_index_many(self, context: &Buffer) -> ops::RangeTo<usize> {
        self.into_slice_index(context)
    }
}

impl<T: BufferIndex<SliceIndex = usize> + Copy> BufferIndexMany for ops::RangeToInclusive<T> {
    type SliceIndexMany = ops::RangeToInclusive<usize>;

    #[inline]
    fn into_slice_index_many(self, context: &Buffer) -> ops::RangeToInclusive<usize> {
        self.into_slice_index(context)
    }
}
impl<T: BufferIndex<SliceIndex = usize> + Copy> BufferIndexMany for ops::RangeFrom<T> {
    type SliceIndexMany = ops::RangeFrom<usize>;

    #[inline]
    fn into_slice_index_many(self, context: &Buffer) -> ops::RangeFrom<usize> {
        self.into_slice_index(context)
    }
}

impl BufferIndexMany for ops::RangeFull {
    type SliceIndexMany = ops::RangeFull;

    #[inline]
    fn into_slice_index_many(self, context: &Buffer) -> ops::RangeFull {
        self.into_slice_index(context)
    }
}
