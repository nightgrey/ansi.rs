use taffy::style_helpers::FromLength;
use ansi::{Color};
use geometry::{Size, Axis};

// Base properties
#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub enum Dimension {
    #[default]
    Auto,
    Length(u32),
    Percent(f32),
}

impl From<Dimension> for taffy::Dimension {
    fn from(value: Dimension) -> Self {
        match value {
            Dimension::Auto => taffy::Dimension::auto(),
            Dimension::Length(val) => taffy::Dimension::length(val as f32),
            Dimension::Percent(val) => taffy::Dimension::percent(val),
        }
    }
}
impl From<Dimension> for taffy::LengthPercentage {
    fn from(value: Dimension) -> Self {
        match value {
            Dimension::Auto => taffy::LengthPercentage::from_length(0.0),
            Dimension::Length(val) => taffy::LengthPercentage::length(val as f32),
            Dimension::Percent(val) => taffy::LengthPercentage::percent(val),
        }
    }
}
impl From<Dimension> for taffy::LengthPercentageAuto {
    fn from(value: Dimension) -> Self {
        match value {
            Dimension::Auto => taffy::LengthPercentageAuto::auto(),
            Dimension::Length(val) => taffy::LengthPercentageAuto::length(val as f32),
            Dimension::Percent(val) => taffy::LengthPercentageAuto::percent(val),
        }
    }
}

pub type Edges = geometry::Edges<Dimension>;

pub type AlignSelf = taffy::AlignItems;
pub type AlignContent = taffy::AlignContent;
pub type AlignItems = taffy::AlignItems;

pub type JustifySelf = taffy::AlignItems;
pub type JustifyContent = taffy::AlignContent;
pub type JustifyItems = taffy::AlignItems;

// Flex properties

pub type FlexWrap = taffy::FlexWrap;
pub type FlexDirection = taffy::FlexDirection;

// Decoration properties

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum Border {
    #[default]
    None,
    Solid
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum FontWeight {
    #[default]
    Normal,
    Bold,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum TextDecorationLine {
    #[default]
    None,
    Underline,
    LineThrough,
}

// Other properties
pub type Overflow = taffy::Overflow;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Layout {
    pub color: Color,
    pub background_color: Color,
    pub text_decoration: TextDecorationLine,
    pub font_weight: FontWeight,

    pub padding: Edges,
    pub margin: Edges,
    pub border: Border,
    pub size: Size<Dimension>,
    pub max_size: Size<Dimension>,
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

    // Flexbox container properties
    /// Which direction does the main axis flow in?
    pub flex_direction: FlexDirection,
    /// Should elements wrap, or stay in a single line?
    pub flex_wrap: FlexWrap,

    // Flexbox item properties
    /// Sets the initial main axis size of the item
    pub flex_basis: Dimension,
    /// The relative rate at which this item grows when it is expanding to fill space
    ///
    /// 0.0 is the default value, and this value must be positive.
    pub flex_grow: f32,
    /// The relative rate at which this item shrinks when it is contracting to fit into space
    ///
    /// 1.0 is the default value, and this value must be positive.
    pub flex_shrink: f32,


    pub overflow: Axis<Overflow>,
    pub scrollbar_width: usize,
}

impl From<Layout> for taffy::Style {
    fn from(value: Layout) -> Self {
        Self {
            padding: taffy::Rect {
                left: value.padding.left.into(),
                right: value.padding.right.into(),
                top: value.padding.top.into(),
                bottom: value.padding.bottom.into(),
            },
            margin: taffy::Rect {
                left: value.margin.left.into(),
                right: value.margin.right.into(),
                top: value.margin.top.into(),
                bottom: value.margin.bottom.into(),
            },
            border: match value.border {
                Border::None => taffy::Rect::zero(),
                Border::Solid => taffy::Rect {
                    left: taffy::LengthPercentage::length(1.0),
                    right: taffy::LengthPercentage::length(1.0),
                    top: taffy::LengthPercentage::length(1.0),
                    bottom: taffy::LengthPercentage::length(1.0),
                },
            },
            size: taffy::Size {
                width: value.size.width.into(),
                height: value.size.height.into(),
            },
            max_size: taffy::Size {
                width: value.max_size.width.into(),
                height: value.max_size.height.into(),
            },
            min_size: taffy::Size {
                width: value.min_size.width.into(),
                height: value.min_size.height.into(),
            },
            align_items: value.align_items,
            align_self: value.align_self,
            justify_items: value.justify_items,
            justify_self: value.justify_self,
            align_content: value.align_content,
            justify_content: value.justify_content,
            flex_direction: value.flex_direction,
            flex_wrap: value.flex_wrap,
            flex_basis: value.flex_basis.into(),
            flex_grow: value.flex_grow,
            flex_shrink: value.flex_shrink,
            overflow: taffy::Point { x: value.overflow.horizontal, y: value.overflow.vertical },
            scrollbar_width: value.scrollbar_width as f32,
            ..Default::default()
        }
    }
}