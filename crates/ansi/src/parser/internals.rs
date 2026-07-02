use crate::params::{Param, Parameters, Params};
use derive_more::{Deref, DerefMut};
use std::borrow::Borrow;
use arrayvec::ArrayVec;
use utils::{NestedMut};

const UTF8_CONTINUATION_MASK: u8 = 0b0011_1111;

#[derive(Debug)]
pub struct Utf8 {
    inner: [u8; char::MAX_LEN_UTF8],
    codepoint: u32,
    len: u8,
}

const impl Utf8 {
    pub const REPLACEMENT_CHARACTER: Self = Self {
        inner: [239, 191, 189, 0],
        codepoint: char::REPLACEMENT_CHARACTER as u32,
        len: 3,
    };
    pub const EMPTY: Self = Self {
        inner: [0; char::MAX_LEN_UTF8],
        codepoint: 0,
        len: 0,
    };

    pub fn new() -> Self {
        Self {
            inner: [0; char::MAX_LEN_UTF8],
            codepoint: 0,
            len: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn is_partial(&self) -> bool {
        // If the byte at [len - 1] is 0 for a multi-byte sequence (len >= 2), the last continuation byte hasn't been written yet.
        self.len >= 2 && self.inner[(self.len - 1) as usize] == 0
    }

    #[inline]
    pub fn set_byte_1(&mut self, byte: u8) {
        self.codepoint |= (byte & UTF8_CONTINUATION_MASK) as u32;
        self.set(byte, 1);
    }

    #[inline]
    pub fn set_byte_2_top(&mut self, byte: u8) {
        self.codepoint |= ((byte & 0b0001_1111) as u32) << 6;
        self.set_top(byte, 2);
    }

    #[inline]
    pub fn set_byte_2(&mut self, byte: u8) {
        self.codepoint |= ((byte & UTF8_CONTINUATION_MASK) as u32) << 6;
        self.set(byte, 2);
    }

    #[inline]
    pub fn set_byte_3_top(&mut self, byte: u8) {
        self.codepoint |= ((byte & 0b0000_1111) as u32) << 12;
        self.set_top(byte, 3);
    }

    #[inline]
    pub fn set_byte_3(&mut self, byte: u8) {
        self.codepoint |= ((byte & UTF8_CONTINUATION_MASK) as u32) << 12;
        self.set(byte, 3);
    }

    #[inline]
    pub fn set_byte_4_top(&mut self, byte: u8) {
        self.codepoint |= ((byte & 0b0000_0111) as u32) << 18;
        self.set_top(byte, 4);
    }

    #[inline]
    fn set(&mut self, byte: u8, from: usize) {
        debug_assert!(from <= self.len as usize);
        self.inner[self.len as usize - from] = byte;
    }

    #[inline]
    fn set_top(&mut self, byte: u8, len: usize) {
        debug_assert!(len <= char::MAX_LEN_UTF8);
        self.len = len as u8;
        self.inner[0] = byte;
    }

    #[inline]
    pub fn clear(&mut self) {
        *self = Self::EMPTY;
    }

    #[inline]
    pub fn as_char(&self) -> char {
        unsafe { char::from_u32_unchecked(self.codepoint) }
    }
}

const impl AsRef<[u8]> for Utf8 {
    fn as_ref(&self) -> &[u8] {
        &self.inner[..self.len as usize]
    }
}

const impl Default for Utf8 {
    fn default() -> Self {
        Self::EMPTY
    }
}


/// Accumulator for the numeric parameter string of an ECMA-48 CSI or DCS
/// control sequence.
///
/// A parameter string is a sequence of decimal digits separated by `;` and `:`.
/// The accumulator distinguishes two levels:
///
/// | Separator | Meaning |
/// |-----------|---------|
/// | `;`       | Starts a new *main* parameter. |
/// | `:`       | Starts a new *sub-parameter* inside the current main parameter. |
///
/// The storage is capped at 32 entries, the maximum parameter count defined by
/// ECMA-48. The type derefs to an [`ArrayVec`] of [`Param`] values so the
/// finished list can be inspected directly.
///
/// # Parsing state
///
/// Because the first value of a main parameter and the first value of a
/// sub-parameter look identical until the next separator arrives, the
/// accumulator keeps three pieces of in-flight state:
///
/// * `current` – the decimal value currently being built from ASCII digits.
/// * `in_group` – whether a `:` has been seen since the last `;`, i.e. whether
///   `current` belongs to an active sub-parameter group.
/// * `is_pending` – whether a trailing `;` has opened a fresh main-parameter
///   slot that has not yet received a value.
///
/// # Semantics
///
/// * Empty main parameters default to `0`. Leading or repeated `;` separators
///   therefore create a `[0]` main parameter, and a trailing `;` before the
///   final character (e.g. `CSI 4 ; m`) also emits a final `[0]`.
/// * Empty sub-parameters default to `0`, so `1::3` is parsed as
///   `[[1, 0, 3]]`.
/// * A sequence with no separator at all, such as `CSI m`, leaves the
///   parameter list empty.
/// * Numeric values saturate at [`u16::MAX`]; values and capacity limits are
///   hard caps. The control sequence still dispatches with whatever parameters
///   fit, rather than panicking.
///
/// # Lifecycle
///
/// Feed bytes with [`InternalParameters::push_digit`], then call the
/// appropriate separator handler ([`InternalParameters::push_sub`] for `:`
/// and [`InternalParameters::push_main`] for `;`). When the final character
/// of the sequence is reached, call [`InternalParameters::flush`] to materialize
/// any pending value. The accumulator can be reused with
/// [`InternalParameters::clear`].
#[derive(Deref, DerefMut, Debug, Default, Clone)]
pub struct ParametersAccumulator {
    #[deref]
    #[deref_mut]
    inner: ArrayVec<Param, 32>, // 32 is the maximum capacity for parameters according to ECMA-48.
    current: Option<u16>,
    /// Whether the main parameter currently being built has sub-parameters,
    /// i.e. a `:` separator has been seen since the last `;`. This is the one
    /// bit that distinguishes `1:2` (sub) from `1;2` (new param) — `inner`
    /// alone can't, since both leave `[1]` behind after the first value.
    in_group: bool,
    /// Whether a `;` has opened a fresh main-parameter slot that has not yet
    /// been closed. A trailing `;` (e.g. `CSI 4 ; m`) leaves an empty slot that
    /// must still materialize as a default `0` at dispatch — ECMA-48 / vte
    /// semantics — whereas `CSI m` (no separator at all) stays empty. `finish`
    /// reads this to tell the two apart.
    is_pending: bool,
}

impl ParametersAccumulator {
    #[inline]
    pub fn push_digit(&mut self, digit: u8) {
        self.current = Some(
            self.current
                .unwrap_or(0)
                .saturating_mul(10)
                .saturating_add((digit - b'0') as u16),
        );
    }

    #[inline]
    pub fn push_sub(&mut self) {
        let current = self.current.take().unwrap_or(0);
        self.flush_sub(current);
    }

    #[inline]
    pub fn push_main(&mut self) {
        match self.current.take() {
            Some(val) => self.flush_value(val),
            // Trailing empty sub (`1:;`) defaults to 0 within the group.
            None if self.in_group => self.flush_sub(0),
            // Empty main param (`;`, `1;;`) becomes a new `[0]` group.
            // Drop on overflow: the CSI/DCS still dispatches with the capped
            // params rather than panicking.
            None => {
                let _ = self.inner.try_push(Param::None);
            }
        }

        self.in_group = false;
        // The `;` opened a fresh main-parameter slot; if nothing fills it before
        // dispatch, [`InternalParameters::flush`] turns it into a default `0`.
        self.is_pending = true;
    }

    #[inline]
    pub fn flush(&mut self) {
        match self.current.take() {
            Some(val) => self.flush_value(val),
            None if self.in_group => self.flush_sub(0),
            // Trailing `;` (`CSI 4 ; m`) → the open slot defaults to `0`.
            None if self.is_pending => {
                let _ = self.inner.try_push(Param::None);
            }
            None => {}
        }
    }

    fn flush_value(&mut self, val: u16) {
        if self.in_group {
            let _ = self.inner.try_push(Param::Sub(val));
        } else {
            let _ = self.inner.try_push(Param::Main(val));
        }
    }

    fn flush_sub(&mut self, val: u16) {
        if self.in_group {
            let _ = self.inner.try_push(Param::Sub(val));
        } else {
            // Only enter the group if the value actually landed, so `in_group`
            // bookkeeping stays consistent on overflow.
            if let Ok(_) =self.inner.try_push(Param::Main(val)) {
                self.in_group = true;
            }
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear();
        self.current = None;
        self.in_group = false;
        self.is_pending = false;
    }
}

impl AsRef<Params> for ParametersAccumulator {
    fn as_ref(&self) -> &Params {
        &Params::new(self.inner.as_slice())
    }
}

impl Borrow<Params> for ParametersAccumulator {
    fn borrow(&self) -> &Params {
        self.as_ref()
    }
}

pub type IntermediatesAccumulator = ArrayVec<u8, 2>;
