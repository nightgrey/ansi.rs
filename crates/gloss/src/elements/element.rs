use std::borrow::Cow;
use bon::{bon, builder};
use derive_more::{Deref, DerefMut};
use geometry::{Bound, Step};
use crate::{Display, ElementKind, Layout};
use tree::{id};

id!(pub struct ElementId);

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct Element<'a> {
    pub kind: ElementKind<'a>,
    #[deref]
    #[deref_mut]
    pub layout: Layout,
}

#[allow(non_snake_case)]
impl<'a> Element<'a> {
    pub fn Span(text: impl Into<Cow<'a, str>>) -> Self {
        Self {
            kind: ElementKind::Span(text.into()),
            layout: Layout {
                display: Display::Inline,
                ..Default::default()
            },
        }
    }

    pub fn Div() -> Self {
        Self {
            kind: ElementKind::Div,
            layout: Layout {
                display: Display::Block,
                ..Default::default()
            },
        }
    }
}
