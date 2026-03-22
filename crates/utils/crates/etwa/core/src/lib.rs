pub use etwa_derive::Etwa;

/// A trait for types with a distinguished "none" and "some" state,
/// providing [`Option`]-like combinators without wrapping.
///
/// # Note: `Option::Some(TNone)` -> `TNone` -> `Option::None` roundtrip
///
/// The `From` impls treat the none variant as `Option::None`.
/// This means `Some(Color::None)` converts to `Color::None` which
/// converts back to `Option::None` — the roundtrip is intentionally
/// lossy. In an ideal world `Some(TNone)` wouldn't exist, but
/// when it does, it collapses to none.
#[allow(non_upper_case_globals, non_snake_case)]
pub trait Etwa: Sized {
    /// No value.
    const None: Self;

    /// Returns `true` if [`Self`] is a [`Self::None`] value.
    ///
    /// # Examples
    ///
    /// ```
    /// use etwa::Etwa;
    ///
    /// enum Foo {
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
    /// use etwa::Etwa;
    ///
    /// enum Foo {
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

    /// Maps [`Self`] by applying a function to a contained value (if "Some") or returns [`Self::None`] (if [`Self::None`]).
    ///
    /// # Examples
    ///
    /// Gets the color space of an <code>Color</code> as an
    /// <code>ColorSpace</code>, consuming the original:
    ///
    /// ```
    /// let x = Color::Rgb(255, 0, 0);
    /// // `Color::map` takes self *by value*, consuming `maybe_some_color`
    /// let maybe_color_space = x.map(|c| c.color_space());
    /// assert_eq!(maybe_color_space, Some(ColorSpace::Rgb));
    ///
    /// let x: Color = Color::None;
    /// assert_eq!(x.map(|c| c.color_space()), None);
    /// ```
    #[inline]
    fn map<U>(self, f: impl FnOnce(Self) -> U) -> Option<U> {
        if self.is_some() { Some(f(self)) } else { None }
    }

    /// Returns [`Etwa::None`] if the option is [`Self::None`], otherwise calls `f` with the
    /// wrapped value and returns the result.
    ///
    /// Some languages call this operation flatmap.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::ascii::Char::Cancel;
    ///
    /// fn stringified_space(color: Color) -> String {
    ///     color.map(|space| Some(space.to_string()))
    /// }
    ///
    /// assert_eq!(Color::Rgb(255, 0, 0).and_then(stringified_space), Some("RGB".to_string()));
    /// assert_eq!(Color::None.and_then(stringified_space), None);
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
    /// the result of a function call, it is recommended to use [`map_or_else`],
    /// which is lazily evaluated.
    ///
    /// [`map_or_else`]: Option::map_or_else
    ///
    /// # Examples
    ///
    /// ```
    /// let x = Some("foo");
    /// assert_eq!(x.map_or(42, |v| v.len()), 3);
    ///
    /// let x: Option<&str> = None;
    /// assert_eq!(x.map_or(42, |v| v.len()), 42);
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
    /// let k = 21;
    ///
    /// let x = Some("foo");
    /// assert_eq!(x.map_or_else(|| 2 * k, |v| v.len()), 3);
    ///
    /// let x: Option<&str> = None;
    /// assert_eq!(x.map_or_else(|| 2 * k, |v| v.len()), 42);
    /// ```
    ///
    /// # Handling a Result-based fallback
    ///
    /// A somewhat common occurrence when dealing with optional values
    /// in combination with [`Result<T, E>`] is the case where one wants to invoke
    /// a fallible fallback if the option is not present.  This example
    /// parses a command line argument (if present), or the contents of a file to
    /// an integer.  However, unlike accessing the command line argument, reading
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
    fn map_or_else<U>(
        self,
        default: impl FnOnce() -> U,
        f: impl FnOnce(Self) -> U,
    ) -> U {
        if self.is_some() { f(self) } else { default() }
    }

    /// Maps an `Option<T>` to a `U` by applying function `f` to the contained
    /// value if the option is [`Some`], otherwise if [`None`], returns the
    /// [default value] for the type `U`.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(result_option_map_or_default)]
    ///
    /// let x: Option<&str> = Some("hi");
    /// let y: Option<&str> = None;
    ///
    /// assert_eq!(x.map_or_default(|x| x.len()), 2);
    /// assert_eq!(y.map_or_default(|y| y.len()), 0);
    /// ```
    ///
    /// [default value]: Default::default
    #[inline]
    fn map_or_default<U: Default>(self, f: impl FnOnce(Self) -> U) -> U {
        if self.is_some() { f(self) } else { U::default() }
    }

    /// Returns the option if it contains a value, otherwise returns `optb`.
    ///
    /// Arguments passed to `or` are eagerly evaluated; if you are passing the
    /// result of a function call, it is recommended to use [`or_else`], which is
    /// lazily evaluated.
    ///
    /// [`or_else`]: Option::or_else
    ///
    /// # Examples
    ///
    /// ```
    /// let x = Some(2);
    /// let y = None;
    /// assert_eq!(x.or(y), Some(2));
    ///
    /// let x = None;
    /// let y = Some(100);
    /// assert_eq!(x.or(y), Some(100));
    ///
    /// let x = Some(2);
    /// let y = Some(100);
    /// assert_eq!(x.or(y), Some(2));
    ///
    /// let x: Option<u32> = None;
    /// let y = None;
    /// assert_eq!(x.or(y), None);
    /// ```
    #[inline]
    fn or(self, fallback: Self) -> Self {
        if self.is_some() { self } else { fallback }
    }

    /// Returns the option if it contains a value, otherwise calls `f` and
    /// returns the result.
    ///
    /// # Examples
    ///
    /// ```
    /// fn nobody() -> Option<&'static str> { None }
    /// fn vikings() -> Option<&'static str> { Some("vikings") }
    ///
    /// assert_eq!(Some("barbarians").or_else(vikings), Some("barbarians"));
    /// assert_eq!(None.or_else(vikings), Some("vikings"));
    /// assert_eq!(None.or_else(nobody), None);
    /// ```
    #[inline]
    fn or_else(self, f: impl FnOnce() -> Self) -> Self {
        if self.is_some() { self } else { f() }
    }

    /// Returns [`None`] if the option is [`None`], otherwise returns `optb`.
    ///
    /// Arguments passed to `and` are eagerly evaluated; if you are passing the
    /// result of a function call, it is recommended to use [`and_then`], which is
    /// lazily evaluated.
    ///
    /// [`and_then`]: Option::and_then
    ///
    /// # Examples
    ///
    /// ```
    /// let x = Some(2);
    /// let y: Option<&str> = None;
    /// assert_eq!(x.and(y), None);
    ///
    /// let x: Option<u32> = None;
    /// let y = Some("foo");
    /// assert_eq!(x.and(y), None);
    ///
    /// let x = Some(2);
    /// let y = Some("foo");
    /// assert_eq!(x.and(y), Some("foo"));
    ///
    /// let x: Option<u32> = None;
    /// let y: Option<&str> = None;
    /// assert_eq!(x.and(y), None);
    /// ```
    #[inline]
    fn and(self, other: Self) -> Self {
        if self.is_some() { other } else { Self::None }
    }

    /// Returns [`Self::None`] if the option is [`Self::None`], otherwise calls `predicate`
    /// with the wrapped value and returns:
    ///
    /// - "Some" if `predicate` returns `true` (where `t` is the wrapped
    ///   value), and
    /// - [`Self::None`] if `predicate` returns `false`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// fn has_blue (color: &Color) -> bool {
    ///     match color {
    ///         Color::Rgb(r, g, b) => *b > 0,
    ///         _ => false,
    ///     }
    /// }
    ///
    /// assert_eq!(Color::None.filter(has_blue), None);
    /// assert_eq!(Color::Rgb(0, 0, 0).filter(has_blue), None);
    /// assert_eq!(Color::Rgb(0, 0, 255).filter(has_blue), Color::Rgb(4, 6, 8));
    /// ```
    #[inline]
    fn filter(self, pred: impl FnOnce(&Self) -> bool) -> Self {
        if self.is_some() && pred(&self) { self } else { Self::None }
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
    /// let mut x = Color::None;
    ///
    /// {
    ///     let y: &mut Color = x.get_or_insert(Color::Red);
    ///     assert_eq!(y, &Color::Red);
    ///
    ///     *y = Color::Blue;
    /// }
    ///
    /// assert_eq!(x, Color::Blue);
    /// ```
    #[inline]
    fn get_or_insert(&mut self, value: Self) -> &mut Self {
        if self.is_none() { *self = value; }
        self
    }

    /// Inserts a value computed from `f` into [`Self`] if it is [`Self::None`],
    /// then returns a mutable reference to the contained value.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut x = Color::None;
    ///
    /// {
    ///     let y: &mut Color = x.get_or_insert_with(|| Color::Red);
    ///     assert_eq!(y, &Color::Red);
    ///
    ///     *y = Color::Blue;
    /// }
    ///
    /// assert_eq!(x, Color::Blue);
    /// ```
    #[inline]
    fn get_or_insert_with(&mut self, f: impl FnOnce() -> Self) -> &mut Self {
        if self.is_none() { let _ = core::mem::replace(self, f()); }
        self
    }

    /// Takes the value out of the option, leaving a [`Self::None`] in its place.
    ///
    /// # Examples
    ///
    /// ```
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
    /// leaving a "Some"" in its place without deinitializing either one.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut x = Some(2);
    /// let old = x.replace(5);
    /// assert_eq!(x, Some(5));
    /// assert_eq!(old, Some(2));
    ///
    /// let mut x = None;
    /// let old = x.replace(3);
    /// assert_eq!(x, Some(3));
    /// assert_eq!(old, None);
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
    /// use etwa::Etwa;
    ///
    /// enum Foo {
    ///     None,
    ///     Something,
    /// }
    ///
    /// let x: Foo = Foo::Something;
    /// assert_eq!(x.etwa(), Some(Foo::Something));
    ///
    /// let x: Foo = Foo::None;
    /// assert_eq!(x.etwa(), None);
    /// ```
    #[inline]
    fn etwa(self) -> Option<Self> {
        if self.is_some() { Some(self) } else { None }
    }

    /// Converts [`Option<Self>`] to [`Self`].
    ///
    /// # Examples
    ///
    /// ```
    /// use etwa::Etwa;
    ///
    /// enum Foo {
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

impl<T> Etwa for Option<T> {
    #[allow(non_upper_case_globals)]
    const None: Self = None;

    #[inline]
    fn is_none(&self) -> bool {
        Option::is_none(self)
    }
}
