use tree::id;
use ansi::Style;
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

impl Element {
    pub fn div() -> Self {
        Self {
            kind: ElementKind::Div,
            style: Style::None,
        }
    }

    pub fn span(content: impl Into<String>) -> Self {
        Self {
            kind: ElementKind::Span(content.into()),
            style: Style::None,
        }
    }

    /// Whether this element should be promoted to its own compositing layer.
    pub fn is_promoting(&self) -> bool {
        false
    }
}

