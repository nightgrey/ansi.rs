use crate::{Id};
use super::LayoutNodeId;

pub trait Bridge {
    fn into_layout_id(self) -> LayoutNodeId;
    fn from_layout_id(id: LayoutNodeId) -> Self;
}

impl<K: Id> Bridge for K {
    #[inline]
    fn into_layout_id(self) -> LayoutNodeId {
        LayoutNodeId::new(self.data().as_ffi())
    }

    #[inline]
    fn from_layout_id(id: LayoutNodeId) -> Self {
        slotmap::KeyData::from_ffi(u64::from(id)).into()
    }
}
