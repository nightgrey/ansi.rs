use core::mem;
use strum::{EnumCount, IntoStaticStr};
use utils_derive::transitions;

transitions! {
    Anywhere {
        0x18       => (Execute, Ground),
        0x1a       => (Execute, Ground),
        0x80..=0x8f => (Execute, Ground),
        0x91..=0x97 => (Execute, Ground),
        0x99       => (Execute, Ground),
        0x9a       => (Execute, Ground),
        0x9c       => (None, Ground),
        0x1b       => (None, Escape),
        0x98       => (None, SosPmApcData),
        0x9e       => (None, SosPmApcData),
        0x9f       => (None, SosPmApcData),
        0x90       => (None, DcsEntry),
        0x9d       => (None, OscData),
        0x9b       => (None, CsiEntry),
    },

    None {

    },


    Ground {
        0x00..=0x17 => Execute,
        0x19       => Execute,
        0x1c..=0x1f => Execute,
        0x20..=0x7f => (Collect, Utf8),
        0xC2..=0xDF => (Collect, Utf8), // UTF8 2 byte sequence
        0xE0..=0xEF => (Collect, Utf8), // UTF8 3 byte sequence
        0xF0..=0xF4 => (Collect, Utf8), // UTF8 4 byte sequence
    }

    Utf8 {
        0x80..=0xBF => (Collect, Utf8), // Continuation byte
    }

    Escape {
        on_entry  => Clear,
        0x00..=0x17 => Execute,
        0x19       => Execute,
        0x1c..=0x1f => Execute,
        0x7f       => Ignore,
        0x20..=0x2f => (Collect, EscapeIntermediate),
        0x30..=0x4f => (EscDispatch, Ground),
        0x51..=0x57 => (EscDispatch, Ground),
        0x59       => (EscDispatch, Ground),
        0x5a       => (EscDispatch, Ground),
        0x5c       => (EscDispatch, Ground),
        0x60..=0x7e => (EscDispatch, Ground),
        0x5b       => (None, CsiEntry),
        0x5d       => (None, OscData),
        0x50       => (None, DcsEntry),
        0x58       => (None, SosPmApcData),
        0x5e       => (None, SosPmApcData),
        0x5f       => (None, SosPmApcData),
    },

    EscapeIntermediate {
        0x00..=0x17 => Execute,
        0x19       => Execute,
        0x1c..=0x1f => Execute,
        0x20..=0x2f => Collect,
        0x7f       => Ignore,
        0x30..=0x7e => (EscDispatch, Ground)
    },

    CsiEntry {
        on_entry  => Clear,
        0x00..=0x17 => Execute,
        0x19       => Execute,
        0x1c..=0x1f => Execute,
        0x7f       => Ignore,
        0x20..=0x2f => (Collect, CsiIntermediate),
        0x3a       => (None, CsiIgnore),
        0x30..=0x39 => (Param, CsiParam),
        0x3b       => (Param, CsiParam),
        0x3c..=0x3f => (Collect, CsiParam),
        0x40..=0x7e => (CsiDispatch, Ground)
    },

    CsiParam {
        0x00..=0x17 => Execute,
        0x19       => Execute,
        0x1c..=0x1f => Execute,
        0x30..=0x39 => Param,
        0x3b       => Param,
        0x7f       => Ignore,
        0x3c..=0x3f => (None, CsiIgnore),
        0x3a        => Param,
        0x20..=0x2f => (Collect, CsiIntermediate),
        0x40..=0x7e => (CsiDispatch, Ground)
    },

    CsiIntermediate {
        0x00..=0x17 => Execute,
        0x19       => Execute,
        0x1c..=0x1f => Execute,
        0x20..=0x2f => Collect,
        0x7f       => Ignore,
        0x30..=0x3f => (None, CsiIgnore),
        0x40..=0x7e => (CsiDispatch, Ground),
    },

    CsiIgnore {
        0x00..=0x17 => Execute,
        0x19       => Execute,
        0x1c..=0x1f => Execute,
        0x20..=0x3f => Ignore,
        0x7f       => Ignore,
        0x40..=0x7e => (None, Ground),
    },

    DcsEntry {
        on_entry  => Clear,
        0x00..=0x17 => Ignore,
        0x19       => Ignore,
        0x1c..=0x1f => Ignore,
        0x7f       => Ignore,
        0x3a       => (None, DcsIgnore),
        0x20..=0x2f => (Collect, DcsIntermediate),
        0x30..=0x39 => (Param, DcsParam),
        0x3b       => (Param, DcsParam),
        0x3c..=0x3f => (Collect, DcsParam),
        0x40..=0x7e => (None, DcsData)
    },


    DcsParam {
        0x00..=0x17 => Ignore,
        0x19       => Ignore,
        0x1c..=0x1f => Ignore,
        0x30..=0x39 => Param,
        0x3b       => Param,
        0x7f       => Ignore,
        0x3a       => Param,
        0x3c..=0x3f => (None, DcsIgnore),
        0x20..=0x2f => (Collect, DcsIntermediate),
        0x40..=0x7e => (None, DcsData)
    },

    DcsIntermediate {
        0x00..=0x17 => Ignore,
        0x19       => Ignore,
        0x1c..=0x1f => Ignore,
        0x20..=0x2f => Collect,
        0x7f       => Ignore,
        0x30..=0x3f => (None, DcsIgnore),
        0x40..=0x7e => (None, DcsData)
    },

    DcsData {
        on_entry  => DcsDispatch,
        0x00..=0x17 => DcsByte,
        0x19       => DcsByte,
        0x1c..=0x1f => DcsByte,
        0x20..=0x7e => DcsByte,
        0x7f       => Ignore,
        on_exit   => DcsTermination
    },

    DcsIgnore {
        0x00..=0x17 => Ignore,
        0x19       => Ignore,
        0x1c..=0x1f => Ignore,
        0x20..=0x7f => Ignore,
    },

    OscData {
        on_entry  => OscDispatch,
        0x00..=0x17 => Ignore,
        0x07       => (None, Ground),
        0x19       => Ignore,
        0x1c..=0x1f => Ignore,
        0x20..=0x7f => OscByte,
        on_exit   => OscTermination
    }


    SosPmApcData {
        0x00..=0x17 => Ignore,
        0x19       => Ignore,
        0x1c..=0x1f => Ignore,
        0x20..=0x7f => Ignore,
    },
}

// NOTE: Removing the unused actions prefixed with `_` will reduce performance.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Default, EnumCount, IntoStaticStr, Debug)]
pub enum Action {
    #[default]
    None = 0,
    Ignore,
    Clear,

    Print,
    Execute,
    Collect,
    Param,
    EscDispatch,
    CsiDispatch,

    DcsDispatch,
    DcsByte,
    DcsTermination,

    OscDispatch,
    OscByte,
    OscTermination,
    _Unused,
}

#[repr(u8)]
#[derive(Copy, EnumCount, IntoStaticStr, Debug)]
#[derive_const(Clone, PartialEq, Eq, Default)]
pub enum State {
    /// No state transition.
    None,
    #[default]
    /// Initial state used to consume characters until an escape-style sequence begins.
    Ground,
    /// UTF-8 byte sequence (`0xC2..=0xDF`, `0xE0..=0xEF`, or `0xF0..=0xF4`) or continuation byte (`0x80..=0xBF`).
    Utf8,
    /// ESC (`ESC` or 0x1B)
    Escape,
    EscapeIntermediate,

    /// CSI (`ESC [` / `0x1B 0x5B` or `0x9B`)
    CsiEntry,
    CsiParam,
    CsiIntermediate,
    CsiIgnore,

    /// DCS (`ESC P` / `0x1B 0x50` or `0x90`)
    DcsEntry,
    DcsParam,
    DcsIntermediate,
    DcsData,
    DcsIgnore,

    /// OSC (`ESC ]` / `0x1B 0x5D` or `0x9B`)
    OscData,

    /// - SOS (`ESC P` / `0x1B 0x50` or `0x98`)
    /// - PM (`ESC ^` / `0x1B 0x5E` or `0x9E`)
    /// - APC (`ESC _` / `0x1B 0x5F` or `0x9F`)
    SosPmApcData,
}

#[inline(always)]
pub const fn pack(action: Action, state: State) -> u8 {
    (action as u8) << 4 | state as u8
}

#[inline(always)]
pub const fn unpack(byte: u8) -> (Action, State) {
    unsafe {
        (
            // Action is stored in top 4 bits
            mem::transmute::<u8, Action>(byte >> 4),
            // State is stored in bottom 4 bits
            mem::transmute::<u8, State>(byte & 0x0F),
        )
    }
}

#[inline(always)]
pub const fn transition(state: State, byte: u8) -> (Action, State) {
    unpack(TRANSITIONS[state as usize][byte as usize])
}

#[inline(always)]
pub const fn entry(state: State) -> Action {
    ENTRY_ACTIONS[state as usize]
}

#[inline(always)]
pub const fn exit(state: State) -> Action {
    EXIT_ACTIONS[state as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unpack_state_action() {
        match unpack(0xE1) {
            (Action::OscTermination, State::Ground) => (),
            _ => panic!("unpack failed"),
        }

        match unpack(0x01) {
            (Action::None, State::Ground) => (),
            _ => panic!("unpack failed"),
        }

        match unpack(0xE5) {
            (Action::OscTermination, State::CsiEntry) => (),
            _ => panic!("unpack failed"),
        }
    }

    #[test]
    fn pack_state_action() {
        assert_eq!(pack(Action::OscTermination, State::Ground), 0xE1);
        assert_eq!(pack(Action::None, State::Ground), 0x01);
        assert_eq!(pack(Action::OscTermination, State::CsiEntry), 0xE5);
    }
}
