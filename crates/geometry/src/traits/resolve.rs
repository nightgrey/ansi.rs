use crate::{Bounded, Column, Point, PointLike, Row};
use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

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
impl<B: Bounded> Resolve<Point, usize> for B {
    fn resolve(&self, value: Point) -> usize {
        value.y as usize * self.width() as usize + value.x as usize
    }

    fn try_resolve(&self, value: Point) -> Option<usize> {
        if value.x < self.width() && value.y < self.height() {
            Some(self.resolve(value))
        } else {
            None
        }
    }
}
impl<B: Bounded> Resolve<Point, Row> for B {
    fn resolve(&self, value: Point) -> Row {
        Row(value.y)
    }

    fn try_resolve(&self, value: Point) -> Option<Row> {
        if value.y < self.height() {
            Some(self.resolve(value))
        } else {
            None
        }
    }
}
impl<B: Bounded> Resolve<Point, Column> for B {
    fn resolve(&self, value: Point) -> Column {
        Column(value.x)
    }

    fn try_resolve(&self, value: Point) -> Option<Column> {
        if value.x < self.width() {
            Some(self.resolve(value))
        } else {
            None
        }
    }
}

// Coordinate
impl<B: Bounded> Resolve<PointLike, usize> for B {
    fn resolve(&self, value: PointLike) -> usize {
        value.1 as usize * self.width() as usize + value.0 as usize
    }

    fn try_resolve(&self, value: PointLike) -> Option<usize> {
        if value.0 < self.width() && value.1 < self.height() {
            Some(self.resolve(value))
        } else {
            None
        }
    }
}
impl<B: Bounded> Resolve<PointLike, Row> for B {
    fn resolve(&self, value: PointLike) -> Row {
        Row(value.1)
    }

    fn try_resolve(&self, value: PointLike) -> Option<Row> {
        if value.1 < self.height() {
            Some(self.resolve(value))
        } else {
            None
        }
    }
}
impl<B: Bounded> Resolve<PointLike, Column> for B {
    fn resolve(&self, value: PointLike) -> Column {
        Column(value.0)
    }

    fn try_resolve(&self, value: PointLike) -> Option<Column> {
        if value.0 < self.width() {
            Some(self.resolve(value))
        } else {
            None
        }
    }
}

// Row
impl<B: Bounded> Resolve<Row, usize> for B {
    fn resolve(&self, value: Row) -> usize {
        (value.into_inner() as usize) * self.width() as usize
    }

    fn try_resolve(&self, value: Row) -> Option<usize> {
        if (value.into_inner() as usize) < self.height() as usize {
            Some(self.resolve(value))
        } else {
            None
        }
    }
}
impl<B: Bounded> Resolve<Row, Point> for B {
    fn resolve(&self, value: Row) -> Point {
        Point::new(0, value.into_inner())
    }

    fn try_resolve(&self, value: Row) -> Option<Point> {
        if (value.into_inner() as usize) < self.height() as usize {
            Some(self.resolve(value))
        } else {
            None
        }
    }
}
impl<B: Bounded> Resolve<Row, Column> for B {
    fn resolve(&self, _value: Row) -> Column {
        Column(0)
    }

    fn try_resolve(&self, value: Row) -> Option<Column> {
        if (value.into_inner() as usize) < self.height() as usize {
            Some(self.resolve(value))
        } else {
            None
        }
    }
}
impl<B: Bounded> Resolve<Row, Range<usize>> for B {
    fn resolve(&self, value: Row) -> Range<usize> {
        let start: usize = self.resolve(value);
        let width = self.width() as usize;
        start..start + width
    }

    fn try_resolve(&self, value: Row) -> Option<Range<usize>> {
        if (value.into_inner() as usize) < self.height() as usize {
            Some(self.resolve(value))
        } else {
            None
        }
    }
}

// Column
impl<B: Bounded> Resolve<Column, Point> for B {
    fn resolve(&self, value: Column) -> Point {
        Point::new(value.into_inner(), 0)
    }

    fn try_resolve(&self, value: Column) -> Option<Point> {
        if (value.into_inner() as usize) < self.width() as usize {
            Some(self.resolve(value))
        } else {
            None
        }
    }
}
impl<B: Bounded> Resolve<Column, usize> for B {
    fn resolve(&self, value: Column) -> usize {
        value.into_inner() as usize
    }

    fn try_resolve(&self, value: Column) -> Option<usize> {
        if (value.into_inner() as usize) < self.width() as usize {
            Some(self.resolve(value))
        } else {
            None
        }
    }
}
impl<B: Bounded> Resolve<Column, Row> for B {
    fn resolve(&self, _value: Column) -> Row {
        Row(0)
    }

    fn try_resolve(&self, value: Column) -> Option<Row> {
        if (value.into_inner() as usize) < self.width() as usize {
            Some(self.resolve(value))
        } else {
            None
        }
    }
}

// usize
impl<B: Bounded> Resolve<usize, Point> for B {
    fn resolve(&self, value: usize) -> Point {
        let w = self.width() as usize;

        Point::new((value % w) as u16, (value / w) as u16)
    }

    fn try_resolve(&self, value: usize) -> Option<Point> {
        if value < self.len() {
            Some(self.resolve(value))
        } else {
            None
        }
    }
}
impl<B: Bounded> Resolve<usize, usize> for B {
    fn resolve(&self, value: usize) -> usize {
        value
    }
}
impl<B: Bounded> Resolve<usize, Row> for B {
    fn resolve(&self, value: usize) -> Row {
        Row((value / self.width() as usize) as u16)
    }

    fn try_resolve(&self, value: usize) -> Option<Row> {
        if value < self.len() {
            Some(self.resolve(value))
        } else {
            None
        }
    }
}
impl<B: Bounded> Resolve<usize, Column> for B {
    fn resolve(&self, value: usize) -> Column {
        Column((value % self.width() as usize) as u16)
    }

    fn try_resolve(&self, value: usize) -> Option<Column> {
        if value < self.len() {
            Some(self.resolve(value))
        } else {
            None
        }
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

    fn try_resolve(&self, value: Range<T>) -> Option<Range<U>> {
        let start = self.try_resolve(value.start)?;
        let end = self.try_resolve(value.end)?;
        Some(start..end)
    }
}

impl<B, T: Clone, U> Resolve<RangeInclusive<T>, RangeInclusive<U>> for B
where
    B: Resolve<T, U>,
{
    fn resolve(&self, value: RangeInclusive<T>) -> RangeInclusive<U> {
        self.resolve(value.start().clone())..=self.resolve(value.end().clone())
    }

    fn try_resolve(&self, value: RangeInclusive<T>) -> Option<RangeInclusive<U>> {
        let start = self.try_resolve(value.start().clone())?;
        let end = self.try_resolve(value.end().clone())?;
        Some(start..=end)
    }
}

impl<B, T, U> Resolve<RangeTo<T>, RangeTo<U>> for B
where
    B: Resolve<T, U>,
{
    fn resolve(&self, value: RangeTo<T>) -> RangeTo<U> {
        ..self.resolve(value.end)
    }

    fn try_resolve(&self, value: RangeTo<T>) -> Option<RangeTo<U>> {
        Some(..self.try_resolve(value.end)?)
    }
}

impl<B, T, U> Resolve<RangeToInclusive<T>, RangeToInclusive<U>> for B
where
    B: Resolve<T, U>,
{
    fn resolve(&self, value: RangeToInclusive<T>) -> RangeToInclusive<U> {
        ..=self.resolve(value.end)
    }

    fn try_resolve(&self, value: RangeToInclusive<T>) -> Option<RangeToInclusive<U>> {
        Some(..=self.try_resolve(value.end)?)
    }
}

impl<B, T, U> Resolve<RangeFrom<T>, RangeFrom<U>> for B
where
    B: Resolve<T, U>,
{
    fn resolve(&self, value: RangeFrom<T>) -> RangeFrom<U> {
        self.resolve(value.start)..
    }

    fn try_resolve(&self, value: RangeFrom<T>) -> Option<RangeFrom<U>> {
        Some(self.try_resolve(value.start)?..)
    }
}

impl<B: Bounded> Resolve<RangeFull, RangeFull> for B {
    fn resolve(&self, _: RangeFull) -> RangeFull {
        ..
    }
}

impl<B: Bounded> Resolve<RangeFull, Range<usize>> for B {
    fn resolve(&self, _: RangeFull) -> Range<usize> {
        0..self.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Point};

    type Bound = crate::Rect;

    fn ctx() -> Bound {
        Bound::new(50, 50, 50, 50)
    }

    macro_rules! assert_resolve {
        ($ctx:tt, $value:expr => $expected:expr, $ty:ty) => {{
            let resolved: $ty = $ctx.resolve($value);
            assert_eq!(
                resolved, $expected,
                "expected {:?} but got {:?}",
                $expected, resolved
            );
        }};
        ($ctx:expr, $value:tt => $expected:tt) => {{
            let resolved = $ctx.resolve($value);
            assert_eq!(
                resolved, $expected,
                "expected {:?} but got {:?}",
                $expected, resolved
            );
        }};
    }

    #[test]
    fn resolve_index() {
        let ctx = ctx();

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