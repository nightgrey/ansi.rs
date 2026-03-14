use tree::id;
use ansi::{Color, Style};
use tree::layout::{Layout, LayoutId};
use crate::LayerId;

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
    pub(crate) layout_id: LayoutId,
    pub layer_id: LayerId,
}

#[allow(non_snake_case)]
impl Element {
    pub fn Div() -> Self {
        Self {
            kind: ElementKind::Div,
            style: Style::EMPTY,
            layout: taffy::Style {
                display: taffy::Display::Flex,
                size: taffy::Size {
                    width: taffy::Dimension::percent(1.0),
                    height: taffy::Dimension::percent(1.0),
                },
                ..Default::default()
            },
            layer_id: LayerId::none(),
            layout_id: LayoutId::none(),
        }
    }

    pub fn Span(content: String) -> Self {
        Self {
            kind: ElementKind::Span(content),
            style: Style::new().foreground(Color::Red),
            layout: Layout::default(),
            layer_id: LayerId::none(),
            layout_id: LayoutId::none(),
        }
    }

    pub fn on(mut self, layer_id: LayerId) -> Self {
        self.layer_id = layer_id;
        self
    }

    /// Whether this element should be promoted to its own compositing layer.
    pub fn is_promoting(&self) -> bool {
        false
    }
}

