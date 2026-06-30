//! Internal support types for grapheme storage.
//!
//! These types are used throughout the grapheme subsystem but are kept
//! separate from the public API surfaces of [`Grapheme`] and [`Graphemes`].

use crate::{Grapheme, GraphemesError};
use std::fmt;

/// Metadata for a resolved arena entry.
///
/// Produced by [`Graphemes::try_resolve`] — carries the slot, byte range
/// (`start..end`) within the arena buffer, and the payload length.
#[derive(Copy, Clone)]
pub struct Entry {
    /// The arena slot that this entry occupies.
    pub slot: Slot,
    /// Byte offset of the payload (past the 2-byte length prefix).
    pub start: usize,
    /// Payload length in bytes (without the prefix).
    pub len: usize,
    /// Byte offset immediately past the payload.
    pub end: usize,
}

impl Entry {
    /// The flat byte offset of this entry's slot within the arena buffer.
    ///
    /// This is `slot.as_usize()` — a convenience for indexing into the
    /// arena's backing `Vec<u8>`.
    #[inline]
    pub fn index(&self) -> usize {
        self.slot.as_usize()
    }
}

/// A 24-bit index into a [`Graphemes`] arena.
///
/// Slots are stored in the low 3 bytes of an extended [`Grapheme`] handle,
/// giving an addressable range of 0..=16,777,214 (16 MiB − 1). The 4th byte
/// is the sentinel tag `0x01`. Direct construction is possible but rare —
/// most slots are obtained via [`Graphemes::try_insert`] or
/// [`Grapheme::slot`].
///
/// Slots are comparable with `u32` for convenience in allocation logic.
#[derive(Copy)]
#[derive_const(Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Slot(u32);

const impl Slot {
    const BITS: u32 = 24;

    pub const MIN: Self = Self(0);
    pub const MAX: Self = Self(u32::MAX >> (u32::BITS - Self::BITS));

    #[inline]
    pub fn try_new(value: u32) -> Result<Self, GraphemesError> {
        if value <= Self::MAX {
            Ok(Self(value))
        } else {
            Err(GraphemesError::Invalid)
        }
    }

    /// Create a new slot.
    ///
    /// Panics if the value is too large.
    #[inline]
    pub fn new(value: u32) -> Self {
        assert!(value <= Self::MAX);

        Self(value)
    }

    #[inline]
    pub fn try_from_grapheme(value: Grapheme) -> Result<Self, GraphemesError> {
        if !value.is_extended() {
            return Err(GraphemesError::Invalid);
        }

        let slice = value.as_bytes();
        // The low 3 bytes hold a 24-bit slot, so the value is always <= MAX.
        Ok(Self(u32::from_le_bytes([slice[0], slice[1], slice[2], 0])))
    }

    #[inline]
    pub fn try_from_usize(value: usize) -> Result<Self, GraphemesError> {
        // Validate before narrowing: a bare `as u32` would alias large values
        // down into the valid 24-bit range.
        if value <= Self::MAX.as_usize() {
            Ok(Self(value as u32))
        } else {
            Err(GraphemesError::Invalid)
        }
    }

    /// Pack this slot into an extended [`Grapheme`] handle.
    ///
    /// Writes the 24-bit slot into the low 3 bytes and sets the 4th byte
    /// to the extended sentinel tag (`0x01`).
    #[inline]
    pub fn into_grapheme(self) -> Grapheme {
        Grapheme::from_bytes_unchecked([
            self.as_u8(),
            (self.as_u32() >> 8) as u8,
            (self.as_u32() >> 16) as u8,
            Grapheme::EXTENDED_TAG,
        ])
    }

    /// The raw `u32` value of this slot.
    #[inline]
    pub fn value(self) -> u32 {
        self.0
    }

    /// The low byte of this slot, as a `u8`.
    #[inline]
    pub fn as_u8(self) -> u8 {
        self.value() as _
    }

    /// The slot value widened to `u32`.
    #[inline]
    pub fn as_u32(self) -> u32 {
        self.value() as _
    }

    /// The slot value widened to `usize`.
    ///
    /// This is the canonical form for indexing into the arena's backing
    /// `Vec<u8>`.
    #[inline]
    pub fn as_usize(self) -> usize {
        self.value() as _
    }
}

impl fmt::Debug for Slot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Slot").field(&self.0).finish()
    }
}

const impl PartialEq<u32> for Slot {
    fn eq(&self, other: &u32) -> bool {
        self.0 == *other
    }
}

const impl PartialOrd<u32> for Slot {
    fn partial_cmp(&self, other: &u32) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(other)
    }
}

const impl PartialEq<Slot> for u32 {
    fn eq(&self, other: &Slot) -> bool {
        self == &other.0
    }
}

const impl PartialOrd<Slot> for u32 {
    fn partial_cmp(&self, other: &Slot) -> Option<std::cmp::Ordering> {
        self.partial_cmp(&other.0)
    }
}
