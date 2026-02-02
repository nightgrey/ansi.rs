use indextree::NodeId;
use geometry::Position;

#[derive(Debug)]
pub enum ElementNodeKind {
    Container,
    Text,
}

#[derive(Debug)]
pub struct ElementNode {
    pub kind: ElementNodeKind,
    pub layer_id: Option<NodeId>,
    pub position: Position
}

impl ElementNode {
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