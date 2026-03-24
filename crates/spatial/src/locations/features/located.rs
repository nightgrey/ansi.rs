use std::marker::Destruct;
use crate::{Area, Column, IntoLocation, Steps, Location, Position, Row, Range, Step};

/// A location paired with its spatial context.
///
/// `Located` is the universal wrapper for "I have a location and I know
/// what space it lives in." It collapses the `StepWithin`, `IntoLocationWithin`,
/// and `SpanWithin` patterns into inherent methods on a single type.
///
/// # Examples
///
/// ```rust
/// # use spatial::{Area, Position, Located};
/// let area = Area::bounds(0, 0, 80, 24);
/// let loc = Located::new(Position::new(0, 0), area);
///
/// // Step forward
/// let next = loc.forward_checked(1);
/// assert_eq!(next.unwrap().value, Position::new(0, 1));
///
/// // Convert to index
/// assert_eq!(loc.into_index(), 0);
/// ```
#[derive(Copy, Debug)]
#[derive_const(Clone)]
pub struct Located<T = Position, Ctx = Area> {
    pub value: T,
    pub context: Ctx,
}

// ─── Construction ──────────────────────────────────────────────────────

impl<T, Ctx> const Located<T, Ctx> {
    #[inline]
    pub fn new(value: T, context: Ctx) -> Self {
        Self { value, context }
    }

    /// Replace the inner value, keeping the same context.
    #[inline]
    pub fn map<U>(self, f: impl [const] FnOnce(T) -> U) -> Located<U, Ctx> where Self:  Sized + [const] Destruct {
        Located { value: f(self.value), context: self.context }
    }

    /// Access the context.
    #[inline]
    pub fn context(&self) -> &Ctx {
        &self.context
    }
}

// ─── Step (forward / backward) ─────────────────────────────────────────

impl<T: Copy, Ctx: ~const Step<T>> const Located<T, Ctx> where Self: Sized + [const] Destruct {
    #[inline]
    pub fn forward_checked(self, count: usize) -> Option<Self> {
        match self.context.forward_checked(self.value, count) {
            Some(next) => Some(Located { value: next, context: self.context }),
            None => None,
        }
    }

    #[inline]
    pub fn forward(self, count: usize) -> Self {
        Located { value: self.context.forward(self.value, count), context: self.context }
    }

    #[inline]
    pub unsafe fn forward_unchecked(self, count: usize) -> Self {
        Located { value: self.context.forward_unchecked(self.value, count), context: self.context }
    }

    #[inline]
    pub fn backward_checked(self, count: usize) -> Option<Self> {
        match self.context.backward_checked(self.value, count) {
            Some(prev) => Some(Located { value: prev, context: self.context }),
            None => None,
        }
    }

    #[inline]
    pub fn backward(self, count: usize) -> Self {
        Located { value: self.context.backward(self.value, count), context: self.context }
    }

    #[inline]
    pub unsafe fn backward_unchecked(self, count: usize) -> Self {
        Located { value: self.context.backward_unchecked(self.value, count), context: self.context }
    }

    #[inline]
    pub fn steps_between(&self, other: T) -> (usize, Option<usize>) {
        self.context.steps_between(self.value, other)
    }
}


// ─── IntoLocation (conversion) ─────────────────────────────────────────

impl<T: Copy, Ctx: ~const IntoLocation<T>> const Located<T, Ctx> where Self: Sized + [const] Destruct {
    #[inline]
    pub fn into_index(&self) -> usize {
        self.context.into_index(self.value)
    }

    #[inline]
    pub fn into_position(&self) -> Position {
        self.context.into_position(self.value)
    }

    #[inline]
    pub fn into_row(&self) -> Row {
        self.context.into_row(self.value)
    }

    #[inline]
    pub fn into_col(&self) -> Column {
        self.context.into_col(self.value)
    }
}

// ─── Span (index ranges) ──────────────────────────────────────────────

impl<T: Copy, Ctx: ~const Range<T>> const Located<T, Ctx> where Self: Sized + [const] Destruct {
    #[inline]
    pub fn start(&self) -> usize {
        self.context.start(self.value)
    }

    #[inline]
    pub fn end(&self) -> usize {
        self.context.end(self.value)
    }

    #[inline]
    pub fn range(&self) -> std::ops::Range<usize> where T: [const] Clone {
        self.context.range(self.value)
    }
}

// ─── Deref to inner value ──────────────────────────────────────────────

impl<T, Ctx> const std::ops::Deref for Located<T, Ctx> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T, Ctx> AsRef<T> for Located<T, Ctx> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

// ─── Convenience constructors on Bounds ────────────────────────────────

impl Area {
    /// Wrap a location with this bounds as context.
    #[inline]
    pub fn locate<T>(&self, value: T) -> Located<T, &Self> {
        Located::new(value, self)
    }

    /// Wrap a location with an owned copy of this bounds as context.
    #[inline]
    pub fn located<T>(self, value: T) -> Located<T, Self> {
        Located::new(value, self)
    }
}
