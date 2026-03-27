use crate::Buffer;
use derive_more::{Deref, DerefMut};
use geometry::Point;

#[derive(Debug, Deref, DerefMut)]
pub struct Layer {
    #[deref]
    #[deref_mut]
    buffer: Buffer,
    pub position: Point,
    pub z_index: i32,
    pub is_dirty: bool,
}

impl Layer {
    pub const EMPTY: Self = Self {
        buffer: Buffer::EMPTY,
        z_index: 0,
        is_dirty: false,
        position: Point::ZERO,
    };

    pub fn new(width: usize, height: usize) -> Self {
        Layer {
            buffer: Buffer::new(width, height),
            z_index: 0,
            is_dirty: false,
            position: Point::default(),
        }
    }
}
