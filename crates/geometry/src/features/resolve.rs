use std::ops::{Range, RangeInclusive, RangeTo, RangeToInclusive, RangeFull, RangeFrom};
use crate::{Point, Rect, Row, Column, Bounded, PointLike};

/// Resolve a context-dependent value.
pub trait Resolve<T, U> {
    /// Resolve value within context of [`Self`].
    fn resolve(&self, value: T) -> U;
}

// Point -> *
impl<B: Bounded> Resolve<Point, usize> for B {
    fn resolve(&self, value: Point) -> usize {
        (value.y - self.min_y()) * self.width() + (value.x - self.min_x())
    }
}

impl<B: Bounded> Resolve<Point, Row> for B {
    fn resolve(&self, value: Point) -> Row {
        Row(value.y - self.min_y())
    }
}


impl<B: Bounded> Resolve<Point, Column> for B {
    fn resolve(&self, value: Point) -> Column {
        Column(value.x - self.min_x())
    }
}

// PointLike -> *
impl<B: Bounded> Resolve<PointLike, usize> for B {
    fn resolve(&self, value: PointLike) -> usize {
        (value.1 - self.min_y()) * self.width() + (value.0 - self.min_x())
    }
}

impl<B: Bounded> Resolve<PointLike, Row> for B {
    fn resolve(&self, value: PointLike) -> Row {
        Row(value.1 - self.min_y())
    }
}


impl<B: Bounded> Resolve<PointLike, Column> for B {
    fn resolve(&self, value: PointLike) -> Column {
        Column(value.0 - self.min_x())
    }
}

// Row -> *
impl<B: Bounded> Resolve<Row, Point> for B {
    fn resolve(&self, value: Row) -> Point {
        Point::new(self.min_x(), value.value())
    }
}

impl<B: Bounded> Resolve<Row, usize> for B {
    fn resolve(&self, value: Row) -> usize {
        (value.value() - self.min_y()) * self.width()
    }
}
impl<B: Bounded> Resolve<Row, Column> for B {
    fn resolve(&self, value: Row) -> Column {
        Column(0)
    }
}

// Column -> *
impl<B: Bounded> Resolve<Column, Point> for B {
    fn resolve(&self, value: Column) -> Point {
        Point::new(value.value(), 0)
    }
}

impl<B: Bounded> Resolve<Column, usize> for B {
    fn resolve(&self, value: Column) -> usize {
        value.value() - self.min_x()
    }
}

impl<B: Bounded> Resolve<Column, Row> for B {
    fn resolve(&self, value: Column) -> Row {
        Row(0)
    }
}

// Index -> *
impl<B: Bounded> Resolve<usize, Point> for B {
    fn resolve(&self, value: usize) -> Point {
        let w = self.width();

        Point::new(value % w + self.min_x(), value / w + self.min_y())
    }
}
impl<B: Bounded> Resolve<usize, PointLike> for B {
    fn resolve(&self, value: usize) -> PointLike {
        let w = self.width();

        (value % w + self.min_x(), value / w + self.min_y())
    }
}

impl<B: Bounded> Resolve<usize, Row> for B {
    fn resolve(&self, value: usize) -> Row {
        Row(value / self.width())
    }
}

impl<B: Bounded> Resolve<usize, Column> for B {
    fn resolve(&self, value: usize) -> Column {
        Column(value % self.width())
    }
}

// Range
impl<R: Resolve<T, usize>, T> Resolve<Range<T>, Range<usize>> for R {
    fn resolve(&self, value: Range<T>) -> Range<usize> {
        self.resolve(value.start)..self.resolve(value.end)
    }
}
impl<R: Resolve<T, usize>, T: Clone> Resolve<RangeInclusive<T>, RangeInclusive<usize>> for R {
    fn resolve(&self, value: RangeInclusive<T>) -> RangeInclusive<usize> {
        self.resolve(value.start().clone())..=self.resolve(value.end().clone())
    }
}

impl<R: Resolve<T, usize>, T> Resolve<RangeTo<T>, RangeTo<usize>> for R {
    fn resolve(&self, value: RangeTo<T>) -> RangeTo<usize> {
        ..self.resolve(value.end)
    }
}

impl<R: Resolve<T, usize>, T> Resolve<RangeToInclusive<T>, RangeToInclusive<usize>> for R {
    fn resolve(&self, value: RangeToInclusive<T>) -> RangeToInclusive<usize> {
        ..=self.resolve(value.end)
    }
}

impl<R: Resolve<T, usize>, T> Resolve<RangeFrom<T>, RangeFrom<usize>> for R {
    fn resolve(&self, value: RangeFrom<T>) -> RangeFrom<usize> {
        self.resolve(value.start)..
    }
}

impl<B: Bounded> Resolve<RangeFull, RangeFull> for B {
    fn resolve(&self, _: RangeFull) -> RangeFull {
        ..
    }
}

// T -> T
impl<B: Bounded> Resolve<Point, Point> for B {
    fn resolve(&self, value: Point) -> Point {
        value
    }
}

impl<B: Bounded> Resolve<PointLike, PointLike> for B {
    fn resolve(&self, value: PointLike) -> PointLike {
        value
    }
}

impl<B: Bounded> Resolve<Row, Row> for B {
    fn resolve(&self, value: Row) -> Row {
        value
    }
}

impl<B: Bounded> Resolve<Column, Column> for B {
    fn resolve(&self, value: Column) -> Column {
        value
    }
}

impl<B: Bounded> Resolve<usize, usize> for B {
    fn resolve(&self, value: usize) -> usize {
        value
    }
}

