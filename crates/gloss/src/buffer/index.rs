use geometry::{Point, PointLike, Row};
use std::ops;
use std::ops::Index;
use std::slice::SliceIndex;
use crate::{Buffer, Cell};

pub trait BufferIndex: Clone {
    type Output: ?Sized;
    type Index: SliceIndex<[Cell], Output = Self::Output>;

    #[inline]
    fn index_of(self, context: &Buffer) -> Self::Index;

    #[inline]
    fn get(self, context: &Buffer) -> Option<&Self::Output> {
        self.index_of(context).get(context.as_ref())
    }

    #[inline]
    fn get_mut(self, context: &mut Buffer) -> Option<&mut Self::Output> {
        self.index_of(context).get_mut(context.as_mut())
    }

    #[inline]
    unsafe fn get_unchecked(self, context: &Buffer) -> *const Self::Output {
        SliceIndex::get_unchecked(self.index_of(context), context.as_ref())
    }

    #[inline]
    unsafe fn get_unchecked_mut(self, context: &mut Buffer) -> *mut Self::Output {
        SliceIndex::get_unchecked_mut(self.index_of(context), context.as_mut())
    }
}

impl BufferIndex for Point {
    type Output = Cell;
    type Index = usize;

    #[inline]
    fn index_of(self, grid: &Buffer) -> usize {
        self.y * grid.width + self.x
    }
}

impl BufferIndex for PointLike {
    type Output = Cell;
    type Index = usize;

    #[inline]
    fn index_of(self, area: &Buffer) -> usize {
        self.1 * area.width + self.0
    }
}

impl BufferIndex for Row {
    type Output = [Cell];
    type Index = ops::Range<usize>;

    #[inline]
    fn index_of(self, area: &Buffer) -> ops::Range<usize> {
        let start = self.value() * area.width;
        start..start + area.width
    }
}

impl BufferIndex for ops::Range<Row> {
    type Output = [Cell];
    type Index = ops::Range<usize>;

    #[inline]
    fn index_of(self, area: &Buffer) -> ops::Range<usize> {
        self.start.value() * area.width..self.end.value() * area.width + area.width
    }
}

impl BufferIndex for ops::RangeTo<Row> {
    type Output = [Cell];
    type Index = ops::RangeTo<usize>;

    #[inline]
    fn index_of(self, area: &Buffer) -> ops::RangeTo<usize> {
        ..self.end.value() * area.width + area.width

    }
}

impl BufferIndex for ops::RangeFrom<Row> {
    type Output = [Cell];
    type Index = ops::RangeFrom<usize>;

    #[inline]
    fn index_of(self, area: &Buffer) -> ops::RangeFrom<usize> {
        self.start.value() * area.width..

    }
}

impl BufferIndex for ops::RangeInclusive<Row> {
    type Output = [Cell];
    type Index = ops::RangeInclusive<usize>;

    #[inline]
    fn index_of(self, area: &Buffer) -> ops::RangeInclusive<usize> {
        self.start().value() * area.width..=self.end().value() * area.width + area.width
    }
}

impl BufferIndex for ops::RangeToInclusive<Row> {
    type Output = [Cell];
    type Index = ops::RangeToInclusive<usize>;

    #[inline]
    fn index_of(self, area: &Buffer) -> ops::RangeToInclusive<usize> {
        ..=self.end.value() * area.width + area.width
    }
}

impl BufferIndex for ops::RangeFull {
    type Output = [Cell];
    type Index = ops::RangeFull;

    #[inline]
    fn index_of(self, _: &Buffer) -> ops::RangeFull {
        ..
    }
}

// Convenience for `Index` and `Position`
impl BufferIndex for usize {
    type Output = Cell;
    type Index = usize;

    #[inline]
    fn index_of(self, _: &Buffer) -> usize {
        self
    }
}

impl<I: BufferIndex<Index = usize>> BufferIndex for ops::Range<I> {
    type Output = [Cell];
    type Index = ops::Range<usize>;

    #[inline]
    fn index_of(self, grid: &Buffer) -> ops::Range<usize> {
        let start = self.start.index_of(grid);
        let end = self.end.index_of(grid);
        start..end
    }
}

impl<I: BufferIndex<Index = usize>> BufferIndex for ops::RangeTo<I> {
    type Output = [Cell];
    type Index = ops::RangeTo<usize>;

    #[inline]
    fn index_of(self, grid: &Buffer) -> ops::RangeTo<usize> {
        let end = self.end.index_of(grid);
        ..end
    }
}


impl<I: BufferIndex<Index = usize>> BufferIndex for ops::RangeFrom<I> {
    type Output = [Cell];
    type Index = ops::RangeFrom<usize>;

    #[inline]
    fn index_of(self, grid: &Buffer) -> ops::RangeFrom<usize> {
        let start = self.start.index_of(grid);
        start..
    }
}

impl<I: BufferIndex<Index = usize>> BufferIndex for ops::RangeInclusive<I> {
    type Output = [Cell];
    type Index = ops::RangeInclusive<usize>;

    #[inline]
    fn index_of(self, grid: &Buffer) -> ops::RangeInclusive<usize> {
        let start = self.start().clone().index_of(grid);
        let end = self.end().clone().index_of(grid);
        start..=end
    }
}

impl<I: BufferIndex<Index = usize>> BufferIndex for ops::RangeToInclusive<I> {
    type Output = [Cell];
    type Index = ops::RangeToInclusive<usize>;

    #[inline]
    fn index_of(self, grid: &Buffer) -> ops::RangeToInclusive<usize> {
        let end = self.end.index_of(grid);
        ..=end
    }
}

