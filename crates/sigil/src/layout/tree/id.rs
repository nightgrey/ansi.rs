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
    fn maybe(self) -> Option<Self> {
        match self.is_none() {
            true => None,
            false => Some(self),
        }
    }

    #[inline]
    fn insert(&mut self, value: Self) -> &mut Self {
        *self = value;

        self
    }

    #[inline]
    fn map<U, F>(self, f: F) -> Option<U>
    where
        F: FnOnce(Self) -> U
    {
        match self.is_some() {
            true => Some(f(self)),
            false => None,
        }
    }

    #[inline]
    fn map_or<U, F>(self, default: U, f: F) -> U
    where
        F: FnOnce(Self) -> U ,
    {
        match self.is_some() {
            true => f(self),
            false => default,
        }
    }

    #[inline]
    fn map_or_else<U, D, F>(self, default: D, f: F) -> U
    where
        D: FnOnce() -> U ,
        F: FnOnce(Self) -> U ,
    {
        match self.is_some() {
            true => f(self),
            false => default(),
        }
    }

    #[inline]
    fn map_or_default<U, F>(self, f: F) -> U
    where
        U: Default,
        F: FnOnce(Self) -> U ,
    {
        match self.is_some() {
            true => f(self),
            false => U::default(),
        }
    }

    #[inline]
    fn get_or_insert(&mut self, value: Self) -> &mut Self {
        self.get_or_insert_with(|| value)
    }

    #[inline]
    fn get_or_insert_default(&mut self) -> &mut Self
    where
        Self: Default,
    {
        self.get_or_insert_with(Self::default)
    }

    #[inline]
    fn get_or_insert_with<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce() -> Self,
    {
        if self.is_none() {
            *self = f();
        }

        self
    }


    #[inline]
    fn and_then<F: FnOnce(Self) -> Self>(self, f: F) -> Self {
        match self.is_none() {
            true => Self::none(),
            false => f(self),
        }
    }

    #[inline]
    fn or(self, default: Self) -> Self {
        match self.is_none() {
            true => default,
            false => self,
        }
    }
    #[inline]
    #[track_caller]
    fn or_else<F>(self, f: F) -> Self
    where
        F: FnOnce() -> Self,
    {
        match self.is_some() {
            true => self,
            false => f(),
        }
    }

    #[inline]
    fn or_none(self) -> Self
    where
        Self: Default,
    {
        match self.is_some() {
            true => self,
            false => Self::default(),
        }
    }


}

#[macro_export]
#[macro_use]
macro_rules! tree_id {
    ( $(#[$outer:meta])* $vis:vis struct $name:ident; $($rest:tt)* ) => {
        use slotmap::Key as _;
        use $crate::TreeId as _;
        slotmap::new_key_type! {
            $(#[$outer])*
            $vis struct $name;

        }

        impl $crate::TreeId for $name {
        }
    };
}
