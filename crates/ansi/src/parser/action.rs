use std::fmt::Debug;
use strum::{EnumCount, IntoStaticStr};

#[repr(u8)]
#[derive_const(Clone, PartialEq, PartialOrd, Ord, Eq, Default, EnumCount, IntoStaticStr)]
#[derive(Copy)]
pub enum Action {
    #[default]
    None,
    /// Forget the current private flag, intermediate characters, final character and
    /// parameters. Fired on entry to `Escape`, `CsiEntry` and `DcsEntry`.
    Clear,
    /// Store the private marker or intermediate character for use when the final
    /// character arrives.
    Collect,

    /// The final character of an escape, CSI or DCS sequence has arrived; dispatch
    /// the corresponding control function.
    Dispatch,

    /// Execute a C0 or C1 control function.
    Execute,

    /// Drop the byte; no observable change to terminal state.
    Ignore,

    /// Collect parameter digits / separators (`0`-`9`, `;`, `:`).
    Param,

    /// In `Ground`, map the code to a glyph and display it.
    /// Accumulate a UTF-8 byte into `self.utf8`. Emits the codepoint and
    /// returns to `Ground` once enough continuation bytes have arrived.
    Print,
}

impl const From<u8> for Action {
    fn from(value: u8) -> Self {
        debug_assert!((value as usize) < Self::COUNT);
        unsafe { std::mem::transmute(value) }
    }
}

impl const From<&u8> for Action {
    fn from(value: &u8) -> Self {
        Self::from(*value)
    }
}

impl const From<usize> for Action {
    fn from(value: usize) -> Self {
        Self::from(value as u8)
    }
}
impl Debug for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Action::{}", <&str>::from(self))
    }
}
