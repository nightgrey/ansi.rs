// Inline styling

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum TextDecoration {
    #[default]
    None,
    Underline,
    LineThrough,
}

impl From<bool> for TextDecoration {
    fn from(value: bool) -> Self {
        if value { Self::Underline } else { Self::None }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum FontWeight {
    #[default]
    Normal,
    Bold,
}

impl From<usize> for FontWeight {
    fn from(value: usize) -> Self {
        match value {
            100..=400 => Self::Normal,
            500..=800 => Self::Bold,
            _ => Self::Normal,
        }
    }
}
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u8)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
}

// Block styling

#[derive(Copy, Debug, Clone, PartialEq, Default)]
pub enum Display {
    /// The node is hidden, and it's children will also be hidden
    None,
    Inline,
    #[default]
    Block,
    Flex,
}

pub type Border = crate::symbols::BorderStyle;

pub type FlexWrap = taffy::FlexWrap;
pub type FlexDirection = taffy::FlexDirection;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ItemAlignment {
    /// Items are packed toward the start of the axis
    Start,
    /// Items are packed toward the end of the axis
    End,
    /// Items are packed along the center of the cross axis
    Center,
    /// Items are aligned such as their baselines align
    Baseline,
    /// Stretch to fill the container
    Stretch,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum ContentAlignment {
    /// Items are packed toward the start of the axis
    Start,
    /// Items are packed toward the end of the axis
    End,
    /// Items are centered around the middle of the axis
    Center,
    /// Items are stretched to fill the container
    Stretch,
    /// The first and last items are aligned flush with the edges of the container (no gap)
    /// The gap between items is distributed evenly.
    SpaceBetween,
    /// The gap between the first and last items is exactly THE SAME as the gap between items.
    /// The gaps are distributed evenly
    SpaceEvenly,
    /// The gap between the first and last items is exactly HALF the gap between items.
    /// The gaps are distributed evenly in proportion to these ratios.
    SpaceAround,
}

pub type AlignSelf = AlignItems;
pub type AlignContent = ContentAlignment;
pub type AlignItems = ItemAlignment;

pub type JustifySelf = JustifyItems;
pub type JustifyContent = ContentAlignment;
pub type JustifyItems = ItemAlignment;

impl Into<taffy::AlignItems> for ItemAlignment {
    fn into(self) -> taffy::AlignItems {
        match self {
            ItemAlignment::Start => taffy::AlignItems::Start,
            ItemAlignment::End => taffy::AlignItems::End,
            ItemAlignment::Center => taffy::AlignItems::Center,
            ItemAlignment::Baseline => taffy::AlignItems::Baseline,
            ItemAlignment::Stretch => taffy::AlignItems::Stretch,
        }
    }
}

impl Into<taffy::AlignContent> for ContentAlignment {
    fn into(self) -> taffy::AlignContent {
        match self {
            ContentAlignment::Start => taffy::AlignContent::Start,
            ContentAlignment::End => taffy::AlignContent::End,
            ContentAlignment::Center => taffy::AlignContent::Center,
            ContentAlignment::Stretch => taffy::AlignContent::Stretch,
            ContentAlignment::SpaceBetween => taffy::AlignContent::SpaceBetween,
            ContentAlignment::SpaceEvenly => taffy::AlignContent::SpaceEvenly,
            ContentAlignment::SpaceAround => taffy::AlignContent::SpaceAround,
        }
    }
}
