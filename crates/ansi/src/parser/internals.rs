use crate::params::{Param, Parameters, Params};
use derive_more::{Deref, DerefMut};
use std::borrow::Borrow;
use std::ops::{Deref, DerefMut};
use arrayvec::ArrayVec;
use utils::{NestedMut};

/// Which separator was encountered in the parameter string.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Separator {
    /// `;` — starts a new main parameter.
    Main,
    /// `:` — starts a new sub-parameter within the current main parameter.
    Sub,
}

// ── Accumulator ────────────────────────────────────────────────────────────

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
/// # Semantics
///
/// * Empty main parameters default to `0`. Leading or repeated `;` separators
///   therefore create a `Param::None`, and a trailing `;` before the final
///   character (e.g. `CSI 4 ; m`) also emits a trailing `Param::None`.
/// * Empty sub-parameters default to `0`, so `1::3` is parsed as
///   `[Main(1), Sub(0), Sub(3)]`.
/// * A sequence with no separator at all, such as `CSI m`, leaves the
///   parameter list empty.
/// * Numeric values saturate at `u16::MAX`; overflow of the 32-entry capacity
///   is silently dropped so dispatch still proceeds with whatever fit.
///
/// # Lifecycle
///
/// ```text
/// for each digit byte:    push_digit(byte)
/// for each `;` separator: push_separator(Separator::Main)  // or push_main()
/// for each `:` separator: push_separator(Separator::Sub)   // or push_sub()
/// at end of sequence:     finish()
/// ```
///
/// Reuse with [`clear`](Self::clear).
#[derive(Debug, Default, Clone, Deref, DerefMut)]
pub struct ParametersAccumulator {
    /// Flat list of parsed parameters.
    #[deref]
    #[deref_mut]
    params: ArrayVec<Param, 32>,
    /// Decimal value being built from incoming ASCII digits.
    current: Option<u16>,
    /// Whether a `:` has been seen since the last `;` — i.e. whether `current`
    /// belongs to an active sub-parameter group.
    in_group: bool,
    /// Whether a `;` has opened a fresh main-parameter slot not yet filled.
    /// Distinguishes `CSI 4 ; m` (trailing `;` → emit `None`) from `CSI m`
    /// (no separator → stay empty).
    is_pending: bool,
}


// --- Core API --------------------------------------------------------------

impl ParametersAccumulator {
    /// Feed one ASCII digit byte (`b'0'`..`b'9'`).
    #[inline]
    pub fn push_digit(&mut self, digit: u8) {
        self.current = Some(
            self.current
                .unwrap_or(0)
                .saturating_mul(10)
                .saturating_add((digit - b'0') as u16),
        );
    }

    /// Feed a separator (`;` or `:`).
    ///
    /// Convenience wrapper around [`push_main`](Self::push_main) and
    /// [`push_sub`](Self::push_sub) for code that already has a
    /// [`Separator`] value.
    #[inline]
    pub fn push_separator(&mut self, separator: Separator) {
        match separator {
            Separator::Main => self.push_main(),
            Separator::Sub => self.push_sub(),
        }
    }

    /// Handle a `;` separator — finishes the current value (if any) and
    /// opens a fresh main-parameter slot.
    #[inline]
    pub fn push_main(&mut self) {
        self.flush();
        self.in_group = false;
        self.is_pending = true;
    }

    /// Handle a `:` separator — finishes the current value (if any) as a
    /// sub-parameter within the current main parameter.
    #[inline]
    pub fn push_sub(&mut self) {
        self.flush_as_sub();
    }

    /// Called when the final character of the sequence is reached.
    ///
    /// Materializes any pending value and resets the in-flight state so the
    /// accumulator is ready for inspection (or reuse after [`clear`](Self::clear)).
    #[inline]
    pub fn finish(&mut self) {
        self.flush();
        // Reset transient state; `params` is left intact for reading.
        self.in_group = false;
        self.is_pending = false;
    }

    /// Reset the accumulator to its initial empty state.
    #[inline]
    pub fn clear(&mut self) {
        self.params.clear();
        self.current = None;
        self.in_group = false;
        self.is_pending = false;
    }

    /// Returns a slice of the finished parameters.
    #[inline]
    pub fn as_slice(&self) -> &[Param] {
        &self.params
    }

    /// Push `current` (or a default) into `params` using whatever group mode
    /// is active, then clear `current`. Also handles the `is_pending` default.
    #[inline]
    pub fn flush(&mut self) {
        match self.current.take() {
            Some(val) => self.push(val),
            None => {
                if self.in_group {
                    // Trailing empty sub (`1:;`) → default 0 within the group.
                    let _ = self.params.try_push(Param::Sub(0));
                } else if self.is_pending {
                    // Trailing `;` (e.g. `CSI 4 ; m`) → default 0 main.
                    let _ = self.params.try_push(Param::None);
                }
                // Else: nothing was pending (e.g. `CSI m`), stay empty.
            }
        }
        self.is_pending = false;
    }

    /// Like [`commit`](Self::flush) but ensures the value
    /// lands as a `Sub` (entering the group if necessary).
    #[inline]
    fn flush_as_sub(&mut self) {
        let val = self.current.take().unwrap_or(0);
        if self.in_group {
            let _ = self.params.try_push(Param::Sub(val));
        } else {
            // First sub in this group: emit as Main, then mark group active.
            if self.params.try_push(Param::Main(val)).is_ok() {
                self.in_group = true;
            }
        }
        self.is_pending = false;
    }

    /// Push a concrete value with the currently active group mode.
    #[inline]
    fn push(&mut self, val: u16) {
        let param = if self.in_group {
            Param::Sub(val)
        } else {
            Param::Main(val)
        };
        let _ = self.params.try_push(param);
        self.is_pending = false;
    }
}

impl AsRef<[Param]> for ParametersAccumulator {
    #[inline]
    fn as_ref(&self) -> &[Param] {
        self.as_slice()
    }
}

impl Borrow<[Param]> for ParametersAccumulator {
    #[inline]
    fn borrow(&self) -> &[Param] {
        self.as_slice()
    }
}



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
