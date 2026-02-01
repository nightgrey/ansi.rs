use compact_str::CompactString;
use derive_more::{Deref, DerefMut, From, Into};
use geometry::Position;
use crate::Buffer;

#[derive(Debug, Deref, DerefMut, From, Into)]
#[repr(transparent)]
pub struct LayerId(pub(super) indextree::NodeId);

#[derive(Debug, Deref, DerefMut)]
pub struct Layer {
    #[deref]
    #[deref_mut]
    buffer: Buffer,
    is_dirty: bool,
    position: Position
}

impl Layer {
    pub fn new(width: usize, height: usize) -> Self {
        Layer {
            buffer: Buffer::new(width, height),
            is_dirty: false,
            position: Position::default(),
        }
    }
}