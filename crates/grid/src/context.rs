use crate::{Bounds, Position, Steps};

/// Type that represents a spatial context.
pub const trait Context {
    #[inline]
    fn min(&self) -> Position;
    
    #[inline]
    fn max(&self) -> Position;

    #[inline]
    fn width(&self) -> usize { self.max().col.saturating_sub(self.min().col) }

    #[inline]
    fn height(&self) -> usize { self.max().row.saturating_sub(self.min().row) }

    #[inline]
    fn area(&self) -> usize { self.width() * self.height() }

    #[inline]
    fn is_empty(&self) -> bool { self.area() == 0 }

    #[inline]
    fn bounds(&self) -> Bounds { Bounds::new(self.min(), self.max()) }

    fn positions(&self) -> Steps where Self: Sized {
        Steps::new(self)
    }
}