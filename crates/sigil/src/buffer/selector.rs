use crate::Buffer;
use geometry::{Col, Position, Rect, Region, SpatialIter};
use std::iter::FusedIterator;
use std::ops::{Range, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};

pub trait BufferSelector {
    fn select(&self, buffer: &Buffer) -> impl Iterator<Item = usize>;
    fn positions(&self, buffer: &Buffer) -> impl Iterator<Item = Position> {
        let width = buffer.width;

        self.select(buffer)
            .map(move |index| Position::from_index(index, width))
    }
}

impl BufferSelector for Col {
    fn select(&self, buffer: &Buffer) -> impl Iterator<Item = usize> {
        (0..buffer.height).map(move |row| row * buffer.width + self.0)
    }
}

impl BufferSelector for Range<Col> {
    fn select(&self, buffer: &Buffer) -> impl Iterator<Item = usize> {
        let start = self.start.0;
        let end = self.end.0;

        (0..buffer.height)
            .flat_map(move |row| (start..end).map(move |col| row * buffer.width + col))
    }
}

impl BufferSelector for Range<Position> {
    fn select(&self, buffer: &Buffer) -> impl Iterator<Item = usize> {
        SpatialIter::new(Region::new(self.start, self.end))
    }
}

impl BufferSelector for RangeInclusive<Position> {
    fn select(&self, buffer: &Buffer) -> impl Iterator<Item = usize> {
        SpatialIter::new(Region::new(*self.start(), *self.end()))
    }
}

impl BufferSelector for RangeFrom<Position> {
    fn select(&self, buffer: &Buffer) -> impl Iterator<Item = usize> {
        SpatialIter::new(Region::new(
            self.start,
            Position::new(buffer.height, buffer.width),
        ))
    }
}

impl BufferSelector for RangeTo<Position> {
    fn select(&self, buffer: &Buffer) -> impl Iterator<Item = usize> {
        SpatialIter::new(Region::new(Position::ZERO, self.end))
    }
}

impl BufferSelector for RangeToInclusive<Position> {
    fn select(&self, buffer: &Buffer) -> impl Iterator<Item = usize> {
        SpatialIter::new(Region::new(Position::ZERO, self.end))
    }
}

impl BufferSelector for Region {
    fn select(&self, buffer: &Buffer) -> impl Iterator<Item = usize> {
        SpatialIter::new(*self)
    }
}
