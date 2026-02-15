use std::iter::FusedIterator;
use crate::{Position, Size};
use std::ops::{IntoBounds, Bound, Bound::*, RangeBounds, Deref, DerefMut, Sub};
use crate::region::Region;
use crate::region::step::{SpatialContext, SpatialStep};

#[derive(Copy, Debug)]
#[derive_const(Clone)]
pub struct SpatialIter<T = Position, C: SpatialContext<T> = Region> {
    context: C,
    item: T,
    done: bool,
}

impl SpatialIter {
    pub const fn new(context: Region) -> Self {
        Self {
            context,
            item: context.min,
            done: context.area() == 0,
        }
    }
}

impl Sub for Position {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            row: self.row - rhs.row,
            col: self.col - rhs.col,
        }
    }
}

impl Iterator for SpatialIter {
    type Item = Position;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let next = self.item;

        match self.context.forward_checked(next, 1) {
            Some(next) => self.item = next,
            None => self.done = true,
        }

        Some(next)
    }
    fn for_each<F>(self, mut f: F)
    where
        Self: Sized,
        F: FnMut(Self::Item),
    {
        if self.done { return; }

        let mut item = self.item;

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
        if self.done { return init; }

        let mut acc = init;

        let mut item = self.item;

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

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.done {
            return (0, Some(0));
        }
        self.context.steps_between(&(self.item), &(self.max))

    }

    #[inline]
    fn count(self) -> usize {
        if self.done {
            return 0;
        }
        let (_, upper) = self.context.steps_between(&(self.item), &(self.max));
        upper.expect("count overflowed usize")
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if let Some(plus_n) = self.context.forward_checked(self.item, n) {
            if plus_n < self.context.max {
                self.item =
                    self.context.forward_checked(plus_n, 1).expect("`Step` invariants not upheld");
                return Some(plus_n);
            }
        }

        self.context.min = self.context.max;
        None
    }


    #[inline]
    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }

    #[inline]
    fn min(mut self) -> Option<Self::Item>
    {
        self.next()
    }

    #[inline]
    fn max(mut self) -> Option<Self::Item>
    {
        self.next_back()
    }

    #[inline]
    fn is_sorted(self) -> bool {
        true
    }
}
impl DoubleEndedIterator for SpatialIter {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let next = self.item;

        match self.context.backward_checked(next, 1) {
            Some(next) => self.item = next,
            None => self.done = true,
        }

        Some(next)
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        if let Some(minus_n) = self.context.backward_checked(self.max, n) {
            if minus_n > self.context.min {
                self.context.max =
                    self.context.backward_checked(minus_n, 1).expect("`Step` invariants not upheld");
                return Some(self.context.max.clone());
            }
        }

        self.context.max = self.context.min;
        None
    }
}

impl ExactSizeIterator for SpatialIter {}
impl FusedIterator for SpatialIter {}

impl const Deref for SpatialIter {
    type Target = Region;
    fn deref(&self) -> &Self::Target {
        &self.context
    }
}
