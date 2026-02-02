use compact_str::CompactString;
use derive_more::{Deref, DerefMut, From, Into};
use geometry::Position;
use crate::Buffer;

#[derive(Debug, Deref, DerefMut)]
pub struct LayerNode {
    #[deref]
    #[deref_mut]
    buffer: Buffer,
    position: Position,
    is_dirty: bool,
}

impl LayerNode {
    pub const ZERO: Self = Self {
        buffer: Buffer::ZERO,
        is_dirty: false,
        position: Position::ZERO
    };

    pub fn new(width: usize, height: usize) -> Self {
        LayerNode {
            buffer: Buffer::new(width, height),
            is_dirty: false,
            position: Position::default(),
        }
    }
}