/// Creates a new node-identifier type that implements [`Id`].
///
/// The generated struct is a thin wrapper around a [`slotmap`] key, inheriting
/// its O(1) lookup, ABA-safe versioning, and compact representation.
///
/// # Examples
///
/// ```rust
/// use tree::id;
///
/// id!(pub struct WidgetId);
/// id!(pub(crate) struct NodeId);
/// ```
#[macro_export]
#[macro_use]
macro_rules! id {
    ( $(#[$outer:meta])* $vis:vis struct $name:ident) => {
        use slotmap::Key as _;
        use $crate::Id as _;
        slotmap::new_key_type! {
            $(#[$outer])*
            $vis struct $name;
        }

        impl $crate::Id for $name { }
    };
}

/// A node identifier with `Option`-like semantics.
///
/// Every [`Id`] is either *some* (points to a valid slot) or *none* (the null
/// sentinel). This trait layers familiar combinators — [`map`](Id::map),
/// [`and_then`](Id::and_then), [`or`](Id::or), etc. — directly onto the key
/// so you can work with structural pointers without constantly wrapping them
/// in `Option`.
///
/// You rarely need to implement this trait by hand; use the [`id!`] macro
/// instead.
pub trait Id: slotmap::Key {
    /// Returns the null (absent) sentinel for this key type.
    #[inline]
    fn none() -> Self {
        Self::null()
    }

    /// Returns `true` if this id is the null sentinel.
    #[inline]
    fn is_none(self) -> bool {
        self.is_null()
    }

    /// Returns `true` if this id points to a valid slot.
    #[inline]
    fn is_some(self) -> bool {
        !self.is_none()
    }

    /// Converts to `Option<Self>`, returning `None` for the null sentinel.
    #[inline]
    fn maybe(self) -> Option<Self> {
        match self.is_none() {
            true => None,
            false => Some(self),
        }
    }

    /// Unconditionally overwrites this id with `value`, returning `&mut Self`.
    #[inline]
    fn insert(&mut self, value: Self) -> &mut Self {
        *self = value;

        self
    }

    /// Applies `f` if non-null, returning `Some(f(self))`, or `None` otherwise.
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

    /// Applies `f` if non-null, or returns `default`.
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

    /// Applies `f` if non-null, or lazily evaluates `default`.
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

    /// Applies `f` if non-null, or returns `U::default()`.
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

    /// Sets `self` to `value` if it is currently null, then returns `&mut Self`.
    #[inline]
    fn get_or_insert(&mut self, value: Self) -> &mut Self {
        self.get_or_insert_with(|| value)
    }

    /// Sets `self` to `Self::default()` if it is currently null.
    #[inline]
    fn get_or_insert_default(&mut self) -> &mut Self
    where
        Self: Default,
    {
        self.get_or_insert_with(Self::default)
    }

    /// Sets `self` to `f()` if it is currently null, then returns `&mut Self`.
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


    /// Returns `f(self)` if non-null, or the null sentinel otherwise.
    #[inline]
    fn and_then<F: FnOnce(Self) -> Self>(self, f: F) -> Self {
        match self.is_none() {
            true => Self::none(),
            false => f(self),
        }
    }

    /// Returns `self` if non-null, or `default`.
    #[inline]
    fn or(self, default: Self) -> Self {
        match self.is_none() {
            true => default,
            false => self,
        }
    }

    /// Returns `self` if non-null, or lazily evaluates `f`.
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

    /// Returns `self` if non-null, or `Self::default()`.
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

/// A general-purpose default id type for trees that don't need a custom key.
id!(pub struct DefaultId);
