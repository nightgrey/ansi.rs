use crate::{LayerId, tree_id, TreeId};
use ansi::{Color, Style};

tree_id!(
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
    pub layer_id: LayerId,
    pub style: Style,
}

impl Element {
    pub  fn container(direction: Direction) -> Self {
        Self {
            kind: ElementKind::Container { direction },
            layer_id: LayerId::none(),
            style: Style::EMPTY,
        }
    }

    pub fn text(content: String) -> Self {
        Self {
            kind: ElementKind::Text(content),
            layer_id: LayerId::none(),
            style: Style::new().foreground(Color::Red),
        }
    }

    /// Whether this element should be promoted to its own compositing layer.
    pub fn promotes(&self) -> bool {
        // For now, only root containers are promoted.
        false
    }
}
