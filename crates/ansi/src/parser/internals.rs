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

#[derive(Deref, DerefMut, Debug, Default, Clone)]
pub struct InternalParameters {
    #[deref]
    #[deref_mut]
    inner: ArrayVec<Param, 32>,
    current: Option<u16>,
    /// Whether the main parameter currently being built has sub-parameters,
    /// i.e. a `:` separator has been seen since the last `;`. This is the one
    /// bit that distinguishes `1:2` (sub) from `1;2` (new param) — `inner`
    /// alone can't, since both leave `[1]` behind after the first value.
    in_group: bool,
}

impl InternalParameters {
    /// ECMA-48 caps parameters at 16383.
    const MAX: u16 = 16383;

    /// Accumulate an ASCII digit into the current sub-parameter value.
    pub fn push_digit(&mut self, digit: u8) {
        self.current = Some(
            self.current
                .unwrap_or(0)
                .saturating_mul(10)
                .saturating_add((digit - b'0') as u16),
        );
    }

    /// Append current parameter as a sub-parameter (`:` separator).
    /// Empty sub-parameters default to 0 to mirror ECMA-48 — `1::3` means `[1, 0, 3]`.
    pub fn push_sub(&mut self) {
        let val = self.current.take().unwrap_or(0);
        self.push_sub_value(val);
    }

    /// Append current parameter as a main parameter (`;` separator).
    /// An empty leading param defaults to 0 — `;1m` means `[[0], [1]]`.
    pub fn push_main(&mut self) {
        match self.current.take() {
            Some(val) => self.push_value(val),
            // Trailing empty sub (`1:;`) defaults to 0 within the group.
            None if self.in_group => self.push_sub_value(0),
            // Empty main param (`;`, `1;;`) becomes a new `[0]` group.
            // Drop on overflow: the CSI/DCS still dispatches with the capped
            // params rather than panicking.
            None => {
                self.inner.push(Param::None);
            }
        }

        self.in_group = false;
    }

    /// Finalize the in-flight param at dispatch time.
    pub fn finish(&mut self) {
        match self.current.take() {
            Some(val) => self.push_value(val),
            None if self.in_group => self.push_sub_value(0),
            None => {}
        }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.current = None;
        self.in_group = false;
    }

    // The push helpers drop values that exceed capacity (`NestedError::Overflow`)
    // instead of panicking. A pathologically long parameter list still
    // dispatches, just with the trailing params capped — mirroring the
    // reference's cap-and-continue behavior, safely.
    fn push_value(&mut self, val: u16) {
        if self.in_group {
            let _ = self.inner.try_push(Param::Sub(val));
        } else {
            let _ = self.inner.try_push(Param::Main(val));
        }
    }

    fn push_sub_value(&mut self, val: u16) {
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
}

impl AsRef<Params> for InternalParameters {
    fn as_ref(&self) -> &Params {
        &Params::new(self.inner.as_slice())
    }
}

impl Borrow<Params> for InternalParameters {
    fn borrow(&self) -> &Params {
        self.as_ref()
    }
}

const NUL: u8 = 0;
const SOH: u8 = 1;
const STX: u8 = 2;
const ETX: u8 = 3;
const EOT: u8 = 4;
const ENQ: u8 = 5;
const ACK: u8 = 6;
const BEL: u8 = 7;
const BS: u8 = 8;
const TAB: u8 = 9;
const LF: u8 = 10;
const VT: u8 = 11;
const FF: u8 = 12;
const CR: u8 = 13;
const SO: u8 = 14;
const SI: u8 = 15;
const DLE: u8 = 16;
const DC1: u8 = 17;
const DC2: u8 = 18;
const DC3: u8 = 19;
const DC4: u8 = 20;
const NAK: u8 = 21;
const SYN: u8 = 22;
const ETB: u8 = 23;
const CAN: u8 = 24;
const EM: u8 = 25;
const SUB: u8 = 26;
const ESC: u8 = 27;
const FS: u8 = 28;
const GS: u8 = 29;
const RS: u8 = 30;
const US: u8 = 31;
const DEL: u8 = 127;

memspan::skip_class! {
    pub fn skip_ascii_graphic_and_utf8(
        ranges = [0x21..=0xFF],
    );
}