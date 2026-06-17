use derive_more::{AsMut, AsRef, Deref, DerefMut, Index, IndexMut};
use utils::{NestedMut, NestedRaw, TryNestedMut};

#[derive(Debug, Default, Clone, PartialEq, Eq, Index, IndexMut, AsRef, AsMut)]
pub struct Utf8 {
    #[index]
    #[index_mut]
    #[as_ref(forward)]
    #[as_mut(forward)]
    inner: [u8; 4],
    len: usize,
}

impl Utf8 {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn set_len(&mut self, len: usize) {
        self.len = len;
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.inner[..self.len]
    }

    #[inline]
    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        &mut self.inner[..self.len]
    }

    #[inline]
    pub fn clear(&mut self) {
        self.len = 0;
    }
}

#[derive(Deref, DerefMut, Debug, Default, Clone)]
pub struct Parameters {
    #[deref]
    #[deref_mut]
    inner: NestedRaw<u16, 32, 32>,
    current: Option<u16>,
    /// Whether the main parameter currently being built has sub-parameters,
    /// i.e. a `:` separator has been seen since the last `;`. This is the one
    /// bit that distinguishes `1:2` (sub) from `1;2` (new param) — `inner`
    /// alone can't, since both leave `[1]` behind after the first value.
    in_group: bool,
}

impl Parameters {
    /// ECMA-48 caps parameters at 16383.
    const MAX: u16 = 16383;

    /// Accumulate an ASCII digit into the current sub-parameter value.
    pub fn push_digit(&mut self, digit: u8) {
        self.current = Some(
            self.current
                .unwrap_or(0)
                .saturating_mul(10)
                .saturating_add((digit - b'0') as u16)
                .min(Self::MAX),
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
                let _ = self.inner.try_push_one(0);
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
            self.push_sub_value(val);
        } else {
            let _ = self.inner.try_push_one(val);
        }
    }

    fn push_sub_value(&mut self, val: u16) {
        if self.in_group {
            let _ = self.inner.try_extend_one(val);
        } else {
            // Only enter the group if the value actually landed, so `in_group`
            // bookkeeping stays consistent on overflow.
            if self.inner.try_push_one(val).is_ok() {
                self.in_group = true;
            }
        }
    }
}
