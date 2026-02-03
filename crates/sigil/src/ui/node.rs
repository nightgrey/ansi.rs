use crate::{LayerId, key};
use geometry::Position;

key!(
    pub struct ElementId;
);

#[derive(Debug)]
pub enum ElementNodeKind {
    Container,
    Text,
}

#[derive(Debug)]
pub struct Element {
    pub kind: ElementNodeKind,
    pub position: Position,
    pub layer_id: Option<LayerId>,
}

impl Element {
    pub const fn container() -> Self {
        Self {
            kind: ElementNodeKind::Container,
            layer_id: None,
            position: Position::ZERO,
        }
    }

    pub const fn text() -> Self {
        Self {
            kind: ElementNodeKind::Text,
            layer_id: None,
            position: Position::ZERO,
        }
    }
}
