use crate::{Bound, Column, Coordinate, Point, Position, Row};
use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};
use number::SaturatingSub;

/// Resolve a context-dependent value.
///
/// # Example
///
/// ```rust
/// # use geometry::{Resolve, Point};
///
/// struct Context {
///     inner: Vec<usize>,
///     width: usize,
///     height: usize
/// }
///
/// # impl Context {
/// #   pub fn new(width: usize, height: usize) -> Self {
/// #     Self { inner: vec![0; width * height], width, height }
/// #   }
/// # }
/// #
/// impl Resolve<Point, usize> for Context {
///     fn resolve(&self, value: Point) -> usize {
///         value.y as usize * self.width + value.x as usize
///     }
/// }
///
/// let ctx = Context::new(5, 5);
/// assert_eq!(ctx.inner.len(), 25);
/// assert_eq!(ctx.resolve(Point::new(1, 2)), 11) // 2 * 5 + 1
/// ```
pub trait Resolve<T, U> {
    /// Given [`T`], resolve [`U`] within [`Self`].
    fn resolve(&self, value: T) -> U;
    /// If possible, given [`T`], resolve [`U`] within [`Self`].
    fn try_resolve(&self, value: T) -> Option<U> {
        Some(self.resolve(value))
    }
}


// Coordinate
impl<B: Bound, P: Coordinate> Resolve<P, usize> for B {
    fn resolve(&self, value: P) -> usize {
        (value.y( ) * self.width() + value.x()) as usize
    }
}

impl<B: Bound, P: Coordinate> Resolve<P, Row> for B {
    fn resolve(&self, value: P) -> Row {
        Row((value.y() as usize))
    }
}

impl<B: Bound, P: Coordinate> Resolve<P, Column> for B {
    fn resolve(&self, value: P) -> Column {
            Column((value.x() as usize))
    }
}

impl<B: Bound> Resolve<Row, usize> for B {
    fn resolve(&self, value: Row) -> usize {
        (value.into_inner()) * self.width() as usize
    }
}
impl<B: Bound, P: Coordinate> Resolve<Row, P> for B {
    fn resolve(&self, value: Row) -> P {
        P::new(self.min_x(), (value.into_inner() as u16))
    }
}

impl<B: Bound> Resolve<Row, Column> for B {
    fn resolve(&self, _value: Row) -> Column {
        Column(0)
    }
}

impl<B: Bound> Resolve<Row, Range<usize>> for B {
    fn resolve(&self, value: Row) -> Range<usize> {
        let start: usize = self.resolve(value);
        let width = self.width() as usize;
        start..start + width
    }
}

// Column
impl<B: Bound, P: Coordinate> Resolve<Column, P> for B {
    fn resolve(&self, value: Column) -> P {
        P::new((value.into_inner() as u16), 0)
    }
}

impl<B: Bound> Resolve<Column, usize> for B {
    fn resolve(&self, value: Column) -> usize {
        value.into_inner()
    }
}

impl<B: Bound> Resolve<Column, Row> for B {
    fn resolve(&self, _value: Column) -> Row {
        Row(0)
    }
}

// usize
impl<B: Bound, P: Coordinate> Resolve<usize, P> for B {
    fn resolve(&self, value: usize) -> P {
        let value = value as u16;
        let w = self.width();

        P::new(
            (value % w),
            (value / w),
        )
    }
}
impl<B: Bound> Resolve<usize, usize> for B {
    fn resolve(&self, value: usize) -> usize {
        value
    }
}

impl<B: Bound> Resolve<usize, Row> for B {
    fn resolve(&self, value: usize) -> Row {
        Row((value / self.width() as usize))
    }
}

impl<B: Bound> Resolve<usize, Column> for B {
    fn resolve(&self, value: usize) -> Column {
        Column(value % self.width() as usize)
    }
}


// ── Ranges ──────────────────────────────────
//
// Any `Resolve<T, U>` automatically extends to all five range kinds.
// For range types whose start/end are themselves resolvable indices
// (e.g. `Row`, `Point`, `Position`), the resulting `Range<U>` is the
// half-open span between the resolved endpoints.
impl<B, T, U> Resolve<Range<T>, Range<U>> for B
where
    B: Resolve<T, U>,
{
    fn resolve(&self, value: Range<T>) -> Range<U> {
        let start: U = self.resolve(value.start);
        let end: U = self.resolve(value.end);
        start..end
    }
}

impl<B, T: Clone, U> Resolve<RangeInclusive<T>, RangeInclusive<U>> for B
where
    B: Resolve<T, U>,
{
    fn resolve(&self, value: RangeInclusive<T>) -> RangeInclusive<U> {
        self.resolve(value.start().clone())..=self.resolve(value.end().clone())
    }
}

impl<B, T, U> Resolve<RangeTo<T>, RangeTo<U>> for B
where
    B: Resolve<T, U>,
{
    fn resolve(&self, value: RangeTo<T>) -> RangeTo<U> {
        ..self.resolve(value.end)
    }
}

impl<B, T, U> Resolve<RangeToInclusive<T>, RangeToInclusive<U>> for B
where
    B: Resolve<T, U>,
{
    fn resolve(&self, value: RangeToInclusive<T>) -> RangeToInclusive<U> {
        ..=self.resolve(value.end)
    }
}

impl<B, T, U> Resolve<RangeFrom<T>, RangeFrom<U>> for B
where
    B: Resolve<T, U>,
{
    fn resolve(&self, value: RangeFrom<T>) -> RangeFrom<U> {
        self.resolve(value.start)..
    }
}

impl<B: Bound> Resolve<RangeFull, RangeFull> for B {
    fn resolve(&self, _: RangeFull) -> RangeFull {
        ..
    }
}

impl<B: Bound> Resolve<RangeFull, Range<usize>> for B {
    fn resolve(&self, _: RangeFull) -> Range<usize> {
        self.min_x() as usize..self.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::traits::bound::Bound as _;
    use super::*;

    type Bound = crate::Rect;

    fn ctx() -> Bound {
        Bound::new(50, 50, 50, 50)
    }

    macro_rules! assert_resolve {
        ($ctx:tt, $value:expr => $expected:expr, $ty:ty) => {{
            let resolved: $ty = $ctx.resolve($value);
            assert_eq!(resolved, $expected, "expected {:?} but got {:?}", $expected, resolved);
        }};
       ($ctx:expr, $value:tt => $expected:tt) => {{
            let resolved = $ctx.resolve($value);
            assert_eq!(resolved, $expected, "expected {:?} but got {:?}", $expected, resolved);
        }};
    }

    #[test]
    fn resolve_index() {
        let ctx = ctx();

        assert_resolve!(ctx, 50 => Position::new(1, 0), Position);
        assert_resolve!(ctx, 50 => Point::new(0, 1), Point);

        assert_resolve!(ctx, 0 => Row(0), Row);

        assert_resolve!(ctx, 49 => Row(0), Row);
        assert_resolve!(ctx, 50 => Row(1), Row);
        assert_resolve!(ctx, 99 => Row(1), Row);
        assert_resolve!(ctx, 100 => Row(2), Row);

        assert_resolve!(ctx, 0 => Column(0), Column);
        assert_resolve!(ctx, 49 => Column(49), Column);
        assert_resolve!(ctx, 50 => Column(0), Column);
        assert_resolve!(ctx, 99 => Column(49), Column);
        assert_resolve!(ctx, 100 => Column(0), Column);
    }

    #[test]
    fn resolve_point() {
        let ctx = ctx();
        assert_resolve!(ctx, Point::new(0, 0) => 0, usize);
        assert_resolve!(ctx, Point::new(50, 50) => 50 * 50 + 50, usize);
        assert_resolve!(ctx, Point::new(25, 25) => 25 * 50 + 25, usize);
        assert_resolve!(ctx, Point::new(10, 10) => 10 * 50 + 10, usize);
    }

    #[test]
    fn resolve_position() {
        let ctx = ctx();
        assert_resolve!(ctx, Position::new(0, 0) => 0, usize);
        assert_resolve!(ctx, Position::new(50, 50) => 50 * 50 + 50, usize);
        assert_resolve!(ctx, Position::new(25, 25) => 25 * 50 + 25, usize);
        assert_resolve!(ctx, Position::new(10, 10) => 10 * 50 + 10, usize);
    }

    #[test]
    fn resolve_row() {
        let ctx = ctx();
        assert_resolve!(ctx, Row(0) => 0, usize);
        assert_resolve!(ctx, Row(25) => 50 * 25, usize);
        assert_resolve!(ctx, Row(50) => 50 * 50, usize);
        assert_resolve!(ctx, Row(10) => 50 * 10, usize);
    }

    #[test]
    fn resolve_column() {
        let ctx = ctx();
        assert_resolve!(ctx, Column(0) => 0, usize);
        assert_resolve!(ctx, Column(25) => 25, usize);
        assert_resolve!(ctx, Column(50) => 50, usize);
        assert_resolve!(ctx, Column(10) => 10, usize);
    }

    #[test]
    fn resolve_ranges() {
        let ctx = ctx();
        assert_resolve!(ctx, 0..50 => 0..50, Range<usize>);
        assert_resolve!(ctx, 50..100 => 50..100, Range<usize>);
        assert_resolve!(ctx, 100..150 => 100..150, Range<usize>);

        assert_resolve!(ctx, 50 => Row(1), Row);
        assert_resolve!(ctx, 0..50 => Row(0)..Row(1), Range<Row>);
        assert_resolve!(ctx, 0..43 => Column(0)..Column(43), Range<Column>);
        assert_resolve!(ctx, Row(21)..Row(20) => Column(0)..Column(0), Range<Column>);
    }
}