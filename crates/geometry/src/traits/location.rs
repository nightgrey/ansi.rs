use crate::{Point, PointLike, Position, PositionLike};

pub trait Locatable: Copy + PartialEq + Eq + PartialOrd + Ord  {
    #[inline]
    fn x(&self) -> u16;

    #[inline]
    fn y(&self) -> u16;

    #[inline]
    fn set_x(&mut self, x: u16);
    
    #[inline]
    fn set_y(&mut self, y: u16);

    #[inline]
    fn set(&mut self, x: u16, y: u16) {
        self.set_x(x);
        self.set_y(y);
    }

    fn point(&self) -> Point {
        Point::new(self.x(), self.y())
    }

    fn position(&self) -> Position {
        Position::new(self.y() as usize, self.x() as usize)
    }
}

impl Locatable for Point {
    fn x(&self) -> u16 {
        self.x
    }
    fn y(&self) -> u16 {
        self.y
    }
    fn set_x(&mut self, x: u16) {
        self.x = x;
    }
    fn set_y(&mut self, y: u16) {
        self.y = y;
    }
}

impl Locatable for PointLike {
    fn x(&self) -> u16 {
        self.0
    }
    fn y(&self) -> u16 {
        self.1
    }

    fn set_x(&mut self, x: u16) {
        self.0 = x;
    }
    fn set_y(&mut self, y: u16) {
        self.1 = y;
    }
}

impl Locatable for Position {
    fn x(&self) -> u16 {
        self.col as u16
    }
    fn y(&self) -> u16 {
        self.row as u16
    }

    fn set_x(&mut self, x: u16) {
        self.col = x as usize;
    }
    fn set_y(&mut self, y: u16) {
        self.row = y as usize;
    }
}

impl Locatable for PositionLike {
    fn x(&self) -> u16 {
        self.0 as u16
    }
    fn y(&self) -> u16 {
        self.1 as u16
    }

    fn set_x(&mut self, x: u16) {
        self.0 = x as usize;
    }
    fn set_y(&mut self, y: u16) {
        self.1 = y as usize;
    }
}
