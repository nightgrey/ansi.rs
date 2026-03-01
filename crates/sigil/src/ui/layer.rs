use crate::{Buffer, NodeRef, NodeRefMut, key};
use compact_str::CompactString;
use derive_more::{Deref, DerefMut, From, Into};
use geometry::Position;

key!(
    pub struct LayerId;
);

#[derive(Debug, Deref, DerefMut)]
pub struct Layer {
    #[deref]
    #[deref_mut]
    buffer: Buffer,
    pub position: Position,
    pub z_index: i32,
    pub is_dirty: bool,
}

impl Layer {
    pub const EMPTY: Self = Self {
        buffer: Buffer::EMPTY,
        z_index: 0,
        is_dirty: false,
        position: Position::ZERO,
    };

    pub fn new(width: usize, height: usize) -> Self {
        Layer {
            buffer: Buffer::new(width, height),
            z_index: 0,
            is_dirty: false,
            position: Position::default(),
        }
    }
}
