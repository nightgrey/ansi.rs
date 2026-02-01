use derive_more::{Deref, DerefMut, From, Into};
use crate::LayerId;

#[derive(Debug, Deref, DerefMut, From, Into)]
#[repr(transparent)]
pub struct NodeId(pub(super) indextree::NodeId);

#[derive(Debug)]
pub enum NodeKind {
    Container,
    Text,
}

#[derive(Debug)]
pub struct Node {
    pub kind: NodeKind,
}
