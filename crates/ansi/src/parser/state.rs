use std::fmt::Debug;
use strum::{EnumCount, IntoStaticStr};

#[repr(u8)]
#[derive_const(Clone, PartialEq, PartialOrd, Ord, Eq, Default, EnumCount, IntoStaticStr)]
#[derive(Copy)]
pub enum State {
    /// Initial state used to consume characters until an escape-style sequence begins;
    /// GL bytes 0x20-0x7F are printed, C0/C1 controls are executed immediately.
    #[default]
    Ground,

    /// Entered on ESC; cancels any unfinished sequence and starts parsing the next one.
    Escape,
    /// Collect zero or more 0x20–0x2F "intermediate" bytes for a final `esc_dispatch`.
    EscapeIntermediate,

    /// First byte of CSI detected (`ESC [` or 0x9B). Digits, intermediate bytes or the
    /// first character from set `<?=>\)` change state; C0 controls are executed on arrival.
    CsiEntry,
    CsiIgnore,
    /// One or more parameter bytes (digits, semicolon) have arrived; 0x3A or second private
    /// marker triggers csi_ignore; intermediate chars after this turn it malformed (→ignore).
    CsiParam,
    /// Exactly one intermediate byte seen; error to see digits while here (→ignore).
    CsiIntermediate,

    /// DCS (`ESC P` or 0x90) seen; same rules as CSI, but C0 bytes are buffered instead of
    /// executed. First byte examined for private marker (0x3C-0x3F) just like CSI entry.
    DcsEntry,
    DcsIgnore,
    DcsIntermediate,
    DcsParam,
    /// Data string phase of a DCS; input is handed to a separate handler until termination
    /// by ST, CAN, SUB or ESC that cancels the string.
    DcsData,
    
    /// Data string phase of an OSC; input is handed to a separate handler until termination
    OscData,
    /// SOS (Start-of-String) string data.
    /// Termina
    SosData,
    /// PM (Private-Message) string data.
    PmData,
    /// APC (Application Program Command) string data.
    ApcData,

    /// UTF-8 decoder state. Bytes are assembled into complete codepoints before being
    /// emitted; invalid sequences are replaced with U+FFFD and the decoder resumes.
    Utf8,
}

impl const From<u8> for State {
    fn from(value: u8) -> Self {
        debug_assert!((value as usize) < Self::COUNT);
        unsafe { std::mem::transmute(value) }
    }
}

impl const From<&u8> for State {
    fn from(value: &u8) -> Self {
        Self::from(*value)
    }
}
impl const From<usize> for State {
    fn from(value: usize) -> Self {
        Self::from(value as u8)
    }
}
impl Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "State::{}", <&str>::from(self))
    }
}
