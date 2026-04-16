use std::ops::{Range, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};
use crate::{Point, Row, Column, Bounded, PointLike, Position, PositionLike};

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

// ── Generic blanket impls for ranges ──────────────────────────────────
//
// Any `Resolve<T, U>` automatically extends to all five range kinds.
// For range types whose start/end are themselves resolvable indices
// (e.g. `Row`, `Point`, `Position`), the resulting `Range<U>` is the
// half-open span between the resolved endpoints.

impl<B, T, U> Resolve<Range<T>, Range<U>> for B
where B: Resolve<T, U> {
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
impl<B: Bounded> Resolve<Point, usize> for B {
    fn resolve(&self, value: Point) -> usize {
        (value.y - self.min_y()) as usize * self.width() as usize
            + (value.x - self.min_x()) as usize
    }
}

impl<B: Bounded> Resolve<Point, Row> for B {
    fn resolve(&self, value: Point) -> Row {
        Row((value.y - self.min_y()) as usize)
    }
}

impl<B: Bounded> Resolve<Point, Column> for B {
    fn resolve(&self, value: Point) -> Column {
        Column((value.x - self.min_x()) as usize)
    }
}

// ── PointLike -> * ────────────────────────────────────────────────────
impl<B: Bounded> Resolve<PointLike, usize> for B {
    fn resolve(&self, value: PointLike) -> usize {
        (value.1 - self.min_y()) as usize * self.width() as usize
            + (value.0 - self.min_x()) as usize
    }
}

impl<B: Bounded> Resolve<PointLike, Row> for B {
    fn resolve(&self, value: PointLike) -> Row {
        Row((value.1 - self.min_y()) as usize)
    }
}

impl<B: Bounded> Resolve<PointLike, Column> for B {
    fn resolve(&self, value: PointLike) -> Column {
        Column((value.0 - self.min_x()) as usize)
    }
}

// ── Position -> * ─────────────────────────────────────────────────────
impl<B: Bounded> Resolve<Position, usize> for B {
    fn resolve(&self, value: Position) -> usize {
        (value.row - self.min_y() as usize) * self.width() as usize
            + (value.col - self.min_x() as usize)
    }
}

impl<B: Bounded> Resolve<Position, Row> for B {
    fn resolve(&self, value: Position) -> Row {
        Row(value.row - self.min_y() as usize)
    }
}

impl<B: Bounded> Resolve<Position, Column> for B {
    fn resolve(&self, value: Position) -> Column {
        Column(value.col - self.min_x() as usize)
    }
}

// ── PositionLike -> * ─────────────────────────────────────────────────
impl<B: Bounded> Resolve<PositionLike, usize> for B {
    fn resolve(&self, value: PositionLike) -> usize {
        (value.0 - self.min_y() as usize) * self.width() as usize
            + (value.1 - self.min_x() as usize)
    }
}

impl<B: Bounded> Resolve<PositionLike, Row> for B {
    fn resolve(&self, value: PositionLike) -> Row {
        Row(value.0 - self.min_y() as usize)
    }
}

impl<B: Bounded> Resolve<PositionLike, Column> for B {
    fn resolve(&self, value: PositionLike) -> Column {
        Column(value.1 - self.min_x() as usize)
    }
}

// ── Row -> * ──────────────────────────────────────────────────────────
impl<B: Bounded> Resolve<Row, Point> for B {
    fn resolve(&self, value: Row) -> Point {
        Point::new(self.min_x(), value.value() as u16)
    }
}

/// Start-of-row cell index. With this in place, the blanket
/// `Resolve<Range<T>, Range<U>>` impl correctly produces the
/// cell range covering `start..end` rows.
impl<B: Bounded> Resolve<Row, usize> for B {
    fn resolve(&self, value: Row) -> usize {
        (value.value() - self.min_y() as usize) * self.width() as usize
    }
}

impl<B: Bounded> Resolve<Row, Column> for B {
    fn resolve(&self, _value: Row) -> Column {
        Column(0)
    }
}

/// Single-row slice: `start..start + width`.
impl<B: Bounded> Resolve<Row, Range<usize>> for B {
    fn resolve(&self, value: Row) -> Range<usize> {
        let start: usize = self.resolve(value);
        let width = self.width() as usize;
        start..start + width
    }
}

// ── Column -> * ───────────────────────────────────────────────────────
impl<B: Bounded> Resolve<Column, Point> for B {
    fn resolve(&self, value: Column) -> Point {
        Point::new(value.value() as u16, 0)
    }
}

impl<B: Bounded> Resolve<Column, Position> for B {
    fn resolve(&self, value: Column) -> Position {
        Position::new(value.value(), 0)
    }
}

impl<B: Bounded> Resolve<Column, usize> for B {
    fn resolve(&self, value: Column) -> usize {
        value.value() - self.min_x() as usize
    }
}

impl<B: Bounded> Resolve<Column, Row> for B {
    fn resolve(&self, _value: Column) -> Row {
        Row(0)
    }
}

// ── Index -> * ────────────────────────────────────────────────────────
impl<B: Bounded> Resolve<usize, Point> for B {
    fn resolve(&self, value: usize) -> Point {
        let w = self.width() as usize;

        Point::new(
            (value % w) as u16 + self.min_x(),
            (value / w) as u16 + self.min_y(),
        )
    }
}

impl<B: Bounded> Resolve<usize, PointLike> for B {
    fn resolve(&self, value: usize) -> PointLike {
        let w = self.width() as usize;

        (
            (value % w) as u16 + self.min_x(),
            (value / w) as u16 + self.min_y(),
        )
    }
}

impl<B: Bounded> Resolve<usize, Position> for B {
    fn resolve(&self, value: usize) -> Position {
        let w = self.width() as usize;

        Position::new(
            (value / w) + self.min_y() as usize,
            (value % w) + self.min_x() as usize,
        )
    }
}

impl<B: Bounded> Resolve<usize, PositionLike> for B {
    fn resolve(&self, value: usize) -> PositionLike {
        let w = self.width() as usize;

        (
            (value / w) + self.min_y() as usize,
            (value % w) + self.min_x() as usize,
        )
    }
}

impl<B: Bounded> Resolve<usize, Row> for B {
    fn resolve(&self, value: usize) -> Row {
        Row(value / self.width() as usize)
    }
}

impl<B: Bounded> Resolve<usize, Column> for B {
    fn resolve(&self, value: usize) -> Column {
        Column(value % self.width() as usize)
    }
}
