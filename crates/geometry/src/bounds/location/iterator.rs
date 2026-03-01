use std::iter::{FusedIterator};
use std::marker::Destruct;
use crate::{Location, Position};
use std::ops::{IntoBounds, Bound::*, RangeBounds, Deref, DerefMut, Sub};
use crate::bounds::Bounds;
use super::{Step, IntoLocation};


#[derive(Copy, Debug)]
#[derive_const(Clone)]
pub struct Iter<T = Position, Context = Bounds> {
    context: Context,
    front: T,
    back: T,
}

impl Iter {
    pub const fn new(context: Bounds) -> Self {
        // For empty regions, set front == back so is_done() is immediately true.
        let front = if context.is_empty() { context.max } else { context.min };
        Self {
            context,
            front,
            back: context.max,
        }
    }
}

impl Iterator for Iter {
    type Item = Position;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.front >= self.back {
            return None;
        }
        let next = self.front;

        match self.context.forward_checked(next, 1) {
            Some(next) => self.front = next,
            None =>  self.front = self.back
        }

        Some(next)
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.front >= self.back {
            return (0, Some(0));
        }
        let count = self.count();
        (count, Some(count))
    }

    #[inline]
    fn count(self) -> usize {
        if self.front >= self.back {
            return 0;
        }
        let current = self.into_index(self.front);
        let remaining = self.area();
        (remaining - current)
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if self.front >= self.back {
            return None;
        }

        if let Some(plus_n) = self.context.forward_checked(self.front, n) {
            if plus_n < self.context.max {
                self.front =
                    self.context.forward_checked(plus_n, 1).expect("`Step` invariants not upheld");
                return Some(plus_n);
            }
        }

        None
    }

    fn for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        if self.front >= self.back { return; }

        let mut item = self.front;

        while item.row < self.max.row {
            let end_col = self.max.col;
            while item.col < end_col {
                f(item);
                item.col += 1;
            }
            item.col = self.min.col;
            item.row += 1;
        }
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        if self.front >= self.back { return init; }

        let mut acc = init;

        let mut item = self.front;

        while item.row < self.max.row {
            let end_col = self.max.col;
            while item.col < end_col {
                acc = f(acc, item);
                item.col += 1;
            }
            item.col = self.min.col;
            item.row += 1;
        }

        acc
    }

    fn max(mut self) -> Option<Self::Item>
    {
        self.next_back()
    }

    #[inline]
    fn min(mut self) -> Option<Self::Item>
    {
        self.next()
    }

    #[inline]
    fn is_sorted(self) -> bool {
        true
    }
}
impl DoubleEndedIterator for Iter {
    #[inline]
    fn next_back(&mut self) -> Option<Position> {
        if self.front >= self.back {
            return None;
        }
        // Decrement first, then yield — back is an exclusive bound.
        match self.context.backward_checked(self.back, 1) {
            Some(prev) => {
                self.back = prev;
                Some(prev)
            }
            None => {
                // back was at min — yield it and mark exhausted.
                let last = self.front;
                self.front = self.back;
                Some(last)
            }
        }
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Position> {
        if self.front >= self.back {
            return None;
        }

        if let Some(minus_n) = self.context.backward_checked(self.back, n) {
            if minus_n < self.context.max {
                self.back =
                    self.context.backward_checked(minus_n, 1).expect("`Step` invariants not upheld");
                return Some(minus_n);
            }
        }

        None
    }
}

impl ExactSizeIterator for Iter {}
impl FusedIterator for Iter {}

impl const Deref for Iter {
    type Target = Bounds;
    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

/// A position that knows its spatial context.
#[derive(Copy, Debug)]
#[derive_const(Clone)]
pub struct Cursor<'a, P = Position> {
    context: &'a Bounds,
    position: P,
}

impl<'a> Cursor<'a, Position> {
    pub const fn new(ctx: &'a Bounds, pos: Position) -> Self {
        Self { context: ctx, position: pos }
    }

    pub fn forward_checked(mut self, n: usize) -> Option<Cursor<'a, Position>> {
        self.context.forward_checked(self.position, n).map(|pos| {
            self.position = pos;
            self
        })
    }

    pub fn forward(mut self, n: usize) -> Cursor<'a, Position> {
        self.position = self.context.forward(self.position, n);
        self
    }

    pub unsafe fn forward_unchecked(mut self, n: usize) -> Cursor<'a, Position> {
        self.position = self.context.forward_unchecked(self.position, n);
        self
    }

    pub fn backward(mut self, n: usize) -> Cursor<'a, Position> {
        self.position = self.context.backward(self.position, n);
        self
    }

    pub fn backward_checked(mut self, n: usize) -> Option<Cursor<'a, Position>> {
        self.context.backward_checked(self.position, n).map(|pos| {
            self.position = pos;
            self
        })
    }

    pub unsafe fn backward_unchecked(mut self, n: usize) -> Cursor<'a, Position> {
        self.position = self.context.backward_unchecked(self.position, n);
        self
    }

    pub fn index(&self) -> usize {
        self.context.into_index(self.position)
    }
}
impl<'a> Iterator for Cursor<'a, Position> {
    type Item = Position;

    fn next(&mut self) -> Option<Position> {
        if self.position >= self.context.max {
            return None;
        }

        let current = self.position;
        match self.context.forward_checked(self.position, 1) {
            Some(next) => { self.position = next; Some(current) }
            None => None, // could use a `done` flag for fused behavior
        }
    }


    fn for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        if self.position >= self.context.max { return; }

        let mut item = self.position;

        while item.row < self.context.max.row {
            let end_col = self.context.max.col;
            while item.col < end_col {
                f(item);
                item.col += 1;
            }
            item.col = self.context.min.col;
            item.row += 1;
        }
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        if self.position >= self.context.max { return init; }

        let mut acc = init;

        let mut item = self.position;

        while item.row < self.context.max.row {
            let end_col = self.context.max.col;
            while item.col < end_col {
                acc = f(acc, item);
                item.col += 1;
            }
            item.col = self.context.min.col;
            item.row += 1;
        }

        acc
    }


    #[inline]
    fn min(mut self) -> Option<Self::Item>
    {
        self.next()
    }

    #[inline]
    fn is_sorted(self) -> bool {
        true
    }

}
impl FusedIterator for Cursor<'_, Position> {}
impl const Deref for Cursor<'_, Position> {
    type Target = Position;
    fn deref(&self) -> &Self::Target {
        &self.position
    }
}
impl AsRef<Position> for Cursor<'_> {
    fn as_ref(&self) -> &Position {
        &self.position
    }
}
#[cfg(test)]
mod tests {
    use super::*;


    #[cfg(test)]
    mod off_by_one {
        use super::*;

        #[test]
        fn from_0() {
            for x in 0..2 {
                for y in 0..2 {
                    let bounds = Bounds::new(Position::new(0, 0), Position::new(x, y));

                    let area = bounds.area();
                    let len = bounds.iter().collect::<Vec<_>>().len();
                    let count = bounds.iter().count();
                    let size_hint = bounds.iter().size_hint().1.unwrap_or(0);

                    dbg!(bounds);
                    assert_eq!(area, area, "area {area} != {area}. bounds={bounds:?}");
                    assert_eq!(area, len, "area {area} != {len}. bounds={bounds:?}");
                    assert_eq!(area,count, "area {area} != {count}. bounds={bounds:?}");
                    assert_eq!(area, size_hint, "area {area} != {size_hint}. bounds={bounds:?}");

                    assert_eq!(len, area, "len {len} != {area}. bounds={bounds:?}");
                    assert_eq!(len, len, "len {len} != {len}. bounds={bounds:?}");
                    assert_eq!(len, count, "len {len} != {count}. bounds={bounds:?}");
                    assert_eq!(len, size_hint, "len {len} != {size_hint}. bounds={bounds:?}");

                    assert_eq!(count, area, "count {count} != {area}. bounds={bounds:?}");
                    assert_eq!(count, len, "count {count} != {len}. bounds={bounds:?}");
                    assert_eq!(count,count, "count {count} != {count}. bounds={bounds:?}");
                    assert_eq!(count, size_hint, "count {count} != {size_hint}. bounds={bounds:?}");

                    assert_eq!(size_hint, area, "size_hint {size_hint} != {area}. bounds={bounds:?}");
                    assert_eq!(size_hint, len, "size_hint {size_hint} != {len}. bounds={bounds:?}");
                    assert_eq!(size_hint,count, "size_hint {size_hint} != {count}. bounds={bounds:?}");
                    assert_eq!(size_hint, size_hint, "size_hint {size_hint} != {size_hint}. bounds={bounds:?}");
                }
            }

        }
        #[test]
        fn from_1() {
            for x in 1..2 {
                for y in 1..3 {

                    let bounds = Bounds::new(Position::new(1, 1), Position::new(x, y));

                    let area = bounds.area();
                    let len = bounds.iter().collect::<Vec<_>>().len().saturating_sub(1);
                    let count = bounds.iter().count();
                    let size_hint = bounds.iter().size_hint().1.unwrap_or(0);

                    assert_eq!(area, area, "area {area} != {area}. bounds={bounds:?}");
                    assert_eq!(area, len, "area {area} != {len}. bounds={bounds:?}");
                    assert_eq!(area,count, "area {area} != {count}. bounds={bounds:?}");
                    assert_eq!(area, size_hint, "area {area} != {size_hint}. bounds={bounds:?}");

                    assert_eq!(len, area, "len {len} != {area}. bounds={bounds:?}");
                    assert_eq!(len, len, "len {len} != {len}. bounds={bounds:?}");
                    assert_eq!(len, count, "len {len} != {count}. bounds={bounds:?}");
                    assert_eq!(len, size_hint, "len {len} != {size_hint}. bounds={bounds:?}");

                    assert_eq!(count, area, "count {count} != {area}. bounds={bounds:?}");
                    assert_eq!(count, len, "count {count} != {len}. bounds={bounds:?}");
                    assert_eq!(count,count, "count {count} != {count}. bounds={bounds:?}");
                    assert_eq!(count, size_hint, "count {count} != {size_hint}. bounds={bounds:?}");

                    assert_eq!(size_hint, area, "size_hint {size_hint} != {area}. bounds={bounds:?}");
                    assert_eq!(size_hint, len, "size_hint {size_hint} != {len}. bounds={bounds:?}");
                    assert_eq!(size_hint,count, "size_hint {size_hint} != {count}. bounds={bounds:?}");
                    assert_eq!(size_hint, size_hint, "size_hint {size_hint} != {size_hint}. bounds={bounds:?}");
                }
            }


        }
        #[test]
        fn to_plus_one() {
            for x in 0..3 {
                for y in 0..3 {
                    let bounds = Bounds::new(Position::new(x, y), Position::new(x + 1, y + 1));

                    let area = bounds.area();
                    let len = bounds.iter().collect::<Vec<_>>().len();
                    let count = bounds.iter().count();
                    let size_hint = bounds.iter().size_hint().1.unwrap_or(0);

                    assert_eq!(area, area, "area {area} != {area}. bounds={bounds:?}");
                    assert_eq!(area, len, "area {area} != {len}. bounds={bounds:?}");
                    assert_eq!(area,count, "area {area} != {count}. bounds={bounds:?}");
                    assert_eq!(area, size_hint, "area {area} != {size_hint}. bounds={bounds:?}");

                    assert_eq!(len, area, "len {len} != {area}. bounds={bounds:?}");
                    assert_eq!(len, len, "len {len} != {len}. bounds={bounds:?}");
                    assert_eq!(len, count, "len {len} != {count}. bounds={bounds:?}");
                    assert_eq!(len, size_hint, "len {len} != {size_hint}. bounds={bounds:?}");

                    assert_eq!(count, area, "count {count} != {area}. bounds={bounds:?}");
                    assert_eq!(count, len, "count {count} != {len}. bounds={bounds:?}");
                    assert_eq!(count,count, "count {count} != {count}. bounds={bounds:?}");
                    assert_eq!(count, size_hint, "count {count} != {size_hint}. bounds={bounds:?}");

                    assert_eq!(size_hint, area, "size_hint {size_hint} != {area}. bounds={bounds:?}");
                    assert_eq!(size_hint, len, "size_hint {size_hint} != {len}. bounds={bounds:?}");
                    assert_eq!(size_hint,count, "size_hint {size_hint} != {count}. bounds={bounds:?}");
                    assert_eq!(size_hint, size_hint, "size_hint {size_hint} != {size_hint}. bounds={bounds:?}");
                }
            }


        }

    }
    #[test]
    #[test]
    fn test_bounds_iter_basic() {
        let bounds = Bounds::new(Position::new(0, 0), Position::new(2, 3));
        let positions: Vec<_> = bounds.iter().collect();

        assert_eq!(positions.len(), 6); // 2 rows * 3 cols

        // Check row-major order
        assert_eq!(positions[0], Position::new(0, 0));
        assert_eq!(positions[1], Position::new(0, 1));
        assert_eq!(positions[2], Position::new(0, 2));
        assert_eq!(positions[3], Position::new(1, 0));
        assert_eq!(positions[4], Position::new(1, 1));
        assert_eq!(positions[5], Position::new(1, 2));
    }

    #[test]
    fn test_bounds_iter_empty_width() {
        let bounds = Bounds::new(Position::new(0, 5), Position::new(0, 5));
        assert_eq!(bounds.iter().count(), 0);
    }

    #[test]
    fn test_bounds_iter_empty_height() {
        let bounds = Bounds::new(Position::new(5, 0), Position::new(5, 1));
        assert_eq!(bounds.iter().count(), 0);
    }

    #[test]
    fn test_bounds_iter_single_cell() {
        let bounds = Bounds::new(Position::new(5, 10), Position::new(6, 11));
        let positions: Vec<_> = bounds.iter().collect();

        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0], Position::new(5, 10));
    }

    #[test]
    fn test_bounds_iter_size_hint() {
        let bounds = Bounds::new(Position::new(0, 0), Position::new(3, 4));
        let iter = bounds.iter();
        let (min, max) = iter.size_hint();

        assert_eq!(min, 12);
        assert_eq!(max, Some(12));

    }

    #[test]
    fn test_bounds_iter_exact_size() {
        let bounds = Bounds::new(Position::new(0, 0), Position::new(5, 10));
        let iter = bounds.iter();

        assert_eq!(iter.count(), 50);
    }

    #[test]
    fn test_bounds_into_iter() {
        let bounds = Bounds::new(Position::new(0, 0), Position::new(2, 2));
        let count = bounds.iter().count();
        assert_eq!(count, 4);
    }

    #[test]
    fn test_bounds_into_iter_ref() {
        let bounds = Bounds::new(Position::new(0, 0), Position::new(3, 3));
        let count = (bounds).iter().count();
        assert_eq!(count, 9);
    }
}