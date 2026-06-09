use derive_more::{Deref, DerefMut};
use utils::{NestedMut, NestedRaw, TryNestedMut};

#[derive(Deref, DerefMut, Debug, Default, Clone)]
pub struct ParametersBuilder {
    #[deref]
    #[deref_mut]
    inner: NestedRaw<u16, 32, 32>,
    current: Option<u16>,
    group_active: bool,
    last_separator: Option<ParameterSeparator>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParameterSeparator {
    Main,
    Sub,
}

impl ParametersBuilder {
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
        self.last_separator = Some(ParameterSeparator::Sub);
    }

    /// Append current parameter as a main parameter (`;` separator).
    /// An empty leading param defaults to 0 — `;1m` means `[[0], [1]]`.
    pub fn push_main(&mut self) {
        match self.current.take() {
            Some(val) => self.push_value(val),
            None if matches!(self.last_separator, Some(ParameterSeparator::Sub)) => {
                self.push_sub_value(0);
            }
            // Drop on overflow: the CSI/DCS still dispatches with the capped
            // params rather than panicking.
            None => {
                let _ = self.inner.try_push_one(0);
            }
        }

        self.group_active = false;
        self.last_separator = Some(ParameterSeparator::Main);
    }

    /// Finalize the in-flight param at dispatch time.
    pub fn finish(&mut self) {
        match self.current.take() {
            Some(val) => self.push_value(val),
            None if matches!(self.last_separator, Some(ParameterSeparator::Sub)) => {
                self.push_sub_value(0);
            }
            None => {}
        }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.current = None;
        self.group_active = false;
        self.last_separator = None;
    }

    // The push helpers drop values that exceed capacity (`NestedError::Overflow`)
    // instead of panicking. A pathologically long parameter list still
    // dispatches, just with the trailing params capped — mirroring the
    // reference's cap-and-continue behavior, safely.
    fn push_value(&mut self, val: u16) {
        if self.group_active {
            self.push_sub_value(val);
        } else {
            let _ = self.inner.try_push_one(val);
        }
    }

    fn push_sub_value(&mut self, val: u16) {
        if self.group_active {
            let _ = self.inner.try_extend_one(val);
        } else {
            // Only mark the group active if the value actually landed, so
            // `group_active` bookkeeping stays consistent on overflow.
            if self.inner.try_push_one(val).is_ok() {
                self.group_active = true;
            }
        }
    }
}
