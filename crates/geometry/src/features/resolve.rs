use utils::Resolve;
use crate::{Point, Rect, Row, Column, Bounded, PointLike};

// Point
impl Resolve<Point, Rect> for Point {
    fn resolve(self, ctx: Rect) -> Point {
        self
    }
}

impl Resolve<usize, Rect> for Point {
    fn resolve(self, ctx: Rect) -> usize {

        (self.y - ctx.min.y) * ctx.width() + (self.x - ctx.min.x)
    }
}

impl Resolve<usize, Rect> for PointLike {
    fn resolve(self, ctx: Rect) -> usize {
        (self.1 - ctx.min.y) * ctx.width() + (self.0 - ctx.min.x)
    }
}

impl Resolve<Row, Rect> for Point {
    fn resolve(self, ctx: Rect) -> Row {
        Row(self.y - ctx.min.y)
    }
}

impl Resolve<Column, Rect> for Point {
    fn resolve(self, ctx: Rect) -> Column {
        Column(self.x - ctx.min.x)
    }
}

// Row
impl Resolve<Point, Rect> for Row {
    fn resolve(self, ctx: Rect) -> Point {
        Point::new(ctx.min.x, self.value())
    }
}

impl Resolve<usize, Rect> for Row {
    fn resolve(self, ctx: Rect) -> usize {
        (self.value() - ctx.min.y) * ctx.width()
    }
}

impl Resolve<Row, Rect> for Row {
    fn resolve(self, ctx: Rect) -> Row {
        Row(self.0 - ctx.min.y)
    }
}

impl Resolve<Column, Rect> for Row {
    fn resolve(self, ctx: Rect) -> Column {
        Column(0)
    }
}

// Column
impl Resolve<Point, Rect> for Column {
    fn resolve(self, ctx: Rect) -> Point {
        Point::new(self.value(), 0)
    }
}

impl Resolve<usize, Rect> for Column {
    fn resolve(self, ctx: Rect) -> usize {
        self.value() - ctx.min.x
    }
}

impl Resolve<Row, Rect> for Column {
    fn resolve(self, ctx: Rect) -> Row {
        Row(0)
    }
}

impl Resolve<Column, Rect> for Column {
    fn resolve(self, ctx: Rect) -> Column {
        self
    }
}

// Linear
impl Resolve<Point, Rect> for usize {
    fn resolve(self, ctx: Rect) -> Point {
        let w = ctx.width();

        Point::new(self % w + ctx.min.x, self / w + ctx.min.y)
    }
}
impl Resolve<PointLike, Rect> for usize {
    fn resolve(self, ctx: Rect) -> PointLike {
        let w = ctx.width();

        (self % w + ctx.min.x, self / w + ctx.min.y)
    }
}

impl Resolve<usize, Rect> for usize {
    fn resolve(self, ctx: Rect) -> usize {
        self
    }
}

impl Resolve<Row, Rect> for usize {
    fn resolve(self, ctx: Rect) -> Row {
        Row(self / ctx.width())
    }
}


// --- Referenced: Rect ---

// Point
impl Resolve<Point, &Rect> for Point {
    fn resolve(self, ctx: &Rect) -> Point {
        self
    }
}

impl Resolve<usize, &Rect> for Point {
    fn resolve(self, ctx: &Rect) -> usize {
        (self.y - ctx.min.y) * ctx.width() + (self.x - ctx.min.x)
    }
}

impl Resolve<Row, &Rect> for Point {
    fn resolve(self, ctx: &Rect) -> Row {
        Row(self.y - ctx.min.y)
    }
}

impl Resolve<Column, &Rect> for Point {
    fn resolve(self, ctx: &Rect) -> Column {
        Column(self.x - ctx.min.x)
    }
}

// Linear
impl Resolve<Point, &Rect> for usize {
    fn resolve(self, ctx: &Rect) -> Point {
        Point::new(
            ctx.min.x + self % ctx.width(),
            ctx.min.y + self / ctx.width(),
        )
    }
}

impl Resolve<usize, &Rect> for usize {
    fn resolve(self, ctx: &Rect) -> usize {
        self
    }
}

impl Resolve<Row, &Rect> for usize {
    fn resolve(self, ctx: &Rect) -> Row {
        Row(self / ctx.width())
    }
}

impl Resolve<Column, &Rect> for usize {
    fn resolve(self, ctx: &Rect) -> Column {
        Column(self % ctx.width())
    }
}
