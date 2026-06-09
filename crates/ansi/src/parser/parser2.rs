use arrayvec::ArrayVec;
use crate::parser::{ByteString, Handler, ParametersBuilder};
use bilge::prelude::*;
use utils::Nested;
use utils_derive::state_machine;

state_machine! {
    #[derive(Copy, Debug, Default)]
    enum Action;
    #[derive(Copy, Debug, Default)]
    enum State;

    // Anywhere transitions (from any state)
   _ => {
        0x18       => [Action::Execute, State::Ground],
        0x1a       => [Action::Execute, State::Ground],
        0x80..0x8f => [Action::Execute, State::Ground],
        0x91..0x97 => [Action::Execute, State::Ground],
        0x99       => [Action::Execute, State::Ground],
        0x9a       => [Action::Execute, State::Ground],
        0x9c       => State::Ground,
        0x1b       => State::Escape,
        0x98       => State::SosPmApcString,
        0x9e       => State::SosPmApcString,
        0x9f       => State::SosPmApcString,
        0x90       => State::DcsEntry,
        0x9d       => State::OscString,
        0x9b       => State::CsiEntry,
    },

    #[default]
    State::Ground =>  {
        0x00..0x17 => Action::Execute,
        0x19       => Action::Execute,
        0x1c..0x1f => Action::Execute,
        0x20..0x7f => Action::Print,
    },

    // State: Escape
    State::Escape => {
        on_entry => Action::Clear,

        0x00..0x17 => Action::Execute,
        0x19       => Action::Execute,
        0x1c..0x1f => Action::Execute,
        0x7f       => Action::Ignore,
        0x20..0x2f => [Action::Intermediate, State::EscapeIntermediate],
        0x30..0x4f => [Action::EscDispatch, State::Ground],
        0x51..0x57 => [Action::EscDispatch, State::Ground],
        0x59       => [Action::EscDispatch, State::Ground],
        0x5a       => [Action::EscDispatch, State::Ground],
        0x5c       => [Action::EscDispatch, State::Ground],
        0x60..0x7e => [Action::EscDispatch, State::Ground],
        0x5b       => State::CsiEntry,
        0x5d       => State::OscString,
        0x50       => State::DcsEntry,
        0x58       => State::SosPmApcString,
        0x5e       => State::SosPmApcString,
        0x5f       => State::SosPmApcString,
    },

    // State: EscapeIntermediate
    State::EscapeIntermediate => {
        0x00..0x17 => Action::Execute,
        0x19       => Action::Execute,
        0x1c..0x1f => Action::Execute,
        0x20..0x2f => Action::Intermediate,
        0x7f       => Action::Ignore,
        0x30..0x7e => [Action::EscDispatch, State::Ground],
    },

    // State: CsiEntry
    State::CsiEntry => {
        on_entry => Action::Clear,

        0x00..0x17 => Action::Execute,
        0x19       => Action::Execute,
        0x1c..0x1f => Action::Execute,
        0x7f       => Action::Ignore,
        0x20..0x2f => [Action::Intermediate, State::CsiIntermediate],
        0x3a       => State::CsiIgnore,
        0x30..0x39 => [Action::Param, State::CsiParam],
        0x3b       => [Action::Param, State::CsiParam],
        0x3c..0x3f => [Action::Intermediate, State::CsiParam],
        0x40..0x7e => [Action::CsiDispatch, State::Ground],
    },

    // State: CsiIgnore
    State::CsiIgnore => {
        0x00..0x17 => Action::Execute,
        0x19       => Action::Execute,
        0x1c..0x1f => Action::Execute,
        0x20..0x3f => Action::Ignore,
        0x7f       => Action::Ignore,
        0x40..0x7e => State::Ground,
    },

    // State: CsiParam
    State::CsiParam => {
        0x00..0x17 => Action::Execute,
        0x19       => Action::Execute,
        0x1c..0x1f => Action::Execute,
        0x30..0x39 => Action::Param,
        0x3b       => Action::Param,
        0x7f       => Action::Ignore,
        0x3a       => State::CsiIgnore,
        0x3c..0x3f => State::CsiIgnore,
        0x20..0x2f => [Action::Intermediate, State::CsiIntermediate],
        0x40..0x7e => [Action::CsiDispatch, State::Ground],
    },

    // State: CsiIntermediate
    State::CsiIntermediate => {
        0x00..0x17 => Action::Execute,
        0x19       => Action::Execute,
        0x1c..0x1f => Action::Execute,
        0x20..0x2f => Action::Intermediate,
        0x7f       => Action::Ignore,
        0x30..0x3f => State::CsiIgnore,
        0x40..0x7e => [Action::CsiDispatch, State::Ground],
    },

    // State: DcsEntry
    State::DcsEntry => {
        on_entry => Action::Clear,

        0x00..0x17 => Action::Ignore,
        0x19       => Action::Ignore,
        0x1c..0x1f => Action::Ignore,
        0x7f       => Action::Ignore,
        0x3a       => State::DcsIgnore,
        0x20..0x2f => [Action::Intermediate, State::DcsIntermediate],
        0x30..0x39 => [Action::Param, State::DcsParam],
        0x3b       => [Action::Param, State::DcsParam],
        0x3c..0x3f => [Action::Intermediate, State::DcsParam],
        0x40..0x7e => State::DcsData,
    },

    // State: DcsIntermediate
    State::DcsIntermediate => {
        0x00..0x17 => Action::Ignore,
        0x19       => Action::Ignore,
        0x1c..0x1f => Action::Ignore,
        0x20..0x2f => Action::Intermediate,
        0x7f       => Action::Ignore,
        0x30..0x3f => State::DcsIgnore,
        0x40..0x7e => State::DcsData,
    },

    // State: DcsIgnore
    State::DcsIgnore => {
        0x00..0x17 => Action::Ignore,
        0x19       => Action::Ignore,
        0x1c..0x1f => Action::Ignore,
        0x20..0x7f => Action::Ignore,
    },

    // State: DcsParam
    State::DcsParam => {

        0x00..0x17 => Action::Ignore,
        0x19       => Action::Ignore,
        0x1c..0x1f => Action::Ignore,
        0x30..0x39 => Action::Param,
        0x3b       => Action::Param,
        0x7f       => Action::Ignore,
        0x3a       => State::DcsIgnore,
        0x3c..0x3f => State::DcsIgnore,
        0x20..0x2f => [Action::Intermediate, State::DcsIntermediate],
        0x40..0x7e => State::DcsData,
    },

    // State: DcsPassthrough
    State::DcsData => {
        on_entry => Action::DcsStart,
        on_exit  => Action::DcsEnd,

        0x00..0x17 => Action::Put,
        0x19       => Action::Put,
        0x1c..0x1f => Action::Put,
        0x20..0x7e => Action::Put,
        0x7f       => Action::Ignore,
    },

    // State: SosPmApcString
    State::SosPmApcString => {
        0x00..0x17 => Action::Ignore,
        0x19       => Action::Ignore,
        0x1c..0x1f => Action::Ignore,
        0x20..0x7f => Action::Ignore,
    },

    // State: OscString
    State::OscString => {
        on_entry => Action::OscStart,
        on_exit  => Action::OscEnd,


        0x00..0x17 => Action::Ignore,
        0x19       => Action::Ignore,
        0x1c..0x1f => Action::Ignore,
        0x20..0x7f => Action::OscPut,
    },
}

impl const Action {
    pub fn is_some(&self) -> bool {
        *self != Action::None
    }

    pub fn is_none(&self) -> bool {
        *self == Action::None
    }
}

impl const State {

    pub fn is_some(&self) -> bool {
        *self != State::None
    }

    pub fn is_none(&self) -> bool {
        *self == State::None
    }
    #[inline(always)]
    pub fn transition(self, byte: u8) -> (Action, Self) {
        transition(self, byte)
    }

    #[inline(always)]
    pub fn exit(self) -> Action {
        exit_action(self)
    }

    #[inline(always)]
    pub fn entry(self) -> Action {
        entry_action(self)
    }

}
#[derive(Debug, Default)]
pub struct Parser {
    pub state: State,

    pub params: ParametersBuilder,
    pub intermediates: ByteString,

    pub utf8: ArrayVec<u8, 4>,
}

impl Parser {
    fn advance(&mut self, handler: &mut impl Handler, bytes: &[u8]) {
        let mut i = 0;

        while i < bytes.len() {
            match self.state {
                State::Ground => i += self.advance_ground(handler, &bytes[i..]),
                _ => {
                    self.transition(handler, bytes[i]);
                    i += 1;
                }
            }
        }
    }

    fn transition(&mut self, handler: &mut impl Handler, byte: u8) {
        let (action, next_state) = self.state.transition(byte);
        let prev_state = self.state;

        if next_state.is_some() {
            let exit_action = prev_state.exit();
            if exit_action.is_some() {
                self.action(handler, exit_action, byte);
            }

            if action.is_some() {
                self.action(handler, action, byte);
            }

            let entry_action = next_state.entry();
            if entry_action.is_some() {
                self.action(handler, entry_action, byte);
            }

            self.state = next_state;
        } else {
            self.action(handler, action, byte);
        }
    }

    fn advance_ground(&mut self, handler: &mut impl Handler, bytes: &[u8]) -> usize {
        0
    }

    #[inline]
    fn action(&mut self, handler: &mut impl Handler, action: Action, byte: u8) {
        match action {
            Action::None | Action::Ignore => {}

            Action::Clear => self.clear(),

            Action::Print => handler.print(byte as char),
            Action::Execute => handler.execute(byte),

            Action::Intermediate => self.intermediates.push(byte),

            Action::Param => match byte {
                b'0'..=b'9' => self.params.push_digit(byte),
                b':' => self.params.push_sub(),
                b';' => self.params.push_main(),
                _ => {}
            },

            Action::EscDispatch => handler.esc(self.intermediates.as_ref(), byte),
            Action::CsiDispatch => {
                self.params.finish();
                handler.csi(
                    self.params.as_nested_slice(),
                    self.intermediates.as_ref(),
                    byte as char,
                );
            }

            Action::DcsDispatch => {
                self.params.finish();
                handler.dcs(
                    self.params.as_nested_slice(),
                    self.intermediates.as_ref(),
                    byte as char,
                );
            }
            Action::DcsByte => handler.dcs_byte(byte),
            Action::DcsTermination => handler.dcs_termination(byte),

            Action::OscDispatch => handler.osc(),
            Action::OscByte => handler.osc_byte(byte),
            Action::OscTermination => handler.osc_termination(byte),
        }
    }

    /// Reset parameter / intermediate / data buffers.
    pub fn clear(&mut self) {
        self.params.clear();
        self.intermediates.clear();
        self.utf8.clear();
    }
}

