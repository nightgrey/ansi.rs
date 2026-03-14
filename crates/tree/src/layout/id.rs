use crate::{id, Id};
use super::InternalLayoutId;

id!(pub struct LayoutId);

pub trait Bridge {
    fn into_layout(self) -> InternalLayoutId;
    fn from_layout(id: InternalLayoutId) -> Self;
}

impl<K: Id> Bridge for K {
    #[inline]
    fn into_layout(self) -> InternalLayoutId {
        InternalLayoutId::new(self.data().as_ffi())
    }

    #[inline]
    fn from_layout(id: InternalLayoutId) -> Self {
        Self::from(slotmap::KeyData::from_ffi(u64::from(id)))
    }
}
