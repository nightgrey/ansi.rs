pub trait TreeId: slotmap::Key {
    #[inline]
    fn none() -> Self {
        Self::null()
    }

    #[inline]
    fn is_none(self) -> bool {
        self.is_null()
    }

    #[inline]
    fn is_some(self) -> bool {
        !self.is_none()
    }

    #[inline]
    fn as_option(self) -> Option<Self> {
        match self.is_none() {
            true => None,
            false => Some(self),
        }
    }

    #[inline]
    fn and_then<F: FnOnce(Self) -> Self>(self, f: F) -> Self {
        match self.is_none() {
            true => Self::none(),
            false => f(self),
        }
    }

    #[inline]
    fn or(self, other: Self) -> Self {
        match self.is_none() {
            true => other,
            false => self,
        }
    }

    #[inline]
    fn or_else<F: FnOnce() -> Self>(self, f: F) -> Self {
        match self.is_none() {
            true => f(),
            false => self,
        }
    }
}
#[macro_export]
#[macro_use]
macro_rules! tree_id {
    ( $(#[$outer:meta])* $vis:vis struct $name:ident; $($rest:tt)* ) => {
        use slotmap::Key as _;
        slotmap::new_key_type! {
            $(#[$outer])*
            $vis struct $name;

        }

        impl $crate::TreeId for $name {}
    };
}
