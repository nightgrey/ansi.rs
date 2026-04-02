use derive_more::{Deref, DerefMut, From};
use super::Dimension;
use crate::Available;

/// A set of edges with a specific value for each edge.
#[derive(Copy, Debug, Clone, PartialEq, Default, Deref, DerefMut)]
#[repr(transparent)]
pub struct Edges(geometry::Edges<Dimension>);

impl Edges {
    pub const AUTO: Self = Self::all(Dimension::Auto);
    pub const ZERO: Self = Self::all(Dimension::Length(0));

    pub const fn auto() -> Self {
        Self::AUTO
    }

    pub const fn length(value: u32) -> Self {
        Self::new(Dimension::Length(value), Dimension::Length(value), Dimension::Length(value), Dimension::Length(value))
    }

    pub const fn percent(value: f32) -> Self {
        Self::new(Dimension::Percent(value), Dimension::Percent(value), Dimension::Percent(value), Dimension::Percent(value))
    }

    pub const fn new(top: impl [const] Into<Dimension>, right: impl [const] Into<Dimension>, bottom: impl [const] Into<Dimension>, left: impl [const] Into<Dimension>) -> Self {
        Self(geometry::Edges::new(top.into(), right.into(), bottom.into(), left.into()))
    }

    pub const fn all(value: impl [const] Into<Dimension> + Copy) -> Self {
        Self::new(value.into(), value.into(), value.into(), value.into())
    }

    pub const fn horizontal(value: impl [const] Into<Dimension> + Copy) -> Self {
        Self::new(value.into(), value.into(), value.into(), value.into())
    }

    pub const fn vertical(value: impl [const] Into<Dimension> + Copy) -> Self {
        Self::new(value.into(), value.into(), value.into(), value.into())
    }

    pub  fn or(mut self, other: Self) -> Self {
        self.top = self.top.or(other.top);
        self.right = self.right.or(other.right);
        self.bottom = self.bottom.or(other.bottom);
        self.left = self.left.or(other.left);
        self
    }
}


impl<T: Into<Dimension>> From<T> for Edges {
    fn from(value: T) -> Self {
        let value = value.into();
        Self::new(value, value, value, value)
    }
}

impl<T: Into<Dimension>> From<(T, T)> for Edges {
    fn from(value: (T, T)) -> Self {
        let (vertical, horizontal) = (value.0.into(), value.1.into());
        Self::new(vertical, horizontal, vertical, horizontal)
    }
}

impl<T: Into<Dimension>> From<(T, T, T, T)> for Edges {
    fn from(value: (T, T, T, T)) -> Self {
        Self::new(
            value.0.into(),
            value.1.into(),
            value.2.into(),
            value.3.into(),
        )
    }
}

impl Into<taffy::Rect<taffy::LengthPercentageAuto>> for Edges {
    fn into(self) -> taffy::Rect<taffy::LengthPercentageAuto> {
        taffy::Rect {
            left: self.left.into(),
            right: self.right.into(),
            top: self.top.into(),
            bottom: self.bottom.into(),
        }
    }
}

impl Into<taffy::Rect<taffy::LengthPercentage>> for Edges {
    fn into(self) -> taffy::Rect<taffy::LengthPercentage> {
        taffy::Rect {
            left: self.left.into(),
            right: self.right.into(),
            top: self.top.into(),
            bottom: self.bottom.into(),
        }
    }
}

impl Into<taffy::Rect<taffy::Dimension>> for Edges {
    fn into(self) -> taffy::Rect<taffy::Dimension> {
        taffy::Rect {
            left: self.left.into(),
            right: self.right.into(),
            top: self.top.into(),
            bottom: self.bottom.into(),
        }
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Deref, DerefMut, From)]
#[repr(transparent)]
pub struct Space(geometry::Size<Available>);

impl Space {
    pub const ZERO: Self = Self(geometry::Size::new(Available::Definite(0), Available::Definite(0)));
    pub const MIN: Self = Self(geometry::Size::new(Available::Min, Available::Min));
    pub const MAX: Self = Self(geometry::Size::new(Available::Max, Available::Max));

    pub const fn new(width: impl [const] Into<Available>, height: impl [const] Into<Available>) -> Self {
        Self(geometry::Size::new(width.into(), height.into()))
    }

    pub const fn definite(value: impl [const] Into<Available> + Copy) -> Self {
        Self::new(value.into(), value.into())
    }

    pub const fn min() -> Self {
        Self::new(Available::Min, Available::Min)
    }

    pub const fn max() -> Self {
        Self::new(Available::Max, Available::Max)
    }
}

impl From<geometry::Size> for Space {
    fn from(value: geometry::Size) -> Self {
        Self::new(Available::Definite(value.width as u32), Available::Definite(value.height as u32))
    }
}

impl Into<taffy::Size<taffy::AvailableSpace>> for Space {
    fn into(self) -> taffy::Size<taffy::AvailableSpace> {
        taffy::Size {
            width: match self.width {
                Available::Definite(w) => taffy::AvailableSpace::Definite(w as f32),
                Available::Min => taffy::AvailableSpace::MinContent,
                Available::Max => taffy::AvailableSpace::MaxContent,
            },
            height: match self.height {
                Available::Definite(h) => taffy::AvailableSpace::Definite(h as f32),
                Available::Min => taffy::AvailableSpace::MinContent,
                Available::Max => taffy::AvailableSpace::MaxContent,
            },
        }
    }
}


#[derive(Copy, Debug, Clone, PartialEq, Default, Deref, DerefMut)]
#[repr(transparent)]
pub struct Gap(geometry::Axis<Dimension>);
impl Gap {
    pub const ZERO: Self = Self(geometry::Axis::new(Dimension::Length(0), Dimension::Length(0)));
    pub const AUTO: Self = Self(geometry::Axis::new(Dimension::Auto, Dimension::Auto));

    pub const fn auto() -> Self {
        Self::AUTO
    }

    pub const fn length(value: u32) -> Self {
        Self::new(Dimension::Length(value), Dimension::Length(value))
    }

    pub const fn percent(value: f32) -> Self {
        Self::new(Dimension::Percent(value), Dimension::Percent(value))
    }

    pub const fn new(horizontal: impl [const] Into<Dimension>, vertical: impl [const] Into<Dimension>) -> Self {
        Self(geometry::Axis::new(horizontal.into(), vertical.into()))
    }

    pub const fn horizontal(value: impl [const] Into<Dimension> + Copy) -> Self {
        Self::new(value.into(), Dimension::Auto)
    }

    pub const fn vertical(value: impl [const] Into<Dimension> + Copy) -> Self {
        Self::new(Dimension::Auto, value.into())
    }
    
    pub const fn both(value: impl [const] Into<Dimension> + Copy) -> Self {
        Self(geometry::Axis::both(value.into()))
    }

}