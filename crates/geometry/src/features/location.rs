use crate::{One, Point, Zero};


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
