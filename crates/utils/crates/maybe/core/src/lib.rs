pub use maybe_derive::Maybe;

/// A trait for types with a distinguished "none" and "some" state,
/// providing [`Option`]-like combinators without wrapping.
#[allow(non_upper_case_globals, non_snake_case)]
pub trait Maybe: Sized {
    /// No value.
    const None: Self;

    /// Returns `true` if [`Self`] is a [`Self::None`] value.
    ///
    /// # Examples
    ///
    /// ```
    /// use maybe::Maybe;
    ///
    /// #[derive(Maybe, Default, Debug, PartialEq)]
    /// enum Foo {
    ///     #[default]
    ///     None,
    ///     Something,
    /// }
    ///
    /// let x: Foo = Foo::Something;
    /// assert_eq!(x.is_none(), false);
    ///
    /// let x: Foo = Foo::None;
    /// assert_eq!(x.is_none(), true);
    /// ```
    fn is_none(&self) -> bool;

    /// Returns `true` if [`Self`] is a "Some" value.
    ///
    /// # Examples
    ///
    /// ```
    /// use maybe::Maybe;
    ///
    /// #[derive(Maybe, Default, Debug, PartialEq)]
    /// enum Foo {
    ///     #[default]
    ///     None,
    ///     Something,
    /// }
    ///
    /// let x: Foo = Foo::Something;
    /// assert_eq!(x.is_some(), true);
    ///
    /// let x: Foo = Foo::None;
    /// assert_eq!(x.is_some(), false);
    /// ```
    #[inline]
    fn is_some(&self) -> bool {
        !self.is_none()
    }

    /// Maps [`Self`] by applying a function to a contained value (if "Some") or returns `None` (if [`Self::None`]).
    ///
    /// # Examples
    ///
    /// ```
    /// # use maybe::Maybe;
    /// # #[derive(Maybe, Default, Clone, Copy, Debug, PartialEq)]
    /// # enum Color {
    /// #     #[default]
    /// #     None,
    /// #     Rgb(u8, u8, u8),
    /// # }
    /// let x = Color::Rgb(255, 0, 0);
    /// assert_eq!(x.map(|_| "rgb"), Some("rgb"));
    ///
    /// let x: Color = Color::None;
    /// assert_eq!(x.map(|_| "rgb"), None);
    /// ```
    #[inline]
    fn map<U>(self, f: impl FnOnce(Self) -> U) -> Option<U> {
        if self.is_some() { Some(f(self)) } else { None }
    }

    /// Returns [`Self::None`] if the value is [`Self::None`], otherwise calls `f` with the
    /// wrapped value and returns the result.
    ///
    /// Some languages call this operation flatmap.
    ///
    /// # Examples
    ///
    /// ```
    /// # use maybe::Maybe;
    /// # #[derive(Maybe, Default, Clone, Copy, Debug, PartialEq)]
    /// # enum Color {
    /// #     #[default]
    /// #     None,
    /// #     Rgb(u8, u8, u8),
    /// # }
    /// fn space_name(color: Color) -> &'static str {
    ///     match color {
    ///         Color::Rgb(..) => "RGB",
    ///         _ => "",
    ///     }
    /// }
    ///
    /// assert_eq!(Color::Rgb(255, 0, 0).and_then(space_name), Some("RGB"));
    /// assert_eq!(Color::None.and_then(space_name), None);
    /// ```
    ///
    /// Often used to chain fallible operations that may return [`Self::None`].
    ///
    /// ```
    /// let arr_2d = [["A0", "A1"], ["B0", "B1"]];
    ///
    /// let item_0_1 = arr_2d.get(0).and_then(|row| row.get(1));
    /// assert_eq!(item_0_1, Some(&"A1"));
    ///
    /// let item_2_0 = arr_2d.get(2).and_then(|row| row.get(0));
    /// assert_eq!(item_2_0, None);
    /// ```
    #[inline]
    fn and_then<U>(self, f: impl FnOnce(Self) -> U) -> Option<U> {
        if self.is_some() { Some(f(self)) } else { None }
    }

    /// Returns the provided default result (if none),
    /// or applies a function to the contained value (if any).
    ///
    /// Arguments passed to `map_or` are eagerly evaluated; if you are passing
    /// the result of a function call, it is recommended to use [`Self::map_or_else`],
    /// which is lazily evaluated.
    ///
    /// # Examples
    ///
    /// ```
    /// # use maybe::Maybe;
    /// # #[derive(Maybe, Default, Clone, Copy, Debug, PartialEq)]
    /// # enum Color {
    /// #     #[default]
    /// #     None,
    /// #     Rgb(u8, u8, u8),
    /// # }
    /// let x = Color::Rgb(255, 0, 0);
    /// assert_eq!(x.map_or(42, |_| 3), 3);
    ///
    /// let x: Color = Color::None;
    /// assert_eq!(x.map_or(42, |_| 3), 42);
    /// ```
    #[inline]
    fn map_or<U>(self, default: U, f: impl FnOnce(Self) -> U) -> U {
        if self.is_some() { f(self) } else { default }
    }

    /// Computes a default function result (if none), or
    /// applies a different function to the contained value (if any).
    ///
    /// # Basic examples
    ///
    /// ```
    /// # use maybe::Maybe;
    /// # #[derive(Maybe, Default, Clone, Copy, Debug, PartialEq)]
    /// # enum Color {
    /// #     #[default]
    /// #     None,
    /// #     Rgb(u8, u8, u8),
    /// # }
    /// let k = 21;
    ///
    /// let x = Color::Rgb(255, 0, 0);
    /// assert_eq!(x.map_or_else(|| 2 * k, |_| 3), 3);
    ///
    /// let x: Color = Color::None;
    /// assert_eq!(x.map_or_else(|| 2 * k, |_| 3), 42);
    /// ```
    ///
    /// # Handling a Result-based fallback
    ///
    /// A somewhat common occurrence when dealing with optional values
    /// in combination with [`Result<T, E>`] is the case where one wants to invoke
    /// a fallible fallback if the value is not present. This example
    /// parses a command line argument (if present), or the contents of a file to
    /// an integer. However, unlike accessing the command line argument, reading
    /// the file is fallible, so it must be wrapped with `Ok`.
    ///
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let v: u64 = std::env::args()
    ///    .nth(1)
    ///    .map_or_else(|| std::fs::read_to_string("/etc/someconfig.conf"), Ok)?
    ///    .parse()?;
    /// #   Ok(())
    /// # }
    /// ```
    #[inline]
    fn map_or_else<U>(self, default: impl FnOnce() -> U, f: impl FnOnce(Self) -> U) -> U {
        if self.is_some() { f(self) } else { default() }
    }

    /// Maps a [`Self`] to a `U` by applying function `f` to the contained
    /// value if the value is "Some", otherwise if [`Self::None`], returns the
    /// [default value] for the type `U`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use maybe::Maybe;
    /// # #[derive(Maybe, Default, Clone, Copy, Debug, PartialEq)]
    /// # enum Color {
    /// #     #[default]
    /// #     None,
    /// #     Rgb(u8, u8, u8),
    /// # }
    /// let x = Color::Rgb(255, 0, 0);
    /// let y: Color = Color::None;
    ///
    /// assert_eq!(x.map_or_default(|_| 3usize), 3);
    /// assert_eq!(y.map_or_default(|_| 3usize), 0);
    /// ```
    ///
    /// [default value]: Default::default
    #[inline]
    fn map_or_default<U: Default>(self, f: impl FnOnce(Self) -> U) -> U {
        if self.is_some() {
            f(self)
        } else {
            U::default()
        }
    }

    /// Returns the value if it contains a value, otherwise returns `other`.
    ///
    /// Arguments passed to `or` are eagerly evaluated; if you are passing the
    /// result of a function call, it is recommended to use [`Self::or_else`], which is
    /// lazily evaluated.
    ///
    /// # Examples
    ///
    /// ```
    /// # use maybe::Maybe;
    /// # #[derive(Maybe, Default, Clone, Copy, Debug, PartialEq)]
    /// # enum Color {
    /// #     #[default]
    /// #     None,
    /// #     Rgb(u8, u8, u8),
    /// # }
    /// let x = Color::Rgb(255, 0, 0);
    /// let y = Color::None;
    /// assert_eq!(x.or(y), Color::Rgb(255, 0, 0));
    ///
    /// let x = Color::None;
    /// let y = Color::Rgb(0, 255, 0);
    /// assert_eq!(x.or(y), Color::Rgb(0, 255, 0));
    ///
    /// let x = Color::Rgb(255, 0, 0);
    /// let y = Color::Rgb(0, 255, 0);
    /// assert_eq!(x.or(y), Color::Rgb(255, 0, 0));
    ///
    /// let x: Color = Color::None;
    /// let y = Color::None;
    /// assert_eq!(x.or(y), Color::None);
    /// ```
    #[inline]
    fn or(self, other: Self) -> Self {
        if self.is_some() { self } else { other }
    }

    /// Returns the value if it contains a value, otherwise calls `f` and
    /// returns the result.
    ///
    /// # Examples
    ///
    /// ```
    /// # use maybe::Maybe;
    /// # #[derive(Maybe, Default, Clone, Copy, Debug, PartialEq)]
    /// # enum Color {
    /// #     #[default]
    /// #     None,
    /// #     Rgb(u8, u8, u8),
    /// # }
    /// fn none_color() -> Color { Color::None }
    /// fn red_color() -> Color { Color::Rgb(255, 0, 0) }
    ///
    /// assert_eq!(Color::Rgb(0, 255, 0).or_else(red_color), Color::Rgb(0, 255, 0));
    /// assert_eq!(Color::None.or_else(red_color), Color::Rgb(255, 0, 0));
    /// assert_eq!(Color::None.or_else(none_color), Color::None);
    /// ```
    #[inline]
    fn or_else(self, f: impl FnOnce() -> Self) -> Self {
        if self.is_some() { self } else { f() }
    }

    /// Returns [`Self::None`] if the value is [`Self::None`], otherwise returns `other`.
    ///
    /// Arguments passed to `and` are eagerly evaluated; if you are passing the
    /// result of a function call, it is recommended to use [`Self::and_then`], which is
    /// lazily evaluated.
    ///
    /// # Examples
    ///
    /// ```
    /// # use maybe::Maybe;
    /// # #[derive(Maybe, Default, Clone, Copy, Debug, PartialEq)]
    /// # enum Color {
    /// #    #[default]
    /// #    #[none]
    /// #    None,
    /// #    Rgb(u8, u8, u8),
    /// #    Index(u8),
    /// # }
    ///
    /// let x = Color::Rgb(255, 0, 0);
    /// let y: Color = Color::None;
    /// assert_eq!(x.and(y), Color::None);
    ///
    /// let x: Color = Color::None;
    /// let y = Color::Rgb(0, 255, 0);
    /// assert_eq!(x.and(y), Color::None);
    ///
    /// let x = Color::Rgb(255, 0, 0);
    /// let y = Color::Rgb(0, 255, 0);
    /// assert_eq!(x.and(y), Color::Rgb(0, 255, 0));
    ///
    /// let x: Color = Color::None;
    /// let y: Color = Color::None;
    /// assert_eq!(x.and(y), Color::None);
    /// ```
    #[inline]
    fn and(self, other: Self) -> Self {
        if self.is_some() { other } else { Self::None }
    }

    /// Returns [`Self::None`] if the value is [`Self::None`], otherwise calls `predicate`
    /// with the wrapped value and returns:
    ///
    /// - "Some" if `predicate` returns `true` (where `t` is the wrapped
    ///   value), and
    /// - [`Self::None`] if `predicate` returns `false`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use maybe::Maybe;
    /// # #[derive(Maybe, Default, Clone, Copy, Debug, PartialEq)]
    /// # enum Color {
    /// #    #[default]
    /// #    None,
    /// #    Black,
    /// #    Red,
    /// #    Green,
    /// #    Blue,
    /// #    Rgb(u8, u8, u8),
    /// #    Index(u8),
    /// # }
    /// fn has_blue(color: &Color) -> bool {
    ///     match color {
    ///         Color::Rgb(r, g, b) => *b > 0,
    ///         _ => false,
    ///     }
    /// }
    ///
    /// assert_eq!(Color::None.filter(has_blue), Color::None);
    /// assert_eq!(Color::Rgb(0, 0, 0).filter(has_blue), Color::None);
    /// assert_eq!(Color::Rgb(0, 0, 255).filter(has_blue), Color::Rgb(0, 0, 255));
    /// ```
    #[inline]
    fn filter(self, pred: impl FnOnce(&Self) -> bool) -> Self {
        if self.is_some() && pred(&self) {
            self
        } else {
            Self::None
        }
    }

    /// Inserts `value` into [`Self`] if it is [`Self::None`], then
    /// returns a mutable reference to the contained value.
    ///
    /// See also [`Self::insert`], which updates the value even if
    /// [`Self`] already is "Some".
    ///
    /// # Examples
    ///
    /// ```
    /// # use maybe::Maybe;
    /// # #[derive(Maybe, Default, Clone, Copy, Debug, PartialEq)]
    /// # enum Color {
    /// #     #[default]
    /// #     None,
    /// #     Rgb(u8, u8, u8),
    /// # }
    /// let mut x = Color::None;
    ///
    /// {
    ///     let y: &mut Color = x.get_or_insert(Color::Rgb(255, 0, 0));
    ///     assert_eq!(y, &Color::Rgb(255, 0, 0));
    ///
    ///     *y = Color::Rgb(0, 0, 255);
    /// }
    ///
    /// assert_eq!(x, Color::Rgb(0, 0, 255));
    /// ```
    #[inline]
    fn get_or_insert(&mut self, value: Self) -> &mut Self {
        if self.is_none() {
            *self = value;
        }
        self
    }

    /// Inserts a value computed from `f` into [`Self`] if it is [`Self::None`],
    /// then returns a mutable reference to the contained value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use maybe::Maybe;
    /// # #[derive(Maybe, Default, Clone, Copy, Debug, PartialEq)]
    /// # enum Color {
    /// #     #[default]
    /// #     None,
    /// #     Rgb(u8, u8, u8),
    /// # }
    /// let mut x = Color::None;
    ///
    /// {
    ///     let y: &mut Color = x.get_or_insert_with(|| Color::Rgb(255, 0, 0));
    ///     assert_eq!(y, &Color::Rgb(255, 0, 0));
    ///
    ///     *y = Color::Rgb(0, 0, 255);
    /// }
    ///
    /// assert_eq!(x, Color::Rgb(0, 0, 255));
    /// ```
    #[inline]
    fn get_or_insert_with(&mut self, f: impl FnOnce() -> Self) -> &mut Self {
        if self.is_none() {
            let _ = core::mem::replace(self, f());
        }
        self
    }

    /// Takes the value out of the option, leaving a [`Self::None`] in its place.
    ///
    /// # Examples
    ///
    /// ```
    /// # use maybe::Maybe;
    /// # #[derive(Maybe, Default, Clone, Copy, Debug, PartialEq)]
    /// # enum Color {
    /// #     #[default]
    /// #     None,
    /// #     Rgb(u8, u8, u8),
    /// # }
    /// let mut x = Color::Rgb(255, 0, 0);
    /// let y = x.take();
    /// assert_eq!(x, Color::None);
    /// assert_eq!(y, Color::Rgb(255, 0, 0));
    ///
    /// let mut x: Color = Color::None;
    /// let y = x.take();
    /// assert_eq!(x, Color::None);
    /// assert_eq!(y, Color::None);
    /// ```
    #[inline]
    fn take(&mut self) -> Self {
        core::mem::replace(self, Self::None)
    }

    /// Replaces the actual value in the option by the value given in parameter,
    /// returning the old value if present,
    /// leaving a "Some" in its place without deinitializing either one.
    ///
    /// # Examples
    ///
    /// ```
    /// # use maybe::Maybe;
    /// # #[derive(Maybe, Default, Clone, Copy, Debug, PartialEq)]
    /// # enum Color {
    /// #     #[default]
    /// #     None,
    /// #     Rgb(u8, u8, u8),
    /// # }
    /// let mut x = Color::Rgb(255, 0, 0);
    /// let old = x.replace(Color::Rgb(0, 255, 0));
    /// assert_eq!(x, Color::Rgb(0, 255, 0));
    /// assert_eq!(old, Color::Rgb(255, 0, 0));
    ///
    /// let mut x = Color::None;
    /// let old = x.replace(Color::Rgb(0, 0, 255));
    /// assert_eq!(x, Color::Rgb(0, 0, 255));
    /// assert_eq!(old, Color::None);
    /// ```
    #[inline]
    fn replace(&mut self, value: Self) -> Self {
        core::mem::replace(self, value)
    }

    /// Converts [`Self`] to [`Option<Self>`].
    ///
    /// # Examples
    ///
    /// ```
    /// use maybe::Maybe;
    ///
    /// #[derive(Maybe, Default, Debug, PartialEq)]
    /// enum Foo {
    ///     #[default]
    ///     None,
    ///     Something,
    /// }
    ///
    /// let x: Foo = Foo::Something;
    /// assert_eq!(x.maybe(), Some(Foo::Something));
    ///
    /// let x: Foo = Foo::None;
    /// assert_eq!(x.maybe(), None);
    /// ```
    #[inline]
    fn maybe(self) -> Option<Self> {
        if self.is_some() { Some(self) } else { None }
    }

    /// Converts [`Option<Self>`] to [`Self`].
    ///
    /// # Examples
    ///
    /// ```
    /// use maybe::Maybe;
    ///
    /// #[derive(Maybe, Default, Debug, PartialEq)]
    /// enum Foo {
    ///     #[default]
    ///     None,
    ///     Something,
    /// }
    ///
    /// let x: Option<Foo> = Some(Foo::Something);
    /// assert_eq!(Foo::from_option(x), Foo::Something);
    ///
    /// let x: Option<Foo> = None;
    /// assert_eq!(Foo::from_option(x), Foo::None);
    /// ```
    #[inline]
    fn from_option(option: Option<Self>) -> Self {
        option.unwrap_or_else(|| Self::None)
    }
}

impl<T> Maybe for Option<T> {
    #[allow(non_upper_case_globals)]
    const None: Self = None;

    #[inline]
    fn is_none(&self) -> bool {
        Option::is_none(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Maybe;

    #[derive(Maybe, Default, Clone, Copy, Debug, PartialEq)]
    enum Color {
        #[default]
        None,
        Black,
        Red,
        Green,
        Blue,
        Rgb(u8, u8, u8),
        Index(u8),
    }

    #[derive(Maybe, Default, Clone, Copy, Debug, PartialEq)]
    enum Weight {
        #[none]
        Unset,
        #[default]
        Normal,
        Bold,
    }

    // ---- none resolution ----

    mod none_resolution {
        use super::*;

        #[test]
        fn auto_detected_by_name() {
            assert_eq!(Color::None, Color::None);
            assert!(Color::None.is_none());
        }

        #[test]
        fn explicit_attr() {
            assert_eq!(Weight::None, Weight::Unset);
            assert!(Weight::Unset.is_none());
        }

        #[test]
        fn some_variants_are_some() {
            assert!(Color::Red.is_some());
            assert!(Color::Black.is_some());
            assert!(Weight::Normal.is_some());
            assert!(Weight::Bold.is_some());
        }
    }

    // ---- default ----

    mod default_impl {
        use super::*;

        #[test]
        fn default_is_none_when_unspecified() {
            assert_eq!(Color::default(), Color::None);
        }

        #[test]
        fn default_respects_maybe_default_attr() {
            assert_eq!(Weight::default(), Weight::Normal);
            // none and default are distinct
            assert_ne!(Weight::None, Weight::default());
        }
    }

    // ---- maybe / conversions ----

    mod conversions {
        use super::*;

        #[test]
        fn maybe_some() {
            assert_eq!(Color::Red.maybe(), Some(Color::Red));
        }

        #[test]
        fn maybe_none() {
            assert_eq!(Color::None.maybe(), None);
        }

        #[test]
        fn from_option_some() {
            let c: Color = Color::from_option(Some(Color::Red));
            assert_eq!(c, Color::Red);
        }

        #[test]
        fn from_option_none() {
            let c: Color = Color::from_option(None);
            assert_eq!(c, Color::None);
        }

        #[test]
        fn into_option_from_some() {
            let opt: Option<Color> = Color::Red.into();
            assert_eq!(opt, Some(Color::Red));
        }

        #[test]
        fn into_option_from_none() {
            let opt: Option<Color> = Color::None.into();

            assert_eq!(opt, Some(Color::None));
        }

        #[test]
        fn from_impl_some() {
            let c: Color = Some(Color::Blue).into();
            assert_eq!(c, Color::Blue);
        }

        #[test]
        fn from_impl_none() {
            let c: Color = Option::<Color>::None.into();
            assert_eq!(c, Color::None);
        }

        #[test]
        fn roundtrip() {
            let c: Color = Some(Color::None).into();
            assert_eq!(c, Color::None);
            let opt: Option<Color> = c.into();
            assert_eq!(opt, Some(Color::None));
        }
    }

    // ---- map ----

    mod map {
        use super::*;

        #[test]
        fn map_some() {
            let c = Color::Red.map(|_| Color::Blue);
            assert_eq!(c, Color::Blue);
        }

        #[test]
        fn map_none() {
            let c = Color::None.map(|_| Color::Blue);
            assert_eq!(c, Color::None);
        }

        #[test]
        fn map_or_some() {
            let s = Color::Red.map_or("fallback", |c| match c {
                Color::Red => "red",
                _ => "other",
            });
            assert_eq!(s, "red");
        }

        #[test]
        fn map_or_none() {
            let s = Color::None.map_or("fallback", |_| "value");
            assert_eq!(s, "fallback");
        }

        #[test]
        fn map_or_else_some() {
            let s = Color::Red.map_or_else(|| "fallback", |_| "value");
            assert_eq!(s, "value");
        }

        #[test]
        fn map_or_else_none() {
            let s = Color::None.map_or_else(|| "fallback", |_| "value");
            assert_eq!(s, "fallback");
        }
    }

    // ---- and_then (cross-type) ----

    mod and_then {
        use super::*;

        #[test]
        fn and_then_some_to_some() {
            let w = Color::Red.and_then(|_| Weight::Bold);
            assert_eq!(w, Weight::Bold);
        }

        #[test]
        fn and_then_some_to_none() {
            let w = Color::Red.and_then(|_| Weight::Unset);
            assert_eq!(w, Weight::Unset);
        }

        #[test]
        fn and_then_none_propagates() {
            let w = Color::None.and_then(|_| Weight::Bold);
            assert_eq!(w, Weight::Unset); // UNone
        }

        #[test]
        fn and_then_same_type() {
            let c = Color::Red.and_then(|_| Color::Green);
            assert_eq!(c, Color::Green);

            let c = Color::None.and_then(|_| Color::Green);
            assert_eq!(c, Color::None);
        }
    }

    // ---- or ----

    mod or_family {
        use super::*;

        #[test]
        fn or_some_ignores_fallback() {
            assert_eq!(Color::Red.or(Color::Blue), Color::Red);
        }

        #[test]
        fn or_none_uses_fallback() {
            assert_eq!(Color::None.or(Color::Blue), Color::Blue);
        }

        #[test]
        fn or_else_some() {
            assert_eq!(Color::Red.or_else(|| Color::Blue), Color::Red);
        }

        #[test]
        fn or_else_none() {
            assert_eq!(Color::None.or_else(|| Color::Blue), Color::Blue);
        }

        #[test]
        fn or_else_lazy() {
            let mut called = false;
            Color::Red.or_else(|| {
                called = true;
                Color::Blue
            });
            assert!(!called);
        }
    }

    // ---- and ----

    mod and_family {
        use super::*;

        #[test]
        fn and_both_some() {
            assert_eq!(Color::Red.and(Color::Blue), Color::Blue);
        }

        #[test]
        fn and_first_none() {
            assert_eq!(Color::None.and(Color::Blue), Color::None);
        }

        #[test]
        fn and_second_none() {
            assert_eq!(Color::Red.and(Color::None), Color::None);
        }
    }

    // ---- filter ----

    mod filter {
        use super::*;

        #[test]
        fn filter_passes() {
            let c = Color::Red.filter(|c| matches!(c, Color::Red));
            assert_eq!(c, Color::Red);
        }

        #[test]
        fn filter_rejects() {
            let c = Color::Red.filter(|c| matches!(c, Color::Blue));
            assert_eq!(c, Color::None);
        }

        #[test]
        fn filter_none_stays_none() {
            let c = Color::None.filter(|_| true);
            assert_eq!(c, Color::None);
        }
    }

    // ---- mutating ----

    mod mutating {
        use super::*;

        #[test]
        fn take_extracts_and_resets() {
            let mut c = Color::Red;
            let taken = c.take();
            assert_eq!(taken, Color::Red);
            assert_eq!(c, Color::None);
        }

        #[test]
        fn take_none_stays_none() {
            let mut c = Color::None;
            let taken = c.take();
            assert_eq!(taken, Color::None);
            assert_eq!(c, Color::None);
        }

        #[test]
        fn replace_returns_old() {
            let mut c = Color::Red;
            let old = c.replace(Color::Blue);
            assert_eq!(old, Color::Red);
            assert_eq!(c, Color::Blue);
        }

        #[test]
        fn get_or_insert_when_none() {
            let mut c = Color::None;
            c.get_or_insert(Color::Green);
            assert_eq!(c, Color::Green);
        }

        #[test]
        fn get_or_insert_when_some() {
            let mut c = Color::Red;
            c.get_or_insert(Color::Green);
            assert_eq!(c, Color::Red);
        }

        #[test]
        fn get_or_insert_with_lazy() {
            let mut called = false;
            let mut c = Color::Red;
            c.get_or_insert_with(|| {
                called = true;
                Color::Green
            });
            assert!(!called);
            assert_eq!(c, Color::Red);
        }
    }

    // ---- Option<T> blanket impl ----

    mod option_impl {
        use super::*;

        #[test]
        fn option_none() {
            assert_eq!(Option::<i32>::None, None);
            assert!(Option::<i32>::None.is_none());
        }

        #[test]
        fn option_some() {
            assert!(Some(42).is_some());
        }

        #[test]
        fn option_maybe() {
            assert_eq!(Some(42).maybe(), Some(Some(42)));
            assert_eq!(Option::<i32>::None.maybe(), None);
        }

        #[test]
        fn option_or() {
            assert_eq!(None.or(Some(42)), Some(42));
            assert_eq!(Some(1).or(Some(42)), Some(1));
        }

        #[test]
        fn option_and_then_cross_type() {
            let c = Some(42).and_then(|n| if n > 0 { Some(Color::Red) } else { None });
            assert_eq!(c, Some(Color::Red));

            let c = Option::<i32>::None.and_then(|_| Some(Color::Red));
            assert_eq!(c, Color::None);
        }

        #[test]
        fn option_map_or() {
            assert_eq!(Some(2).map_or(0, |v| v * 3), 6);
            assert_eq!(Option::<i32>::None.map_or(0, |_| 99), 0);
        }

        #[test]
        fn option_take() {
            let mut o = Some(42);
            let taken = Maybe::take(&mut o);
            assert_eq!(taken, Some(42));
            assert_eq!(o, None);
        }
    }

    // ---- generic code over Maybe ----

    mod generic {
        use super::*;

        fn fallback<T: Maybe>(a: T, b: T) -> T {
            a.or(b)
        }

        fn reset<T: Maybe>(val: &mut T) {
            *val = T::None;
        }

        #[test]
        fn generic_fallback_enum() {
            assert_eq!(fallback(Color::None, Color::Red), Color::Red);
            assert_eq!(fallback(Color::Blue, Color::Red), Color::Blue);
        }

        #[test]
        fn generic_fallback_option() {
            assert_eq!(fallback(None, Some(42)), Some(42));
            assert_eq!(fallback(Some(1), Some(42)), Some(1));
        }

        #[test]
        fn generic_reset() {
            let mut c = Color::Red;
            reset(&mut c);
            assert_eq!(c, Color::None);

            let mut o = Some(42);
            reset(&mut o);
            assert_eq!(o, None);
        }
    }
}
