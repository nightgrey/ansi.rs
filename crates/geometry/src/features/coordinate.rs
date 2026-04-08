use crate::{Column, Point, PointLike, Row};

pub trait Coordinated: Copy + Into<Point> + From<Point> {
    fn new(x: usize, y: usize) -> Self;
    fn x(&self) -> usize;
    fn y(&self) -> usize;
}

impl Coordinated for Point {
    fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
    fn x(&self) -> usize {
        self.x
    }
    fn y(&self) -> usize {
        self.y
    }
}

impl Coordinated for PointLike {
    fn new(x: usize, y: usize) -> Self {
        (x, y).into()
    }
    fn x(&self) -> usize {
        self.0
    }
    fn y(&self) -> usize {
        self.1
    }
}

impl From<PointLike> for Point {
    fn from(value: PointLike) -> Self {
        Self::new(value.0, value.1)
    }
}

impl From<Point> for PointLike {
    fn from(value: Point) -> Self {
        (value.x, value.y)
    }
}

impl Coordinated for Row {
    fn new(x: usize, y: usize) -> Self {
        Self(y)
    }
    fn x(&self) -> usize {
        0
    }
    fn y(&self) -> usize {
        self.value()
    }
}

impl From<Row> for Point {
    fn from(value: Row) -> Self {
        Point::new(0, value.0)
    }
}

impl From<Point> for Row {
    fn from(value: Point) -> Self {
        Row(value.y)
    }
}

impl Coordinated for Column {
    fn new(x: usize, y: usize) -> Self {
        Self(x)
    }
    fn x(&self) -> usize {
        self.value()
    }
    fn y(&self) -> usize {
        0
    }
}

impl From<Column> for Point {
    fn from(value: Column) -> Self {
        Point::new(value.0, 0)
    }
}

impl From<Point> for Column {
    fn from(value: Point) -> Self {
        Column(value.x)
    }
}


