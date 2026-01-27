use crate::geometry::{Rect, Size};
use crate::Point;

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Edges {
    pub top: usize,
    pub right: usize,
    pub bottom: usize,
    pub left: usize,
}

impl Edges {
    pub const ZERO: Self = Self {
        top: 0,
        right: 0,
        bottom: 0,
        left: 0,
    };

    pub const fn new(top: usize, right: usize, bottom: usize, left: usize) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
}

    pub const fn sides(x: usize, y: usize) -> Self {
        Self {
            top: y,
            right: x,
            bottom: y,
            left: x,
        }
    }

    pub const fn all(n: usize) -> Self {
        Self {
            top: n,
            right: n,
            bottom: n,
            left: n,
        }
    }

    pub fn horizontal(&self) -> usize {
        self.left + self.right
    }

    pub fn vertical(&self) -> usize {
        self.top + self.bottom
    }
}

impl Rect {
    pub const fn shrink(&self, edges: &Edges) -> Self {
        Self {
            min: Point {
                x: self.min.x + edges.left,
                y: self.min.y + edges.top,
            },
            max: Point {
                x: self.max.x.saturating_sub(edges.right),
                y: self.max.y.saturating_sub(edges.bottom),
            },
        }
    }

}
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum Align {
    #[default]
    Start,
    Center,
    End,
}
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Alignment {
    pub x: Align,
    pub y: Align,
}

impl Alignment {
    pub fn offset(&self, outer: Size, inner: Size) -> Point {
        Point {
            x: match self.x {
                Align::Start => 0,
                Align::Center => outer.width.saturating_sub(inner.width) / 2,
                Align::End => outer.width.saturating_sub(inner.width),
            },
            y: match self.y {
                Align::Start => 0,
                Align::Center => outer.height.saturating_sub(inner.height) / 2,
                Align::End => outer.height.saturating_sub(inner.height),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Constraint {
    #[default]
    Auto,
    Min(usize),
    Max(usize),
    Fixed(usize),
    Between(usize, usize),
    Fill,
}

impl Constraint {
    pub fn clamp(&self, value: usize) -> usize {
        match self {
            Self::Auto => value,
            &Self::Min(min) => min.max(value),
            &Self::Max(max) => max.min(value),
            &Self::Fixed(fixed) => fixed.min(value),
            &Self::Between(min, max) => min.max(value).min(max),
            Self::Fill => value,
        }
    }

    pub fn min(&self) -> Option<usize> {
        match *self {
            Self::Min(min) | Self::Between(min, ..) | Self::Fixed(min) => Some(min),
            _ => None,
        }
    }

    pub fn min_or(&self, default: usize) -> usize {
        self.min().unwrap_or(default)
    }

    pub fn max(&self) -> Option<usize> {
        match *self {
            Self::Max(max) | Self::Between(_, max) | Self::Fixed(max) => Some(max),
            _ => None,
        }
    }

    pub fn max_or(&self, default: usize) -> usize {
        self.max().unwrap_or(default)
    }

    pub fn fixed(&self) -> Option<usize> {
        match self {
            Self::Fixed(fixed) => Some(*fixed),
            _ => None,
        }
    }

    /// Merge with other constraint
    pub fn constrain(&self, other: Constraint) -> Constraint {
        match (*self, other) {
            // self is Auto → inherit other
            (Self::Auto, other) => other,
            // self is Fill → expand to other's max
            (Self::Fill, other) => Constraint::Fixed(other.max_or(usize::MAX)),
            // self specifies → use self, but clamp to other bounds
            (constraint, _) => constraint,
        }
    }

    pub fn shrink(&self, amount: usize) -> Self {
        match *self {
            Self::Auto | Self::Fill => *self,
            Self::Min(n) => Self::Min(n.saturating_sub(amount)),
            Self::Max(n) => Self::Max(n.saturating_sub(amount)),
            Self::Fixed(n) => Self::Fixed(n.saturating_sub(amount)),
            Self::Between(min, max) => Self::Between(
                min.saturating_sub(amount),
                max.saturating_sub(amount),
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Constraints {
    pub width: Constraint,
    pub height: Constraint,
}

#[allow(non_snake_case)]
impl Constraints {
    pub fn Fixed(width: usize, height: usize) -> Self {
        Self::new(Constraint::Fixed(width), Constraint::Fixed(height))
    }

    pub fn Max(width: usize, height: usize) -> Self {
        Self::new(Constraint::Max(width), Constraint::Max(height))
    }

    pub fn Min(width: usize, height: usize) -> Self {
        Self::new(Constraint::Min(width), Constraint::Min(height))
    }

    pub fn Auto() -> Self {
        Self::new(Constraint::Auto, Constraint::Auto)
    }

    pub fn new(width: Constraint, height: Constraint) -> Self {
        Self { width, height }
    }

    pub fn clamp(&self, width: usize, height: usize) -> Size {
        Size {
            width: self.width.clamp(width),
            height: self.height.clamp(height),
        }
    }
    pub fn constrain(&self, other: Constraints) -> Constraints {
       Self {
           width: self.width.constrain(other.width),
           height: self.height.constrain(other.height),
       }
    }

    pub fn shrink(&self, insets: &Edges) -> Self {
        Self {
            width: self.width.shrink(insets.horizontal()),
            height: self.height.shrink(insets.vertical()),
        }
    }

    pub fn min(&self) -> Size {
        Size {
            width: self.width.min_or(0),
            height: self.height.min_or(0),
        }
    }

    pub fn max(&self) -> Size {
        Size {
            width: self.width.max_or(0),
            height: self.height.max_or(0),
        }
    }
}

impl From<Rect> for Constraints {
    fn from(value: Rect) -> Self {
        Self::Fixed(value.width(), value.height())
    }
}