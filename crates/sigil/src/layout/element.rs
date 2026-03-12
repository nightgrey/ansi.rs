use tree::id;
use ansi::{Color, Style};
use crate::LayerId;

id!(pub struct ElementId);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElementKind {
    Div,
    Span(String),
}

#[derive(Debug)]
pub struct Element {
    pub kind: ElementKind,
    pub style: Style,
    pub layout: taffy::Style,
    pub layer_id: LayerId,
    pub(crate) taffy_node: taffy::NodeId,
}

#[allow(non_snake_case)]
impl Element {
    pub fn Div(direction: Direction) -> Self {
        let flex_direction = match direction {
            Direction::Horizontal => taffy::FlexDirection::Row,
            Direction::Vertical => taffy::FlexDirection::Column,
        };
        Self {
            kind: ElementKind::Div,
            style: Style::EMPTY,
            layout: taffy::Style {
                flex_direction,
                size: taffy::Size {
                    width: taffy::Dimension::percent(1.0),
                    height: taffy::Dimension::percent(1.0),
                },
                flex_grow: 1.0,
                ..Default::default()
            },
            layer_id: LayerId::none(),
            taffy_node: taffy::NodeId::new(0),
        }
    }

    pub fn Span(content: String) -> Self {
        Self {
            kind: ElementKind::Span(content),
            style: Style::new().foreground(Color::Red),
            layout: taffy::Style {
                flex_grow: 1.0,
                ..Default::default()
            },
            layer_id: LayerId::none(),
            taffy_node: taffy::NodeId::new(0),
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
