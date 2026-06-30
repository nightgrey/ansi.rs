use crate::{Buffer, BufferIndex};
use geometry::{Point, PointLike, Resolve, Row};
use std::ops;

pub enum IndexKind {
    Single(usize),
    Range(ops::Range<usize>),
}

pub trait BufferIndexExt: Clone + BufferIndex {
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
    fn into_point(self, context: &Buffer) -> Point {
        let index = self.into_index(context) as u16;
        Point::new((index % context.width()), (index / context.width()))
    }
    fn into_range(self, context: &Buffer) -> ops::Range<usize> {
        self.start(context)..self.end(context)
    }
}

impl BufferIndexExt for usize {
    #[inline]
    fn kind(&self, _: &Buffer) -> IndexKind {
        IndexKind::Single(*self)
    }

    #[inline]
    fn len(&self, _: &Buffer) -> usize {
        1
    }

    #[inline]
    fn start(&self, _: &Buffer) -> usize {
        *self
    }

    #[inline]
    fn end(&self, _: &Buffer) -> usize {
        *self + 1
    }

    #[inline]
    fn within(&self, context: &Buffer) -> bool {
        *self < context.len()
    }
}

impl BufferIndexExt for Point {
    #[inline]
    fn kind(&self, context: &Buffer) -> IndexKind {
        IndexKind::Single(self.into_index(context))
    }

    #[inline]
    fn len(&self, context: &Buffer) -> usize {
        1
    }

    #[inline]
    fn start(&self, context: &Buffer) -> usize {
        context.resolve(*self)
    }

    #[inline]
    fn end(&self, context: &Buffer) -> usize {
        self.start(context) + 1
    }

    #[inline]
    fn within(&self, context: &Buffer) -> bool {
        self.x < context.width() && self.y < context.height()
    }

    #[inline]
    fn into_point(self, context: &Buffer) -> Point {
        self.into()
    }
}

impl BufferIndexExt for PointLike {
    #[inline]
    fn kind(&self, context: &Buffer) -> IndexKind {
        IndexKind::Single(self.into_index(context))
    }

    #[inline]
    fn len(&self, context: &Buffer) -> usize {
        1
    }

    #[inline]
    fn start(&self, context: &Buffer) -> usize {
        context.resolve(*self)
    }

    #[inline]
    fn end(&self, context: &Buffer) -> usize {
        self.start(context) + 1
    }

    #[inline]
    fn within(&self, context: &Buffer) -> bool {
        self.0 < context.width() && self.1 < context.height()
    }

    #[inline]
    fn into_point(self, context: &Buffer) -> Point {
        self.into()
    }
}
impl BufferIndexExt for Row {
    #[inline]
    fn kind(&self, context: &Buffer) -> IndexKind {
        IndexKind::Range(self.into_range(context))
    }

    #[inline]
    fn len(&self, context: &Buffer) -> usize {
        context.width() as usize
    }

    #[inline]
    fn start(&self, context: &Buffer) -> usize {
        context.resolve(*self)
    }

    #[inline]
    fn end(&self, context: &Buffer) -> usize {
        self.start(context) + context.width() as usize
    }

    #[inline]
    fn within(&self, context: &Buffer) -> bool {
        self.into_inner() < context.height()
    }

    #[inline]
    fn into_point(self, context: &Buffer) -> Point {
        Point::new(0, self.into_inner())
    }
}

impl<T: BufferIndexExt + BufferIndex<Index = usize>> BufferIndexExt for ops::Range<T> {
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
        self.start.clone().into_slice_index(context)
    }

    #[inline]
    fn end(&self, context: &Buffer) -> usize {
        self.end.clone().into_slice_index(context)
    }

    #[inline]
    fn within(&self, context: &Buffer) -> bool {
        self.start(context) < context.len() && self.end(context) <= context.len()
    }
}


impl<T: BufferIndexExt + BufferIndex<Index = usize>> BufferIndexExt for ops::RangeInclusive<T> {
    #[inline]
    fn kind(&self, context: &Buffer) -> IndexKind {
        IndexKind::Range(BufferIndexExt::start(self, context)..BufferIndexExt::end(self, context))
    }

    #[inline]
    fn len(&self, context: &Buffer) -> usize {
        BufferIndexExt::end(self, context) - BufferIndexExt::start(self, context)
    }

    #[inline]
    fn start(&self, context: &Buffer) -> usize {
       ops::RangeInclusive::start(self).clone().into_slice_index(context)
    }

    #[inline]
    fn end(&self, context: &Buffer) -> usize {
        ops::RangeInclusive::end(self).clone().into_slice_index(context) + 1
    }

    #[inline]
    fn within(&self, context: &Buffer) -> bool {
        BufferIndexExt::start(self, context) < context.len() && BufferIndexExt::end(self, context) <= context.len()
    }
}


impl<T: BufferIndexExt + BufferIndex<Index = usize>> BufferIndexExt for ops::RangeTo<T> {
    #[inline]
    fn kind(&self, context: &Buffer) -> IndexKind {
        let end = self.end.clone().into_slice_index(context);
        IndexKind::Range(0..end)
    }

    #[inline]
    fn len(&self, context: &Buffer) -> usize {
        self.end(context)
    }

    #[inline]
    fn start(&self, context: &Buffer) -> usize {
        0
    }

    #[inline]
    fn end(&self, context: &Buffer) -> usize {
        self.end.clone().into_slice_index(context)
    }

    #[inline]
    fn within(&self, context: &Buffer) -> bool {
        BufferIndexExt::end(self, context) <= context.len()
    }
    #[inline]
    fn into_point(self, context: &Buffer) -> Point {
        Point::ZERO
    }
}

impl<T: BufferIndexExt + BufferIndex<Index = usize>> BufferIndexExt for ops::RangeToInclusive<T> {
    #[inline]
    fn kind(&self, context: &Buffer) -> IndexKind {
        IndexKind::Range(0..BufferIndexExt::end(self, context))
    }

    #[inline]
    fn len(&self, context: &Buffer) -> usize {
        self.end(context)
    }

    #[inline]
    fn start(&self, context: &Buffer) -> usize {
        0
    }

    #[inline]
    fn end(&self, context: &Buffer) -> usize {
        self.end.clone().into_slice_index(context) + 1
    }

    #[inline]
    fn within(&self, context: &Buffer) -> bool {
        BufferIndexExt::end(self, context) <= context.len()
    }
    #[inline]
    fn into_point(self, context: &Buffer) -> Point {
        Point::ZERO
    }
}
impl<T: BufferIndexExt + BufferIndex<Index = usize>> BufferIndexExt for ops::RangeFrom<T> {
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
        self.start.clone().into_slice_index(context)
    }

    #[inline]
    fn end(&self, context: &Buffer) -> usize {
        context.len()
    }

    #[inline]
    fn within(&self, context: &Buffer) -> bool {
        BufferIndexExt::start(self, context) < context.len()
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
    fn start(&self, _: &Buffer) -> usize {
        0
    }

    #[inline]
    fn end(&self, context: &Buffer) -> usize {
        context.len()
    }

    #[inline]
    fn within(&self, context: &Buffer) -> bool {
        true
    }

    #[inline]
    fn into_point(self, context: &Buffer) -> Point {
        Point::ZERO
    }
}