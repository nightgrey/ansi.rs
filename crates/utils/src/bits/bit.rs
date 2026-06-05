//! A compact, const-friendly bitflag/set toolkit.
//!
//! Three concepts, three names:
//!
//! * [`Bit`]       — a single flag (one enum variant).
//! * [`Bits`]    — a *set* of flags; the plural of `Bit`.
//! * [`Bit::Repr`] — the unsigned integer that physically stores the bits.
//!
//! Unlike a generic `Bits<B>` newtype, the *set* is a concrete type minted by
//! the [`bits!`] macro **on the user's side** (`struct Attributes(u16)`), so it
//! can carry its own inherent methods, trait impls and documentation. The
//! shared behaviour still lives here: [`Bits`] is a `const trait` whose
//! default method bodies implement the whole set algebra, so the macro only
//! emits the parts that genuinely cannot be generic — the type itself, its
//! `Repr`/`Bit` wiring, and the operator/conversion impls that the orphan rules
//! force to be defined next to the concrete type.
//!
//! Almost every method accepts `impl Into<Self>`, so a bare `Bit`, a set, and a
//! borrowed/owned mix are all interchangeable arguments.

use core::fmt::Debug;
use std::marker::Destruct;
use crate::other::Base;
/// A single flag: one variant of a bit enum.
///
/// Deliberately small. A flag does not need to be a whole boolean algebra —
/// it only needs to know its representation and the universe of valid bits.
/// All set behaviour lives on [`Bits`].
pub const trait Bit: Sized
+ [ const ] Base
+ [ const ] Destruct
+ Copy
+ [ const ] PartialEq
+ [ const ] Eq
+ [ const ] PartialOrd
+ Debug
+ [ const ] Ord
+ [ const ] Into<Self::Repr>
+ 'static
{
    /// Every flag, in declaration order. Drives iteration and counting.
    const LIST: &'static [(Self, &'static str)];
    /// Number of declared flags.
    const COUNT: usize = Self::LIST.len();
}