use std::ops::{Range, RangeInclusive, RangeTo, RangeToInclusive, RangeFull, RangeFrom};
use crate::{Point, Row, Column, Bounded, PointLike, Map, Position, PositionLike};

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

// Point -> *
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

impl<B: Bounded> Resolve<Range<Point>, Range<usize>> for B {
    fn resolve(&self, value: Range<Point>) -> Range<usize> {
        self.resolve(value.start)..self.resolve(value.end)
    }
}

impl<B: Bounded> Resolve<RangeInclusive<Point>, RangeInclusive<usize>> for B {
    fn resolve(&self, value: RangeInclusive<Point>) -> RangeInclusive<usize> {
        self.resolve(*value.start())..=self.resolve(*value.end())
    }
}

impl<B: Bounded> Resolve<RangeTo<Point>, RangeTo<usize>> for B {
    fn resolve(&self, value: RangeTo<Point>) -> RangeTo<usize> {
        ..self.resolve(value.end)
    }
}

impl<B: Bounded> Resolve<RangeToInclusive<Point>, RangeToInclusive<usize>> for B {
    fn resolve(&self, value: RangeToInclusive<Point>) -> RangeToInclusive<usize> {

        ..=self.resolve(value.end)
    }
}

impl<B: Bounded> Resolve<RangeFrom<Point>, RangeFrom<usize>> for B {
    fn resolve(&self, value: RangeFrom<Point>) -> RangeFrom<usize> {
        self.resolve(value.start)..
    }
}


// PointLike -> *
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


impl<B: Bounded> Resolve<Range<PointLike>, Range<usize>> for B {
    fn resolve(&self, value: Range<PointLike>) -> Range<usize> {
        self.resolve(value.start)..self.resolve(value.end)
    }
}


impl<B: Bounded> Resolve<RangeInclusive<PointLike>, RangeInclusive<usize>> for B {
    fn resolve(&self, value: RangeInclusive<PointLike>) -> RangeInclusive<usize> {
        self.resolve(*value.start())..=self.resolve(*value.end())
    }
}

impl<B: Bounded> Resolve<RangeTo<PointLike>, RangeTo<usize>> for B {
    fn resolve(&self, value: RangeTo<PointLike>) -> RangeTo<usize> {
        ..self.resolve(value.end)
    }
}

impl<B: Bounded> Resolve<RangeToInclusive<PointLike>, RangeToInclusive<usize>> for B {
    fn resolve(&self, value: RangeToInclusive<PointLike>) -> RangeToInclusive<usize> {

        ..=self.resolve(value.end)
    }
}

impl<B: Bounded> Resolve<RangeFrom<PointLike>, RangeFrom<usize>> for B {
    fn resolve(&self, value: RangeFrom<PointLike>) -> RangeFrom<usize> {
        self.resolve(value.start)..
    }
}

// Point -> *
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

impl<B: Bounded> Resolve<Range<Position>, Range<usize>> for B {
    fn resolve(&self, value: Range<Position>) -> Range<usize> {
        self.resolve(value.start)..self.resolve(value.end)
    }
}

impl<B: Bounded> Resolve<RangeInclusive<Position>, RangeInclusive<usize>> for B {
    fn resolve(&self, value: RangeInclusive<Position>) -> RangeInclusive<usize> {
        self.resolve(*value.start())..=self.resolve(*value.end())
    }
}

impl<B: Bounded> Resolve<RangeTo<Position>, RangeTo<usize>> for B {
    fn resolve(&self, value: RangeTo<Position>) -> RangeTo<usize> {
        ..self.resolve(value.end)
    }
}

impl<B: Bounded> Resolve<RangeToInclusive<Position>, RangeToInclusive<usize>> for B {
    fn resolve(&self, value: RangeToInclusive<Position>) -> RangeToInclusive<usize> {

        ..=self.resolve(value.end)
    }
}

impl<B: Bounded> Resolve<RangeFrom<Position>, RangeFrom<usize>> for B {
    fn resolve(&self, value: RangeFrom<Position>) -> RangeFrom<usize> {
        self.resolve(value.start)..
    }
}


// PositionLike -> *
impl<B: Bounded> Resolve<PositionLike, usize> for B {
    fn resolve(&self, value: PositionLike) -> usize {
        (value.0 - self.min_y() as usize) * self.width() as usize
            + (value.1 - self.min_x() as usize)
    }
}

impl<B: Bounded> Resolve<PositionLike, Row> for B {
    fn resolve(&self, value: PositionLike) -> Row {
        Row((value.0 - self.min_y() as usize))
    }
}


impl<B: Bounded> Resolve<PositionLike, Column> for B {
    fn resolve(&self, value: PositionLike) -> Column {
        Column((value.1 - self.min_x() as usize))
    }
}


impl<B: Bounded> Resolve<Range<PositionLike>, Range<usize>> for B {
    fn resolve(&self, value: Range<PositionLike>) -> Range<usize> {
        self.resolve(value.start)..self.resolve(value.end)
    }
}


impl<B: Bounded> Resolve<RangeInclusive<PositionLike>, RangeInclusive<usize>> for B {
    fn resolve(&self, value: RangeInclusive<PositionLike>) -> RangeInclusive<usize> {
        self.resolve(*value.start())..=self.resolve(*value.end())
    }
}

impl<B: Bounded> Resolve<RangeTo<PositionLike>, RangeTo<usize>> for B {
    fn resolve(&self, value: RangeTo<PositionLike>) -> RangeTo<usize> {
        ..self.resolve(value.end)
    }
}

impl<B: Bounded> Resolve<RangeToInclusive<PositionLike>, RangeToInclusive<usize>> for B {
    fn resolve(&self, value: RangeToInclusive<PositionLike>) -> RangeToInclusive<usize> {

        ..=self.resolve(value.end)
    }
}

impl<B: Bounded> Resolve<RangeFrom<PositionLike>, RangeFrom<usize>> for B {
    fn resolve(&self, value: RangeFrom<PositionLike>) -> RangeFrom<usize> {
        self.resolve(value.start)..
    }
}


// Row -> *
impl<B: Bounded> Resolve<Row, Point> for B {
    fn resolve(&self, value: Row) -> Point {
        Point::new(self.min_x(), value.value() as u16)
    }
}

impl<B: Bounded> Resolve<Row, usize> for B {
    fn resolve(&self, value: Row) -> usize {
        (value.value() - self.min_y() as usize) * self.width() as usize
    }
}
impl<B: Bounded> Resolve<Row, Column> for B {
    fn resolve(&self, value: Row) -> Column {
        Column(0)
    }
}


impl<B: Bounded> Resolve<Row, Range<usize>> for B {
    fn resolve(&self, value: Row) -> Range<usize> {
        let y = *value;
        let width = self.width() as usize;

        y * width..y * width + width
    }
}


impl<B: Bounded> Resolve<Range<Row>, Range<usize>> for B {
    fn resolve(&self, value: Range<Row>) -> Range<usize> {
        let width = self.width() as usize;

        *value.start * width..*value.end * width + width
    }
}


impl<B: Bounded> Resolve<RangeInclusive<Row>, RangeInclusive<usize>> for B {
    fn resolve(&self, value: RangeInclusive<Row>) -> RangeInclusive<usize> {
        let width = self.width() as usize;

        **value.start() * width..=**value.end() * width + width
    }
}


impl<B: Bounded> Resolve<RangeTo<Row>, RangeTo<usize>> for B {
    fn resolve(&self, value: RangeTo<Row>) -> RangeTo<usize> {
        let width = self.width() as usize;

        ..*value.end * width + width
    }
}

impl<B: Bounded> Resolve<RangeToInclusive<Row>, RangeToInclusive<usize>> for B {
    fn resolve(&self, value: RangeToInclusive<Row>) -> RangeToInclusive<usize> {
        let width = self.width() as usize;

        ..=*value.end * width + width
    }
}

impl<B: Bounded> Resolve<RangeFrom<Row>, RangeFrom<usize>> for B {
    fn resolve(&self, value: RangeFrom<Row>) -> RangeFrom<usize> {
        let width = self.width() as usize;

        *value.start * width..
    }
}


// Column -> *
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
    fn resolve(&self, value: Column) -> Row {
        Row(0)
    }
}

// Index -> *
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
            (value % w) + self.min_x() as usize ,
        )
    }
}

impl<B: Bounded> Resolve<usize, PositionLike> for B {
    fn resolve(&self, value: usize) -> PositionLike {
        let w = self.width() as usize;

        (
            (value / w) + self.min_y() as usize,
            (value % w) + self.min_x()  as usize,
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
