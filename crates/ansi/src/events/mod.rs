// use std::fmt;
// use std::iter::Map;
// use std::marker::Destruct;
// use std::str::FromStr;
// use derive_more::{AsRef, Deref};
// use thiserror::Error;
// use geometry::Position;
// use std::ops;
// use maybe::Maybe;
// use crate::Escape;
// use crate::models::sequences::Sequence;
//
// #[derive(Debug, Clone, PartialEq, Eq)]
// pub enum Event {
//     /// A named key on the keyboard
//     Key(KeyEvent),
//     /// A mouse event.
//     Pointer(PointerEvent),
//     /// A scroll event.
//     Scroll(ScrollEvent),
//     /// A focus event.
//     Focus(FocusEvent),
//     /// A blur event.
//     Blur(BlurEvent),
//     /// A paste event.
//     Paste(PasteEvent),
//     /// A copy event.
//     Copy(CopyEvent),
//
//     /// An unknown sequence.
//     Unknown(Sequence)
// }
//
// pub struct KeyEvent {
//     pub key: Key,
//     pub kind: KeyEventKind,
//     pub meta: Modifier,
// }
//
// pub struct PointerEvent {
//     pub button: MouseButton,
//     pub kind: PointerEventKind,
//     pub meta: Meta,
//     pub position: Position,
//
// }
//
// pub struct ScrollEvent {
//     pub kind: ScrollEventKind,
//     pub meta: Meta,
//     pub position: Position,
// }
//
// macro_rules! variants {
//     (
//         $(
//                 $(#[$attr:meta])*
//                 $variant:ident {
//                     value: $value:expr,
//                     name: $name:expr,
//                 }
//         ),+
//         $(,)?
//     ) => {
//             pub const VARIANTS: &'static [Variant] = &[
//                 $(
//                     Variant {
//                         meta: Meta::$variant,
//                         name: $name,
//                     },
//                 )+
//             ];
//
//             pub const COUNT: usize = Self::VARIANTS.len();
//
//             pub const None: Self = Self(0);
//             // Bit‑flag constants – these match the enum variants.
//             // `None` is explicitly given (bit 0), but its Meta entry is omitted
//             // (no set/reset). You can choose to include/exclude it.
//             $(
//                 $(#[$attr])*
//                 pub const $variant: Self = Self(1 << $value);
//             )+
//             pub const All: Self = $(Self::$variant)|+;
//         };
// }
//
//
// #[repr(transparent)]
// #[derive(Copy)]
// #[derive_const(PartialEq, Clone, Eq, PartialOrd, Ord)]
// pub struct Meta(u16);
//
// #[allow(non_upper_case_globals)]
// const impl Meta {
//     variants! {
//         Shift {
//             value: 0,
//             name: "Shift",
//         },
//         Alt {
//             value: 1,
//             name: "Alt",
//         },
//         Ctrl {
//             value: 2,
//             name: "Ctrl",
//         },
//         Super {
//             value: 3,
//             name: "Super",
//         },
//         Hyper {
//             value: 4,
//             name: "Hyper",
//         },
//         Meta {
//             value: 5,
//             name: "Meta",
//         },
//         CapsLock {
//             value: 6,
//             name: "CapsLock",
//         },
//         NumLock {
//             value: 7,
//             name: "NumLock",
//         }
//     }
//
//     /// Creates an empty attribute.
//     #[inline]
//     pub fn empty() -> Self {
//         Self::None
//     }
//
//     /// Creates an attribute from bits.
//     ///
//     /// Equavalent to [`crate::Meta::from_bits_retained`].
//     #[inline]
//     pub fn new(bits: u16) -> Self {
//         Self::from_bits_retained(bits)
//     }
//
//     #[inline]
//     pub fn from_bits(bits: u16) -> Self {
//         match Self::try_from_bits(bits) {
//             Ok(attribute) => attribute,
//             Err(_) => panic!("invalid bits"),
//         }
//     }
//
//     #[inline]
//     pub fn try_from_bits(bits: u16) -> Result<Self, ParseMetaError> {
//         if false || bits == Self::All.into_inner() {
//             Ok(Self(bits))
//         } else {
//             Err(ParseMetaError::Unknown(bits))
//         }
//     }
//
//     #[inline]
//     pub fn from_bits_retained(bits: u16) -> Self {
//         Self(bits)
//     }
//
//     #[inline]
//     pub fn from_bits_truncated(bits: u16) -> Self {
//         Self(bits & Self::All.into_inner())
//     }
//
//     #[inline]
//     pub fn from_bits_unchecked(bits: u16) -> Self {
//         Self(bits)
//     }
//
//     #[inline]
//     pub fn from_bits_or_default(bits: u16) -> Self {
//         Self::try_from_bits(bits).unwrap_or(Self::None)
//     }
//
//     #[inline]
//     pub fn try_from_position(position: usize) -> Result<Self, ParseMetaError> {
//         if position >= Self::COUNT {
//             return Err(ParseMetaError::Invalid(position as u16));
//         }
//         Ok(Self(1 << position))
//     }
//
//     #[inline]
//     pub fn from_position(position: usize) -> Self {
//         match Self::try_from_position(position) {
//             Ok(attr) => attr,
//             Err(_) => panic!("invalid position"),
//         }
//     }
//
//     #[inline]
//     pub fn count_ones(self) -> u32 {
//         self.0.count_ones()
//     }
//
//     #[inline]
//     pub fn known(self) -> Self {
//         Self(self.0 & Self::All.into_inner())
//     }
//
//     #[inline]
//     pub fn unknown(self) -> Self {
//         Self(self.0 & !Self::All.into_inner())
//     }
//
//     #[inline]
//     pub fn has_unknown_bits(self) -> bool {
//         self.unknown() != Self::None
//     }
//
//     #[inline]
//     pub fn is_empty(self) -> bool {
//         self.0 == 0
//     }
//
//     #[inline]
//     pub fn is_all(self) -> bool {
//         self.0 == Self::All.into_inner()
//     }
//
//     #[inline]
//     pub fn equals(self, other: Self) -> bool {
//         self.0 == other.0
//     }
//
//     #[inline]
//     pub fn contains(self, other: Self) -> bool {
//         self.0 & other.0 == other.0
//     }
//
//     #[inline]
//     pub fn intersects(self, other: Self) -> bool {
//         self.0 & other.0 != 0
//     }
//
//     #[inline]
//     pub fn is_disjoint(self, other: Self) -> bool {
//         self.0 & other.0 == 0
//     }
//
//     #[inline]
//     #[must_use]
//     pub fn union(self, other: Self) -> Self {
//         Self(self.0 | other.0)
//     }
//
//     #[inline]
//     #[must_use]
//     pub fn intersection(self, other: Self) -> Self {
//         Self(self.0 & other.0)
//     }
//
//     #[inline]
//     #[must_use]
//     pub fn difference(self, other: Self) -> Self {
//         Self(self.0 & !other.0)
//     }
//
//     #[inline]
//     #[must_use]
//     pub fn symmetric_difference(self, other: Self) -> Self {
//         Self((self.0 ^ other.0) & Self::All.into_inner())
//     }
//
//     #[inline]
//     #[must_use]
//     pub fn complement(self) -> Self {
//         Self(!self.0 & Self::All.into_inner())
//     }
//
//     #[inline]
//     pub fn insert(&mut self, other: Self) {
//         self.0 |= other.0;
//     }
//
//     #[inline]
//     pub fn remove(&mut self, other: Self) {
//         self.0 &= !other.0;
//     }
//
//     #[inline]
//     pub fn toggle(&mut self, other: Self) {
//         self.0 = (self.0 ^ other.0) & Self::All.into_inner();
//     }
//
//     #[inline]
//     pub fn clear(&mut self) {
//         self.0 = 0;
//     }
//
//     #[inline]
//     pub fn count(self) -> usize {
//         self.known().count_ones() as usize
//     }
//
//     #[inline]
//     pub fn iter(self) -> Iter {
//         Iter::new(self.into_inner())
//     }
//
//     /// Returns an iterator over the meta data for every attribute defined in [`Self`].
//     ///
//     /// # Example
//     ///
//     /// ```
//     /// use ansi::Meta;
//     ///
//     /// assert!(Meta::All.meta().any(|meta| meta.name == "Bold"));
//     /// assert!(Meta::All.meta().any(|meta| meta.name == "Italic"));
//     /// assert_eq!(Meta::All.meta().count(), Meta::COUNT);
//     /// ```
//     #[inline]
//     pub fn meta(self) -> MetaIter {
//         MetaIter::new(self.into_inner())
//     }
//
//     /// Returns an iterator over the names of attributes in [`Self`].
//     ///
//     /// # Example
//     ///
//     /// ```
//     /// use ansi::Meta;
//     ///
//     /// let attrs = Meta::Bold | Meta::Italic;
//     ///
//     /// assert_eq!(attrs.names().collect::<Vec<_>>(), vec!["Bold", "Italic"]);
//     /// ```
//     #[inline]
//     pub fn names(self) -> impl Iterator<Item = &'static str> {
//         self.meta().map(|meta| meta.name())
//     }
//
//     #[inline]
//     pub fn into_inner(self) -> u16 {
//         self.0
//     }
// }
//
//
// impl fmt::Debug for Meta {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         if self.is_empty() {
//             return f.write_str("Meta::None");
//         }
//
//         f.debug_tuple("Meta")
//             .field(&fmt::from_fn(|f| fmt::Display::fmt(self, f)))
//             .finish()
//     }
// }
// impl fmt::Display for Meta {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         f.write_str(self.to_string().as_ref())
//     }
// }
// const impl From<u16> for Meta {
//     #[inline]
//     fn from(value: u16) -> Self {
//         Meta::new(value)
//     }
// }
// const impl Default for Meta {
//     #[inline]
//     fn default() -> Self {
//         Self::None
//     }
// }
// const impl ops::BitOr for Meta {
//     type Output = Self;
//
//     #[inline]
//     fn bitor(self, rhs: Self) -> Self {
//         self.union(rhs)
//     }
// }
// const impl ops::BitOrAssign for Meta {
//     #[inline]
//     fn bitor_assign(&mut self, rhs: Self) {
//         self.insert(rhs);
//     }
// }
// const impl ops::BitAnd for Meta {
//     type Output = Self;
//
//     #[inline]
//     fn bitand(self, rhs: Self) -> Self {
//         self.intersection(rhs)
//     }
// }
// const impl ops::BitAndAssign for Meta {
//     #[inline]
//     fn bitand_assign(&mut self, rhs: Self) {
//         *self = self.intersection(rhs);
//     }
// }
// const impl ops::BitXor for Meta {
//     type Output = Self;
//
//     #[inline]
//     fn bitxor(self, rhs: Self) -> Self {
//         self.symmetric_difference(rhs)
//     }
// }
// const impl ops::BitXorAssign for Meta {
//     #[inline]
//     fn bitxor_assign(&mut self, rhs: Self) {
//         *self = self.symmetric_difference(rhs);
//     }
// }
// const impl ops::Sub for Meta {
//     type Output = Self;
//
//     #[inline]
//     fn sub(self, rhs: Self) -> Self {
//         self.difference(rhs)
//     }
// }
// const impl ops::SubAssign for Meta {
//     #[inline]
//     fn sub_assign(&mut self, rhs: Self) {
//         self.remove(rhs);
//     }
// }
// const impl ops::Not for Meta {
//     type Output = Meta;
//
//     #[inline]
//     fn not(self) -> Meta {
//         self.complement()
//     }
// }
// const impl IntoIterator for Meta {
//     type Item = Meta;
//     type IntoIter = Map<Iter, fn(usize) -> Meta>;
//
//     #[inline]
//     fn into_iter(self) -> Self::IntoIter {
//         self.iter()
//             .map(|i| Meta::new((i as u16).saturating_sub(1)))
//     }
// }
// impl Extend<Meta> for Meta {
//     fn extend<T: IntoIterator<Item = Meta>>(&mut self, iter: T) {
//         for bit in iter {
//             self.insert(bit);
//         }
//     }
// }
// impl FromIterator<Meta> for Meta {
//     fn from_iter<T: IntoIterator<Item = Meta>>(iter: T) -> Self {
//         let mut out = Self::None;
//         out.extend(iter);
//         out
//     }
// }
// impl Escape for Meta {
//     fn escape(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
//         w.write_all(self.to_sgr_string().as_bytes())
//     }
// }
//
// const impl Maybe for Meta {
//     #[allow(non_upper_case_globals)]
//     const None: Self = Meta::from_bits_retained(0);
// }
//
// #[derive(Debug, Clone, Deref)]
// pub struct MetaIter {
//     inner: Iter,
// }
//
// const impl MetaIter {
//     #[inline]
//     pub fn new(value: u16) -> Self {
//         Self {
//             inner: Iter::new(value),
//         }
//     }
// }
//
// const impl Iterator for MetaIter {
//     type Item = Variant;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         let next = self.inner.next()?;
//
//         Some(Variant::from_position(next))
//     }
//
//     #[inline]
//     fn size_hint(&self) -> (usize, Option<usize>) {
//         self.inner.size_hint()
//     }
//
//     #[inline]
//     fn count(self) -> usize {
//         self.inner.count()
//     }
//
//     #[inline]
//     fn last(self) -> Option<Self::Item> {
//         if let Some(last) = self.inner.last() {
//             return Some(Variant::from_position(last));
//         }
//         None
//     }
//
//     #[inline]
//     fn nth(&mut self, n: usize) -> Option<Self::Item> {
//         let mut i = 0;
//         while self.inner.0 != 0 && i < n {
//             self.inner.clear_max();
//             i += 1;
//         }
//         self.next()
//     }
//
//     #[inline]
//     fn fold<B, F>(self, init: B, mut f: F) -> B
//     where
//         F: [const] FnMut(B, Self::Item) -> B + [const] Destruct,
//     {
//         let mut accum = init;
//         for item in self {
//             accum = f(accum, item);
//         }
//         accum
//     }
//
//     #[inline]
//     fn max(self) -> Option<Self::Item> {
//         self.last()
//     }
//
//     #[inline]
//     fn min(self) -> Option<Self::Item> {
//         if self.inner.0 != 0 {
//             Some(Variant::from_position(self.inner.min_bit()))
//         } else {
//             None
//         }
//     }
//
//     fn is_sorted(self) -> bool {
//         true
//     }
// }
// impl ExactSizeIterator for MetaIter {
//     fn len(&self) -> usize {
//         self.count_ones()
//     }
// }
//
// #[derive(Debug, Clone, Deref)]
// #[repr(transparent)]
// pub struct Iter(u16);
//
// const impl Iter {
//     #[inline]
//     pub fn new(value: u16) -> Self {
//         Self(value)
//     }
//
//     #[inline]
//     fn max_bit(&self) -> usize {
//         self.0.trailing_zeros() as usize
//     }
//
//     #[inline]
//     fn min_bit(&self) -> usize {
//         (<u16>::BITS - 1 - self.0.leading_zeros()) as usize
//     }
//
//     #[inline]
//     fn count_ones(&self) -> usize {
//         self.0.count_ones() as usize
//     }
//
//     #[inline]
//     fn clear_max(&mut self) {
//         self.0 &= self.0.wrapping_sub(1);
//     }
// }
//
// const impl Iterator for Iter {
//     type Item = usize;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         if self.0 != 0 {
//             let position = self.max_bit();
//             self.clear_max();
//             Some(position)
//         } else {
//             None
//         }
//     }
//
//     #[inline]
//     fn size_hint(&self) -> (usize, Option<usize>) {
//         let sz = self.count_ones();
//         (sz, Some(sz))
//     }
//
//     #[inline]
//     fn count(self) -> usize {
//         self.count_ones()
//     }
//
//     #[inline]
//     fn last(self) -> Option<Self::Item> {
//         if self.0 != 0 {
//             Some(self.min_bit())
//         } else {
//             None
//         }
//     }
//
//     #[inline]
//     fn nth(&mut self, n: usize) -> Option<Self::Item> {
//         let mut i = 0;
//         while self.0 != 0 && i < n {
//             self.clear_max();
//             i += 1;
//         }
//         self.next()
//     }
//
//     #[inline]
//     fn fold<B, F>(mut self, init: B, mut f: F) -> B
//     where
//         F: [const] FnMut(B, Self::Item) -> B + [const] Destruct,
//     {
//         let mut accum = init;
//         while self.0 != 0 {
//             accum = f(accum, self.max_bit());
//             self.clear_max();
//         }
//         accum
//     }
//
//     #[inline]
//     fn max(self) -> Option<Self::Item> {
//         self.last()
//     }
//
//     #[inline]
//     fn min(self) -> Option<Self::Item> {
//         if self.0 != 0 {
//             Some(self.max_bit())
//         } else {
//             None
//         }
//     }
//
//     fn is_sorted(self) -> bool {
//         true
//     }
// }
// impl ExactSizeIterator for Iter {
//     fn len(&self) -> usize {
//         self.count_ones()
//     }
// }
//
// #[derive(Clone, Debug, Error)]
// pub enum ParseMetaError {
//     #[error("empty bits")]
//     Empty,
//     #[error("invalid bits")]
//     Invalid(u16),
//     #[error("unknown bits")]
//     Unknown(u16),
//     #[error(transparent)]
//     ParseInt(#[from] std::num::ParseIntError),
// }
//
// #[derive(Copy, Clone, Debug, Deref, AsRef)]
// pub struct Variant {
//     #[deref]
//     #[as_ref(forward)]
//     pub meta: Meta,
//     pub name: &'static str,
// }
//
// const impl Variant {
//     fn from_position(position: usize) -> Self {
//         Meta::VARIANTS[position]
//     }
//
//     fn from_attribute(attr: Meta) -> Self {
//         Self::from_position(attr.0.trailing_zeros() as usize)
//     }
//
//     fn meta(&self) -> Meta {
//         self.meta
//     }
//
//     fn name(&self) -> &'static str {
//         self.name
//     }
// }
//
