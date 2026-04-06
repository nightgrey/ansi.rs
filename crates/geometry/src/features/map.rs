use crate::{Edges, Point, Rect, Size};

pub trait Map<T, U> {
    type Output;

    fn map<F>(self, f: F) -> Self::Output
    where
        F: Fn(T) -> U;
}

impl<T, U> Map<T, U> for Rect<T> {
    type Output = Rect<U>;

    fn map<F>(self, f: F) -> Self::Output
    where
        F: Fn(T) -> U,
    {
        Rect {
            min: f(self.min),
            max: f(self.max),
        }
    }
}

impl<T, U> Map<T, U> for Size<T> {
    type Output = Size<U>;

    fn map<F>(self, f: F) -> Self::Output
    where
        F: Fn(T) -> U,
    {
        Size {
            width: f(self.width),
            height: f(self.height),
        }
    }
}

impl<T, U> Map<T, U> for Point<T> {
    type Output = Point<U>;

    fn map<F>(self, f: F) -> Self::Output
    where
        F: Fn(T) -> U,
    {
        Point {
            x: f(self.x),
            y: f(self.y),
        }
    }
}

impl<T, U> Map<T, U> for Edges<T> {
    type Output = Edges<U>;

    fn map<F>(self, f: F) -> Self::Output
    where
        F: Fn(T) -> U,
    {
        Edges {
            top: f(self.top),
            right: f(self.right),
            bottom: f(self.bottom),
            left: f(self.left),
        }
    }
}
