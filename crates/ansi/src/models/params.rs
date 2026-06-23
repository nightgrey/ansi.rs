use crate::parser::State;
use std::borrow::{Borrow, BorrowMut};
use std::fmt;
use std::fmt::{Debug, Display};
use std::iter::FusedIterator;
use std::marker::Destruct;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use thiserror::Error;

const _: () = assert!(size_of::<Param>() == 4);

/// Ergonomic, pattern-matchable view of a packed parameter.
#[derive(Copy, Eq, Hash, PartialOrd, Ord)]
#[derive_const(Clone, PartialEq)]
pub enum Param {
    Main(u16),
    Sub(u16),
}

const impl Param {
    pub const None: Self = Self::Main(0);

    pub fn value(self) -> u16 {
        match self {
            Self::Main(value) | Self::Sub(value) => value,
        }
    }

    #[inline]
    pub fn is_main(self) -> bool {
        matches!(self, Self::Main(_))
    }

    #[inline]
    pub fn is_sub(self) -> bool {
        matches!(self, Self::Sub(_))
    }

    #[inline]
    pub fn separator(&self) -> &'static str {
        if self.is_sub() { ":" } else { ";" }
    }
}
const impl Default for Param {
    #[inline]
    fn default() -> Self {
        Self::Main(0)
    }
}
const impl AsRef<u16> for Param {
    #[inline]
    fn as_ref(&self) -> &u16 {
        match self {
            Self::Main(value) | Self::Sub(value) => value,
        }
    }
}

const impl AsMut<u16> for Param {
    #[inline]
    fn as_mut(&mut self) -> &mut u16 {
        match self {
            Self::Main(value) | Self::Sub(value) => value,
        }
    }
}

const impl Borrow<u16> for Param {
    #[inline]
    fn borrow(&self) -> &u16 {
        self.as_ref()
    }
}

const impl BorrowMut<u16> for Param {
    #[inline]
    fn borrow_mut(&mut self) -> &mut u16 {
        self.as_mut()
    }
}

const impl PartialEq<u16> for Param {
    #[inline]
    fn eq(&self, other: &u16) -> bool {
        PartialEq::eq(self.as_ref(), other)
    }
}

const impl PartialEq<Param> for u16 {
    #[inline]
    fn eq(&self, other: &Param) -> bool {
        PartialEq::eq(self, other.as_ref())
    }
}

const impl PartialEq<&Param> for Param {
    #[inline]
    fn eq(&self, other: &&Param) -> bool {
        PartialEq::eq(self.as_ref(), other.as_ref())
    }
}

const impl PartialEq<Param> for &Param {
    #[inline]
    fn eq(&self, other: &Param) -> bool {
        PartialEq::eq(self.as_ref(), other.as_ref())
    }
}

const impl PartialOrd<u16> for Param {
    #[inline]
    fn partial_cmp(&self, other: &u16) -> Option<std::cmp::Ordering> {
        PartialOrd::partial_cmp(self.as_ref(), other)
    }
}

const impl PartialOrd<Param> for u16 {
    #[inline]
    fn partial_cmp(&self, other: &Param) -> Option<std::cmp::Ordering> {
        PartialOrd::partial_cmp(other.as_ref(), self)
    }
}

impl Display for Param {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(self.as_ref(), f)
    }
}

impl Debug for Param {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple(if self.is_main() {
            "Param::Main"
        } else {
            "Param::Sub"
        })
        .field(self.as_ref())
        .finish()
    }
}

macro_rules! impl_from {
    ($($ty:ty),*) => {
        $(
            const impl From<$ty> for Param {
                #[inline]
                fn from(value: $ty) -> Self {
                    Self::Main(value as u16)
                }
            }
        )*
    };
}

impl_from!(i8, i16, i32, i64, i128, isize, u8, u32, u64, u128, usize);

const impl From<Param> for u16 {
    #[inline]
    fn from(value: Param) -> Self {
        value.value()
    }
}

// ============================================================================
// Params — the borrowed, unsized counterpart to `Parameters`.
//
// `Params : Parameters  ::  str : String  ::  [T] : Vec<T>`
//
// It is a thin `#[repr(transparent)]` newtype over `[Param]`, so every
// `&Params` is just a fat pointer to a `[Param]` with the same length
// metadata. All the *logic* that only needs to read a sequence of params
// (display, grouping, …) lives here; `Parameters` re-exposes it for free
// through `Deref`, exactly like `String` borrows `str`'s methods.
// ============================================================================

/// Borrowed view over a sequence of [`Param`]s.
///
/// This is the unsized, slice-like counterpart to [`Parameters`]. Obtain one
/// by dereferencing a [`Parameters`], via [`Params::new`], or by parsing into a
/// [`Box<Params>`].
#[derive(PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Params(pub [Param]);

const impl Params {
    /// Wraps a `&[Param]` as a `&Params` without copying.
    #[inline]
    pub fn new(slice: &[Param]) -> &Params {
        // SAFETY: `Params` is `#[repr(transparent)]` over `[Param]`, so the two
        // share an identical layout *and* pointer metadata (the slice length).
        // The cast preserves the fat-pointer's length; only the nominal type
        // changes.
        unsafe { &*(slice as *const [Param] as *const Params) }
    }

    /// Wraps a `&mut [Param]` as a `&mut Params` without copying.
    #[inline]
    pub fn new_mut(slice: &mut [Param]) -> &mut Params {
        // SAFETY: see `Params::new`; transparent layout makes this sound.
        unsafe { &mut *(slice as *mut [Param] as *mut Params) }
    }

    #[inline]
    pub fn as_slice(&self) -> &[Param] {
        &self.0
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [Param] {
        &mut self.0
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Params {
    #[inline]
    pub const fn first(&self) -> Option<&u16> {
        self.0.first().map(Param::as_ref)
    }

    #[inline]
    pub const fn last(&self) -> Option<&u16> {
        self.0.last().map(Param::as_ref)
    }

    #[inline]
    pub const fn get(&self, index: usize) -> Option<&u16> {
        self.0.get(index).map(Param::as_ref)
    }

    /// Iterates over logical main-parameter groups.
    ///
    /// For well-formed sequences, each group is:
    ///
    /// ```text
    /// main sub*
    /// ```
    ///
    /// For example, `16:5:10;2;9;3:4` yields four groups:
    ///
    /// ```text
    /// 16 with [5, 10]
    /// 2  with []
    /// 9  with []
    /// 3  with [4]
    /// ```
    #[inline]
    pub fn groups(&self) -> Groups<'_> {
        Groups(self.0.iter())
    }

    #[inline]
    pub fn values(&self) -> Values<'_> {
        Values(self.0.iter())
    }
}

const impl Deref for Params {
    type Target = [Param];

    #[inline]
    fn deref(&self) -> &[Param] {
        &self.0
    }
}

const impl DerefMut for Params {
    #[inline]
    fn deref_mut(&mut self) -> &mut [Param] {
        &mut self.0
    }
}

const impl AsRef<[Param]> for Params {
    #[inline]
    fn as_ref(&self) -> &[Param] {
        &self.0
    }
}

const impl AsMut<[Param]> for Params {
    #[inline]
    fn as_mut(&mut self) -> &mut [Param] {
        &mut self.0
    }
}

const impl<'a> From<&'a [Param]> for &'a Params {
    #[inline]
    fn from(slice: &'a [Param]) -> Self {
        Params::new(slice)
    }
}

const impl<'a> From<&'a mut [Param]> for &'a mut Params {
    #[inline]
    fn from(slice: &'a mut [Param]) -> Self {
        Params::new_mut(slice)
    }
}

const impl<'a> Default for &'a Params {
    #[inline]
    fn default() -> Self {
        Params::new(&[])
    }
}

impl ToOwned for Params {
    type Owned = Parameters;

    #[inline]
    fn to_owned(&self) -> Parameters {
        Parameters(self.0.to_vec())
    }
}

impl From<&Params> for Box<Params> {
    #[inline]
    fn from(params: &Params) -> Self {
        let boxed: Box<[Param]> = Box::from(&params.0);
        // SAFETY: transparent layout (see `Params::new`).
        unsafe { Box::from_raw(Box::into_raw(boxed) as *mut Params) }
    }
}

impl From<Box<Params>> for Parameters {
    #[inline]
    fn from(boxed: Box<Params>) -> Self {
        // SAFETY: transparent layout; recover the `Box<[Param]>` we started from.
        let slice: Box<[Param]> = unsafe { Box::from_raw(Box::into_raw(boxed) as *mut [Param]) };
        Parameters(slice.into_vec())
    }
}

const impl PartialEq<[Param]> for Params {
    #[inline]
    fn eq(&self, other: &[Param]) -> bool {
        PartialEq::eq(&self.0, other)
    }
}

const impl PartialEq<Params> for [Param] {
    #[inline]
    fn eq(&self, other: &Params) -> bool {
        PartialEq::eq(self, &other.0)
    }
}

const impl PartialEq<Parameters> for Params {
    #[inline]
    fn eq(&self, other: &Parameters) -> bool {
        PartialEq::eq(&self.0, other.0.as_slice())
    }
}

const impl PartialEq<Params> for Parameters {
    #[inline]
    fn eq(&self, other: &Params) -> bool {
        PartialEq::eq(self.0.as_slice(), &other.0)
    }
}

impl Display for Params {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, param) in self.0.iter().enumerate() {
            if i > 0 {
                f.write_str(param.separator())?;
            }
            write!(f, "{}", param.value())?;
        }
        Ok(())
    }
}

impl Debug for Params {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.0.iter()).finish()
    }
}

impl<'a> IntoIterator for &'a Params {
    type Item = &'a Param;
    type IntoIter = std::slice::Iter<'a, Param>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut Params {
    type Item = &'a mut Param;
    type IntoIter = std::slice::IterMut<'a, Param>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl FromStr for Box<Params> {
    type Err = ParseParamsError;

    /// Parses directly into a heap-allocated [`Box<Params>`] — the owned-but-
    /// not-growable form, analogous to `Box<str>`.
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.parse::<Parameters>()?.into_boxed_params())
    }
}

/// Owned, growable parameter storage.
///
/// Example logical sequence:
///
/// ```text
/// [Main(16), Sub(5), Sub(10), Main(2), Main(9), Main(3), Sub(4)]
/// ```
///
/// Represents:
///
/// ```text
/// 16:5:10;2;9;3:4
/// ```
#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct Parameters(Vec<Param>);

impl Parameters {
    #[inline]
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    #[inline]
    pub const fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }

    #[inline]
    pub const fn from_vec(vec: Vec<Param>) -> Self {
        Self(vec)
    }

    /// Borrows as the unsized [`Params`] view.
    #[inline]
    pub const fn as_params(&self) -> &Params {
        Params::new(self.0.as_slice())
    }

    /// Mutably borrows as the unsized [`Params`] view.
    #[inline]
    pub const fn as_mut_params(&mut self) -> &mut Params {
        Params::new_mut(self.0.as_mut_slice())
    }

    /// Consumes into a fixed-size [`Box<Params>`] (no spare capacity).
    #[inline]
    pub fn into_boxed_params(self) -> Box<Params> {
        let boxed: Box<[Param]> = self.0.into_boxed_slice();
        // SAFETY: `Params` is transparent over `[Param]`.
        unsafe { Box::from_raw(Box::into_raw(boxed) as *mut Params) }
    }

    #[inline]
    pub fn into_vec(self) -> Vec<Param> {
        self.0
    }

    #[inline]
    pub const fn as_vec(&self) -> &Vec<Param> {
        &self.0
    }

    #[inline]
    pub const fn as_mut_vec(&mut self) -> &mut Vec<Param> {
        &mut self.0
    }

    #[inline]
    pub const fn get(&self, index: usize) -> Option<&u16> {
        self.0.get(index).map(Param::as_ref)
    }

    #[inline]
    pub fn push(&mut self, param: Param) {
        self.0.push(param);
    }

    #[inline]
    pub fn pop(&mut self) -> Option<Param> {
        self.0.pop()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    #[inline]
    pub fn truncate(&mut self, len: usize) {
        self.0.truncate(len);
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }

    #[inline]
    pub fn push_many<I, P>(&mut self, iter: I)
    where
        I: IntoIterator<Item = P>,
        P: Into<Param>,
    {
        self.0.extend(iter.into_iter().map(Into::into));
    }
}

const impl Deref for Parameters {
    type Target = Params;

    #[inline]
    fn deref(&self) -> &Params {
        self.as_params()
    }
}

const impl DerefMut for Parameters {
    #[inline]
    fn deref_mut(&mut self) -> &mut Params {
        self.as_mut_params()
    }
}

const impl AsRef<Params> for Parameters {
    #[inline]
    fn as_ref(&self) -> &Params {
        self.as_params()
    }
}

const impl AsRef<[Param]> for Parameters {
    #[inline]
    fn as_ref(&self) -> &[Param] {
        self.0.as_slice()
    }
}

const impl AsMut<[Param]> for Parameters {
    #[inline]
    fn as_mut(&mut self) -> &mut [Param] {
        self.0.as_mut_slice()
    }
}

const impl Borrow<Params> for Parameters {
    #[inline]
    fn borrow(&self) -> &Params {
        self.as_params()
    }
}

const impl BorrowMut<Params> for Parameters {
    #[inline]
    fn borrow_mut(&mut self) -> &mut Params {
        self.as_mut_params()
    }
}

impl Display for Parameters {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&**self, f)
    }
}

impl Debug for Parameters {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&**self, f)
    }
}

impl Extend<Param> for Parameters {
    #[inline]
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = Param>,
    {
        self.0.extend(iter);
    }
}

impl Extend<u16> for Parameters {
    #[inline]
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = u16>,
    {
        self.0.extend(iter.into_iter().map(Param::Main));
    }
}

impl FromIterator<Param> for Parameters {
    #[inline]
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Param>,
    {
        Self(iter.into_iter().collect())
    }
}

impl FromIterator<u16> for Parameters {
    #[inline]
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = u16>,
    {
        Self(iter.into_iter().map(Param::Main).collect())
    }
}

const impl From<Vec<Param>> for Parameters {
    #[inline]
    fn from(value: Vec<Param>) -> Self {
        Self(value)
    }
}

const impl From<Parameters> for Vec<Param>
where
    Parameters: [const] Destruct,
{
    #[inline]
    fn from(value: Parameters) -> Self {
        value.0
    }
}

impl IntoIterator for Parameters {
    type Item = Param;
    type IntoIter = std::vec::IntoIter<Param>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Parameters {
    type Item = &'a Param;
    type IntoIter = std::slice::Iter<'a, Param>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut Parameters {
    type Item = &'a mut Param;
    type IntoIter = std::slice::IterMut<'a, Param>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

// ============================================================================
// Iteration
// ============================================================================

#[derive(Clone)]
pub struct Groups<'a>(std::slice::Iter<'a, Param>);

impl Groups<'_> {
    #[inline]
    pub fn as_slice(&self) -> &[Param] {
        self.0.as_slice()
    }
}

impl<'a> Iterator for Groups<'a> {
    type Item = Group<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let slice = self.0.as_slice();
        if slice.is_empty() {
            return None;
        }

        let mut to = 1;
        self.0.next();

        while to < slice.len() && slice[to].is_sub() {
            to += 1;
            self.0.next();
        }

        Some(Group(slice[..to].iter()))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.0.len();
        // Each group consumes ≥1 element, so at most `remaining` groups; at
        // least 1 if anything is left. NOT exact — hence no `ExactSizeIterator`.
        (usize::from(remaining > 0), Some(remaining))
    }
}

impl FusedIterator for Groups<'_> {}

#[derive(Clone)]
pub struct Group<'a>(std::slice::Iter<'a, Param>);

impl Group<'_> {
    #[inline]
    pub fn as_slice(&self) -> &[Param] {
        self.0.as_slice()
    }
}

impl<'a> Iterator for Group<'a> {
    type Item = &'a Param;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl AsRef<[Param]> for Group<'_> {
    #[inline]
    fn as_ref(&self) -> &[Param] {
        self.as_slice()
    }
}

impl DoubleEndedIterator for Group<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back()
    }
}

impl FusedIterator for Group<'_> {}
// Valid here: `Group` is 1:1 with `slice::Iter` and delegates `size_hint`.
impl ExactSizeIterator for Group<'_> {}

#[derive(Clone, Debug)]
pub struct Values<'a>(std::slice::Iter<'a, Param>);

impl Values<'_> {
    #[inline]
    pub fn as_slice(&self) -> &[Param] {
        self.0.as_slice()
    }
}

impl<'a> Iterator for Values<'a> {
    type Item = u16;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|param| param.value())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl DoubleEndedIterator for Values<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<u16> {
        self.0.next_back().map(|param| param.value())
    }
}

impl ExactSizeIterator for Values<'_> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl FusedIterator for Values<'_> {}

// ============================================================================
// Parsing
// ============================================================================

/// Error returned by [`Parameters::from_str`] / [`<Box<Params>>::from_str`].
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum ParseParamsError {
    #[error("invalid byte at index {0}")]
    InvalidByte(usize),
    #[error("parameter value overflow at index {0}")]
    Overflow(usize),
}

impl FromStr for Parameters {
    type Err = ParseParamsError;

    /// Parses an ANSI-like parameter string.
    ///
    /// Examples:
    ///
    /// ```text
    /// 16:5:10;2;9;3:4
    /// ```
    ///
    /// Empty fields are parsed as zero:
    ///
    /// ```text
    /// "1;;2" => Main(1), Main(0), Main(2)
    /// "1::2" => Main(1), Sub(0), Sub(2)
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(Self::new());
        }

        let bytes = s.as_bytes();
        let mut params = Self::with_capacity(estimate_param_count(s));

        let mut state = State::CsiEntry;
        let mut i = 0;
        let mut current: Option<u16> = None;
        let mut in_group = false;

        while i < bytes.len() {
            let byte = bytes[i];
            let (new_state, _action) = state.transition(byte);

            if new_state != State::CsiParam {
                return Err(ParseParamsError::InvalidByte(i));
            }

            match byte {
                b'0'..=b'9' => {
                    let digit = (byte - b'0') as u16;
                    match current
                        .unwrap_or(0)
                        .checked_mul(10)
                        .and_then(|x| x.checked_add(digit))
                    {
                        Some(next) => current = Some(next),
                        None => return Err(ParseParamsError::Overflow(i)),
                    }
                }
                b':' => {
                    let value = current.take().unwrap_or(0);
                    params.push(if in_group {
                        Param::Sub(value)
                    } else {
                        Param::Main(value)
                    });
                    in_group = true;
                }
                b';' => {
                    match current.take() {
                        Some(value) => {
                            params.push(if in_group {
                                Param::Sub(value)
                            } else {
                                Param::Main(value)
                            });
                        }
                        // Trailing empty sub (`1:;`) defaults to 0 within the group.
                        None if in_group => params.push(Param::Sub(0)),
                        // Empty main (`;`, `1;;`) opens a fresh `[0]` group, so it
                        // is a *Main*, not a Sub.
                        None => params.push(Param::Main(0)),
                    }
                    in_group = false;
                }
                _ => {}
            }

            i += 1;
            state = new_state;
        }

        if let Some(value) = current {
            params.push(if in_group {
                Param::Sub(value)
            } else {
                Param::Main(value)
            });
        }

        Ok(params)
    }
}

#[inline]
fn estimate_param_count(s: &str) -> usize {
    1 + s
        .as_bytes()
        .iter()
        .filter(|&&byte| byte == b';' || byte == b':')
        .count()
}

// ----------------------------------------------------------------------------
// Macro
// ----------------------------------------------------------------------------

#[macro_export]
macro_rules! params {
    // Emptyto_owned
    () => {
        {
                $crate::Parameters::new()
        }
    };
    // [_]
    ($($elem:literal),+ $(,)?) => (
        {
            $crate::Parameters::from_iter([$($elem),*])
        }
    );
    // [[_]]
    ($([$($elem:literal),* $(,)?]),+ $(,)?) => (
        {
            let mut params = $crate::Parameters::new();
            $(
                params.push_many([$($elem),*]);
            )+
            params
        }
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn param() {
        let main = Param::Main(16);
        let sub = Param::Sub(5);

        assert_eq!(main, 16);
        assert!(main.is_main());
        assert!(!main.is_sub());
        assert_eq!(main, Param::Main(16));

        assert_eq!(sub, 5);
        assert!(sub.is_sub());
        assert!(!sub.is_main());
        assert_eq!(sub, Param::Sub(5));
    }

    #[test]
    fn params_view_is_transparent() {
        let owned: Parameters = "16:5:10;2;9;3:4".parse().unwrap();
        let view: &Params = &owned; // Deref<Target = Params>

        assert_eq!(view.len(), owned.as_vec().len());
        assert_eq!(view.to_string(), "16:5:10;2;9;3:4");
        assert_eq!(view, &*owned);
        assert_eq!(owned, *view); // cross-type PartialEq
    }

    #[test]
    fn params_new_roundtrip() {
        let raw = [Param::Main(1), Param::Sub(2), Param::Main(3)];
        let view = Params::new(&raw);
        assert_eq!(view.as_slice(), &raw);
        assert_eq!(view.to_string(), "1:2;3");

        // ToOwned bridge
        let owned: Parameters = view.to_owned();
        assert_eq!(&owned, view);
    }

    #[test]
    fn boxed_params_roundtrip() {
        let boxed: Box<Params> = "1:2;3".parse().unwrap();
        assert_eq!(boxed.to_string(), "1:2;3");

        let owned = Parameters::from(boxed);
        assert_eq!(owned.to_string(), "1:2;3");

        let reboxed = owned.into_boxed_params();
        assert_eq!(reboxed.len(), 3);
    }

    #[test]
    fn display_roundtrip() {
        let mut params = Parameters::new();
        params.push(Param::Main(16));
        params.push(Param::Sub(5));
        params.push(Param::Sub(10));
        params.push(Param::Main(2));
        params.push(Param::Main(9));
        params.push(Param::Main(3));
        params.push(Param::Sub(4));

        let str = params.to_string();
        assert_eq!(str, "16:5:10;2;9;3:4");
        assert_eq!(params, Parameters::from_str(&str).unwrap());
    }

    #[test]
    fn groups() {
        let params: Parameters = "16:5:10;2;9;3:4".parse().unwrap();

        // grouping now lives on the borrowed view; reachable via Deref
        let groups: Vec<_> = params
            .groups()
            .map(|group| group.map(|p| p.value()).collect::<Vec<_>>())
            .collect();

        assert_eq!(groups, vec![vec![16, 5, 10], vec![2], vec![9], vec![3, 4]]);
    }

    #[test]
    fn parse_empty_fields() {
        let params: Parameters = "1;;2;3::4;5".parse().unwrap();
        let tokens: Vec<_> = params.iter().collect();

        assert_eq!(
            tokens,
            vec![
                &Param::Main(1),
                &Param::Main(0),
                &Param::Main(2),
                &Param::Main(3),
                &Param::Sub(0),
                &Param::Sub(4),
                &Param::Main(5),
            ]
        );
    }

    #[test]
    fn invalid() {
        let err = "1;A".parse::<Parameters>().unwrap_err();
        assert!(matches!(err, ParseParamsError::InvalidByte { .. }));
    }

    #[test]
    fn overflow() {
        let err = "999999".parse::<Parameters>().unwrap_err();
        assert!(matches!(err, ParseParamsError::Overflow { .. }));
    }
}
