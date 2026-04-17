use std::ops::Add;
use crate::{Point, Rect};

pub trait Translate<T = Self> {
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

impl<T: Copy + Add<T, Output = T>> Translate<Point<T>> for Point<T> {
    type Output = Self;

    fn translate(self, by: &Point<T>) -> Self::Output {
        Self {
            x: self.x + by.x,
            y: self.y + by.y,
        }
    }
}

impl<T: Copy + Add<T, Output = T>> Translate<T> for Point<T> {
    type Output = Self;

    fn translate(self, by: &T) -> Self::Output {
        Self {
            x: self.x + *by,
            y: self.y + *by,
        }
    }
}