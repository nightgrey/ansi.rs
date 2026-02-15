use std::iter::FusedIterator;
use crate::{Position, Size};
use std::ops::{IntoBounds, Bound, Bound::*, RangeBounds, Deref, DerefMut, Sub};
use crate::region::Region;

/// A trait for spatial contexts that can be stepped through with `T`.
///
/// This is the context required to determine valid steps for `T`.
/// Forward stepping follows row-major order (left to right within rows, top to bottom between rows).
pub const trait SpatialContext<T = Position> {
    /// Returns the bounds on the number of steps required to get from `start` to `end`,
    /// following row-major order within the given bounds.
    ///
    /// Returns `(usize::MAX, None)` if the number of steps would overflow `usize`,
    /// or is infinite.
    ///
    /// # Invariants
    /// * `steps_between(bounds, start, end) == (n, Some(n))` iff
    ///   `forward_checked(bounds, start, n) == Some(end)`
    /// * `steps_between(bounds, start, end) == (n, Some(n))` only if `start <= end` within bounds
    /// * `steps_between(bounds, start, end) == (0, Some(0))` iff `start == end`
    /// * `steps_between(bounds, start, end) == (0, None)` if `start > end` within bounds
    fn steps_between(
        &self,
        start: &T,
        end: &T,
    ) -> (usize, Option<usize>);

    /// Returns the value obtained by taking `count` forward steps
    /// from `start` within `bounds`, following row-major order.
    ///
    /// Returns `None` if the step would go past the bounds.
    ///
    /// # Invariants
    /// * `forward_checked(bounds, a, 0) == Some(a)`
    /// * `forward_checked(bounds, a, n).and_then(|x| forward_checked(bounds, x, m))
    ///   == forward_checked(bounds, a, n.checked_add(m))`
    fn forward_checked(
        &self,
        start: T,
        count: usize,
    ) -> Option<T>;

    /// Returns the value obtained by taking `count` forward steps
    /// from `start` within `bounds`.
    ///
    /// # Panics
    /// Panics if the step would overflow the bounds.
    fn forward(&self, start: T, count: usize) -> T {
        self.forward_checked(start, count)
            .expect("overflow in `SpatialStep::forward`")
    }

    /// Returns the value obtained by taking `count` forward steps
    /// from `start` within `bounds`.
    ///
    /// # Safety
    /// The result must not overflow the bounds. Calling this with
    /// parameters that would overflow is undefined behavior.
    ///
    /// # Invariants
    /// If no overflow occurs, this is equivalent to `forward`.
    unsafe fn forward_unchecked(&self, start: T, count: usize) -> T {
        self.forward(start, count)
    }

    /// Returns the value obtained by taking `count` backward steps
    /// from `start` within `bounds`, following row-major order.
    ///
    /// Returns `None` if the step would go before the bounds.
    fn backward_checked(
        &self,
        start: T,
        count: usize,
    ) -> Option<T>;

    /// Returns the value obtained by taking `count` backward steps
    /// from `start` within `bounds`.
    ///
    /// # Panics
    /// Panics if the step would overflow the bounds.
    fn backward(&self, start: T, count: usize) -> T {
        self.backward_checked(start, count)
            .expect("overflow in `SpatialStep::backward`")
    }

    /// Returns the value obtained by taking `count` backward steps
    /// from `start` within `bounds`.
    ///
    /// # Safety
    /// The result must not underflow the bounds. Calling this with
    /// parameters that would underflow is undefined behavior.
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
pub const trait SpatialStep: Clone + Sized {
    /// The bounds type that provides stepping context.
    type Context: [const] SpatialContext<Self>;

    /// Returns the bounds on the number of steps required to get from `start` to `end`,
    /// following row-major order within the given bounds.
    ///
    /// Returns `(usize::MAX, None)` if the number of steps would overflow `usize`,
    /// or is infinite.
    ///
    /// # Invariants
    /// * `steps_between(context, start, end) == (n, Some(n))` iff
    ///   `forward_checked(context, start, n) == Some(end)`
    /// * `steps_between(context, start, end) == (n, Some(n))` only if `start <= end` within context
    /// * `steps_between(context, start, end) == (0, Some(0))` iff `start == end`
    /// * `steps_between(context, start, end) == (0, None)` if `start > end` within context
    fn steps_between(
        start: &Self,
        end: &Self,
        context: &Self::Context,
    ) -> (usize, Option<usize>) {
        context.steps_between(start, end)
    }

    /// Returns the value obtained by taking `count` forward steps
    /// from `start` within `context`, following row-major order.
    ///
    /// Returns `None` if the step would go past the context.
    ///
    /// # Invariants
    /// * `forward_checked(context, a, 0) == Some(a)`
    /// * `forward_checked(context, a, n).and_then(|x| forward_checked(context, x, m))
    ///   == forward_checked(context, a, n.checked_add(m))`
    fn forward_checked(
        start: Self,
        count: usize,
        context: &Self::Context,
    ) -> Option<Self> {
        context.forward_checked(start, count)
    }

    /// Returns the value obtained by taking `count` forward steps
    /// from `start` within `context`.
    ///
    /// # Panics
    /// Panics if the step would overflow the context.
    fn forward(start: Self, count: usize, context: &Self::Context) -> Self {
        Self::forward_checked(start, count, context)
            .expect("overflow in `SpatialStep::forward`")
    }

    /// Returns the value obtained by taking `count` forward steps
    /// from `start` within `context`.
    ///
    /// # Safety
    /// The result must not overflow the context. Calling this with
    /// parameters that would overflow is undefined behavior.
    ///
    /// # Invariants
    /// If no overflow occurs, this is equivalent to `forward`.
    unsafe fn forward_unchecked(start: Self, count: usize, context: &Self::Context) -> Self {
        Self::forward(start, count, context)
    }

    /// Returns the value obtained by taking `count` backward steps
    /// from `start` within `context`, following row-major order.
    ///
    /// Returns `None` if the step would go before the context.
    fn backward_checked(
        start: Self,
        count: usize,
        context: &Self::Context,
    ) -> Option<Self> {
        context.backward_checked(start, count)
    }

    /// Returns the value obtained by taking `count` backward steps
    /// from `start` within `context`.
    ///
    /// # Panics
    /// Panics if the step would overflow the context.
    fn backward(start: Self, count: usize, context: &Self::Context) -> Self {
        Self::backward_checked(start, count, context)
            .expect("overflow in `SpatialStep::backward`")
    }

    /// Returns the value obtained by taking `count` backward steps
    /// from `start` within `context`.
    ///
    /// # Safety
    /// The result must not underflow the context. Calling this with
    /// parameters that would underflow is undefined behavior.
    unsafe fn backward_unchecked(start: Self, count: usize, context: &Self::Context) -> Self {
        Self::backward(start, count, context)
    }
}
