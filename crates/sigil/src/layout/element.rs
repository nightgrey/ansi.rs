use crate::{LayerId, key};
use ansi::{Color, Style};

key!(
    pub struct ElementId;
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElementKind {
    Container { direction: Direction },
    Text(String),
}

#[derive(Debug)]
pub struct Element {
    pub kind: ElementKind,
    pub layer_id: Option<LayerId>,
    pub style: Style,
}

impl Element {
    pub const fn container(direction: Direction) -> Self {
        Self {
            kind: ElementKind::Container { direction },
            layer_id: None,
            style: Style::EMPTY,
        }
    }

    pub const fn text(content: String) -> Self {
        Self {
            kind: ElementKind::Text(content),
            layer_id: None,
            style: Style::new().foreground(Color::Red),
        }
    }

    /// Whether this element should be promoted to its own compositing layer.
    pub fn promotes(&self) -> bool {
        // For now, only root containers are promoted.
        false
    }
}
