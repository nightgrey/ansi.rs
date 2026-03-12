use crate::{tree_id, LayerId};
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
    pub style: Style,
    pub layer_id: LayerId,
}

impl Element {
    pub fn container(direction: Direction) -> Self {
        Self {
            kind: ElementKind::Container { direction },
            style: Style::EMPTY,
            layer_id: LayerId::none(),
        }
    }

    pub fn text(content: String) -> Self {
        Self {
            kind: ElementKind::Text(content),
            style: Style::new().foreground(Color::Red),
            layer_id: LayerId::none(),
        }
    }
    
    pub fn on(mut self, layer_id: LayerId) -> Self {
        self.layer_id = layer_id;
        self
    }

    /// Whether this element should be promoted to its own compositing layer.
    pub fn is_promoting(&self) -> bool {
        // For now, only root containers are promoted.
        false
    }
}
