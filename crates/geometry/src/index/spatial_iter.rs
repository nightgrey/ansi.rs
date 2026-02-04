use std::iter::FusedIterator;
use std::ops::Range;
use crate::{Point, Position, Rect, Region, Size};

/// Iterator over indexes in a 2D spatial rectangular region
#[derive(Clone, Debug)]
pub struct SpatialIter {
    row: Range<usize>,
    col: Range<usize>,

    index: usize,
    end: usize,
}

impl SpatialIter {
    pub fn new(region_like: impl Into<Region>) -> Self {
        let region = region_like.into();

        let width = region.width();
        let height = region.height();

        let row = region.min.row..region.max.row;
        let col = region.min.col..region.max.col;

        Self {
            row,
            col,
            index: 0,
            end: height * width,
        }
    }

    pub const fn bounds(x: usize, y: usize, width: usize, height: usize) -> Self {
        let row = y..y + height;
        let col = x..x + width;

        Self {
            row,
            col,
            index: 0,
            end: height * width,
        }
    }
}

impl Iterator for SpatialIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.end {
            return None;
        }

        let coord = self.index;
        self.index += 1;
        Some(coord)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.index = self.index.saturating_add(n);
        self.next()
    }
}

impl DoubleEndedIterator for SpatialIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index >= self.end {
            return None;
        }

        self.end -= 1;
        Some(self.end)
    }
}

impl ExactSizeIterator for SpatialIter {
    fn len(&self) -> usize {
        self.end.saturating_sub(self.index)
    }
}
impl FusedIterator for SpatialIter {}

pub struct PositionsIter(SpatialIter);

impl PositionsIter {
    pub  fn new(region_like: impl Into<Region>) -> Self {
        Self(SpatialIter::new(region_like))
    }

    pub const fn bounds(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self(SpatialIter::bounds(x, y, width, height))
    }

    fn to_position(&self, index: usize) -> Position {
        let width = self.0.col.end - self.0.col.start;
        Position {
            row: self.0.row.start + index / width,
            col: self.0.col.start + index % width,
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
    pub  fn new(region_like: impl Into<Region>) -> Self {
        Self(SpatialIter::new(region_like))
    }

    pub const fn bounds(x: usize, y: usize, width: usize, height: usize) -> Self {
        Self(SpatialIter::bounds(x, y, width, height))
    }

    fn to_point(&self, index: usize) -> Point {
        let width = self.0.col.end - self.0.col.start;
        Point {
            x: self.0.col.start + index % width,
            y: self.0.row.start + index / width,
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
        let width = self.0.col.end - self.0.col.start;

        Some(Point {
            x: self.0.col.start + index % width,
            y: self.0.row.start + index / width,
        })
    }
}