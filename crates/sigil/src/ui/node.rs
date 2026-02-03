use indextree::NodeId;
use geometry::Position;
use crate::key;

key!(pub struct ElementId;);

#[derive(Debug)]
pub enum ElementNodeKind {
    Container,
    Text,
}

#[derive(Debug)]
pub struct Element {
    pub kind: ElementNodeKind,
    pub layer_id: Option<NodeId>,
    pub position: Position
}

impl Element {
    pub const fn container() -> Self {
        Self {
            kind: ElementNodeKind::Container,
            layer_id: None,
            position: Position::ZERO
        }
    }

    pub const fn text() -> Self {
        Self {
            kind: ElementNodeKind::Text,
            layer_id: None,
            position: Position::ZERO
        }
    }
}