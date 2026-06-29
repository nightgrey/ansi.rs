use std::fmt;
use crate::{Grapheme, GraphemesError};

#[derive(Copy, Clone)]
pub struct Entry {
    pub slot: Slot,
    pub start: usize,
    pub len: usize,
    pub end: usize,
}

impl Entry {
    #[inline]
    pub fn index(&self) -> usize {
        self.slot.as_usize()
    }
}


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

    #[inline]
    pub fn into_grapheme(self) -> Grapheme {
        Grapheme::from_bytes_unchecked([self.as_u8(), (self.as_u32() >> 8) as u8, (self.as_u32() >> 16) as u8, Grapheme::EXTENDED_TAG])
    }

    #[inline]
    pub fn value(self) -> u32 {
        self.0
    }

    #[inline]
    pub fn as_u8(self) -> u8 {
        self.value() as _
    }

    #[inline]
    pub fn as_u32(self) -> u32 {
        self.value() as _
    }

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
