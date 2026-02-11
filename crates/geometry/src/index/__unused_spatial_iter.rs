use crate::{Bounds, Point, Position, Rect, Size};
use std::iter::FusedIterator;
use std::ops::Range;

/// Iterator over indexes in a 2D spatial rectangular region
#[derive(Clone, Debug)]
pub struct SpatialIter {
    rows: Range<usize>,
    cols: Range<usize>,

    start: usize,
    end: usize,
}

impl SpatialIter {
    pub fn new(bounds_like: impl Into<Bounds>) -> Self {
        let region = bounds_like.into();

        let width = region.width();
        let height = region.height();

        let row = region.min.row..region.max.row;
        let col = region.min.col..region.max.col;

        Self {
            rows: row,
            cols: col,
            start: 0,
            end: height * width,
        }
    }

    pub const fn bounds(x: usize, y: usize, width: usize, height: usize) -> Self {
        let row = y..y + height;
        let col = x..x + width;

        Self {
            rows: row,
            cols: col,
            start: 0,
            end: height * width,
        }
    }

    pub const fn positions(self) -> PositionsIter {
        PositionsIter(self)
    }

    pub const fn points(self) -> PointsIter {
        PointsIter(self)
    }
}

impl Iterator for SpatialIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            return None;
        }

        let coord = self.start;
        self.start += 1;
        Some(coord)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.start = self.start.saturating_add(n);
        self.next()
    }
}

impl DoubleEndedIterator for SpatialIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            return None;
        }

        self.end -= 1;
        Some(self.end)
    }
}

impl ExactSizeIterator for SpatialIter {
    fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }
}
impl FusedIterator for SpatialIter {}

pub struct PositionsIter(SpatialIter);

impl PositionsIter {
    pub fn new(region_like: impl Into<Bounds>) -> Self {
        Self(SpatialIter::new(region_like))
    }

    pub const fn bounds(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self(SpatialIter::bounds(x, y, width, height))
    }

    fn to_position(&self, index: usize) -> Position {
        let width = self.0.cols.end - self.0.cols.start;
        Position {
            row: self.0.rows.start + index / width,
            col: self.0.cols.start + index % width,
        }
    }
}
impl Iterator for PositionsIter {
    type Item = Position;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|index| self.to_position(index))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth(n).map(|index| self.to_position(index))
    }
}
impl DoubleEndedIterator for PositionsIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|index| self.to_position(index))
    }
}
impl ExactSizeIterator for PositionsIter {
    fn len(&self) -> usize {
        self.0.len()
    }
}

pub struct PointsIter(SpatialIter);

impl PointsIter {
    pub fn new(region_like: impl Into<Bounds>) -> Self {
        Self(SpatialIter::new(region_like))
    }

    pub const fn bounds(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self(SpatialIter::bounds(x, y, width, height))
    }

    fn to_point(&self, index: usize) -> Point {
        let width = self.0.cols.end - self.0.cols.start;
        Point {
            x: self.0.cols.start + index % width,
            y: self.0.rows.start + index / width,
        }
    }
}

impl Iterator for PointsIter {
    type Item = Point;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|index| self.to_point(index))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth(n).map(|index| self.to_point(index))
    }
}
impl DoubleEndedIterator for PointsIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        let index = self.0.next_back()?;
        let width = self.0.cols.end - self.0.cols.start;

        Some(Point {
            x: self.0.cols.start + index % width,
            y: self.0.rows.start + index / width,
        })
    }
}
