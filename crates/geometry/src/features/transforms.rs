use std::ops::Add;
use crate::{Point, Rect};

pub trait Translate<T = Point> {
    type Output;
    fn translate(self, by: &T) -> Self::Output;
}

impl<T: Copy + Add<T, Output = T>> Translate<T> for Rect<T> {
    type Output = Self;

    fn translate(self, by: &T) -> Self::Output {
        Self {
            min: self.min + *by,
            max: self.max + *by,
        }
    }
}
