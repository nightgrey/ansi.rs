use taffy::prelude::TaffyZero;

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
