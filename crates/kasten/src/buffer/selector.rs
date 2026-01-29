use std::iter::FusedIterator;
use std::ops::{Range, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};
use crate::{Buffer, Col, Position, Region};

pub trait BufferSelector {
    fn select<'a>(&'a self, buffer: &'a Buffer) -> impl Iterator<Item = usize> + 'a;
    fn positions<'a>(&'a self, buffer: &'a Buffer) -> impl Iterator<Item = Position> + 'a {
        let width = buffer.width();

        self.select(buffer).map(move |index| Position::from_index(index, width))
    }
}

impl BufferSelector for Col {
    fn select(&self, buffer: &Buffer) -> impl Iterator<Item = usize> {
        (0..buffer.height()).map(move |row| row * buffer.width() + self.0)
    }
}

impl BufferSelector for Range<Col> {
    fn select(&self, buffer: &Buffer) -> impl Iterator<Item = usize> {
        let start = self.start.0;
        let end = self.end.0;

        (0..buffer.height()).flat_map(move |row| {
            (start..end).map(move |col| row * buffer.width() + col)
        })
    }
}


impl BufferSelector for Range<Position> {
    fn select(&self, buffer: &Buffer) -> impl Iterator<Item = usize> {
        SelectorIter::new(Region::new(self.start, self.end))
    }
}

impl BufferSelector for RangeInclusive<Position> {
    fn select(&self, buffer: &Buffer) -> impl Iterator<Item = usize> {
        SelectorIter::new(Region::new(*self.start(), *self.end()))
    }
}

impl BufferSelector for RangeFrom<Position> {
    fn select(&self, buffer: &Buffer) -> impl Iterator<Item = usize> {
        SelectorIter::new(Region::new(self.start, Position::new(buffer.height(), buffer.width())))
    }
}

impl BufferSelector for RangeTo<Position> {
    fn select(&self, buffer: &Buffer) -> impl Iterator<Item = usize> {
        SelectorIter::new(Region::new(Position::ZERO, self.end))
    }

}

impl BufferSelector for RangeToInclusive<Position> {
    fn select(&self, buffer: &Buffer) -> impl Iterator<Item = usize> {
        SelectorIter::new(Region::new(Position::ZERO, self.end))
    }

}

impl BufferSelector for Region {
    fn select(&self, buffer: &Buffer) -> impl Iterator<Item = usize> {
        SelectorIter::new(*self)
    }
}

/// Iterator over positions in a region, row-by-row.
#[derive(Clone, Debug)]
pub struct SelectorIter {
    row: Range<usize>,
    col: Range<usize>,

    index: usize,
    end: usize,
}

impl SelectorIter {
    #[inline]
    const fn new(region: Region) -> Self {
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

    fn to_position(&self, index: usize) -> usize {
        let width = self.col.end - self.col.start;
        if width == 0 {
            return 0;
        }

        self.row.start + index / width + (self.col.start + index % width) * self.row.end
    }
}

impl Iterator for SelectorIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.end {
            return None;
        }

        let next = self.index;
        self.index += 1;
        Some(next)
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

impl DoubleEndedIterator for SelectorIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index >= self.end {
            return None;
        }

        self.end -= 1;
        Some(self.end)
    }
}

impl ExactSizeIterator for SelectorIter {
    fn len(&self) -> usize {
        self.end.saturating_sub(self.index)
    }
}
impl FusedIterator for SelectorIter {}