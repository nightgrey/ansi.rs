use std::iter::FusedIterator;
use std::marker::Destruct;
use std::ops::{*, Bound::*};
use crate::Position;

#[derive(Clone, Debug)]
pub struct SpatialRangeIter {
    range: Range<Position>,
    index: Position,
}

impl SpatialRangeIter {
    pub const fn new(range: &(impl [const] RangeBounds<Position> + [const] Destruct)) -> Self {
        let start = match range.start_bound() {
            Included(&p) => p,
            Excluded(&p) => Position {  row: p.row + 1, col: p.col + 1 },
            Unbounded => Position::MIN,
        };
        let end = match range.end_bound() {
            Included(&p) => Position {  row: p.row + 1, col: p.col + 1 },
            Excluded(&p) => p,
            Unbounded => Position::MAX,
        };
        Self { index: start, range: start..end }
    }
}

impl Iterator for SpatialRangeIter {
    type Item = Position;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index.row >= self.range.end.row {
            return None;
        }

        let mut pos = self.index;

        pos.col += 1;
        if pos.col >= self.range.end.col  {
            pos.col = self.range.start.col;
            pos.row += 1;
        }

        Some(std::mem::replace(&mut self.index, pos))
    }


    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.index >= self.range.end {
            return (0, Some(0));
        }
        let width = self.range.end.col.saturating_sub(self.range.start.col);

        let remaining = (self.range.end.row - self.index.row)
            .saturating_mul(width)
            .saturating_sub(self.index.col - self.range.start.col);

        (remaining, Some(remaining))
    }

    #[inline]
    fn count(self) -> usize {
        self.size_hint().0
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        if self.index >= self.range.end {
            return None;
        }

        Some(Position {
            row: self.range.end.row - 1,
            col: self.range.end.col  - 1,
        })
    }

    #[inline]
    fn min(mut self) -> Option<Self::Item>
    {
        self.next()
    }
}
impl FusedIterator for SpatialRangeIter {}
impl ExactSizeIterator for SpatialRangeIter {}

pub trait SpatialRange: Sized + RangeBounds<Position> {
    fn iter(&self) -> SpatialRangeIter {
        SpatialRangeIter::new(self)
    }
}

impl SpatialRange for Range<Position> {}
impl SpatialRange for RangeFrom<Position> {}
impl SpatialRange for RangeTo<Position> {}
impl SpatialRange for RangeInclusive<Position> {}
impl SpatialRange for RangeToInclusive<Position> {}
impl SpatialRange for (Bound<Position>, Bound<Position>) {}
