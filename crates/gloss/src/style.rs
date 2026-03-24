use bon::{bon, Builder};
use bon::__::IsSet;
use ansi::Color;
use geometry::{Axis, Size};

pub type BorderStyle = crate::symbols::BorderStyle;
pub enum Available {
    Definite(u32),
    Min,
    Max,
}

impl From<Available> for taffy::AvailableSpace {
    fn from(value: Available) -> Self {
        match value {
            Available::Definite(val) => taffy::AvailableSpace::Definite(val as f32),
            Available::Min => taffy::AvailableSpace::MinContent,
            Available::Max => taffy::AvailableSpace::MaxContent,
        }
    }
}

pub type Space = Size<Available>;


// Base properties
#[derive(Copy, Debug, Clone, PartialEq, Default)]
pub enum Dimension {
    #[default]
    Auto,
    Length(u32),
    Percent(f32),
}

impl From<Dimension> for taffy::LengthPercentage {
    fn from(value: Dimension) -> Self {
        match value {
            Dimension::Auto => taffy::LengthPercentage::length(0.0),
            Dimension::Length(val) => taffy::LengthPercentage::length(val as f32),
            Dimension::Percent(val) => taffy::LengthPercentage::percent(val),
        }
    }
}

impl From<Dimension> for taffy::LengthPercentageAuto {
    fn from(input: Dimension) -> Self {
        match input {
            Dimension::Auto => taffy::LengthPercentageAuto::auto(),
            Dimension::Length(val) => taffy::LengthPercentageAuto::length(val as f32),
            Dimension::Percent(val) => taffy::LengthPercentageAuto::percent(val),
        }
    }
}

impl From<Dimension> for taffy::Dimension {
    fn from(input: Dimension) -> Self {
        match input {
            Dimension::Auto => taffy::Dimension::auto(),
            Dimension::Length(val) => taffy::Dimension::length(val as f32),
            Dimension::Percent(val) => taffy::Dimension::percent(val),
        }
    }
}

#[derive(Copy, Debug, Clone, PartialEq, Default)]
pub enum Display {
    /// The node is hidden, and it's children will also be hidden
    None,
    Inline,
    Block,
    #[default]
    Flex,

}
pub use geometry::Edges;

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

impl From<ItemAlignment> for taffy::AlignItems {
    fn from(value: ItemAlignment) -> Self {
        match value {
            ItemAlignment::Start => taffy::AlignItems::Start,
            ItemAlignment::End => taffy::AlignItems::End,
            ItemAlignment::Center => taffy::AlignItems::Center,
            ItemAlignment::Baseline => taffy::AlignItems::Baseline,
            ItemAlignment::Stretch => taffy::AlignItems::Stretch,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
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

impl From<ContentAlignment> for taffy::AlignContent {
    fn from(value: ContentAlignment) -> Self {
        match value {
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

pub type AlignSelf = AlignItems;
pub type AlignContent = ContentAlignment;
pub type AlignItems = ItemAlignment;

pub type JustifySelf = JustifyItems;
pub type JustifyContent = ContentAlignment;
pub type JustifyItems = ItemAlignment;

pub type Overflow = taffy::Overflow;

// Flex properties

pub type FlexWrap = taffy::FlexWrap;
pub type FlexDirection = taffy::FlexDirection;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum TextDecorationLine {
    #[default]
    None,
    Underline,
    LineThrough,
}
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum FontWeight {
    #[default]
    Normal,
    Bold,
}
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
}


// Decoration properties
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum Border {
    #[default]
    None,
    Solid
}

pub struct Layout {

}

#[derive(Clone, Builder, Copy, PartialEq, Debug)]
#[builder(derive(Debug))]
pub struct Style {
    #[builder(default)]
    pub color: Color,
    #[builder(default)]
    pub background_color: Color,
    #[builder(default)]
    pub text_decoration: TextDecorationLine,
    #[builder(default)]
    pub font_weight: FontWeight,
    #[builder(default)]
    pub font_style: FontStyle,

    #[builder(default)]
    pub display: Display,

    #[builder(default = Edges { left: Dimension::Auto, right: Dimension::Auto, top: Dimension::Auto, bottom: Dimension::Auto })]
    pub padding: Edges<Dimension>,
    #[builder(default = Edges { left: Dimension::Auto, right: Dimension::Auto, top: Dimension::Auto, bottom: Dimension::Auto })]
    pub margin: Edges<Dimension>,

    #[builder(default)]
    pub border: Border,

    #[builder(default = Size { width: Dimension::Auto, height: Dimension::Auto })]
    pub size: Size<Dimension>,
    #[builder(default = Size { width: Dimension::Auto, height: Dimension::Auto })]
    pub max_size: Size<Dimension>,
    #[builder(default = Size { width: Dimension::Auto, height: Dimension::Auto })]
    pub min_size: Size<Dimension>,

    // Alignment properties
    /// How this node's children aligned in the cross/block axis?
    pub align_items: Option<AlignItems>,
    /// How this node should be aligned in the cross/block axis
    /// Falls back to the parents [`AlignItems`] if not set
    pub align_self: Option<AlignSelf>,
    /// How this node's children should be aligned in the inline axis
    pub justify_items: Option<JustifyItems>,

    /// How this node should be aligned in the inline axis
    /// Falls back to the parents [`JustifyItems`] if not set
    pub justify_self: Option<AlignItems>,
    /// How should content contained within this item be aligned in the cross/block axis
    pub align_content: Option<AlignContent>,
    /// How should content contained within this item be aligned in the main/inline axis
    pub justify_content: Option<JustifyContent>,



    /// Which direction does the main axis flow in?
    #[builder(default)]
    pub flex_direction: FlexDirection,
    /// Should elements wrap, or stay in a single line?
    #[builder(default)]
    pub flex_wrap: FlexWrap,
    /// Sets the initial main axis size of the item
    #[builder(default)]
    pub flex_basis: Dimension,
    /// The relative rate at which this item grows when it is expanding to fill space
    ///
    /// 0.0 is the default value, and this value must be positive.
    #[builder(default)]
    pub flex_grow: f32,
    /// The relative rate at which this item shrinks when it is contracting to fit into space
    ///
    /// 1.0 is the default value, and this value must be positive.
    #[builder(default)]
    pub flex_shrink: f32,

    #[builder(default)]
    pub gap: Axis<Dimension>,
}
#[allow(non_snake_case, non_upper_case_globals)]
impl Style {
    pub const Default: Self = Self {
        color: Color::None,
        background_color: Color::None,
        text_decoration: TextDecorationLine::None,
        font_weight: FontWeight::Normal,
        font_style: FontStyle::Normal,
        display: Display::Flex,
        padding: Edges { top: Dimension::Auto, right: Dimension::Auto, bottom: Dimension::Auto, left: Dimension::Auto },
        margin: Edges { top: Dimension::Auto, right: Dimension::Auto, bottom: Dimension::Auto, left: Dimension::Auto },
        border: Border::None,
        size: Size { width: Dimension::Auto, height: Dimension::Auto },
        max_size: Size { width: Dimension::Auto, height: Dimension::Auto },
        min_size: Size { width: Dimension::Auto, height: Dimension::Auto },
        align_items: None,
        align_self: None,
        justify_items: None,
        justify_self: None,
        align_content: None,
        justify_content: None,
        flex_direction: FlexDirection::Column,
        flex_wrap: FlexWrap::NoWrap,
        flex_basis: Dimension::Auto,
        flex_grow: 0.0,
        flex_shrink: 1.0,
        gap: Axis { horizontal: Dimension::Auto, vertical: Dimension::Auto },
    };
    pub const Inline: Self = Self {
             display: Display::Inline,
             ..Self::Default
    };
    pub const Block: Self = Self {
             display: Display::Block,
             ..Self::Default
    };
    pub const Flex: Self =  Self {
             display: Display::Flex,
             ..Self::Default
    };

    pub fn new() -> StyleBuilder {
        Style::builder()
    }
}
impl Default for Style {
    fn default() -> Self {
        Self::Default
    }
}
impl taffy::CoreStyle for Style {
    type CustomIdent = String;

    #[inline(always)]
    fn box_generation_mode(&self) -> taffy::BoxGenerationMode {
        match self.display {
            Display::None => taffy::BoxGenerationMode::None,
            _ => taffy::BoxGenerationMode::Normal,
        }
    }
    #[inline(always)]
    fn is_block(&self) -> bool {
        matches!(self.display, Display::Block)
    }
    #[inline(always)]
    fn size(&self) -> taffy::Size<taffy::Dimension> {
        taffy::Size {
            width: self.size.width.into(),
            height: self.size.height.into(),
        }
    }
    #[inline(always)]
    fn min_size(&self) -> taffy::Size<taffy::Dimension> {
        taffy::Size {
            width: self.min_size.width.into(),
            height: self.min_size.height.into(),
        }
    }
    #[inline(always)]
    fn max_size(&self) -> taffy::Size<taffy::Dimension> {
        taffy::Size {
            width: self.max_size.width.into(),
            height: self.max_size.height.into(),
        }
    }

    #[inline(always)]
    fn margin(&self) -> taffy::Rect<taffy::LengthPercentageAuto> {
        taffy::Rect {
            left: self.margin.left.into(),
            right: self.margin.right.into(),
            top: self.margin.top.into(),
            bottom: self.margin.bottom.into(),
        }
    }
    #[inline(always)]
    fn padding(&self) -> taffy::Rect<taffy::LengthPercentage> {
        taffy::Rect {
            left: self.padding.left.into(),
            right: self.padding.right.into(),
            top: self.padding.top.into(),
            bottom: self.padding.bottom.into(),
        }
    }
    #[inline(always)]
    fn border(&self) -> taffy::Rect<taffy::LengthPercentage> {
        match self.border {
            Border::None => taffy::Rect::zero(),
            Border::Solid => taffy::Rect::length(1.0),
        }
    }
}

impl taffy::BlockContainerStyle for Style {
}

impl taffy::BlockItemStyle for Style {
}

impl taffy::FlexboxContainerStyle for Style {
    #[inline(always)]
    fn flex_direction(&self) -> FlexDirection {
        self.flex_direction
    }
    #[inline(always)]
    fn flex_wrap(&self) -> taffy::FlexWrap {
        self.flex_wrap
    }
    #[inline(always)]
    fn gap(&self) -> taffy::Size<taffy::LengthPercentage> {
        taffy::Size {
            width: self.gap.horizontal.into(),
            height: self.gap.vertical.into(),
        }
    }
    #[inline(always)]
    fn align_content(&self) -> Option<taffy::AlignContent> {
        self.align_content.map(Into::into)
    }
    #[inline(always)]
    fn align_items(&self) -> Option<taffy::AlignItems> {
        self.align_items.map(Into::into)
    }
    #[inline(always)]
    fn justify_content(&self) -> Option<taffy::JustifyContent> {
        self.justify_content.map(Into::into)
    }
}


impl taffy::FlexboxItemStyle for Style {
    #[inline(always)]
    fn flex_basis(&self) -> taffy::Dimension {
        self.flex_basis.into()
    }
    #[inline(always)]
    fn flex_grow(&self) -> f32 {
        self.flex_grow
    }
    #[inline(always)]
    fn flex_shrink(&self) -> f32 {
        self.flex_shrink
    }
    #[inline(always)]
    fn align_self(&self) -> Option<taffy::AlignSelf> {
        self.align_self.map(Into::into)
    }
}
