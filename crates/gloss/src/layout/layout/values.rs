use taffy::prelude::TaffyZero;

use derive_more::{Deref, DerefMut, From};

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

impl From<Edges> for taffy::Rect<taffy::LengthPercentageAuto> {
    fn from(edges: Edges) -> Self {
        taffy::Rect {
            left: edges.left.into(),
            right: edges.right.into(),
            top: edges.top.into(),
            bottom: edges.bottom.into(),
        }
    }
}

impl From<Edges> for taffy::Rect<taffy::LengthPercentage> {
    fn from(edges: Edges) -> Self {
        taffy::Rect {
            left: edges.left.into(),
            right: edges.right.into(),
            top: edges.top.into(),
            bottom: edges.bottom.into(),
        }
    }
}

impl From<Edges> for taffy::Rect<taffy::Dimension> {
    fn from(edges: Edges) -> Self {
        taffy::Rect {
            left: edges.left.into(),
            right: edges.right.into(),
            top: edges.top.into(),
            bottom: edges.bottom.into(),
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

impl From<Space> for taffy::Size<taffy::AvailableSpace> {
    fn from(space: Space) -> Self {
        taffy::Size {
            width: match space.width {
                Available::Definite(w) => taffy::AvailableSpace::Definite(w as f32),
                Available::Min => taffy::AvailableSpace::MinContent,
                Available::Max => taffy::AvailableSpace::MaxContent,
            },
            height: match space.height {
                Available::Definite(h) => taffy::AvailableSpace::Definite(h as f32),
                Available::Min => taffy::AvailableSpace::MinContent,
                Available::Max => taffy::AvailableSpace::MaxContent,
            },
        }
    }
}


#[derive(Copy, Debug, Clone, PartialEq, Default, Deref, DerefMut)]
#[repr(transparent)]
pub struct Gap(geometry::Sides<Dimension>);
impl Gap {
    pub const ZERO: Self = Self(geometry::Sides::new(Dimension::Length(0), Dimension::Length(0)));
    pub const AUTO: Self = Self(geometry::Sides::new(Dimension::Auto, Dimension::Auto));

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
        Self(geometry::Sides::new(horizontal.into(), vertical.into()))
    }

    pub const fn horizontal(value: impl [const] Into<Dimension> + Copy) -> Self {
        Self::new(value.into(), Dimension::Auto)
    }

    pub const fn vertical(value: impl [const] Into<Dimension> + Copy) -> Self {
        Self::new(Dimension::Auto, value.into())
    }

    pub const fn both(value: impl [const] Into<Dimension> + Copy) -> Self {
        Self(geometry::Sides::both(value.into()))
    }

}

impl From<Gap> for taffy::Size<taffy::LengthPercentage> {
    fn from(value: Gap) -> Self {
        taffy::Size {
            width: value.horizontal.into(),
            height: value.vertical.into(),
        }
    }
}
#[derive(Copy, Debug, Clone, PartialEq, Default, Deref, DerefMut)]
#[repr(transparent)]
pub struct Size(geometry::Size<Dimension>);

impl Size {
    pub const ZERO: Self = Self(geometry::Size::new(Dimension::Length(0), Dimension::Length(0)));
    pub const AUTO: Self = Self(geometry::Size::new(Dimension::Auto, Dimension::Auto));

    pub const fn auto() -> Self {
        Self::AUTO
    }

    pub const fn length(value: u32) -> Self {
        Self::new(Dimension::Length(value), Dimension::Length(value))
    }

    pub const fn percent(value: f32) -> Self {
        Self::new(Dimension::Percent(value), Dimension::Percent(value))
    }

    pub const fn new(width: impl [const] Into<Dimension>, height: impl [const] Into<Dimension>) -> Self {
        Self(geometry::Size::new(width.into(), height.into()))
    }
}

impl From<Size> for taffy::Size<taffy::Dimension> {
    fn from(value: Size) -> Self {
        taffy::Size {
            width: value.width.into(),
            height: value.height.into(),
        }
    }
}

impl From<Size> for taffy::Size<taffy::LengthPercentage> {
    fn from(value: Size) -> Self {
        taffy::Size {
            width: value.width.into(),
            height: value.height.into(),
        }
    }
}

impl From<Size> for taffy::Size<taffy::LengthPercentageAuto> {
    fn from(value: Size) -> Self {
        taffy::Size {
            width: value.width.into(),
            height: value.height.into(),
        }
    }
}

#[derive(Copy, Debug, Clone, PartialEq)]
pub enum Available {
    /// The amount of space available is the specified number of pixels
    Definite(u32),
    /// The amount of space available is indefinite and the node should be laid out under a min-content constraint
    Min,
    /// The amount of space available is indefinite and the node should be laid out under a max-content constraint
    Max,
}

impl const From<u32> for Available {
    fn from(value: u32) -> Self {
        Self::Definite(value)
    }
}

impl From<usize> for Available {
    fn from(value: usize) -> Self {
        Self::Definite(value as u32)
    }
}
impl const From<Option<Available>> for Available {
    fn from(value: Option<Available>) -> Self {
        value.unwrap_or(Self::Max)
    }
}

// Base properties
#[derive(Copy, Debug, Clone, PartialEq, Default)]
pub enum Dimension {
    #[default]
    Auto,
    Length(u32),
    Percent(f32),
}

impl Dimension {
    pub const DEFAULT: Self = Self::Auto;
    pub const ZERO: Self = Self::Length(0);
    pub const MAX: Self = Self::Percent(1.0);
    
    pub const fn or(self, other: Self) -> Self {
        match (self, other) {
            (Self::Auto, x) => x,
            (x, Self::Auto) => x,
            _ => self,
        }
    }
}

impl const From<u32> for Dimension {
    fn from(value: u32) -> Self {
        Self::Length(value)
    }
}

impl const From<f32> for Dimension {
    fn from(value: f32) -> Self {
        Self::Percent(value)
    }
}

impl const From<Option<Dimension>> for Dimension {
    fn from(value: Option<Dimension>) -> Self {
        value.unwrap_or(Self::Auto)
    }
}

impl From<Dimension> for taffy::LengthPercentage {
    fn from(dim: Dimension) -> Self {
        match dim {
            Dimension::Auto => taffy::LengthPercentage::ZERO,
            Dimension::Length(val) => taffy::LengthPercentage::length(val as f32),
            Dimension::Percent(val) => taffy::LengthPercentage::percent(val),
        }
    }
}

impl From<Dimension> for taffy::LengthPercentageAuto {
    fn from(dim: Dimension) -> Self {
        match dim {
            Dimension::Auto => taffy::LengthPercentageAuto::auto(),
            Dimension::Length(val) => taffy::LengthPercentageAuto::length(val as f32),
            Dimension::Percent(val) => taffy::LengthPercentageAuto::percent(val),
        }
    }
}

impl From<Dimension> for taffy::Dimension {
    fn from(dim: Dimension) -> Self {
        match dim {
            Dimension::Auto => taffy::Dimension::auto(),
            Dimension::Length(val) => taffy::Dimension::length(val as f32),
            Dimension::Percent(val) => taffy::Dimension::percent(val),
        }
    }
}
