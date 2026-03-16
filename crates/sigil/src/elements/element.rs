use tree::id;
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
}

#[allow(non_snake_case)]
impl Element {
    pub fn Div() -> Self {
        Self {
            kind: ElementKind::Div,
            style: Style::EMPTY,
        }
    }

    pub fn Span(content: String) -> Self {
        Self {
            kind: ElementKind::Span(content),
            style: Style::new().foreground(Color::Red),
            
        }
    }

    /// Whether this element should be promoted to its own compositing layer.
    pub fn is_promoting(&self) -> bool {
        false
    }
}

