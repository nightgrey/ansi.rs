use bitflags::bitflags_match;
use compact_str::CompactString;
use crate::style::*;
use ansi::Color;

bitflags::bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
    struct Property: u32 {
        const Color = 1 << 0;
        const BackgroundColor = 1 << 1;
        const TextDecoration = 1 << 2;
        const FontWeight = 1 << 3;
        const FontStyle = 1 << 4;
        const Display = 1 << 5;
        const Padding = 1 << 6;
        const Margin = 1 << 7;
        const Border = 1 << 8;
        const MinWidth = 1 << 9;
        const MinHeight = 1 << 10;
        const Width = 1 << 11;
        const Height = 1 << 12;
        const MaxWidth = 1 << 13;
        const MaxHeight = 1 << 14;
        const AlignItems = 1 << 15;
        const AlignSelf = 1 << 16;
        const JustifyItems = 1 << 17;
        const JustifySelf = 1 << 18;
        const AlignContent = 1 << 19;
        const JustifyContent = 1 << 20;
        const FlexDirection = 1 << 21;
        const FlexWrap = 1 << 22;
        const FlexBasis = 1 << 23;
        const FlexGrow = 1 << 24;
        const FlexShrink = 1 << 25;
        const Gap = 1 << 26;
    }
}

impl Property {
    #[allow(non_upper_case_globals)]
    pub const None: Self = Self::empty();
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Style {
    properties: Property,

    pub(crate) min_width: Dimension,
    pub(crate) min_height: Dimension,
    pub(crate) width: Dimension,
    pub(crate) height: Dimension,
    pub(crate) max_width: Dimension,
    pub(crate) max_height: Dimension,

    pub(crate) padding: Edges,
    pub(crate) margin: Edges,
    pub(crate) gap: Gap,

    pub(crate) color: Color,
    pub(crate) background_color: Color,

    // Flex
    pub(crate) flex_direction: FlexDirection,
    pub(crate) flex_wrap: FlexWrap,
    pub(crate) flex_basis: Dimension,
    pub(crate) flex_grow: f32,
    pub(crate) flex_shrink: f32,

    pub(crate) align_items: AlignItems,
    pub(crate) align_self: AlignSelf,
    pub(crate) align_content: AlignContent,
    pub(crate) justify_items: JustifyItems,
    pub(crate) justify_self: AlignSelf,
    pub(crate) justify_content: JustifyContent,

    pub(crate) text_decoration: TextDecorationLine,
    pub(crate) font_weight: FontWeight,
    pub(crate) font_style: FontStyle,

    pub(crate) display: Display,
    pub(crate) border: Border,

}

#[allow(non_upper_case_globals)]
impl Style {
    pub const DEFAULT: Style = Style {
        properties: Property::None,
        display: Display::Flex,
        margin: Edges::ZERO,
        padding: Edges::ZERO,
        border: Border::None,
        width: Dimension::ZERO,
        height: Dimension::ZERO,
        min_width: Dimension::ZERO,
        min_height: Dimension::ZERO,
        max_width: Dimension::ZERO,
        max_height: Dimension::ZERO,
        gap: Gap::ZERO,
        // Alignment
        align_items: AlignItems::Start,
        align_self: AlignSelf::Start,
        justify_items: JustifyItems::Start,
        justify_self: JustifySelf::Start,
        align_content: AlignContent::Start,
        justify_content: JustifyContent::Start,
        // Flexbox
        text_decoration: TextDecorationLine::None,
        font_weight: FontWeight::Normal,
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::NoWrap,
        flex_grow: 0.0,
        flex_shrink: 1.0,
        flex_basis: Dimension::Auto,
        color: Color::None,
        background_color: Color::None,
        font_style: FontStyle::Normal,
    };

    /// Applies the properties of `other` from `self`.
    pub fn inherit(self, other: Self) -> Self {
        let mut style = self;
        for property in other.properties.iter() {
            bitflags_match!(property, {
                Property::Color => style.color = other.color,
                Property::BackgroundColor => style.background_color = other.background_color,
                Property::TextDecoration => style.text_decoration = other.text_decoration,
                Property::FontWeight => style.font_weight = other.font_weight,
                Property::FontStyle => style.font_style = other.font_style,
                Property::Display => style.display = other.display,
                Property::Padding => style.padding = other.padding,
                Property::Margin => style.margin = other.margin,
                Property::Border => style.border = other.border,
                Property::Width => style.width = other.width,
                Property::Height => style.height = other.height,
                Property::MinWidth => style.min_width = other.min_width,
                Property::MinHeight => style.min_height = other.min_height,
                Property::MaxWidth => style.max_width = other.max_width,
                Property::MaxHeight => style.max_height = other.max_height,
                Property::AlignItems => style.align_items = other.align_items,
                Property::AlignSelf => style.align_self = other.align_self,
                Property::JustifyItems => style.justify_items = other.justify_items,
                Property::JustifySelf => style.justify_self = other.justify_self,
                Property::AlignContent => style.align_content = other.align_content,
                Property::JustifyContent => style.justify_content = other.justify_content,
                Property::FlexDirection => style.flex_direction = other.flex_direction,
                Property::FlexWrap => style.flex_wrap = other.flex_wrap,
                Property::FlexBasis => style.flex_basis = other.flex_basis,
                Property::FlexGrow => style.flex_grow = other.flex_grow,
                Property::FlexShrink => style.flex_shrink = other.flex_shrink,
                Property::Gap => style.gap = other.gap,
                _ => unreachable!("This property is not supported: {:?}", property),
            });
        }

        style
    }

    /// Applies the properties of `self` onto `other`.
    pub fn propagate(self, other: Self) -> Self {
        other.inherit(self)
    }

    pub fn is_default(&self) -> bool {
        self == &Self::DEFAULT
    }
}

macro_rules! property {
    (fn $property_ident:ident($arg_ident:ident: $arg_ty:ty) -> $property_ty:ident $f:block) => {
        ::paste::paste! {
            /// Sets the property and returns a new Style.
            pub fn [<$property_ident>](mut self, $arg_ident: $arg_ty) -> Self {
                self.properties.insert(Property::$property_ty);
                self.$property_ident = $f;
                self
            }

            /// Sets the property and returns a mutable reference to the Style.
            pub fn [<set_ $property_ident>](&mut self, $arg_ident: $arg_ty) -> &mut Self {
                self.properties.insert(Property::$property_ty);
                self.$property_ident = $f;
                self
            }

            /// Unsets the property and returns a mutable reference to the Style.
            pub fn [<unset_ $property_ident>](&mut self) -> &mut Self {
                self.properties.remove(Property::$property_ty);
                self.$property_ident = Self::DEFAULT.$property_ident;
                self
            }

            /// Returns the property.
            pub fn [<get_ $property_ident>](&self) -> $arg_ty {
                if self.[<has_ $property_ident>]() { self.$property_ident } else { Self::DEFAULT.$property_ident }
            }

            /// Checks if the property is set.
            pub fn [<has_ $property_ident>](&self) -> bool {
                self.properties.contains(Property::$property_ty)
            }
        }
    };
    (fn $property_ident:ident($arg_ident:ident: Into<$arg_ty:ty>) -> $property_ty:ident) => {
        property!(
            fn $property_ident($arg_ident: impl Into<$arg_ty>) -> $property_ty {
                $arg_ident.into()
            }
        );
    };
    (fn $property_ident:ident($arg_ident:ident: $arg_ty:ty) -> $property_ty:ident) => {
        property!(
            fn $property_ident($arg_ident: $arg_ty) -> $property_ty {
                $arg_ident
            }
        );
    };
}

impl Style {

    property!(fn color(value: Into<Color>) -> Color);
    property!(fn background_color(value: Into<Color>) -> BackgroundColor);
    property!(fn text_decoration(value: TextDecorationLine) -> TextDecoration);
    property!(fn font_weight(value: Into<FontWeight>) -> FontWeight);
    property!(fn font_style(value: FontStyle) -> FontStyle);
    property!(fn display(value: Display) -> Display);
    property!(fn padding(value: Into<Edges>) -> Padding);
    property!(fn margin(value: Into<Edges>) -> Margin);
    property!(fn border(value: Border) -> Border);
    property!(fn min_width(value: Into<Dimension>) -> MinWidth);
    property!(fn min_height(value: Into<Dimension>) -> MinHeight);
    property!(fn width(value: Into<Dimension>) -> Width);
    property!(fn height(value: Into<Dimension>) -> Height);
    property!(fn max_width(value: Into<Dimension>) -> MaxWidth);
    property!(fn max_height(value: Into<Dimension>) -> MaxHeight);
    property!(fn align_items(value: AlignItems) -> AlignItems);
    property!(fn align_self(value: AlignSelf) -> AlignSelf);
    property!(fn justify_items(value: JustifyItems) -> JustifyItems);
    property!(fn justify_self(value: JustifySelf) -> JustifySelf);
    property!(fn align_content(value: AlignContent) -> AlignContent);
    property!(fn justify_content(value: JustifyContent) -> JustifyContent);
    property!(fn flex_direction(value: FlexDirection) -> FlexDirection);
    property!(fn flex_wrap(value: FlexWrap) -> FlexWrap);
    property!(fn flex_basis(value: Into<Dimension>) -> FlexBasis);
    property!(fn flex_grow(value: f32) -> FlexGrow);
    property!(fn flex_shrink(value: f32) -> FlexShrink);
    property!(fn gap(value: Gap) -> Gap);
}
impl Into<ansi::Style> for Style {
    fn into(self) -> ansi::Style {
        let mut attributes = ansi::Attribute::empty();

        match self.font_weight {
            FontWeight::Normal => (),
            FontWeight::Bold => attributes.insert(ansi::Attribute::Bold),
        };

        match self.text_decoration {
            TextDecorationLine::None => (),
            TextDecorationLine::Underline => attributes.insert(ansi::Attribute::Underline),
            TextDecorationLine::LineThrough => attributes.insert(ansi::Attribute::Strikethrough),
        };

        match self.font_style {
            FontStyle::Normal => (),
            FontStyle::Italic => attributes.insert(ansi::Attribute::Italic),
        };

        ansi::Style {
            attributes,
            foreground: self.color,
            background: self.background_color,
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
    fn box_generation_mode(&self) -> taffy::BoxGenerationMode {
        match self.get_display() {
            Display::None => taffy::BoxGenerationMode::None,
            _ => taffy::BoxGenerationMode::Normal,
        }
    }

    #[inline(always)]
    fn is_block(&self) -> bool {
        matches!(self.get_display(), Display::Block)
    }

    fn box_sizing(&self) -> taffy::BoxSizing {
        taffy::BoxSizing::BorderBox
    }

    #[inline(always)]
    fn size(&self) -> taffy::Size<taffy::Dimension> {
        if !(self.has_width() && self.has_height()) {
            return taffy::Size::auto();
        }
        taffy::Size {
            width: self.width.into(),
            height: self.height.into(),
        }
    }
    #[inline(always)]
    fn min_size(&self) -> taffy::Size<taffy::Dimension> {
        if !(self.has_min_width() && self.has_min_height()) {
            return taffy::Size::auto();
        }
        taffy::Size {
            width: self.min_width.into(),
            height: self.min_height.into(),
        }
    }
    #[inline(always)]
    fn max_size(&self) -> taffy::Size<taffy::Dimension> {
        if !(self.has_max_width() && self.has_max_height()) {
            return taffy::Size::auto();
        }
        taffy::Size {
            width: self.max_width.into(),
            height: self.max_height.into(),
        }
    }

    #[inline(always)]
    fn margin(&self) -> taffy::Rect<taffy::LengthPercentageAuto> {
        if !self.has_margin() {
            return taffy::Rect::auto();
        }
        self.margin.into()
    }
    #[inline(always)]
    fn padding(&self) -> taffy::Rect<taffy::LengthPercentage> {
        if !self.has_padding() {
            return taffy::Rect::zero();
        }
        self.padding.into()
    }
    #[inline(always)]
    fn border(&self) -> taffy::Rect<taffy::LengthPercentage> {
        if !self.has_border() {
            return taffy::Rect::zero();
        }

        let edges = self.get_border().into_edges();

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
        self.get_flex_direction()
    }
    #[inline(always)]
    fn flex_wrap(&self) -> taffy::FlexWrap {
        self.get_flex_wrap()
    }
    #[inline(always)]
    fn gap(&self) -> taffy::Size<taffy::LengthPercentage> {
        taffy::Size {
            width: self.get_gap().horizontal.into(),
            height: self.get_gap().vertical.into(),
        }
    }
    #[inline(always)]
    fn align_content(&self) -> Option<taffy::AlignContent> {
        Some(self.get_align_content().into())
    }
    #[inline(always)]
    fn align_items(&self) -> Option<taffy::AlignItems> {
        Some(self.get_align_items().into())
    }
    #[inline(always)]
    fn justify_content(&self) -> Option<taffy::JustifyContent> {
        Some(self.get_justify_content().into())
    }
}

impl taffy::FlexboxItemStyle for Style {
    #[inline(always)]
    fn flex_basis(&self) -> taffy::Dimension {
        self.get_flex_basis().into().into()
    }
    #[inline(always)]
    fn flex_grow(&self) -> f32 {
        self.get_flex_grow()
    }
    #[inline(always)]
    fn flex_shrink(&self) -> f32 {
        self.get_flex_shrink()
    }
    #[inline(always)]
    fn align_self(&self) -> Option<taffy::AlignSelf> {
        Some(self.get_align_self().into())
    }
}
