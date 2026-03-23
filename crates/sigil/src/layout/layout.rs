use taffy::style_helpers::FromLength;
use ansi::{Color};
use geometry::{Size, Axis};

// Base properties
#[derive_const(Clone, PartialEq, Default)]
#[derive(Copy, Debug)]
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

impl Layout {
    pub const DEFAULT: Self = Self {
        color: Color::None,
        background_color: Color::None,
        text_decoration: TextDecorationLine::None,
        font_weight: FontWeight::Normal,

        padding: Edges::default(),
        margin: Edges::default(),
        border: Border::None,
        size: Size::default(),
        max_size: Size::default(),
        min_size: Size::default(),

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

        overflow: Axis {
            horizontal: Overflow::Visible,
            vertical: Overflow::Visible,
        },
        scrollbar_width: 0,
    };
    pub fn clone_into(&self, layout: &mut taffy::Style) {
        layout.padding.left = self.padding.left.into();
        layout.padding.right = self.padding.right.into();
        layout.padding.top = self.padding.top.into();
        layout.padding.bottom = self.padding.bottom.into();
        layout.margin.left = self.margin.left.into();
        layout.margin.right = self.margin.right.into();
        layout.margin.top = self.margin.top.into();
        layout.margin.bottom = self.margin.bottom.into();
        layout.border = match self.border {
            Border::None => taffy::Rect::zero(),
            Border::Solid => taffy::Rect {
                left: taffy::LengthPercentage::length(1.0),
                right: taffy::LengthPercentage::length(1.0),
                top: taffy::LengthPercentage::length(1.0),
                bottom: taffy::LengthPercentage::length(1.0),
            },
        };
        layout.size.width = self.size.width.into();
        layout.size.height = self.size.height.into();
        layout.max_size.width = self.max_size.width.into();
        layout.max_size.height = self.max_size.height.into();
        layout.min_size.width = self.min_size.width.into();
        layout.min_size.height = self.min_size.height.into();
        layout.align_items = self.align_items;
        layout.align_self = self.align_self;
        layout.justify_items = self.justify_items;
        layout.justify_self = self.justify_self;
        layout.align_content = self.align_content;
        layout.justify_content = self.justify_content;
        layout.flex_direction = self.flex_direction;
        layout.flex_wrap = self.flex_wrap;
        layout.flex_basis = self.flex_basis.into();
        layout.flex_grow = self.flex_grow;
        layout.flex_shrink = self.flex_shrink;
        layout.overflow.x = self.overflow.horizontal;
        layout.overflow.y = self.overflow.vertical;

        layout.scrollbar_width = self.scrollbar_width as f32;
    }
}

impl From<Layout> for taffy::Style {
    fn from(value: Layout) -> Self {
        let mut taffy_style = Self::default();
        value.clone_into(&mut taffy_style);
        taffy_style
    }
}