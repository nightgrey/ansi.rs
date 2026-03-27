use crate::{Point};

pub const trait Zero {
    const ZERO: Self;
}

impl const Zero for usize {
    const ZERO: Self = 0;
}
// 
// impl const Zero for Position {
//     const ZERO: Self = Self::MIN;
// }

impl const Zero for Point {
    const ZERO: Self = Self::ZERO;
}

pub const trait Location<T: Copy = usize>: Copy {
    fn new(x: T, y: T) -> Self;
}
// 
// impl<T: Copy> const Location<T> for Position<T> {
//     fn new(x: T, y: T) -> Self {
//         Position::new(y, x)
//     }
// }

impl<T: Copy> const Location<T> for Point<T> {
    fn new(x: T, y: T) -> Self {
        Point::new(x, y)
    }
}
