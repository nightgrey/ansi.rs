use crate::{Buffer, key};
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
    position: Position,
    is_dirty: bool,
}

impl Layer {
    pub const ZERO: Self = Self {
        buffer: Buffer::ZERO,
        is_dirty: false,
        position: Position::ZERO,
    };

    pub fn new(width: usize, height: usize) -> Self {
        Layer {
            buffer: Buffer::new(width, height),
            is_dirty: false,
            position: Position::default(),
        }
    }
}
