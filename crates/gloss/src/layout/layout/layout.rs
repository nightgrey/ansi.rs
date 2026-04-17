use compact_str::CompactString;
use ansi::Color;
use super::*;
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Layout {
    // Block
    pub display: Display,
    pub size: Size,
    pub min_size: Size,
    pub max_size: Size,
    // Inline


    // This
    pub padding: Edges,
    pub margin: Edges,
    pub border: Border,

    // Flex
    pub flex_direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub flex_basis: Length,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub gap: Gap,
    pub align_items: Option<AlignItems>,
    pub align_self: Option<AlignSelf>,
    pub align_content: Option<AlignContent>,
    pub justify_items: Option<JustifyItems>,
    pub justify_self: Option<JustifySelf>,
    pub justify_content: Option<JustifyContent>,

    // Inline (inheritable)
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub text_decoration: Option<TextDecoration>,
    pub font_weight: Option<FontWeight>,
    pub font_style: Option<FontStyle>,
}

#[allow(non_upper_case_globals)]
impl Layout {
    pub const DEFAULT: Layout = Layout {
        display: Display::Block,
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
        flex_basis: Length::Auto,
        font_style: None,
    };

    pub fn inherit(self, parent: Self) -> Self {
        let mut next = self;

        fn inherit<T>(parent: Option<T>, current: Option<T>) -> Option<T> {
            match (parent, current) {
                (Some(_), Some(x)) => Some(x),
                (Some(x), None) => Some(x),
                (None, Some(x)) => Some(x),
                (None, None) => None,
            }
        }

        next.color = inherit(parent.color, self.color);
        next.background = inherit(parent.background, self.background);
        next.text_decoration = inherit(parent.text_decoration, self.text_decoration);
        next.font_style = inherit(parent.font_style, self.font_style);

        next
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

impl From<ansi::Style> for Layout {
    fn from(style: ansi::Style) -> Self {
        let mut result = Self::DEFAULT;

        if style.attributes.contains(ansi::Attribute::Bold) {
            result.font_weight = Some(FontWeight::Bold);
        }
        if style.attributes.contains(ansi::Attribute::Italic) {
            result.font_style = Some(FontStyle::Italic);
        }
        if style.attributes.contains(ansi::Attribute::Underline) {
            result.text_decoration = Some(TextDecoration::Underline);
        }
        if style.attributes.contains(ansi::Attribute::Strikethrough) {
            result.text_decoration = Some(TextDecoration::Strikethrough);
        }

        result.color = Some(style.foreground);
        result.background = Some(style.background);

        result
    }
}

impl From<Layout> for ansi::Style {
    fn from(style: Layout) -> Self {
        if style.is_default() { return ansi::Style::default() }
        let mut attributes = ansi::Attribute::empty();

        match style.font_weight {
            Some(FontWeight::Bold) => attributes.insert(ansi::Attribute::Bold),
            _ => (),
        };

        match style.text_decoration {
            Some(TextDecoration::Underline) => attributes.insert(ansi::Attribute::Underline),
            Some(TextDecoration::Strikethrough) => attributes.insert(ansi::Attribute::Strikethrough),
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

impl Default for Layout {
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl taffy::CoreStyle for Layout {
    type CustomIdent = CompactString;

    #[inline(always)]
    fn is_block(&self) -> bool {
        matches!(self.display, Display::Block)
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

impl taffy::BlockContainerStyle for Layout {}

impl taffy::BlockItemStyle for Layout {}

impl taffy::FlexboxContainerStyle for Layout {
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

impl taffy::FlexboxItemStyle for Layout {
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
