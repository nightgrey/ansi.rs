use tree::{id, Layout};
use ansi::{Color, Style};
id!(pub struct ElementId);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElementKind {
    Div,
    Span(String),
}

#[derive(Debug)]
pub struct Element {
    pub kind: ElementKind,
    pub style: Style,
    pub layout: Layout,
}

#[allow(non_snake_case)]
impl Element {
    pub fn Div() -> Self {
        Self {
            kind: ElementKind::Div,
            style: Style::EMPTY,
            layout: Layout::default(),
        }
    }

    pub fn Span(content: String) -> Self {
        Self {
            kind: ElementKind::Span(content),
            style: Style::new().foreground(Color::Red),
            layout: Layout::default(),
        }
    }

    /// Whether this element should be promoted to its own compositing layer.
    pub fn is_promoting(&self) -> bool {
        false
    }
}

