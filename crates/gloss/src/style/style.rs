use compact_str::CompactString;
use crate::style::*;
use ansi::Color;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Style {
    pub min_size: Size,
    pub size: Size,
    pub max_size: Size,
    pub padding: Edges,
    pub margin: Edges,
    pub gap: Gap,
    pub flex_direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub flex_basis: Dimension,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub align_items: Option<AlignItems>,
    pub align_self: Option<AlignSelf>,
    pub align_content: Option<AlignContent>,
    pub justify_items: Option<JustifyItems>,
    pub justify_self: Option<AlignSelf>,
    pub justify_content: Option<JustifyContent>,
    pub border: Border,
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub text_decoration: Option<TextDecoration>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
    pub display: Display,
}

#[allow(non_upper_case_globals)]
impl Style {
    pub const DEFAULT: Style = Style {
        display: Display::Flex,
        margin: Edges::ZERO,
        padding: Edges::ZERO,
        border: Border::None,
        color: None,
        background: None,
        text_decoration: None,
        font_weight: None,
        size: Size::AUTO,
        min_size: Size::AUTO,
        max_size: Size::AUTO,
        gap: Gap::ZERO,
        // Alignment
        align_items: None,
        align_self: None,
        justify_items: None,
        justify_self: None,
        align_content: None,
        justify_content: None,
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::NoWrap,
        flex_grow: 0.0,
        flex_shrink: 1.0,
        flex_basis: Dimension::Auto,
        font_style: None,
    };

    pub fn inherit(self, from: Self) -> Self {
        let mut result = self;

        match (from.color, self.color) {
            (Some(_), Some(x)) => {
                result.color = Some(x);
            },
            (Some(x), None) => {
                result.color = Some(x);
            }
            (None, Some(x)) => {
                result.color = Some(x);
            }
            (None, None) => (),
        };

        match (from.background, self.background) {
            (Some(_), Some(x)) => {
                result.background = Some(x);
            },
            (Some(x), None) => {
                result.background = Some(x);
            }
            (None, Some(x)) => {
                result.background = Some(x);
            }
            (None, None) => (),
        };
        
        match (from.text_decoration, self.text_decoration) {
            (Some(_), Some(x)) => {
                result.text_decoration = Some(x);
            }
            (Some(x), None) => {
                result.text_decoration = Some(x);
            }
            (None, Some(x)) => {
                result.text_decoration = Some(x);
            }
            (None, None) => (),
        };
        
        match (from.font_style, self.font_style) {
            (Some(_), Some(x)) => {
                result.font_style = Some(x);
            },
            (Some(x), None) => {
                result.font_style = Some(x);
            },
            (None, Some(x)) => {
                result.font_style = Some(x);
            },
            (None, None) => (),
        };

        result
    }


    pub fn is_default(&self) -> bool {
        self == &Self::DEFAULT
    }
}

macro_rules! property {
    ($bitset_property:ident -> fn $struct_property:ident($arg_ident:ident: Into<$arg_ty:ty>) -> $arg_return:ty) => {
        property!(
            $bitset_property -> fn $struct_property($arg_ident: impl Into<$arg_ty>) -> $arg_return {
                $arg_ident.into()
            }
        );
    };

    ($bitset_property:ident -> fn $struct_property:ident($arg_ident:ident: $arg_ty:ty) -> $arg_return:ty $f:block) => {
        ::paste::paste! {
            /// Sets the property and returns a new Style.
            pub fn [<$struct_property>](mut self, $arg_ident: $arg_ty) -> Self {
                self.properties.insert(Property::$bitset_property);
                self.$struct_property = $f;
                self
            }

            /// Sets the property and returns a mutable reference to the Style.
            pub fn [<set_ $struct_property>](&mut self, $arg_ident: $arg_ty) -> &mut Self {
                self.properties.insert(Property::$bitset_property);
                self.$struct_property = $f;
                self
            }

            /// Unsets the property and returns a mutable reference to the Style.
            pub fn [<unset_ $struct_property>](&mut self) -> &mut Self {
                self.properties.remove(Property::$bitset_property);
                self.$struct_property = Self::DEFAULT.$struct_property;
                self
            }

            /// Returns the property.
            pub fn [<get_ $struct_property>](&self) -> $arg_return {
                if self.[<has_ $struct_property>]() { self.$struct_property } else { Self::DEFAULT.$struct_property }
            }

            /// Checks if the property is set.
            pub fn [<maybe_ $struct_property>](&self) -> Option<$arg_return> {
                if self.properties.contains(Property::$bitset_property) { Some(self.$struct_property) } else { None }
            }
            /// Checks if the property is set.
            pub fn [<has_ $struct_property>](&self) -> bool {
                self.properties.contains(Property::$bitset_property)
            }
        }
    };
    ($property_ty:ident -> fn $fn_name:ident($arg_ident:ident: $arg_ty:ty) -> $arg_return:ty) => {
        property!(
            $property_ty -> fn $fn_name($arg_ident: $arg_ty) -> $arg_return {
                $arg_ident
            }
        );
    };
}


impl From<Style> for ansi::Style {
    fn from(style: Style) -> Self {
        if style.is_default() { return ansi::Style::default() }
        let mut attributes = ansi::Attribute::empty();

        match style.font_weight {
            Some(FontWeight::Bold) => attributes.insert(ansi::Attribute::Bold),
            _ => (),
        };

        match style.text_decoration {
            Some(TextDecoration::Underline) => attributes.insert(ansi::Attribute::Underline),
            Some(TextDecoration::LineThrough) => attributes.insert(ansi::Attribute::Strikethrough),
            _ => (),
        };

        match style.font_style {
            Some(FontStyle::Italic) => attributes.insert(ansi::Attribute::Italic),
            _ => (),
        };

        ansi::Style {
            attributes,
            foreground: style.color.unwrap_or_default(),
            background: style.background.unwrap_or_default(),
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl taffy::CoreStyle for Style {
    type CustomIdent = CompactString;

    #[inline(always)]
    fn is_block(&self) -> bool {
        matches!(self.display, Display::Flex)
    }

    fn box_sizing(&self) -> taffy::BoxSizing {
        taffy::BoxSizing::BorderBox
    }

    #[inline(always)]
    fn size(&self) -> taffy::Size<taffy::Dimension> {
        self.size.into()
    }
    #[inline(always)]
    fn min_size(&self) -> taffy::Size<taffy::Dimension> {
        self.min_size.into()
    }
    #[inline(always)]
    fn max_size(&self) -> taffy::Size<taffy::Dimension> {
        self.max_size.into()
    }

    #[inline(always)]
    fn margin(&self) -> taffy::Rect<taffy::LengthPercentageAuto> {
        self.margin.into()
    }
    #[inline(always)]
    fn padding(&self) -> taffy::Rect<taffy::LengthPercentage> {
        self.padding.into()
    }
    #[inline(always)]
    fn border(&self) -> taffy::Rect<taffy::LengthPercentage> {
        let edges = self.border.into_edges();

        taffy::Rect {
            left: taffy::LengthPercentage::length(edges.left as f32),
            right: taffy::LengthPercentage::length(edges.right as f32),
            top: taffy::LengthPercentage::length(edges.top as f32),
            bottom: taffy::LengthPercentage::length(edges.bottom as f32),
        }

    }
}

impl taffy::BlockContainerStyle for Style {}

impl taffy::BlockItemStyle for Style {}

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
        self.gap.into()
    }
    #[inline(always)]
    fn align_content(&self) -> Option<taffy::AlignContent> {
        self.align_content.map(|a| a.into())
    }
    #[inline(always)]
    fn align_items(&self) -> Option<taffy::AlignItems> {
        self.align_items.map(|a| a.into())
    }
    #[inline(always)]
    fn justify_content(&self) -> Option<taffy::JustifyContent> {
        self.justify_content.map(|a| a.into())
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
        self.align_self.map(|a| a.into())
    }
}
