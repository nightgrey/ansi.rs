use std::borrow::Cow;
use derive_more::{Deref, DerefMut};
use crate::{Display, ElementKind, Style};
use tree::id;

id!(pub struct ElementId);

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct Element<'a> {
    pub kind: ElementKind<'a>,
    #[deref]
    #[deref_mut]
    pub style: Style,
}

#[allow(non_snake_case)]
impl<'a> Element<'a> {
    pub fn Span(text: impl Into<Cow<'a, str>>) -> Self {
        Self {
            kind: ElementKind::Span(text.into()),
            style: Style {
                display: Display::Inline,
                ..Default::default()
            },
        }
    }

    pub fn Div() -> Self {
        Self {
            kind: ElementKind::Div,
            style: Style {
                display: Display::Block,
                ..Default::default()
            },
        }
    }
}
