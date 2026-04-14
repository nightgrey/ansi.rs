use std::borrow::Cow;
use bitflags::bitflags;
use derive_more::{Deref, DerefMut};
use crate::{Display, Style};
use tree::id;

id!(pub struct NodeId);

#[derive(Clone, Debug)]
pub enum NodeKind<'a> {
    Span(Cow<'a, str>),
    Div,
}

#[derive(Clone, Debug, Deref, DerefMut)]
pub struct Node<'a> {
    pub kind: NodeKind<'a>,
    #[deref]
    #[deref_mut]
    pub style: Style,
}

#[allow(non_snake_case)]
impl<'a> Node<'a> {
    pub fn Span(text: impl Into<Cow<'a, str>>) -> Self {
        Self {
            kind: NodeKind::Span(text.into()),
            style: Style {
                display: Display::Inline,
                ..Default::default()
            },
        }
    }

    pub fn Div() -> Self {
        Self {
            kind: NodeKind::Div,
            style: Style {
                display: Display::Block,
                ..Default::default()
            },
        }
    }
}

bitflags! {
    #[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
    pub struct Dirty: u8 {
        const Style   = 1 << 0;
        const Measure = 1 << 1;
        const Layout  = 1 << 2;
    }
}

#[derive(Debug, Clone)]
pub struct LayoutNode {
    pub cache: taffy::Cache,
    pub unrounded_layout: taffy::Layout,
    pub final_layout: taffy::Layout,
    pub dirty: Dirty,
}

impl Default for LayoutNode {
    fn default() -> Self {
        Self {
            cache: taffy::Cache::default(),
            unrounded_layout: taffy::Layout::default(),
            final_layout: taffy::Layout::default(),
            dirty: Dirty::Style | Dirty::Measure | Dirty::Layout,
        }
    }
}
