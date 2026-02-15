use std::iter;
use geometry::{Bounds, Col, Position as Pos, Position, Row};
use std::ops::{Range, Bound, IntoBounds, RangeBounds};
use super::super::{Buffer};
use super::{BoundsIter, };

pub trait Selector {
    type Iter: Iterator<Item = usize>;

    fn iter(self, of: &Buffer) -> Self::Iter;

    fn len(&self, of: &Buffer) -> usize;
    fn is_empty(&self, of: &Buffer) -> bool {
        self.len(of) == 0
    }
}

pub trait IntoSelectorBounds: SelectorBounds {
    fn into_bounds(self, of: &Buffer) -> (Bound<Position>, Bound<Position>);
    fn intersect(self, other: Self, of: &Buffer) -> (Bound<Position>, Bound<Position>) {
        let self_bounds = self.into_bounds(of);
        let other_bounds = other.into_bounds(of);
        IntoBounds::intersect(self_bounds, other_bounds)
    }

    /// Union with another selector (covering both areas). Returns (start_bound, end_bound).
    fn union(
        self,
        other: impl IntoSelectorBounds,
        of: &Buffer
    ) -> (Bound<Position>, Bound<Position>) {
        fn pos(b: Bound<Position>) -> Position {
            match b {
                Bound::Included(p) | Bound::Excluded(p) => p,
                Bound::Unbounded => Position { row: 0, col: 0 },
            }
        }

        fn end_pos(b: Bound<Position>) -> Position {
            match b {
                Bound::Included(p) | Bound::Excluded(p) => p,
                Bound::Unbounded => Position { row: usize::MAX, col: usize::MAX },
            }
        }

        // Min of starts
        let (self_start, other_start) = self.into_bounds(of);
        let start = match (&self_start, &other_start) {
            (Bound::Unbounded, _) | (_, Bound::Unbounded) => Bound::Unbounded,
            (a, b) => {
                let pos_a = pos(*a);
                let pos_b = pos(*b);
                let min_pos = if pos_a <= pos_b { pos_a } else { pos_b };
                // If either included, result is included
                match (a, b) {
                    (Bound::Excluded(_), Bound::Excluded(_)) => Bound::Excluded(min_pos),
                    _ => Bound::Included(min_pos),
                }
            }
        };

        // Max of ends
        let (self_end, other_end) = other.into_bounds(of);
        let end = match (&self_end, &other_end) {
            (Bound::Unbounded, _) | (_, Bound::Unbounded) => Bound::Unbounded,
            (a, b) => {
                let pos_a = end_pos(*a);
                let pos_b = end_pos(*b);
                let max_pos = if pos_a >= pos_b { pos_a } else { pos_b };
                // If either excluded, result is excluded
                match (a, b) {
                    (Bound::Included(_), Bound::Included(_)) => Bound::Included(max_pos),
                    _ => Bound::Excluded(max_pos),
                }
            }
        };

        (start, end)
    }

    fn clip(self, other: impl IntoSelectorBounds, of: &Buffer) -> (Bound<Position>, Bound<Position>) {
        let self_bounds = self.into_bounds(of);
        let other_bounds = other.into_bounds(of);
        IntoBounds::intersect(other_bounds, self_bounds)
    }


}

pub trait SelectorBounds: Selector + Sized {
    fn start_bound(&self) -> Bound<&Position>;
    fn end_bound(&self) -> Bound<&Position>;

    fn min(&self) -> &Position;
    fn max(&self) -> &Position;

    fn contains(&self, position: Position) -> bool {
        (match self.start_bound() {
            Bound::Included(start) => start <= &position,
            Bound::Excluded(start) => start < &position,
            Bound::Unbounded => true,
        }) && (match self.end_bound() {
            Bound::Included(end) => &position <= end,
            Bound::Excluded(end) => &position < end,
            Bound::Unbounded => true,
        })
    }

    fn is_empty(&self) -> bool {
        self.max() <= self.min()
    }
}

// row wrapping for linear bounds
const fn wrap(p: Position, width: usize) -> Position {
    if p.col >= width {
        Position {
            row: p.row + p.col / width,
            col: p.col % width,
        }
    } else { p }
}

impl Selector for usize {
    type Iter = iter::Once<usize>;
    fn iter(self, _of: &Buffer) -> Self::Iter {
        std::iter::once(self)
    }
    fn len(&self, _of: &Buffer) -> usize {
        1
    }
    fn is_empty(&self, _of: &Buffer) -> bool {
        false
    }
}

impl IntoSelectorBounds for usize {
    fn start_bound(&self, of: &Buffer) -> Bound<Position> {
        Bound::Included(Position { row: self / of.width, col: self % of.width })
    }

    fn end_bound(&self, of: &Buffer) -> Bound<Position> {
        Bound::Included(Position::new(self / of.width, self % of.width))
    }
}

impl Selector for Position {
    type Iter = std::iter::Once<usize>;
    fn iter(self, of: &Buffer) -> Self::Iter {
        std::iter::once(of.index_of(self))
    }
    fn len(&self, of: &Buffer) -> usize {
        1
    }
    fn is_empty(&self, of: &Buffer) -> bool {
        false
    }
}

impl SelectorBounds for Position {
    fn start_bound(&self, _of: &Buffer) -> Bound<Position> {
        Bound::Included(*self)
    }
    fn end_bound(&self, _of: &Buffer) -> Bound<Position> {
        Bound::Included(*self)
    }
}

impl Selector for Row {
    type Iter =Range<usize>;
    fn iter(self, of: &Buffer) -> Self::Iter {
        let start = self.0 * of.width;
        start..start + of.width
    }

    fn len(&self, _of: &Buffer) -> usize {
        self.iter(_of).len()
    }

    fn is_empty(&self, _of: &Buffer) -> bool {
        false
    }
}

impl SelectorBounds for Row {
    fn start_bound(&self, of: &Buffer) -> Bound<Pos> {
        Bound::Included(Position::new(self.0, 0))
    }
    fn end_bound(&self, of: &Buffer) -> Bound<Pos> {
        Bound::Included(Position::new(self.0 + 1, of.width))
    }
}

impl Selector for Col {
    type Iter = iter::StepBy<Range<usize>>;
    fn iter(self, of: &Buffer) -> Self::Iter {
        (self.0..of.len()).step_by(of.width)
    }

    fn len(&self, _of: &Buffer) -> usize {
        _of.height
    }

    fn is_empty(&self, _of: &Buffer) -> bool {
        false
    }
}

impl SelectorBounds for Col {
    fn start_bound(&self, _of: &Buffer) -> Bound<Position> {
        Bound::Included(Position::new(0, self.0))
    }
    fn end_bound(&self, _of: &Buffer) -> Bound<Position> {
        Bound::Included(Position::new(1, self.0 + 1))
    }
}

impl Selector for (Bound<Position>, Bound<Position>) {
    type Iter =  BoundsIter;

    fn iter(self, of: &Buffer) -> Self::Iter {
        SelectorBounds::into_concrete_bounds(self, of).iter(of)
    }

    fn len(&self, of: &Buffer) -> usize {
        SelectorBounds::into_concrete_bounds(self, of).area()
    }
}
impl SelectorBounds for (Bound<Position>, Bound<Position>) {
    fn start_bound(&self, _of: &Buffer) -> Bound<Position> {
        self.0
    }
    fn end_bound(&self, _of: &Buffer) -> Bound<Position> {
        self.1
    }
}

impl Selector for (Bound<&Position>, Bound<&Position>) {
    type Iter =  BoundsIter;

    fn iter(self, of: &Buffer) -> Self::Iter {
        SelectorBounds::into_concrete_bounds(self, of).iter(of)
    }

    fn len(&self, of: &Buffer) -> usize {
        SelectorBounds::into_concrete_bounds(self, of).area()
    }
}
impl SelectorBounds for (Bound<Position>, Bound<Position>) {
    fn start_bound(&self, _of: &Buffer) -> Bound<Position> {
        self.0
    }
    fn end_bound(&self, _of: &Buffer) -> Bound<Position> {
        self.1
    }
}
impl Selector for Bounds {
    type Iter = BoundsIter;

    fn iter(self, of: &Buffer) -> Self::Iter {
        BoundsIter::new(self, of.width)
    }

    fn len(&self, of: &Buffer) -> usize {
        self.area()
    }

    fn is_empty(&self, of: &Buffer) -> bool {
        self.area() == 0
    }
}
impl SelectorBounds for Bounds {
    fn start_bound(&self, _of: &Buffer) -> Bound<Position> {
        Bound::Included(self.min)
    }

    fn end_bound(&self, _of: &Buffer) -> Bound<Position> {
        Bound::Excluded(self.max)
    }
}

impl<'a, S: Selector + Copy> Selector for &'a S {
    type Iter = S::Iter;
    fn iter(self, of: &Buffer) -> Self::Iter { (*self).iter(of) }
    fn len(&self, of: &Buffer) -> usize { (*self).len(of) }
    fn is_empty(&self, of: &Buffer) -> bool { (*self).is_empty(of) }
}
impl<'a, S: SelectorBounds + Copy> SelectorBounds for &'a S {
    fn start_bound(&self, of: &Buffer) -> Bound<Position> {
        (*self).start_bound(of)
    }
    fn end_bound(&self, of: &Buffer) -> Bound<Position> {
        (*self).end_bound(of)
    }
}