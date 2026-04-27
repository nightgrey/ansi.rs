use core::mem;
use strum::{EnumCount, IntoStaticStr};
use utils_derive::transitions;

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
    _Unused
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Default, EnumCount, IntoStaticStr, Debug)]
pub enum State {
    None = 0,
    #[default]
    Ground,
    Utf8,
    Escape,
    EscapeIntermediate,

    CsiEntry,
    CsiParam,
    CsiIntermediate,

    CsiIgnore,

    DcsEntry,
    DcsParam,
    DcsIntermediate,
    DcsString,
    DcsIgnore,

    OscString,

    SosPmApcString,
}

transitions!(0, {
    Anywhere {
        0x18       => (Execute, Ground),
        0x1a       => (Execute, Ground),
        0x80..=0x8f => (Execute, Ground),
        0x91..=0x97 => (Execute, Ground),
        0x99       => (Execute, Ground),
        0x9a       => (Execute, Ground),
        0x9c       => (None, Ground),
        0x1b       => (None, Escape),
        0x98       => (None, SosPmApcString),
        0x9e       => (None, SosPmApcString),
        0x9f       => (None, SosPmApcString),
        0x90       => (None, DcsEntry),
        0x9d       => (None, OscString),
        0x9b       => (None, OscString),
    },

    Ground {
        0x00..=0x17 => Execute,
        0x19       => Execute,
        0x1c..=0x1f => Execute,
        0x20..=0x7f => Print,
    }
    
    Utf8 {}

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
        0x5d       => (None, OscString),
        0x50       => (None, DcsEntry),
        0x58       => (None, SosPmApcString),
        0x5e       => (None, SosPmApcString),
        0x5f       => (None, SosPmApcString),
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
        0x3a        => (None, CsiIgnore),
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
        0x40..=0x7e => (None, DcsString)
    },


    DcsParam {
        0x00..=0x17 => Ignore,
        0x19       => Ignore,
        0x1c..=0x1f => Ignore,
        0x30..=0x39 => Param,
        0x3b       => Param,
        0x7f       => Ignore,
        0x3a       => (None, DcsIgnore),
        0x3c..=0x3f => (None, DcsIgnore),
        0x20..=0x2f => (Collect, DcsIntermediate),
        0x40..=0x7e => (None, DcsString)
    },

    DcsIntermediate {
        0x00..=0x17 => Ignore,
        0x19       => Ignore,
        0x1c..=0x1f => Ignore,
        0x20..=0x2f => Collect,
        0x7f       => Ignore,
        0x30..=0x3f => (None, DcsIgnore),
        0x40..=0x7e => (None, DcsString)
    },

    DcsString {
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

    OscString {
        on_entry  => OscDispatch,
        0x00..=0x17 => Ignore,
        0x19       => Ignore,
        0x1c..=0x1f => Ignore,
        0x20..=0x7f => OscByte,
        on_exit   => OscTermination
    }


    SosPmApcString {
        0x00..=0x17 => Ignore,
        0x19       => Ignore,
        0x1c..=0x1f => Ignore,
        0x20..=0x7f => Ignore,
    },

});

/// Unpack a u8 into a State and Action
///
/// The implementation of this assumes that there are *precisely* 16 variants
/// for both Action and State. Furthermore, it assumes that the enums are
/// tag-only; that is, there is no data in any variant.
///
/// Bad things will happen if those invariants are violated.
#[inline(always)]
pub fn unpack(byte: u8) -> (Action, State) {
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
pub const fn pack(action: Action, state: State) -> u8 {
    (action as u8) << 4 | state as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unpack_state_action() {
        match unpack(0xEE) {
            (Action::OscTermination, State::Ground) => (),
            _ => panic!("unpack failed"),
        }

        match unpack(0x0E) {
            (Action::None, State::Ground) => (),
            _ => panic!("unpack failed"),
        }

        match unpack(0xE0) {
            (Action::OscTermination, State::CsiEntry) => (),
            _ => panic!("unpack failed"),
        }
    }

    #[test]
    fn pack_state_action() {
        assert_eq!(pack(Action::OscTermination, State::Ground), 0xEE);
        assert_eq!(pack(Action::None, State::Ground), 0x0E);
        assert_eq!(pack(Action::OscTermination, State::CsiEntry), 0xE0);
    }
}