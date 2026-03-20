use ansi::{Color};
use geometry::{Size, Axis};

use taffy::{geometry as g, style as s, self as t};
type style = t::Style;

// Base properties


pub type Edges = geometry::Edges;

// Alignment properties

mod alignment {
    #[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
    pub enum Align {
        #[default]
        Start,
        Center,
        End,
    }

    #[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
    pub enum AlignItems {
        #[default]
        /// Items are packed toward the start of the axis
        Start,
        /// Items are packed along the center of the cross axis
        Center,
        /// Items are packed toward the end of the axis
        End,

        /// Items are aligned such as their baselines align
        Baseline,
        /// Stretch to fill the container
        Stretch,
    }

    #[derive(Copy, Clone, PartialEq, Eq, Debug)]
    pub enum AlignContent {
        /// Items are packed toward the start of the axis
        Start,
        /// Items are centered around the middle of the axis
        Center,
        /// Items are packed toward the end of the axis
        End,

        /// The first and last items are aligned flush with the edges of the container (no gap)
        /// The gap between items is distributed evenly.
        SpaceBetween,
        /// The gap between the first and last items is exactly THE SAME as the gap between items.
        /// The gaps are distributed evenly
        SpaceEvenly,
        /// The gap between the first and last items is exactly HALF the gap between items.
        /// The gaps are distributed evenly in proportion to these ratios.
        SpaceAround,

        /// Items are stretched to fill the container
        Stretch,
    }
}

pub type Align = alignment::Align;

pub type AlignSelf = alignment::AlignItems;
pub type AlignContent = alignment::AlignContent;
pub type AlignItems = alignment::AlignItems;

pub type JustifySelf = alignment::AlignItems;
pub type JustifyContent = alignment::AlignContent;
pub type JustifyItems = alignment::AlignItems;

// Flex properties

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum FlexWrap {
    /// Items will not wrap and stay on a single line
    #[default]
    NoWrap,
    /// Items will wrap according to this item's [`FlexDirection`]
    Wrap,
    /// Items will wrap in the opposite direction to this item's [`FlexDirection`]
    WrapReverse,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum FlexDirection {
    /// Defines +x as the main axis
    ///
    /// Items will be added from left to right in a row.
    #[default]
    Row,
    /// Defines +y as the main axis
    ///
    /// Items will be added from top to bottom in a column.
    Column,
    /// Defines -x as the main axis
    ///
    /// Items will be added from right to left in a row.
    RowReverse,
    /// Defines -y as the main axis
    ///
    /// Items will be added from bottom to top in a column.
    ColumnReverse,
}

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
pub type Overflow = t::Overflow;

struct Layout {
    pub color: Color,
    pub background_color: Color,
    pub text_decoration: TextDecorationLine,
    pub font_weight: FontWeight,

    pub overflow: Axis<Overflow>,

    pub padding: Edges,
    pub margin: Edges,
    pub border: Border,
    pub size: Size,
    pub max_size: Size,
    pub min_size: Size,

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
    pub flex_basis: t::Dimension,
    /// The relative rate at which this item grows when it is expanding to fill space
    ///
    /// 0.0 is the default value, and this value must be positive.
    pub flex_grow: f32,
    /// The relative rate at which this item shrinks when it is contracting to fit into space
    ///
    /// 1.0 is the default value, and this value must be positive.
    pub flex_shrink: f32,
}
