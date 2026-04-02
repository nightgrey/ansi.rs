use utils::Resolve;
use crate::{Point, Rect, Row, Column, Bounded, PointLike};

// Point
impl Resolve<Point, Rect> for Point {
    fn resolve(self, context: Rect) -> Point {
        self
    }
}

impl Resolve<usize, Rect> for Point {
    fn resolve(self, context: Rect) -> usize {
        (self.y - context.min.y) * context.width() + (self.x - context.min.x)
    }
}

impl Resolve<usize, Rect> for PointLike {
    fn resolve(self, context: Rect) -> usize {
        (self.1 - context.min.y) * context.width() + (self.0 - context.min.x)
    }
}

impl Resolve<Row, Rect> for Point {
    fn resolve(self, context: Rect) -> Row {
        Row(self.y - context.min.y)
    }
}

impl Resolve<Column, Rect> for Point {
    fn resolve(self, context: Rect) -> Column {
        Column(self.x - context.min.x)
    }
}

// Row
impl Resolve<Point, Rect> for Row {
    fn resolve(self, context: Rect) -> Point {
        Point::new(context.min.x, self.value())
    }
}

impl Resolve<usize, Rect> for Row {
    fn resolve(self, context: Rect) -> usize {
        (self.value() - context.min.y) * context.width()
    }
}

impl Resolve<Row, Rect> for Row {
    fn resolve(self, context: Rect) -> Row {
        Row(self.0 - context.min.y)
    }
}

impl Resolve<Column, Rect> for Row {
    fn resolve(self, context: Rect) -> Column {
        Column(0)
    }
}

// Column
impl Resolve<Point, Rect> for Column {
    fn resolve(self, context: Rect) -> Point {
        Point::new(self.value(), 0)
    }
}

impl Resolve<usize, Rect> for Column {
    fn resolve(self, context: Rect) -> usize {
        self.value() - context.min.x
    }
}

impl Resolve<Row, Rect> for Column {
    fn resolve(self, context: Rect) -> Row {
        Row(0)
    }
}

impl Resolve<Column, Rect> for Column {
    fn resolve(self, context: Rect) -> Column {
        self
    }
}

// Linear
impl Resolve<Point, Rect> for usize {
    fn resolve(self, context: Rect) -> Point {
        Point::new(
            context.min.x + self % context.width(),
            context.min.y + self / context.width(),
        )
    }
}


impl Resolve<usize, Rect> for usize {
    fn resolve(self, context: Rect) -> usize {
        self
    }
}

impl Resolve<Row, Rect> for usize {
    fn resolve(self, context: Rect) -> Row {
        Row(self / context.width())
    }
}


// --- Referenced: Rect ---

// Point
impl Resolve<Point, &Rect> for Point {
    fn resolve(self, context: &Rect) -> Point {
        self
    }
}

impl Resolve<usize, &Rect> for Point {
    fn resolve(self, context: &Rect) -> usize {
        (self.y - context.min.y) * context.width() + (self.x - context.min.x)
    }
}

impl Resolve<Row, &Rect> for Point {
    fn resolve(self, context: &Rect) -> Row {
        Row(self.y - context.min.y)
    }
}

impl Resolve<Column, &Rect> for Point {
    fn resolve(self, context: &Rect) -> Column {
        Column(self.x - context.min.x)
    }
}

// Linear
impl Resolve<Point, &Rect> for usize {
    fn resolve(self, context: &Rect) -> Point {
        Point::new(
            context.min.x + self % context.width(),
            context.min.y + self / context.width(),
        )
    }
}

impl Resolve<usize, &Rect> for usize {
    fn resolve(self, context: &Rect) -> usize {
        self
    }
}

impl Resolve<Row, &Rect> for usize {
    fn resolve(self, context: &Rect) -> Row {
        Row(self / context.width())
    }
}

impl Resolve<Column, &Rect> for usize {
    fn resolve(self, context: &Rect) -> Column {
        Column(self % context.width())
    }
}

/// Trait to encapsulate behaviour to resolve a value from a context.
pub trait ContextualResolve<Into, With> {
    /// Resolve a value.
    ///
    /// Panics if the value is unable to resolve.
    #[inline(always)]
    fn resolve(self, with: With) -> Into;
}

impl<'a, Into, Context, T: Resolve<Into, &'a Context>> ContextualResolve<Into, T> for &'a Context {
    #[inline(always)]
    fn resolve(self, with: T) -> Into {
        with.resolve(self)
    }
}

