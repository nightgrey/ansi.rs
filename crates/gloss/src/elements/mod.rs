pub mod element;
pub mod kind;

use bon::builder;
pub use element::*;
pub use kind::*;


use crate::layout::*;
use ansi::Color;

#[builder]
pub fn div<'a>(
    #[builder(default)]
    display: Display,
    #[builder(default)]
    min_size: Size,
    #[builder(default)]
    size: Size,
    #[builder(default)]
    max_size: Size,
    #[builder(default)]
    padding: Edges,
    #[builder(default)]
    margin: Edges,
    #[builder(default)]
    gap: Gap,
    #[builder(default)]
    flex_direction: FlexDirection,
    #[builder(default)]
    flex_wrap: FlexWrap,
    #[builder(default)]
    flex_basis: Length,
    #[builder(default)]
    flex_grow: f32,
    #[builder(default)]
    flex_shrink: f32,
    #[builder(default)]
    border: Border,

    align_items: Option<AlignItems>,
    align_self: Option<AlignSelf>,
    align_content: Option<AlignContent>,
    justify_items: Option<JustifyItems>,
    justify_self: Option<AlignSelf>,
    justify_content: Option<JustifyContent>,
    color: Option<Color>,
    background: Option<Color>,
    text_decoration: Option<TextDecoration>,
    font_weight: Option<FontWeight>,
    font_style: Option<FontStyle>,

) -> Element<'a> {
    Element {
        kind: ElementKind::Div,
        layout: Layout {
            min_size,
            size,
            max_size,
            padding,
            margin,
            gap,
            flex_direction,
            flex_wrap,
            flex_basis,
            flex_grow,
            flex_shrink,
            align_items,
            align_self,
            align_content,
            justify_items,
            justify_self,
            justify_content,
            border: border,
            color,
            background,
            text_decoration,
            font_weight,
            font_style,
            display,
        }
    }
}

#[test]
fn qwe() {
    let element = div().display(Display::Inline);
    let el = element.call();
}