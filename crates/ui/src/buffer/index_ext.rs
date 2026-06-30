use crate::{Buffer, BufferIndex, Cell};
use geometry::{Point, PointLike, Resolve, Row};
use std::{iter, ops};

pub enum IndexKind {
    Single(usize),
    Range(ops::Range<usize>),
}

pub trait BufferIndexExt: BufferIndex {
    fn kind(&self, context: &Buffer) -> IndexKind;

    fn len(&self, context: &Buffer) -> usize;
    fn start(&self, context: &Buffer) -> usize;
    fn end(&self, context: &Buffer) -> usize;

    fn is_empty(&self, context: &Buffer) -> bool {
        self.len(context) == 0
    }

    fn within(&self, context: &Buffer) -> bool;

    fn into_index(self, context: &Buffer) -> usize {
        self.start(context)
    }

    fn as_index(&self, context: &Buffer) -> usize {
        self.clone().into_index(context)
    }

    fn into_point(self, context: &Buffer) -> Point {
        let index = self.into_index(context) as u16;
        let width = context.width();
        Point::new((index % width), (index / width))
    }

    fn as_point(&self, context: &Buffer) -> Point {
        self.clone().into_point(context)
    }

    fn into_range(self, context: &Buffer) -> ops::Range<usize> {
        self.start(context)..self.end(context)
    }

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

    // @TODO
    // /// Returns an iterator over the output at this location, if in bounds.
    // fn iter(self, context: &Buffer) -> impl Iterator<Item=&Cell>;
    // 
    // /// Returns a mutable iterator over the output at this location, if in bounds.
    // fn iter_mut(self, context: &mut Buffer) -> impl Iterator<Item=&mut Cell>;
}

// --------------------------------------------------------------------------
// usize, Point, PointLike, Row – unchanged (except the trait bound is satisfied)
// --------------------------------------------------------------------------

impl BufferIndexExt for usize {
    #[inline]
    fn kind(&self, _: &Buffer) -> IndexKind {
        IndexKind::Single(*self)
    }

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

    #[inline]
    fn as_index(&self, context: &Buffer) -> usize {
        BufferIndexExt::into_index(*self, context)
    }

    #[inline]
    fn as_point(&self, context: &Buffer) -> Point {
        BufferIndexExt::into_point(*self, context)
    }

    #[inline]
    fn as_range(&self, context: &Buffer) -> ops::Range<usize> {
        BufferIndexExt::into_range(*self, context)
    }
}

impl BufferIndexExt for Point {
    #[inline]
    fn kind(&self, context: &Buffer) -> IndexKind {
        IndexKind::Single(self.into_index(context))
    }
    #[inline]
    fn len(&self, context: &Buffer) -> usize { 1 }
    #[inline]
    fn start(&self, context: &Buffer) -> usize { context.resolve(*self) }
    #[inline]
    fn end(&self, context: &Buffer) -> usize { self.start(context) + 1 }
    #[inline]
    fn within(&self, context: &Buffer) -> bool {
        self.x < context.width() && self.y < context.height()
    }
    #[inline]
    fn into_point(self, context: &Buffer) -> Point { self.into() }

    #[inline]
    fn as_index(&self, context: &Buffer) -> usize {
        BufferIndexExt::into_index(*self, context)
    }

    #[inline]
    fn as_point(&self, context: &Buffer) -> Point {
        BufferIndexExt::into_point(*self, context)
    }

    #[inline]
    fn as_range(&self, context: &Buffer) -> ops::Range<usize> {
        BufferIndexExt::into_range(*self, context)
    }
}

impl BufferIndexExt for PointLike {
    #[inline]
    fn kind(&self, context: &Buffer) -> IndexKind {
        IndexKind::Single(self.into_index(context))
    }
    #[inline]
    fn len(&self, context: &Buffer) -> usize { 1 }
    #[inline]
    fn start(&self, context: &Buffer) -> usize { context.resolve(*self) }
    #[inline]
    fn end(&self, context: &Buffer) -> usize { self.start(context) + 1 }
    #[inline]
    fn within(&self, context: &Buffer) -> bool {
        self.0 < context.width() && self.1 < context.height()
    }
    #[inline]
    fn into_point(self, context: &Buffer) -> Point { self.into() }

    #[inline]
    fn as_index(&self, context: &Buffer) -> usize {
        BufferIndexExt::into_index(*self, context)
    }

    #[inline]
    fn as_point(&self, context: &Buffer) -> Point {
        BufferIndexExt::into_point(*self, context)
    }

    #[inline]
    fn as_range(&self, context: &Buffer) -> ops::Range<usize> {
        BufferIndexExt::into_range(*self, context)
    }
}

impl BufferIndexExt for Row {
    #[inline]
    fn kind(&self, context: &Buffer) -> IndexKind {
        IndexKind::Range(self.into_range(context))
    }
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
    fn into_point(self, context: &Buffer) -> Point {
        Point::new(0, self.into_inner())
    }

    #[inline]
    fn as_index(&self, context: &Buffer) -> usize {
        BufferIndexExt::into_index(*self, context)
    }

    #[inline]
    fn as_point(&self, context: &Buffer) -> Point {
        BufferIndexExt::into_point(*self, context)
    }

    #[inline]
    fn as_range(&self, context: &Buffer) -> ops::Range<usize> {
        BufferIndexExt::into_range(*self, context)
    }
}

impl<T: BufferIndex<SliceIndex=usize> + Copy> BufferIndexExt for ops::Range<T> {
    #[inline]
    fn kind(&self, context: &Buffer) -> IndexKind {
        IndexKind::Range(self.start(context)..self.end(context))
    }
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
    fn kind(&self, context: &Buffer) -> IndexKind {
        IndexKind::Range(BufferIndexExt::start(self, context)..BufferIndexExt::end(self, context))
    }
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
    fn kind(&self, context: &Buffer) -> IndexKind {
        let end = self.end.as_slice_index(context);
        IndexKind::Range(0..end)
    }
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
    fn kind(&self, context: &Buffer) -> IndexKind {
        IndexKind::Range(self.start(context)..self.end(context))
    }
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
    fn kind(&self, context: &Buffer) -> IndexKind {
        IndexKind::Range(self.start(context)..self.end(context))
    }
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
    fn kind(&self, context: &Buffer) -> IndexKind {
        IndexKind::Range(0..context.len())
    }
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

    #[inline]
    fn as_index(&self, context: &Buffer) -> usize {
        BufferIndexExt::into_index(*self, context)
    }

    #[inline]
    fn as_point(&self, context: &Buffer) -> Point {
        BufferIndexExt::into_point(*self, context)
    }

    #[inline]
    fn as_range(&self, context: &Buffer) -> ops::Range<usize> {
        BufferIndexExt::into_range(*self, context)
    }
}

pub enum IndexIter {
    Single(iter::Once<usize>),
    Range(ops::Range<usize>),
}

impl Iterator for IndexIter {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<usize> {
        match self {
            Self::Single(once) => once.next(),
            Self::Range(range) => range.next(),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Single(once) => once.size_hint(),
            Self::Range(range) => range.size_hint(),
        }
    }
}

impl DoubleEndedIterator for IndexIter {
    #[inline]
    fn next_back(&mut self) -> Option<usize> {
        match self {
            Self::Single(once) => once.next_back(),
            Self::Range(range) => range.next_back(),
        }
    }
}

impl ExactSizeIterator for IndexIter {}

impl std::iter::FusedIterator for IndexIter {}