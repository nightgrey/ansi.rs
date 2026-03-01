use crate::{Column, IntoLocation, Position, Row, Location, Bounds};

/// Provides the spatial context needed to step through positions in row-major
/// order within a bounded 2D region.
///
/// This is the "grid" that gives meaning to forward/backward movement —
/// without it, a bare `Position` doesn't know when to wrap to the next row.
pub const trait Step<T = Position> {
    /// Number of row-major steps from `start` to `end`.
    ///
    /// Returns `(n, Some(n))` when `start <= end` within bounds,
    /// or `(0, None)` when `start > end`.
    fn steps_between(&self, start: T, end: T) -> (usize, Option<usize>);

    /// Move `count` steps forward in row-major order, or `None` if out of bounds.
    fn forward_checked(&self, start: T, count: usize) -> Option<T>;

    /// Like `forward_checked`, but panics on overflow.
    fn forward(&self, start: T, count: usize) -> T {
        self.forward_checked(start, count)
            .expect("overflow in Step::forward")
    }

    /// Like `forward_checked`, without bounds checking.
    ///
    /// # Safety
    /// The result must remain within bounds.
    unsafe fn forward_unchecked(&self, start: T, count: usize) -> T {
        self.forward(start, count)
    }

    /// Move `count` steps backward in row-major order, or `None` if out of bounds.
    fn backward_checked(&self, start: T, count: usize) -> Option<T>;

    /// Like `backward_checked`, but panics on underflow.
    fn backward(&self, start: T, count: usize) -> T {
        self.backward_checked(start, count)
            .expect("underflow in Step::backward")
    }

    /// Like `backward_checked`, without bounds checking.
    ///
    /// # Safety
    /// The result must remain within bounds.
    unsafe fn backward_unchecked(&self, start: T, count: usize) -> T {
        self.backward(start, count)
    }
}

impl const Step<Position> for Bounds {
    fn steps_between(&self, start: Position, end: Position) -> (usize, Option<usize>) {
        if start > end {
            return (0, None);
        }
        let current = self.into_index(start);
        let remaining = self.into_index(end);

        let dist = remaining - current;

        (dist, Some(dist))
    }

    fn forward_checked(&self, start: Position, count: usize) -> Option<Position> {
        // Fast path for single step (Iterator usage).
        if count == 1 {
            let mut next = start;
            next.col += 1;

            if next.col >= self.max.col {
                next.col = self.min.col;
                next.row += 1;

                if next.row >= self.max.row {
                    return None;
                }
            }

            return Some(next);
        }

        // General path for arbitrary steps.
        let index = self.into_index(start).checked_add(count)?;
        if index >= self.area() {
            return None;
        }

        Some(self.into_position(index))
    }

    fn backward_checked(&self, start: Position, count: usize) -> Option<Position> {
        // Fast path: stay on the same row.
        if start.row < self.max.row && count <= start.col - self.min.col {
            return Some(Position::new(start.row, start.col - count));
        }
        // General path: linearize through the exclusive end.
        let idx = if start >= self.max { self.area() } else { self.into_index(start) };
        let target = idx.checked_sub(count)?;
        Some(self.into_position(target))
    }
}

impl const Step<Row> for Bounds {
    fn steps_between(&self, start: Row, end: Row) -> (usize, Option<usize>) {
        if start.value() <= end.value() {
            let steps = end.value() - start.value();
            (steps, Some(steps))
        } else {
            (0, None)
        }
    }

    fn forward_checked(&self, start: Row, count: usize) -> Option<Row> {
        let row = start.value().checked_add(count)?;
        if row >= self.max.row {
            return None;
        }
        Some(Row(row))
    }

    fn backward_checked(&self, start: Row, count: usize) -> Option<Row> {
        let row = start.value().checked_sub(count)?;
        if row < self.min.row {
            return None;
        }
        Some(Row(row))
    }
}

impl const Step<Column> for Bounds {
    fn steps_between(&self, start: Column, end: Column) -> (usize, Option<usize>) {
        if start.value() <= end.value() {
            let steps = end.value() - start.value();
            (steps, Some(steps))
        } else {
            (0, None)
        }
    }

    fn forward_checked(&self, start: Column, count: usize) -> Option<Column> {
        let col = start.value().checked_add(count)?;
        if col >= self.max.col {
            return None;
        }
        Some(Column(col))
    }

    fn backward_checked(&self, start: Column, count: usize) -> Option<Column> {
        let col = start.value().checked_sub(count)?;
        if col < self.min.col {
            return None;
        }
        Some(Column(col))
    }
}
