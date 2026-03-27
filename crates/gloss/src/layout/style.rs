use bitflags::bitflags_match;
use compact_str::CompactString;
use crate::layout::*;
use ansi::Color;
macro_rules! property {
    (fn $field:ident($value_ident:ident: $value_ty:ty) -> $variant:ident $convert:block) => {
        ::paste::paste! {
            pub fn [<set_ $field>](&mut self, $value_ident: $value_ty) -> &mut Self {
                self.properties.insert(Property::$variant);
                self.$field = $convert;
                self
            }

            pub fn [<with_ $field>](&mut self, $value_ident: $value_ty) -> &mut Self {
                self.properties.insert(Property::$variant);
                self.$field = $convert;
                self
            }

            pub fn [<$field>](&self) -> $value_ty {
                self.$field
            }

            pub fn [<no_ $field>](mut self) -> Self {
                self.properties.remove(Property::$variant);
                self.$field = Self::Default.$field;
                self
            }

            pub fn [<has_ $field>](&self) -> bool {
                self.properties.contains(Property::$variant)
            }
        }
    };
    (fn $field:ident($value_ident:ident: impl Into<$value_ty:ty>) -> $variant:ident) => {
        ::paste::paste! {
            pub fn [<set_ $field>](&mut self, $value_ident: impl Into<$value_ty>) -> &mut Self {
                self.properties.insert(Property::$variant);
                self.$field = $value_ident.into();
                self
            }

            pub fn [<with_ $field>](&mut self, $value_ident: impl Into<$value_ty>) -> &mut Self {
                self.properties.insert(Property::$variant);
                self.$field = $value_ident.into();
                self
            }

            pub fn [<$field>](&self) -> $value_ty {
                self.$field
            }

            pub fn [<no_ $field>](mut self) -> Self {
                self.properties.remove(Property::$variant);
                self.$field = Self::Default.$field;
                self
            }

            pub fn [<has_ $field>](&self) -> bool {
                self.properties.contains(Property::$variant)
            }
        }
    };
    (fn $field:ident($value_ident:ident: $value_ty:ty) -> $variant:ident) => {
        property!(
            fn $field($value_ident: $value_ty) -> $variant {
                $value_ident
            }
        );
    };
}

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

    min_width: Dimension,
    min_height: Dimension,
    width: Dimension,
    height: Dimension,
    max_width: Dimension,
    max_height: Dimension,

    padding: Edges,
    margin: Edges,
    gap: Gap,

    color: Color,
    background_color: Color,

    // Flex
    flex_direction: FlexDirection,
    flex_wrap: FlexWrap,
    flex_basis: Dimension,
    flex_grow: f32,
    flex_shrink: f32,

    align_items: AlignItems,
    align_self: AlignSelf,
    align_content: AlignContent,
    justify_items: JustifyItems,
    justify_self: AlignSelf,
    justify_content: JustifyContent,

    text_decoration: TextDecorationLine,
    font_weight: FontWeight,
    font_style: FontStyle,

    display: Display,
    border: Border,

}

#[allow(non_upper_case_globals)]
impl Style {
    pub const Default: Self = Self {
        properties: Property::None,
        color: Color::None,
        background_color: Color::None,
        text_decoration: TextDecorationLine::None,
        font_weight: FontWeight::Normal,
        font_style: FontStyle::Normal,
        display: Display::Block,
        padding: Edges::Auto,
        margin: Edges::Auto,
        border: Border::None,
        min_width: Dimension::Auto,
        min_height: Dimension::Auto,
        width: Dimension::Auto,
        height: Dimension::Auto,
        max_width: Dimension::Auto,
        max_height: Dimension::Auto,
        align_items: AlignItems::Start,
        align_self: AlignSelf::Start,
        justify_items: JustifyItems::Start,
        justify_self: JustifySelf::Start,
        align_content: AlignContent::Start,
        justify_content: JustifyContent::Start,
        flex_direction: FlexDirection::Column,
        flex_wrap: FlexWrap::NoWrap,
        flex_basis: Dimension::Auto,
        flex_grow: 0.0,
        flex_shrink: 1.0,
        gap: Gap::Auto,
    };

    pub fn inherit_from(self, other: Self) -> Self {
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

    pub fn is_default(&self) -> bool {
        *self == Self::Default
    }
    property!(fn color(value: impl Into<Color>) -> Color);
    property!(fn background_color(value: impl Into<Color>) -> BackgroundColor);
    property!(fn text_decoration(value: TextDecorationLine) -> TextDecoration);
    property!(fn font_weight(value: impl Into<FontWeight>) -> FontWeight);
    property!(fn font_style(value: FontStyle) -> FontStyle);
    property!(fn display(value: Display) -> Display);
    property!(fn padding(value: impl Into<Edges>) -> Padding);
    property!(fn margin(value: impl Into<Edges>) -> Margin);
    property!(fn border(value: Border) -> Border);
    property!(fn min_width(value: impl Into<Dimension>) -> MinWidth);
    property!(fn min_height(value: impl Into<Dimension>) -> MinHeight);
    property!(fn width(value: impl Into<Dimension>) -> Width);
    property!(fn height(value: impl Into<Dimension>) -> Height);
    property!(fn max_width(value: impl Into<Dimension>) -> MaxWidth);
    property!(fn max_height(value: impl Into<Dimension>) -> MaxHeight);
    property!(fn align_items(value: AlignItems) -> AlignItems);
    property!(fn align_self(value: AlignSelf) -> AlignSelf);
    property!(fn justify_items(value: JustifyItems) -> JustifyItems);
    property!(fn justify_self(value: JustifySelf) -> JustifySelf);
    property!(fn align_content(value: AlignContent) -> AlignContent);
    property!(fn justify_content(value: JustifyContent) -> JustifyContent);
    property!(fn flex_direction(value: FlexDirection) -> FlexDirection);
    property!(fn flex_wrap(value: FlexWrap) -> FlexWrap);
    property!(fn flex_basis(value: impl Into<Dimension>) -> FlexBasis);
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
        Self::Default
    }
}

impl taffy::CoreStyle for Style {
    type CustomIdent = CompactString;

    #[inline(always)]
    fn box_generation_mode(&self) -> taffy::BoxGenerationMode {
        match self.display() {
            Display::None => taffy::BoxGenerationMode::None,
            _ => taffy::BoxGenerationMode::Normal,
        }
    }

    #[inline(always)]
    fn is_block(&self) -> bool {
        matches!(self.display(), Display::Block)
    }

    fn box_sizing(&self) -> taffy::BoxSizing {
        taffy::BoxSizing::BorderBox
    }

    #[inline(always)]
    fn size(&self) -> taffy::Size<taffy::Dimension> {
        taffy::Size {
            width: self.width().into(),
            height: self.height().into(),
        }
    }
    #[inline(always)]
    fn min_size(&self) -> taffy::Size<taffy::Dimension> {
        taffy::Size {
            width: self.min_width().into(),
            height: self.min_height().into(),
        }
    }
    #[inline(always)]
    fn max_size(&self) -> taffy::Size<taffy::Dimension> {
        taffy::Size {
            width: self.max_width().into(),
            height: self.max_height().into(),
        }
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
       let edges = self.border().into_edges();

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
        self.flex_direction()
    }
    #[inline(always)]
    fn flex_wrap(&self) -> taffy::FlexWrap {
        self.flex_wrap()
    }
    #[inline(always)]
    fn gap(&self) -> taffy::Size<taffy::LengthPercentage> {
        taffy::Size {
            width: self.gap().horizontal.into(),
            height: self.gap().vertical.into(),
        }
    }
    #[inline(always)]
    fn align_content(&self) -> Option<taffy::AlignContent> {
        Some(self.align_content().into())
    }
    #[inline(always)]
    fn align_items(&self) -> Option<taffy::AlignItems> {
        Some(self.align_items().into())
    }
    #[inline(always)]
    fn justify_content(&self) -> Option<taffy::JustifyContent> {
        Some(self.justify_content().into())
    }
}

impl taffy::FlexboxItemStyle for Style {
    #[inline(always)]
    fn flex_basis(&self) -> taffy::Dimension {
        self.flex_basis().into()
    }
    #[inline(always)]
    fn flex_grow(&self) -> f32 {
        self.flex_grow()
    }
    #[inline(always)]
    fn flex_shrink(&self) -> f32 {
        self.flex_shrink()
    }
    #[inline(always)]
    fn align_self(&self) -> Option<taffy::AlignSelf> {
        Some(self.align_self().into())
    }
}
