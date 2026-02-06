use std::{iter, ops};
use std::ops::{*};
use geometry::{Bounds, Col, Point, Position, Rect, Row, SpatialIter};
use crate::Buffer;

/// Unified access to buffer ranges, supporting both linear and spatial indexing
pub trait BufferRange {
    type Iter: Iterator<Item = usize>;

    /// Spatial bounding box (always available)
    fn spatial(&self, buffer: &Buffer) -> Bounds;

    /// Contiguous linear range, if applicable
    /// - Returns `Some(range)` for linear ranges and full-row spatial regions
    /// - Returns `None` for partial-row regions (non-contiguous in memory)
    fn linear(&self, buffer: &Buffer) -> Option<Range<usize>>;

    /// Total cell count
    fn len(&self, buffer: &Buffer) -> usize;

    /// Check if empty
    fn is_empty(&self, buffer: &Buffer) -> bool {
        self.len(buffer) == 0
    }

    /// Iterate over linear indices (row-major order)
    fn iter(&self, buffer: &Buffer) -> Self::Iter;

    fn positions(&self, buffer: &Buffer) -> impl Iterator<Item = Position> {
        self.iter(buffer).map(|i| buffer.position_of(i))
    }
}

// One cell


// ============================================================================
// Linear usize Ranges
// ============================================================================
impl BufferRange for usize {
    type Iter = iter::Once<usize>;
    fn spatial(&self, buffer: &Buffer) -> Bounds {
        Bounds::new(
            buffer.position_of(*self),
            buffer.position_of(*self + 1),
        )
    }

    fn linear(&self, _buffer: &Buffer) -> Option<Range<usize>> {
        Some(*self..*self + 1)
    }

    fn len(&self, _buffer: &Buffer) -> usize {
        1
    }

    fn iter(&self, _buffer: &Buffer) -> Self::Iter {
        iter::once(*self)
    }
}

impl BufferRange for Range<usize> {
    type Iter = Self;
    fn spatial(&self, buffer: &Buffer) -> Bounds {
        Bounds::new(
            buffer.position_of(self.start),
            buffer.position_of(self.end),
        )
    }

    fn linear(&self, _buffer: &Buffer) -> Option<Range<usize>> {
        Some(self.clone())
    }

    fn len(&self, _buffer: &Buffer) -> usize {
        self.end.saturating_sub(self.start)
    }

    fn iter(&self, _buffer: &Buffer) -> Self::Iter {
        self.clone().into_iter()
    }
}

impl BufferRange for RangeInclusive<usize> {
    type Iter = Self;
    fn spatial(&self, buffer: &Buffer) -> Bounds {
        Bounds::new(
            buffer.position_of(*self.start()),
            buffer.position_of(*self.end() + 1),
        )
    }

    fn linear(&self, _buffer: &Buffer) -> Option<Range<usize>> {
        Some(*self.start()..*self.end() + 1)
    }

    fn len(&self, _buffer: &Buffer) -> usize {
        self.end().saturating_sub(*self.start()) + 1
    }

    fn iter(&self, _buffer: &Buffer) -> Self::Iter {
        self.clone().into_iter()
    }
}

impl BufferRange for RangeTo<usize> {
    type Iter = Range<usize>;
    fn spatial(&self, buffer: &Buffer) -> Bounds {
        Bounds::new(
            Position::ZERO,
            buffer.position_of(self.end),
        )
    }

    fn linear(&self, _buffer: &Buffer) -> Option<Range<usize>> {
        Some(0..self.end)
    }

    fn len(&self, _buffer: &Buffer) -> usize {
        self.end
    }

    fn iter(&self, _buffer: &Buffer) -> Self::Iter {
        (0..self.end).into_iter()
    }
}

impl BufferRange for RangeToInclusive<usize> {
    type Iter = Range<usize>;
    fn spatial(&self, buffer: &Buffer) -> Bounds {
        Bounds::new(
            Position::ZERO,
            buffer.position_of(self.end + 1),
        )
    }

    fn linear(&self, _buffer: &Buffer) -> Option<Range<usize>> {
        Some(0..self.end + 1)
    }

    fn len(&self, _buffer: &Buffer) -> usize {
        self.end + 1
    }

    fn iter(&self, _buffer: &Buffer) -> Self::Iter {
        0..self.end + 1
    }
}

impl BufferRange for RangeFrom<usize> {
    type Iter = Range<usize>;
    fn spatial(&self, buffer: &Buffer) -> Bounds {
        Bounds::new(
            buffer.position_of(self.start),
            buffer.position_of(buffer.len()),
        )
    }

    fn linear(&self, buffer: &Buffer) -> Option<Range<usize>> {
        Some(self.start..buffer.len())
    }

    fn len(&self, buffer: &Buffer) -> usize {
        buffer.len().saturating_sub(self.start)
    }

    fn iter(&self, buffer: &Buffer) -> Self::Iter {
        self.start..buffer.len()
    }
}

impl BufferRange for RangeFull {
    type Iter = Range<usize>;
    fn spatial(&self, buffer: &Buffer) -> Bounds {
        Bounds::new(
            Position::ZERO,
            Position::new(buffer.height, buffer.width),
        )
    }

    fn linear(&self, buffer: &Buffer) -> Option<Range<usize>> {
        Some(0..buffer.len())
    }

    fn len(&self, buffer: &Buffer) -> usize {
        buffer.len()
    }

    fn iter(&self, buffer: &Buffer) -> Self::Iter {
        0..buffer.len()
    }
}

// ============================================================================
// Bounds (Spatial)
// ============================================================================

impl BufferRange for Bounds {
    type Iter = SpatialIter;
    fn spatial(&self, _buffer: &Buffer) -> Bounds {
        *self
    }

    fn linear(&self, buffer: &Buffer) -> Option<Range<usize>> {
        // Only contiguous for single row or full-width multi-row
        if self.height() == 1 {
            let start = self.min.row * buffer.width + self.min.col;
            Some(start..start + self.width())
        } else if self.width() == buffer.width {
            let start = self.min.row * buffer.width;
            Some(start..start + self.area())
        } else {
            None
        }
    }

    fn len(&self, _buffer: &Buffer) -> usize {
        self.area()
    }

    fn iter(&self, _buffer: &Buffer) -> Self::Iter {
        SpatialIter::new(*self)
    }
}
impl BufferRange for Position {
    type Iter = iter::Once<usize>;
    fn spatial(&self, _buffer: &Buffer) -> Bounds {
        Bounds::new(*self, *self + 1)
    }

    fn linear(&self, buffer: &Buffer) -> Option<Range<usize>> {
        Some(self.row * buffer.width + self.col..(self.row + 1) * buffer.width + self.col + 1)
    }

    fn len(&self, _buffer: &Buffer) -> usize {
        1
    }

    fn iter(&self, buffer: &Buffer) -> Self::Iter {
        iter::once(self.row * buffer.width + self.col)
    }
}

// ============================================================================
// Position Ranges
// ============================================================================

impl BufferRange for Range<Position> {
    type Iter = SpatialIter;
    fn spatial(&self, _buffer: &Buffer) -> Bounds {
        Bounds::new(self.start, self.end)
    }

    fn linear(&self, buffer: &Buffer) -> Option<Range<usize>> {
        Bounds::new(self.start, self.end).linear(buffer)
    }

    fn len(&self, _buffer: &Buffer) -> usize {
        Bounds::new(self.start, self.end).area()
    }

    fn iter(&self, buffer: &Buffer) -> Self::Iter {
        SpatialIter::new(self.spatial(buffer))
    }
}

impl BufferRange for RangeInclusive<Position> {
    type Iter = SpatialIter;

    fn spatial(&self, _buffer: &Buffer) -> Bounds {
        Bounds::new(*self.start(), Position::new(self.end().row + 1, self.end().col + 1))
    }

    fn linear(&self, buffer: &Buffer) -> Option<Range<usize>> {
        self.spatial(buffer).linear(buffer)
    }

    fn len(&self, _buffer: &Buffer) -> usize {
        self.spatial(&Buffer::ZERO).area()
    }

    fn iter(&self, buffer: &Buffer) -> Self::Iter {
        SpatialIter::new(self.spatial(buffer))
    }
}

impl BufferRange for RangeTo<Position> {
    type Iter = SpatialIter;
    fn spatial(&self, _buffer: &Buffer) -> Bounds {
        Bounds::new(Position::ZERO, self.end)
    }

    fn linear(&self, buffer: &Buffer) -> Option<Range<usize>> {
        self.spatial(buffer).linear(buffer)
    }

    fn len(&self, _buffer: &Buffer) -> usize {
        self.spatial(&Buffer::ZERO).area()
    }

    fn iter(&self, buffer: &Buffer) -> Self::Iter {
        SpatialIter::new(self.spatial(buffer))
    }
}

impl BufferRange for RangeToInclusive<Position> {
    type Iter = SpatialIter;
    fn spatial(&self, _buffer: &Buffer) -> Bounds {
        Bounds::new(Position::ZERO, Position::new(self.end.row + 1, self.end.col + 1))
    }

    fn linear(&self, buffer: &Buffer) -> Option<Range<usize>> {
        self.spatial(buffer).linear(buffer)
    }

    fn len(&self, _buffer: &Buffer) -> usize {
        self.spatial(&Buffer::ZERO).area()
    }

    fn iter(&self, buffer: &Buffer) -> Self::Iter {
        SpatialIter::new(self.spatial(buffer))
    }
}

impl BufferRange for RangeFrom<Position> {
    type Iter = SpatialIter;
    fn spatial(&self, buffer: &Buffer) -> Bounds {
        Bounds::new(self.start, Position::new(buffer.height, buffer.width))
    }

    fn linear(&self, buffer: &Buffer) -> Option<Range<usize>> {
        self.spatial(buffer).linear(buffer)
    }

    fn len(&self, buffer: &Buffer) -> usize {
        self.spatial(buffer).area()
    }

    fn iter(&self, buffer: &Buffer) -> Self::Iter {
        SpatialIter::new(self.spatial(buffer))
    }
}

// ============================================================================
// Rect (Point-based)
// ============================================================================

impl BufferRange for Rect {
    type Iter = SpatialIter;

    fn spatial(&self, _buffer: &Buffer) -> Bounds {
        Bounds::from(*self)
    }

    fn linear(&self, buffer: &Buffer) -> Option<Range<usize>> {
        self.spatial(buffer).linear(buffer)
    }

    fn len(&self, _buffer: &Buffer) -> usize {
        self.area()
    }

    fn iter(&self, buffer: &Buffer) -> Self::Iter {
        SpatialIter::new(Bounds::from(*self))
    }
}

// ============================================================================
// Point Ranges
// ============================================================================
impl BufferRange for Point {
    type Iter = iter::Once<usize>;
    fn spatial(&self, buffer: &Buffer) -> Bounds {
        Bounds::new(
            Position::new(self.y, self.x),
            Position::new(self.y + 1, self.x + 1),
        )
    }

    fn linear(&self, buffer: &Buffer) -> Option<Range<usize>> {
        Some(self.y * buffer.width + self.x..(self.y + 1) * buffer.width + self.x + 1)
    }

    fn len(&self, _buffer: &Buffer) -> usize {
        1
    }

    fn iter(&self, buffer: &Buffer) -> Self::Iter {
        iter::once(self.y * buffer.width + self.x)
    }
}

impl BufferRange for Range<Point> {
    type Iter = SpatialIter;

    fn spatial(&self, _buffer: &Buffer) -> Bounds {
        Bounds::new(self.start.into(), self.end.into())
    }

    fn linear(&self, buffer: &Buffer) -> Option<Range<usize>> {
        self.spatial(buffer).linear(buffer)
    }

    fn len(&self, _buffer: &Buffer) -> usize {
        self.spatial(&Buffer::ZERO).area()
    }

    fn iter(&self, buffer: &Buffer) -> Self::Iter {
        SpatialIter::new(self.spatial(buffer))
    }
}

impl BufferRange for RangeInclusive<Point> {
    type Iter = SpatialIter;

    fn spatial(&self, _buffer: &Buffer) -> Bounds {
        Bounds::new(
            (*self.start()).into(),
            Position::new(self.end().y + 1, self.end().x + 1)
        )
    }

    fn linear(&self, buffer: &Buffer) -> Option<Range<usize>> {
        self.spatial(buffer).linear(buffer)
    }

    fn len(&self, _buffer: &Buffer) -> usize {
        self.spatial(&Buffer::ZERO).area()
    }

    fn iter(&self, buffer: &Buffer) -> Self::Iter {
        SpatialIter::new(self.spatial(buffer))
    }
}

impl BufferRange for RangeTo<Point> {
    type Iter = SpatialIter;

    fn spatial(&self, _buffer: &Buffer) -> Bounds {
        Bounds::new(Position::ZERO, self.end.into())
    }

    fn linear(&self, buffer: &Buffer) -> Option<Range<usize>> {
        self.spatial(buffer).linear(buffer)
    }

    fn len(&self, _buffer: &Buffer) -> usize {
        self.spatial(&Buffer::ZERO).area()
    }

    fn iter(&self, buffer: &Buffer) -> Self::Iter {
        SpatialIter::new(self.spatial(buffer))
    }
}

impl BufferRange for RangeToInclusive<Point> {
    type Iter = SpatialIter;
    fn spatial(&self, _buffer: &Buffer) -> Bounds {
        Bounds::new(Position::ZERO, Position::new(self.end.y + 1, self.end.x + 1))
    }

    fn linear(&self, buffer: &Buffer) -> Option<Range<usize>> {
        self.spatial(buffer).linear(buffer)
    }

    fn len(&self, _buffer: &Buffer) -> usize {
        self.spatial(&Buffer::ZERO).area()
    }

    fn iter(&self, buffer: &Buffer) -> Self::Iter {
        SpatialIter::new(self.spatial(buffer))
    }
}

impl BufferRange for RangeFrom<Point> {
    type Iter = SpatialIter;
    fn spatial(&self, buffer: &Buffer) -> Bounds {
        Bounds::new(self.start.into(), Position::new(buffer.height, buffer.width))
    }

    fn linear(&self, buffer: &Buffer) -> Option<Range<usize>> {
        self.spatial(buffer).linear(buffer)
    }

    fn len(&self, buffer: &Buffer) -> usize {
        self.spatial(buffer).area()
    }

    fn iter(&self, buffer: &Buffer) -> Self::Iter {
        SpatialIter::new(self.spatial(buffer))
    }
}

// ============================================================================
// Row
// ============================================================================

impl BufferRange for Row {
    type Iter = Range<usize>;
    fn spatial(&self, buffer: &Buffer) -> Bounds {
        let row = self.0;
        let start = Position::new(row, 0);
        let end = Position::new((row + 1).min(buffer.height), buffer.width);
        Bounds::new(start, end)
    }

    fn linear(&self, buffer: &Buffer) -> Option<Range<usize>> {
        let start = self.0 * buffer.width;
        Some(start..start + buffer.width)
    }

    fn len(&self, buffer: &Buffer) -> usize {
        if self.0 < buffer.height { buffer.width } else { 0 }
    }

    fn iter(&self, buffer: &Buffer) -> Self::Iter {
        let start = self.0 * buffer.width;
        start..start + buffer.width
    }
}

impl BufferRange for Range<Row> {
    type Iter = Range<usize>;
    fn spatial(&self, buffer: &Buffer) -> Bounds {
        let start_row = self.start.0;
        let end_row = self.end.0.min(buffer.height);
        Bounds::new(
            Position::new(start_row, 0),
            Position::new(end_row, buffer.width),
        )
    }

    fn linear(&self, buffer: &Buffer) -> Option<Range<usize>> {
        let start = self.start.0 * buffer.width;
        let end = self.end.0.min(buffer.height) * buffer.width;
        Some(start..end)
    }

    fn len(&self, buffer: &Buffer) -> usize {
        let rows = self.end.0.saturating_sub(self.start.0);
        rows.min(buffer.height.saturating_sub(self.start.0)) * buffer.width
    }

    fn iter(&self, buffer: &Buffer) -> Self::Iter {
        let start = self.start.0 * buffer.width;
        let end = self.end.0.min(buffer.height) * buffer.width;
        start..end
    }
}

impl BufferRange for RangeInclusive<Row> {
    type Iter = Range<usize>;
    fn spatial(&self, buffer: &Buffer) -> Bounds {
        let start_row = self.start().0;
        let end_row = (self.end().0 + 1).min(buffer.height);
        Bounds::new(
            Position::new(start_row, 0),
            Position::new(end_row, buffer.width),
        )
    }

    fn linear(&self, buffer: &Buffer) -> Option<Range<usize>> {
        let start = self.start().0 * buffer.width;
        let end = (self.end().0 + 1).min(buffer.height) * buffer.width;
        Some(start..end)
    }

    fn len(&self, buffer: &Buffer) -> usize {
        let rows = self.end().0 - self.start().0 + 1;
        rows.min(buffer.height.saturating_sub(self.start().0)) * buffer.width
    }

    fn iter(&self, buffer: &Buffer) -> Self::Iter {
        let start = self.start().0 * buffer.width;
        let end = (self.end().0 + 1).min(buffer.height) * buffer.width;
        start..end
    }
}

impl BufferRange for RangeTo<Row> {
    type Iter = Range<usize>;
    fn spatial(&self, buffer: &Buffer) -> Bounds {
        Bounds::new(
            Position::new(0, 0),
            Position::new(self.end.0.min(buffer.height), buffer.width),
        )
    }

    fn linear(&self, buffer: &Buffer) -> Option<Range<usize>> {
        let end = self.end.0.min(buffer.height) * buffer.width;
        Some(0..end)
    }

    fn len(&self, buffer: &Buffer) -> usize {
        self.end.0.min(buffer.height) * buffer.width
    }

    fn iter(&self, buffer: &Buffer) -> Self::Iter {
        let end = self.end.0.min(buffer.height) * buffer.width;
        0..end
    }
}

impl BufferRange for RangeToInclusive<Row> {
    type Iter = Range<usize>;
    fn spatial(&self, buffer: &Buffer) -> Bounds {
        Bounds::new(
            Position::new(0, 0),
            Position::new((self.end.0 + 1).min(buffer.height), buffer.width),
        )
    }

    fn linear(&self, buffer: &Buffer) -> Option<Range<usize>> {
        let end = (self.end.0 + 1).min(buffer.height) * buffer.width;
        Some(0..end)
    }

    fn len(&self, buffer: &Buffer) -> usize {
        (self.end.0 + 1).min(buffer.height) * buffer.width
    }

    fn iter(&self, buffer: &Buffer) -> Self::Iter {
        let end = (self.end.0 + 1).min(buffer.height) * buffer.width;
        0..end
    }
}

impl BufferRange for RangeFrom<Row> {
    type Iter = Range<usize>;
    fn spatial(&self, buffer: &Buffer) -> Bounds {
        Bounds::new(
            Position::new(self.start.0, 0),
            Position::new(buffer.height, buffer.width),
        )
    }

    fn linear(&self, buffer: &Buffer) -> Option<Range<usize>> {
        let start = self.start.0 * buffer.width;
        Some(start..buffer.len())
    }

    fn len(&self, buffer: &Buffer) -> usize {
        buffer.height.saturating_sub(self.start.0) * buffer.width
    }

    fn iter(&self, buffer: &Buffer) -> Self::Iter {
        let start = self.start.0 * buffer.width;
        start..buffer.len()
    }
}

// ============================================================================
// Col
// ============================================================================
// @TODO: Iter types
/*
impl BufferRange for Col {
    type Iter = Map<Range<usize>, fn(usize) -> usize>;
    fn spatial(&self, buffer: &Buffer) -> Bounds {
        let col = self.0;

        if col >= buffer.width {
            return Bounds::ZERO;
        }

        Bounds::new(
            Position::new(0, col),
            Position::new(buffer.height, col + 1),
        )
    }

    fn linear(&self, buffer: &Buffer) -> Option<Range<usize>> {
        // A column is never contiguous (unless buffer has width 1)
        if buffer.width == 1 && self.0 == 0 {
            Some(0..buffer.len())
        } else {
            None
        }
    }

    fn len(&self, buffer: &Buffer) -> usize {
        if self.0 < buffer.width { buffer.height } else { 0 }
    }

    fn iter(&self, buffer: &Buffer) -> Self::Iter {
        let col = self.0;
        (0..buffer.height).map(move |row| row * buffer.width + col)
    }
}

impl BufferRange for Range<Col> {
    type Iter = FlatMap<Range<usize>, fn(usize) -> Range<usize>>;
    fn spatial(&self, buffer: &Buffer) -> Bounds {
        let start_col = self.start.0;
        let end_col = self.end.0.min(buffer.width);
        Bounds::new(
            Position::new(0, start_col),
            Position::new(buffer.height, end_col),
        )
    }

    fn linear(&self, buffer: &Buffer) -> Option<Range<usize>> {
        // Only contiguous if selecting all columns (full rows)
        if self.start.0 == 0 && self.end.0 >= buffer.width {
            Some(0..buffer.len())
        } else {
            None
        }
    }

    fn len(&self, buffer: &Buffer) -> usize {
        let cols = self.end.0.saturating_sub(self.start.0).min(buffer.width.saturating_sub(self.start.0));
        cols * buffer.height
    }

    fn iter(&self, buffer: &Buffer) -> Self::Iter {
        let start_col = self.start.0;
        let end_col = self.end.0.min(buffer.width);
        let width = buffer.width;

        (0..buffer.height)
            .flat_map(move |row| (start_col..end_col).map(move |col| row * width + col))
    }
}

impl BufferRange for RangeInclusive<Col> {
    type Iter = ();
    fn spatial(&self, buffer: &Buffer) -> Bounds {
        let start_col = self.start().0;
        let end_col = (self.end().0 + 1).min(buffer.width);
        Bounds::new(
            Position::new(0, start_col),
            Position::new(buffer.height, end_col),
        )
    }

    fn linear(&self, buffer: &Buffer) -> Option<Range<usize>> {
        if self.start().0 == 0 && self.end().0 >= buffer.width - 1 {
            Some(0..buffer.len())
        } else {
            None
        }
    }

    fn len(&self, buffer: &Buffer) -> usize {
        let cols = (self.end().0 - self.start().0 + 1).min(buffer.width.saturating_sub(self.start().0));
        cols * buffer.height
    }

    fn iter(&self, buffer: &Buffer) -> Self::Iter {
        let start_col = self.start().0;
        let end_col = (self.end().0 + 1).min(buffer.width);
        let width = buffer.width;

        (0..buffer.height)
            .flat_map(move |row| (start_col..end_col).map(move |col| row * width + col))
    }
}

impl BufferRange for RangeTo<Col> {
    type Iter = ();
    fn spatial(&self, buffer: &Buffer) -> Bounds {
        let end_col = self.end.0.min(buffer.width);
        Bounds::new(
            Position::new(0, 0),
            Position::new(buffer.height, end_col),
        )
    }

    fn linear(&self, buffer: &Buffer) -> Option<Range<usize>> {
        if self.end.0 >= buffer.width {
            Some(0..buffer.len())
        } else {
            None
        }
    }

    fn len(&self, buffer: &Buffer) -> usize {
        self.end.0.min(buffer.width) * buffer.height
    }

    fn iter(&self, buffer: &Buffer) -> Self::Iter {
        let end_col = self.end.0.min(buffer.width);
        let width = buffer.width;

        (0..buffer.height)
            .flat_map(move |row| (0..end_col).map(move |col| row * width + col))
    }
}

impl BufferRange for RangeToInclusive<Col> {
    type Iter = ();
    fn spatial(&self, buffer: &Buffer) -> Bounds {
        let end_col = (self.end.0 + 1).min(buffer.width);
        Bounds::new(
            Position::new(0, 0),
            Position::new(buffer.height, end_col),
        )
    }

    fn linear(&self, buffer: &Buffer) -> Option<Range<usize>> {
        if self.end.0 >= buffer.width - 1 {
            Some(0..buffer.len())
        } else {
            None
        }
    }

    fn len(&self, buffer: &Buffer) -> usize {
        (self.end.0 + 1).min(buffer.width) * buffer.height
    }

    fn iter(&self, buffer: &Buffer) -> Self::Iter {
        let end_col = (self.end.0 + 1).min(buffer.width);
        let width = buffer.width;

        (0..buffer.height)
            .flat_map(move |row| (0..end_col).map(move |col| row * width + col))
    }
}

impl BufferRange for RangeFrom<Col> {
    type Iter = ();
    fn spatial(&self, buffer: &Buffer) -> Bounds {
        let start_col = self.start.0;
        if start_col >= buffer.width {
            return Bounds::ZERO;
        }
        Bounds::new(
            Position::new(0, start_col),
            Position::new(buffer.height, buffer.width),
        )
    }

    fn linear(&self, buffer: &Buffer) -> Option<Range<usize>> {
        if self.start.0 == 0 {
            Some(0..buffer.len())
        } else {
            None
        }
    }

    fn len(&self, buffer: &Buffer) -> usize {
        buffer.width.saturating_sub(self.start.0) * buffer.height
    }

    fn iter(&self, buffer: &Buffer) -> Self::Iter {
        let start_col = self.start.0;
        let width = buffer.width;
        
        (0..buffer.height)
            .flat_map(move |row| (start_col..width).map(move |col| row * width + col))
    }
}
*/