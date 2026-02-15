use std::iter::FusedIterator;
use crate::{Position, Size};
use std::ops::{IntoBounds, Bound, Bound::*, RangeBounds, Deref, DerefMut, Sub};
use crate::region::Region;

// ─── SpatialContext / SpatialStep ────────────────────────────────────────────

/// Provides the spatial context needed to step through positions in row-major
/// order within a bounded 2D region.
///
/// This is the "grid" that gives meaning to forward/backward movement —
/// without it, a bare `Position` doesn't know when to wrap to the next row.
pub const trait SpatialContext<T = Position> {
    /// Number of row-major steps from `start` to `end`.
    ///
    /// Returns `(n, Some(n))` when `start <= end` within bounds,
    /// or `(0, None)` when `start > end`.
    fn steps_between(&self, start: &T, end: &T) -> (usize, Option<usize>);

    /// Move `count` steps forward in row-major order, or `None` if out of bounds.
    fn forward_checked(&self, start: T, count: usize) -> Option<T>;

    /// Like `forward_checked`, but panics on overflow.
    fn forward(&self, start: T, count: usize) -> T {
        self.forward_checked(start, count)
            .expect("overflow in SpatialContext::forward")
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
            .expect("underflow in SpatialContext::backward")
    }

    /// Like `backward_checked`, without bounds checking.
    ///
    /// # Safety
    /// The result must remain within bounds.
    unsafe fn backward_unchecked(&self, start: T, count: usize) -> T {
        self.backward(start, count)
    }
}

/// A trait for types that can be stepped through within spatial bounds.
///
/// This is the spatial analogue of `Step`, but requires a context (bounds)
/// to determine valid steps. Forward stepping follows row-major order
/// (left to right within rows, top to bottom between rows).
///
/// # Type Parameters
/// * `Context` - The bounds type that provides stepping context
///
/// # Implementation Note
/// Implementors must ensure that all methods respect the bounds context
/// and that forward steps always move in row-major order.
/// A type that can be stepped through a spatial context in row-major order.
///
/// Delegates to [`SpatialContext`] — this trait exists so you can write
/// `Position::forward_checked(pos, n, &region)` in a generic context.
pub const trait SpatialStep: Clone + Sized {
    type Context: ~const SpatialContext<Self>;

    fn steps_between(start: &Self, end: &Self, ctx: &Self::Context) -> (usize, Option<usize>) {
        ctx.steps_between(start, end)
    }

    fn forward_checked(start: Self, count: usize, ctx: &Self::Context) -> Option<Self> {
        ctx.forward_checked(start, count)
    }

    fn forward(start: Self, count: usize, ctx: &Self::Context) -> Self {
        ctx.forward(start, count)
    }

    unsafe fn forward_unchecked(start: Self, count: usize, ctx: &Self::Context) -> Self {
        ctx.forward_unchecked(start, count)
    }

    fn backward_checked(start: Self, count: usize, ctx: &Self::Context) -> Option<Self> {
        ctx.backward_checked(start, count)
    }

    fn backward(start: Self, count: usize, ctx: &Self::Context) -> Self {
        ctx.backward(start, count)
    }

    unsafe fn backward_unchecked(start: Self, count: usize, ctx: &Self::Context) -> Self {
        ctx.backward_unchecked(start, count)
    }
}
