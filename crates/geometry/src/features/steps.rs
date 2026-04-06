use crate::{Bounded, Point, Rect, Row};
use std::iter::FusedIterator;
use crate::{Resolve};

/// Provides the spatial context needed to step through positions in row-major
/// order within a bounded 2D region.
///
/// This is the "grid" that gives meaning to forward/backward movement —
/// without it, a bare `Position` doesn't know when to wrap to the next row.
pub trait Step<T> {
    /// Number of row-major steps from `start` to `end`.
    ///
    /// Returns `(n, Some(n))` when `start <= end` within bounds,
    /// or `(0, None)` when `start > end`.
    #[inline(always)]
    fn steps_between(&self, start: T, end: T) -> (usize, Option<usize>);

    /// Move `count` steps forward in row-major order, or `None` if out of bounds.
    #[inline(always)]
    fn forward_checked(&self, start: T, count: usize) -> Option<T>;

    /// Like `forward_checked`, but panics on overflow.
    #[inline(always)]
    fn forward(&self, start: T, count: usize) -> T {
        self.forward_checked(start, count)
            .expect("overflow in Step::forward")
    }

    /// Like `forward_checked`, without bounds checking.
    ///
    /// # Safety
    /// The result must remain within bounds.
    #[inline(always)]
    unsafe fn forward_unchecked(&self, start: T, count: usize) -> T {
        self.forward(start, count)
    }

    /// Move `count` steps backward in row-major order, or `None` if out of bounds.
    #[inline(always)]
    fn backward_checked(&self, start: T, count: usize) -> Option<T>;

    /// Like `backward_checked`, but panics on underflow.
    #[inline(always)]
    fn backward(&self, start: T, count: usize) -> T {
        self.backward_checked(start, count)
            .expect("underflow in Step::backward")
    }

    /// Like `backward_checked`, without bounds checking.
    ///
    /// # Safety
    /// The result must remain within bounds.
    #[inline(always)]
    unsafe fn backward_unchecked(&self, start: T, count: usize) -> T {
        self.backward(start, count)
    }
}
impl Step<Point> for Rect {
    fn steps_between(&self, start: Point, end: Point) -> (usize, Option<usize>) {
        if start > end {
            return (0, None);
        }
        let current: usize = self.resolve(start);
        let remaining: usize = self.resolve(end);

        let dist = remaining - current;

        (dist, Some(dist))
    }

    fn forward_checked(&self, start: Point, count: usize) -> Option<Point> {
        // Fast path for single step (Iterator usage).
        if count == 1 {
            let mut next = start;
            next.x += 1;

            if next.x >= self.max().x {
                next.x = self.min().x;
                next.y += 1;

                if next.y >= self.max().y {
                    return None;
                }
            }

            return Some(next);
        }

        // General path for arbitrary steps.
        let index: usize = self.resolve(start);

        let index = index.checked_add(count)?;
        if index >= self.len() {
            return None;
        }

        Some(self.resolve(index))
    }

    fn backward_checked(&self, start: Point, count: usize) -> Option<Point> {
        // Fast path: stay on the same row.
        if start.y < self.max().x && count <= start.y - self.min.x {
            return Some(Point::new(start.x, start.y - count));
        }
        // General path: linearize through the exclusive end.
        let idx = if start >= self.max() {
            self.len()
        } else {
            self.resolve(start)
        };
        let target = idx.checked_sub(count)?;
        Some(self.resolve(target))
    }
}

/// Owned, double-ended iterator over every `Position` in a `Bounds`.
///
/// Created by [`Area::iter`].
#[derive(Copy, Debug, Clone)]
pub struct Steps<Ctx: Bounded<Coordinate= T> + Step<T>, T> {
    pub(crate) context: Ctx,
    pub(crate) front: T,
    pub(crate) back: T,
}

impl<Ctx: Bounded<Coordinate= T> + Step<T>, T> Steps<Ctx, T> {
    pub fn new(context: Ctx) -> Self {
        let front = if context.is_empty() {
            context.max()
        } else {
            context.min()
        };
        let back = context.max();

        Self {
            context,
            front,
            back,
        }
    }
}

impl Iterator for Steps<Rect, Point> {
    type Item = Point;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.front >= self.back {
            return None;
        }
        let next = self.front;

        match self.context.forward_checked(next, 1) {
            Some(next) => self.front = next,
            None => self.front = self.back,
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
        let current: usize = self.context.resolve(self.front);
        let remaining = self.context.len();
        remaining - current
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if self.front >= self.back {
            return None;
        }

        if let Some(plus_n) = self.context.forward_checked(self.front, n) {
            if plus_n < self.context.max {
                self.front = self
                    .context
                    .forward_checked(plus_n, 1)
                    .expect("`Step` invariants not upheld");
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
        if self.front >= self.back {
            return;
        }
        let mut pos = self.front;
        while pos.y < self.context.max.y {
            while pos.x < self.context.max.x {
                f(pos);
                pos.x += 1;
            }
            pos.x = self.context.min.x;
            pos.y += 1;
        }
    }

    fn fold<B, F>(self, init: B, mut f: F) -> B
    where
        F: FnMut(B, Self::Item) -> B,
    {
        if self.front >= self.back {
            return init;
        }
        let mut acc = init;
        let mut pos = self.front;
        while pos.y < self.context.max.y {
            while pos.x < self.context.max.x {
                acc = f(acc, pos);
                pos.x += 1;
            }
            pos.x = self.context.min.x;
            pos.y += 1;
        }
        acc
    }

    fn max(mut self) -> Option<Self::Item> {
        self.next_back()
    }

    #[inline]
    fn min(mut self) -> Option<Self::Item> {
        self.next()
    }

    #[inline]
    fn is_sorted(self) -> bool {
        true
    }
}
impl DoubleEndedIterator for Steps<Rect, Point> {
    #[inline]
    fn next_back(&mut self) -> Option<Point> {
        if self.front >= self.back {
            return None;
        }
        match self.context.backward_checked(self.back, 1) {
            Some(prev) => {
                self.back = prev;
                Some(prev)
            }
            None => {
                let last = self.front;
                self.front = self.back;
                Some(last)
            }
        }
    }

    #[inline]
    fn nth_back(&mut self, n: usize) -> Option<Point> {
        if self.front >= self.back {
            return None;
        }

        if let Some(minus_n) = self.context.backward_checked(self.back, n) {
            if minus_n < self.context.max {
                self.back = self
                    .context
                    .backward_checked(minus_n, 1)
                    .expect("`Step` invariants not upheld");
                return Some(minus_n);
            }
        }

        None
    }
}

impl<Ctx: Bounded<Coordinate= T> + Step<T>, T> ExactSizeIterator for Steps<Ctx, T> where Self: Iterator {}
impl<Ctx: Bounded<Coordinate= T> + Step<T>, T> FusedIterator for Steps<Ctx, T> where Self: Iterator {}

// ─── Tests ─────────────────────────────────────────────────────────────

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[cfg(test)]
//     mod off_by_one {
//         use super::*;
//
//         // #[test]
//         // @TODO: Fix this test
//         fn from_0() {
//             for x in 0..2 {
//                 for y in 0..2 {
//                     let bounds = Area::new(Position::new(0, 0), Position::new(x, y));
//
//                     let area = bounds.len();
//                     let len = bounds.iter().collect::<Vec<_>>().len();
//                     let count = bounds.iter().count();
//                     let size_hint = bounds.iter().size_hint().1.unwrap_or(0);
//
//                     assert_eq!(
//                         area, len,
//                         "Area len mismatch: {area} != {len}. bounds={bounds:?}"
//                     );
//                     assert_eq!(
//                         area, count,
//                         "Area count mismatch: {area} != {count}. bounds={bounds:?}"
//                     );
//                     assert_eq!(
//                         area, size_hint,
//                         "Area size hint mismatch: {area} != {size_hint}. bounds={bounds:?}"
//                     );
//                 }
//             }
//         }
//
//         #[test]
//         fn from_1() {
//             for x in 1..2 {
//                 for y in 1..3 {
//                     let bounds = Area::new(Position::new(1, 1), Position::new(x, y));
//
//                     let area = bounds.len();
//                     let len = bounds.iter().collect::<Vec<_>>().len().saturating_sub(1);
//                     let count = bounds.iter().count();
//                     let size_hint = bounds.iter().size_hint().1.unwrap_or(0);
//
//                     assert_eq!(area, len, "area {area} != {len}. bounds={bounds:?}");
//                     assert_eq!(area, count, "area {area} != {count}. bounds={bounds:?}");
//                     assert_eq!(
//                         area, size_hint,
//                         "area {area} != {size_hint}. bounds={bounds:?}"
//                     );
//                 }
//             }
//         }
//
//         #[test]
//         fn to_plus_one() {
//             for x in 0..3 {
//                 for y in 0..3 {
//                     let bounds = Area::new(Position::new(x, y), Position::new(x + 1, y + 1));
//
//                     let area = bounds.len();
//                     let len = bounds.iter().collect::<Vec<_>>().len();
//                     let count = bounds.iter().count();
//                     let size_hint = bounds.iter().size_hint().1.unwrap_or(0);
//
//                     assert_eq!(area, len, "area {area} != {len}. bounds={bounds:?}");
//                     assert_eq!(area, count, "area {area} != {count}. bounds={bounds:?}");
//                     assert_eq!(
//                         area, size_hint,
//                         "area {area} != {size_hint}. bounds={bounds:?}"
//                     );
//                 }
//             }
//         }
//     }
//
//     #[test]
//     fn test_bounds_iter_basic() {
//         let bounds = Area::new(Position::new(0, 0), Position::new(2, 3));
//         let positions: Vec<_> = bounds.iter().collect();
//
//         assert_eq!(positions.len(), 6); // 2 rows * 3 cols
//
//         // Check row-major order
//         assert_eq!(positions[0], Position::new(0, 0));
//         assert_eq!(positions[1], Position::new(0, 1));
//         assert_eq!(positions[2], Position::new(0, 2));
//         assert_eq!(positions[3], Position::new(1, 0));
//         assert_eq!(positions[4], Position::new(1, 1));
//         assert_eq!(positions[5], Position::new(1, 2));
//     }
//
//     #[test]
//     fn test_bounds_iter_empty_width() {
//         let bounds = Area::new(Position::new(0, 5), Position::new(0, 5));
//         assert_eq!(bounds.iter().count(), 0);
//     }
//
//     #[test]
//     fn test_bounds_iter_empty_height() {
//         let bounds = Area::new(Position::new(5, 0), Position::new(5, 1));
//         assert_eq!(bounds.iter().count(), 0);
//     }
//
//     #[test]
//     fn test_bounds_iter_single_cell() {
//         let bounds = Area::new(Position::new(5, 10), Position::new(6, 11));
//         let positions: Vec<_> = bounds.iter().collect();
//
//         assert_eq!(positions.len(), 1);
//         assert_eq!(positions[0], Position::new(5, 10));
//     }
//
//     #[test]
//     fn test_bounds_iter_size_hint() {
//         let bounds = Area::new(Position::new(0, 0), Position::new(3, 4));
//         let iter = bounds.iter();
//         let (min, max) = iter.size_hint();
//
//         assert_eq!(min, 12);
//         assert_eq!(max, Some(12));
//     }
//
//     #[test]
//     fn test_bounds_iter_exact_size() {
//         let bounds = Area::new(Position::new(0, 0), Position::new(5, 10));
//         let iter = bounds.iter();
//
//         assert_eq!(iter.count(), 50);
//     }
//
//     #[test]
//     fn test_bounds_into_iter() {
//         let bounds = Area::new(Position::new(0, 0), Position::new(2, 2));
//         let count = bounds.iter().count();
//         assert_eq!(count, 4);
//     }
//
//     #[test]
//     fn test_bounds_into_iter_ref() {
//         let bounds = Area::new(Position::new(0, 0), Position::new(3, 3));
//         let count = (bounds).iter().count();
//         assert_eq!(count, 9);
//     }
// }
