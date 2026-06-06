use taffy::prelude::TaffyZero;

use derive_more::{Deref, DerefMut, From};

/// A set of edges with a specific value for each edge.
#[derive(Copy, Debug, Clone, PartialEq, Default, Deref, DerefMut)]
#[repr(transparent)]
pub struct Edges(geometry::Edges<Length>);

impl Edges {
    pub const AUTO: Self = Self::all(Length::Auto);
    pub const ZERO: Self = Self::all(Length::Value(0));

    pub const fn auto() -> Self {
        Self::AUTO
    }

    pub const fn length(value: u32) -> Self {
        Self::new(
            Length::Value(value),
            Length::Value(value),
            Length::Value(value),
            Length::Value(value),
        )
    }

    pub const fn percent(value: f32) -> Self {
        Self::new(
            Length::Percent(value),
            Length::Percent(value),
            Length::Percent(value),
            Length::Percent(value),
        )
    }

    pub const fn new(
        top: impl [const] Into<Length>,
        right: impl [const] Into<Length>,
        bottom: impl [const] Into<Length>,
        left: impl [const] Into<Length>,
    ) -> Self {
        Self(geometry::Edges::new(
            top.into(),
            right.into(),
            bottom.into(),
            left.into(),
        ))
    }

    pub const fn all(value: impl [const] Into<Length> + Copy) -> Self {
        Self::new(value.into(), value.into(), value.into(), value.into())
    }

    pub const fn horizontal(value: impl [const] Into<Length> + Copy) -> Self {
        Self::new(value.into(), value.into(), value.into(), value.into())
    }

    pub const fn vertical(value: impl [const] Into<Length> + Copy) -> Self {
        Self::new(value.into(), value.into(), value.into(), value.into())
    }
}

impl<T: Into<Length>> From<T> for Edges {
    fn from(value: T) -> Self {
        let value = value.into();
        Self::new(value, value, value, value)
    }
}

impl<T: Into<Length>> From<(T, T)> for Edges {
    fn from(value: (T, T)) -> Self {
        let (vertical, horizontal) = (value.0.into(), value.1.into());
        Self::new(vertical, horizontal, vertical, horizontal)
    }
}

impl<T: Into<Length>> From<(T, T, T, T)> for Edges {
    fn from(value: (T, T, T, T)) -> Self {
        Self::new(
            value.0.into(),
            value.1.into(),
            value.2.into(),
            value.3.into(),
        )
    }
}

impl<T: From<Length>> From<Edges> for taffy::Rect<T> {
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
    pub const ZERO: Self = Self(geometry::Size::new(
        Available::Pixel(0),
        Available::Pixel(0),
    ));
    pub const MIN: Self = Self(geometry::Size::new(Available::Min, Available::Min));
    pub const MAX: Self = Self(geometry::Size::new(Available::Max, Available::Max));

    pub const fn new(
        width: impl [const] Into<Available>,
        height: impl [const] Into<Available>,
    ) -> Self {
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
        Self::new(
            Available::Pixel(value.width as u32),
            Available::Pixel(value.height as u32),
        )
    }
}

impl<T: From<Available>> From<Space> for taffy::Size<T> {
    fn from(space: Space) -> Self {
        taffy::Size {
            width: space.width.into(),
            height: space.height.into(),
        }
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Default, Deref, DerefMut)]
#[repr(transparent)]
pub struct Gap(geometry::Sides<Length>);

impl Gap {
    pub const ZERO: Self = Self(geometry::Sides::new(Length::Value(0), Length::Value(0)));
    pub const AUTO: Self = Self(geometry::Sides::new(Length::Auto, Length::Auto));

    pub const fn auto() -> Self {
        Self::AUTO
    }

    pub const fn length(value: u32) -> Self {
        Self::new(Length::Value(value), Length::Value(value))
    }

    pub const fn percent(value: f32) -> Self {
        Self::new(Length::Percent(value), Length::Percent(value))
    }

    pub const fn new(
        horizontal: impl [const] Into<Length>,
        vertical: impl [const] Into<Length>,
    ) -> Self {
        Self(geometry::Sides::new(horizontal.into(), vertical.into()))
    }

    pub const fn horizontal(value: impl [const] Into<Length> + Copy) -> Self {
        Self::new(value.into(), Length::Auto)
    }

    pub const fn vertical(value: impl [const] Into<Length> + Copy) -> Self {
        Self::new(Length::Auto, value.into())
    }

    pub const fn both(value: impl [const] Into<Length> + Copy) -> Self {
        Self(geometry::Sides::both(value.into()))
    }
}

impl<T: From<Length>> From<Gap> for taffy::Size<T> {
    fn from(value: Gap) -> Self {
        taffy::Size {
            width: value.horizontal.into(),
            height: value.vertical.into(),
        }
    }
}

/// A size with a specific value for each dimension.
#[derive(Copy, Debug, Clone, PartialEq, Default, Deref, DerefMut)]
#[repr(transparent)]
pub struct Size(geometry::Size<Length>);

impl Size {
    pub const ZERO: Self = Self(geometry::Size::new(Length::Value(0), Length::Value(0)));
    pub const AUTO: Self = Self(geometry::Size::new(Length::Auto, Length::Auto));

    pub const fn auto() -> Self {
        Self::AUTO
    }

    pub const fn length(value: u32) -> Self {
        Self::new(Length::Value(value), Length::Value(value))
    }

    pub const fn percent(value: f32) -> Self {
        Self::new(Length::Percent(value), Length::Percent(value))
    }

    pub const fn new(width: impl [const] Into<Length>, height: impl [const] Into<Length>) -> Self {
        Self(geometry::Size::new(width.into(), height.into()))
    }
}

impl<T: From<Length>> From<Size> for taffy::Size<T> {
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
    Pixel(u32),
    /// The amount of space available is indefinite and the node should be laid out under a min-content constraint
    Min,
    /// The amount of space available is indefinite and the node should be laid out under a max-content constraint
    Max,
}

impl const From<u32> for Available {
    fn from(value: u32) -> Self {
        Self::Pixel(value)
    }
}

impl From<usize> for Available {
    fn from(value: usize) -> Self {
        Self::Pixel(value as u32)
    }
}
impl const From<Option<Available>> for Available {
    fn from(value: Option<Available>) -> Self {
        value.unwrap_or(Self::Max)
    }
}

impl From<Available> for taffy::AvailableSpace {
    fn from(value: Available) -> Self {
        match value {
            Available::Pixel(val) => taffy::AvailableSpace::Definite(val as f32),
            Available::Min => taffy::AvailableSpace::MinContent,
            Available::Max => taffy::AvailableSpace::MaxContent,
        }
    }
}

// Base properties
#[derive(Copy, Debug, Clone, PartialEq, Default)]
pub enum Length {
    #[default]
    Auto,
    Value(u32),
    Percent(f32),
}

impl Length {
    pub const DEFAULT: Self = Self::Auto;
    pub const ZERO: Self = Self::Value(0);
    pub const MIN: Self = Self::ZERO;
    pub const MAX: Self = Self::Percent(1.0);
}

impl const From<u32> for Length {
    fn from(value: u32) -> Self {
        Self::Value(value)
    }
}

impl const From<f32> for Length {
    fn from(value: f32) -> Self {
        Self::Percent(value)
    }
}

impl const From<Option<Length>> for Length {
    fn from(value: Option<Length>) -> Self {
        value.unwrap_or(Self::Auto)
    }
}

impl From<Length> for taffy::LengthPercentage {
    fn from(dim: Length) -> Self {
        match dim {
            Length::Auto => taffy::LengthPercentage::ZERO,
            Length::Value(val) => taffy::LengthPercentage::length(val as f32),
            Length::Percent(val) => taffy::LengthPercentage::percent(val),
        }
    }
}

impl From<Length> for taffy::LengthPercentageAuto {
    fn from(dim: Length) -> Self {
        match dim {
            Length::Auto => taffy::LengthPercentageAuto::auto(),
            Length::Value(val) => taffy::LengthPercentageAuto::length(val as f32),
            Length::Percent(val) => taffy::LengthPercentageAuto::percent(val),
        }
    }
}

impl From<Length> for taffy::Dimension {
    fn from(dim: Length) -> Self {
        match dim {
            Length::Auto => taffy::Dimension::auto(),
            Length::Value(val) => taffy::Dimension::length(val as f32),
            Length::Percent(val) => taffy::Dimension::percent(val),
        }
    }
}

impl From<Length> for taffy::CompactLength {
    fn from(dim: Length) -> Self {
        match dim {
            Length::Auto => taffy::CompactLength::auto(),
            Length::Value(val) => taffy::CompactLength::length(val as f32),
            Length::Percent(val) => taffy::CompactLength::percent(val),
        }
    }
}
