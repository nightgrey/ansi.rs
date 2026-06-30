use crate::{Buffer, BufferIndex, Cell};
use geometry::{Point, PointLike, Resolve, Row};
use std::ops;

/// [`BufferIndex`] extension
pub trait BufferIndexExt: BufferIndex {
    /// Returns the number of elements covered by this index.
    fn len(&self, context: &Buffer) -> usize;

    /// Returns the first element covered by this index.
    fn start(&self, context: &Buffer) -> usize;

    /// Returns the last element covered by this index.
    fn end(&self, context: &Buffer) -> usize;

    /// Returns `true` if this index does not cover any elements.
    fn is_empty(&self, context: &Buffer) -> bool {
        self.len(context) == 0
    }

    /// Returns `true` if this index is within the bounds of the given context.
    fn within(&self, context: &Buffer) -> bool;

    /// Converts `self` into a `usize` index.
    fn into_index(self, context: &Buffer) -> usize {
        self.start(context)
    }

    /// Returns `self` as a `usize` index.
    fn as_index(&self, context: &Buffer) -> usize {
        self.clone().into_index(context)
    }

    /// Converts `self` into a [`Point`].
    fn into_point(self, context: &Buffer) -> Point {
        let index = self.into_index(context);
        let width = context.width();
        if width == 0 {
            return Point::ZERO;
        }
        let width = width as usize;
        // Compute in `usize`, narrow only the resulting coordinates.
        Point::new((index % width) as u16, (index / width) as u16)
    }

    /// Returns `self` as a [`Point`].
    fn as_point(&self, context: &Buffer) -> Point {
        self.clone().into_point(context)
    }

    /// Returns `self` as a [`Range<usize>`].
    fn into_range(self, context: &Buffer) -> ops::Range<usize> {
        self.start(context)..self.end(context)
    }

    /// Returns `self` as a [`Range<usize>`].
    fn as_range(&self, context: &Buffer) -> ops::Range<usize> {
        self.clone().into_range(context)
    }

    /// Returns a shared reference to the output at this location, if in
    /// bounds.
    ///
    /// Normalizes the output to a slice.
    fn get_many<'a>(&self, context: &'a Buffer) -> Option<&'a [Cell]> {
        let range = self.as_range(context);
        context.inner.get(range)
    }

    /// Returns a mutable reference to the output at this location, if in
    /// bounds.
    ///
    /// Normalizes the output to a slice.
    fn get_many_mut<'a>(&self, context: &'a mut Buffer) -> Option<&'a mut [Cell]> {
        let range = self.as_range(context);
        context.inner.get_mut(range)
    }

    /// Iterates the cells at this location. Empty if out of bounds.
    fn iter<'a>(&self, context: &'a Buffer) -> impl Iterator<Item = &'a Cell> {
        self.get_many(context).unwrap_or(&[]).iter()
    }

    /// Mutably iterates the cells at this location. Empty if out of bounds.
    fn iter_mut<'a>(&self, context: &'a mut Buffer) -> impl Iterator<Item = &'a mut Cell> {
        self.get_many_mut(context).unwrap_or(&mut []).iter_mut()
    }
}

// --------------------------------------------------------------------------
// usize, Point, PointLike, Row
// --------------------------------------------------------------------------

impl BufferIndexExt for usize {
    #[inline]
    fn len(&self, _: &Buffer) -> usize { 1 }

    #[inline]
    fn start(&self, _: &Buffer) -> usize { *self }

    #[inline]
    fn end(&self, _: &Buffer) -> usize { *self + 1 }

    #[inline]
    fn within(&self, context: &Buffer) -> bool {
        *self < context.len()
    }
}

impl BufferIndexExt for Point {
    #[inline]
    fn len(&self, _context: &Buffer) -> usize { 1 }
    #[inline]
    fn start(&self, context: &Buffer) -> usize { context.resolve(*self) }
    #[inline]
    fn end(&self, context: &Buffer) -> usize { self.start(context) + 1 }
    #[inline]
    fn within(&self, context: &Buffer) -> bool {
        self.x < context.width() && self.y < context.height()
    }
    #[inline]
    fn into_point(self, _context: &Buffer) -> Point { self }
}

impl BufferIndexExt for PointLike {
    #[inline]
    fn len(&self, _context: &Buffer) -> usize { 1 }
    #[inline]
    fn start(&self, context: &Buffer) -> usize { context.resolve(*self) }
    #[inline]
    fn end(&self, context: &Buffer) -> usize { self.start(context) + 1 }
    #[inline]
    fn within(&self, context: &Buffer) -> bool {
        self.0 < context.width() && self.1 < context.height()
    }
    #[inline]
    fn into_point(self, _context: &Buffer) -> Point { self.into() }
}

impl BufferIndexExt for Row {
    #[inline]
    fn len(&self, context: &Buffer) -> usize { context.width() as usize }
    #[inline]
    fn start(&self, context: &Buffer) -> usize { context.resolve(*self) }
    #[inline]
    fn end(&self, context: &Buffer) -> usize { self.start(context) + context.width() as usize }
    #[inline]
    fn within(&self, context: &Buffer) -> bool {
        self.into_inner() < context.height()
    }
    #[inline]
    fn into_point(self, _context: &Buffer) -> Point {
        Point::new(0, self.into_inner())
    }
}

impl<T: BufferIndex<SliceIndex=usize> + Copy> BufferIndexExt for ops::Range<T> {
    #[inline]
    fn len(&self, context: &Buffer) -> usize {
        self.end(context) - self.start(context)
    }
    #[inline]
    fn start(&self, context: &Buffer) -> usize {
        self.start.as_slice_index(context)
    }
    #[inline]
    fn end(&self, context: &Buffer) -> usize {
        self.end.as_slice_index(context)
    }
    #[inline]
    fn within(&self, context: &Buffer) -> bool {
        let s = self.start(context);
        let e = self.end(context);
        // start ≤ end  AND  start ≤ context.len()  AND  end ≤ context.len()
        s <= e && s <= context.len() && e <= context.len()
    }
}


impl<T: BufferIndex<SliceIndex=usize> + Copy> BufferIndexExt for ops::RangeInclusive<T> {
    #[inline]
    fn len(&self, context: &Buffer) -> usize {
        BufferIndexExt::end(self, context) - BufferIndexExt::start(self, context)   // relies on valid range
    }
    #[inline]
    fn start(&self, context: &Buffer) -> usize {
        ops::RangeInclusive::start(self).as_slice_index(context)
    }
    #[inline]
    fn end(&self, context: &Buffer) -> usize {
        let end_inclusive = ops::RangeInclusive::end(self).as_slice_index(context);
        debug_assert!(end_inclusive < usize::MAX, "inclusive end must not be usize::MAX");
        end_inclusive + 1
    }
    #[inline]
    fn within(&self, context: &Buffer) -> bool {
        let start = BufferIndexExt::start(self, context);
        let end = ops::RangeInclusive::end(self).as_slice_index(context);
        // start ≤ end_inclusive  AND  start < context.len()  AND  e_inclusive < context.len()
        start <= end && start < context.len() && end < context.len()
    }
}


impl<T: BufferIndex<SliceIndex=usize> + Copy> BufferIndexExt for ops::RangeTo<T> {
    #[inline]
    fn len(&self, context: &Buffer) -> usize {
        self.end(context)
    }
    #[inline]
    fn start(&self, _: &Buffer) -> usize { 0 }
    #[inline]
    fn end(&self, context: &Buffer) -> usize {
        self.end.as_slice_index(context)
    }
    #[inline]
    fn within(&self, context: &Buffer) -> bool {
        self.end(context) <= context.len()
    }
    #[inline]
    fn into_point(self, _: &Buffer) -> Point {
        Point::ZERO
    }
}

impl<T: BufferIndex<SliceIndex=usize> + Copy> BufferIndexExt for ops::RangeToInclusive<T> {
    #[inline]
    fn len(&self, context: &Buffer) -> usize {
        self.end(context)
    }
    #[inline]
    fn start(&self, _: &Buffer) -> usize { 0 }
    #[inline]
    fn end(&self, context: &Buffer) -> usize {
        let end_inclusive = self.end.as_slice_index(context);
        debug_assert!(end_inclusive < usize::MAX, "inclusive end must not be usize::MAX");
        end_inclusive + 1
    }
    #[inline]
    fn within(&self, context: &Buffer) -> bool {
        self.end(context) <= context.len()
    }
    #[inline]
    fn into_point(self, _: &Buffer) -> Point {
        Point::ZERO
    }
}
impl<T: BufferIndex<SliceIndex=usize> + Copy> BufferIndexExt for ops::RangeFrom<T> {
    #[inline]
    fn len(&self, context: &Buffer) -> usize {
        self.end(context) - self.start(context)
    }
    #[inline]
    fn start(&self, context: &Buffer) -> usize {
        self.start.as_slice_index(context)
    }
    #[inline]
    fn end(&self, context: &Buffer) -> usize {
        context.len()
    }
    #[inline]
    fn within(&self, context: &Buffer) -> bool {
        // start ≤ context.len()  (end is context.len(), so always valid if start is ok)
        self.start(context) <= context.len()
    }
}

impl BufferIndexExt for ops::RangeFull {
    #[inline]
    fn len(&self, context: &Buffer) -> usize {
        context.len()
    }
    #[inline]
    fn start(&self, _: &Buffer) -> usize { 0 }
    #[inline]
    fn end(&self, context: &Buffer) -> usize { context.len() }
    #[inline]
    fn within(&self, _: &Buffer) -> bool { true }
    #[inline]
    fn into_point(self, _: &Buffer) -> Point {
        Point::ZERO
    }
}
