pub trait Key: slotmap::Key {
    fn is_none(self) -> bool {
        self.is_null()
    }

    fn is_some(self) -> bool {
        !self.is_none()
    }

    fn option(self) -> Option<Self> {
        match self.is_null() {
            true => None,
            false => Some(self),
        }
    }

    fn and_then<F: FnOnce(Self) -> Self>(self, f: F) -> Self {
        match self.is_null() {
            true => Self::null(),
            false => f(self),
        }
    }

    fn or(self, other: Self) -> Self {
        match self.is_null() {
            true => other,
            false => self,
        }
    }

    fn or_else<F: FnOnce() -> Self>(self, f: F) -> Self {
        match self.is_null() {
            true => f(),
            false => self,
        }
    }
}

#[macro_export]
#[macro_use]
macro_rules! key {
    ( $(#[$outer:meta])* $vis:vis struct $name:ident; $($rest:tt)* ) => {
        pub use slotmap::Key;

        slotmap::new_key_type! {
            $(#[$outer])*
            $vis struct $name;

        }

        impl $crate::Key for $name {}
    };
}
