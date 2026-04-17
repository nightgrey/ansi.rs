use std::ops::{Range, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};
use crate::{Point, Row, Column, Bound, PointLike, Position, PositionLike, Location};

/// Resolve a context-dependent value.
///
/// This trait is used to convert values from one context to another.
///
/// # Example
///
/// ```rust
/// use geometry::Resolve;
///
/// struct MyContext<T> {
///     inner: Vec<T>,
///     width: usize,
/// }
///
/// impl<T> Resolve<(usize, usize), usize> for MyContext<T> {
///     fn resolve(&self, value: (usize, usize)) -> usize {
///         value.1 * self.width + value.0
///     }
/// }
///
/// let ctx = MyContext { inner: Vec::from_iter(0..50), width: 5 };
///
/// let index = ctx.resolve((5, 0)); // Result: 25
/// let data = &ctx.inner[index]; // Result: &25
/// ```
pub trait Resolve<T, U> {
    /// Resolve value within context of [`Self`].
    fn resolve(&self, value: T) -> U;
}

// ── Ranges ──────────────────────────────────
//
// Any `Resolve<T, U>` automatically extends to all five range kinds.
// For range types whose start/end are themselves resolvable indices
// (e.g. `Row`, `Point`, `Position`), the resulting `Range<U>` is the
// half-open span between the resolved endpoints.

impl<B, T, U> Resolve<Range<T>, Range<U>> for B where B: Resolve<T, U> {
    fn resolve(&self, value: Range<T>) -> Range<U> {
        self.resolve(value.start)..self.resolve(value.end)
    }
}

impl<B, T: Clone, U> Resolve<RangeInclusive<T>, RangeInclusive<U>> for B
where B: Resolve<T, U> {
    fn resolve(&self, value: RangeInclusive<T>) -> RangeInclusive<U> {
        self.resolve(value.start().clone())..=self.resolve(value.end().clone())
    }
}

impl<B, T, U> Resolve<RangeTo<T>, RangeTo<U>> for B
where B: Resolve<T, U> {
    fn resolve(&self, value: RangeTo<T>) -> RangeTo<U> {
        ..self.resolve(value.end)
    }
}

impl<B, T, U> Resolve<RangeToInclusive<T>, RangeToInclusive<U>> for B
where B: Resolve<T, U> {
    fn resolve(&self, value: RangeToInclusive<T>) -> RangeToInclusive<U> {
        ..=self.resolve(value.end)
    }
}

impl<B, T, U> Resolve<RangeFrom<T>, RangeFrom<U>> for B
where B: Resolve<T, U> {
    fn resolve(&self, value: RangeFrom<T>) -> RangeFrom<U> {
        self.resolve(value.start)..
    }
}

// ── Point -> * ────────────────────────────────────────────────────────
impl<B: Bound, P: Location> Resolve<P, usize> for B {
    fn resolve(&self, value: P) -> usize {
        (value.y() - self.min_y()) as usize * self.width() as usize
            + (value.x() - self.min_x()) as usize
    }
}

impl<B: Bound, P: Location> Resolve<P, Row> for B {
    fn resolve(&self, value: P) -> Row {
        Row((value.y() - self.min_y()) as usize)
    }
}

impl<B: Bound, P: Location> Resolve<P, Column> for B {
    fn resolve(&self, value: P) -> Column {
        Column((value.x() - self.min_x()) as usize)
    }
}

// ── Row -> * ──────────────────────────────────────────────────────────
impl<B: Bound, P: Location> Resolve<Row, P> for B {
    fn resolve(&self, value: Row) -> P {
        P::new(self.min_x(), value.into_inner() as u16)
    }
}

/// Start-of-row cell index. With this in place, the blanket
/// `Resolve<Range<T>, Range<U>>` impl correctly produces the
/// cell range covering `start..end` rows.
impl<B: Bound> Resolve<Row, usize> for B {
    fn resolve(&self, value: Row) -> usize {
        (value.into_inner() - self.min_y() as usize) * self.width() as usize
    }
}

impl<B: Bound> Resolve<Row, Column> for B {
    fn resolve(&self, _value: Row) -> Column {
        Column(0)
    }
}

/// Single-row slice: `start..start + width`.
impl<B: Bound> Resolve<Row, Range<usize>> for B {
    fn resolve(&self, value: Row) -> Range<usize> {
        let start: usize = self.resolve(value);
        let width = self.width() as usize;
        start..start + width
    }
}

// ── Column -> * ───────────────────────────────────────────────────────
impl<B: Bound, P: Location> Resolve<Column, P> for B {
    fn resolve(&self, value: Column) -> P {
        P::new(value.into_inner() as u16, 0)
    }
}

impl<B: Bound> Resolve<Column, usize> for B {
    fn resolve(&self, value: Column) -> usize {
        value.into_inner() - self.min_x() as usize
    }
}

impl<B: Bound> Resolve<Column, Row> for B {
    fn resolve(&self, _value: Column) -> Row {
        Row(0)
    }
}

// ── Index -> * ────────────────────────────────────────────────────────
impl<B: Bound, P: Location> Resolve<usize, P> for B {
    fn resolve(&self, value: usize) -> P {
        let w = self.width() as usize;

        P::new(
            (value % w) as u16 + self.min_x(),
            (value / w) as u16 + self.min_y(),
        )
    }
}

impl<B: Bound> Resolve<usize, Row> for B {
    fn resolve(&self, value: usize) -> Row {
        Row(value / self.width() as usize)
    }
}

impl<B: Bound> Resolve<usize, Column> for B {
    fn resolve(&self, value: usize) -> Column {
        Column(value % self.width() as usize)
    }
}
