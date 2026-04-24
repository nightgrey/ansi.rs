use std::fmt::Debug;
use std::ops::{Index, IndexMut};
use strum::EnumCount;
use super::{Action, State, Transition};

/// DEC ANSI transition table
///
/// https://vt100.net/emu/dec_ansi_parser
#[derive(Clone, Debug)]
pub struct Table {
    inner: [u16; State::COUNT * 256],
    enter: [Action; State::COUNT],
    exit: [Action; State::COUNT],
}

pub static GLOBAL: Table = Table::default();

impl const Table {
    // Table values are encoded as:
    //
    //   inner index:   (state << INDEX_STATE_SHIFT) | byte
    //   inner value:   (action << TRANSITION_ACTION_SHIFT) | next_state
    //
    // `State` uses 5 bits (17 states); `Action` takes the remaining bits.
    //
    // "No transition" is encoded as `Action::None` (0).
    const TRANSITION_ACTION_SHIFT: usize = 5;
    const TRANSITION_STATE_MASK: u16 = 0x1F; // 5 bits for State
    const INDEX_STATE_SHIFT: usize = 8;

    pub const GLOBAL: &'static Self = &GLOBAL;

    pub fn default() -> Self {
        macro_rules! each {
            ($($state:ident),* => |$s:ident| $f:expr) => {
                {
                   $(let $s = State::$state; $f)*
                }
            };
            (|$state:ident| $f:expr) => {
                each!(
                    Ground,
                    Escape,
                    EscapeIntermediate,
                    CsiEntry,
                    CsiIgnore,
                    CsiParam,
                    CsiIntermediate,
                    DcsEntry,
                    DcsIgnore,
                    DcsIntermediate,
                    DcsParam,
                    DcsData,
                    OscData,
                    SosData,
                    PmData,
                    ApcData,
                    Utf8 => |$state| $f
                );
            }
        }

        let mut table = Table::empty();

        // "Anywhere" transitions — apply to every state.
        each!(|state| {
            table.add(0x18, state, Action::Execute, State::Ground);
            table.add(0x1A, state, Action::Execute, State::Ground);
            table.add(0x80..=0x8F, state, Action::Execute, State::Ground);
            table.add(0x91..=0x97, state, Action::Execute, State::Ground);
            table.add(0x99, state, Action::Execute, State::Ground);
            table.add(0x9A, state, Action::Execute, State::Ground);
            table.add(0x9C, state, Action::None, State::Ground);

            table.add(0x1B, state, Action::None, State::Escape);

            table.add(0x98, state, Action::None, State::SosData);
            table.add(0x9E, state, Action::None, State::PmData);
            table.add(0x9F, state, Action::None, State::ApcData);

            table.add(0x90, state, Action::None, State::DcsEntry);
            table.add(0x9D, state, Action::None, State::OscData);
            table.add(0x9B, state, Action::None, State::CsiEntry);
        });

        // State::Ground
        table.add(0x00..=0x17, State::Ground, Action::Execute, State::Ground);
        table.add(0x19, State::Ground, Action::Execute, State::Ground);
        table.add(0x1C..=0x1F, State::Ground, Action::Execute, State::Ground);
        table.add(0x20..=0x7F, State::Ground, Action::Print, State::Ground);
        table.add(0xC2..=0xF4, State::Ground, Action::None, State::Utf8);

        // State::Utf8 — accumulate continuation bytes. `Utf8Collect` manually
        // resets to Ground once the codepoint is complete (same-state
        // transition, so no exit/entry fires). The exit action is Clear so
        // that an abort via anywhere rules (ESC, CAN, SUB, …) drops the
        // half-decoded bytes instead of leaving them in `self.utf8`.
        table.add_enter(State::Utf8, Action::Print);
        table.add(0x80..=0xBF, State::Utf8, Action::Print, State::Utf8);
        // Invalid byte in Utf8 — abort the partial sequence and return to Ground.
        table.add(0xC0..=0xFF, State::Utf8, Action::None, State::Ground);
        table.add_exit(State::Utf8, Action::Clear);

        // State::Escape
        table.add_enter(State::Escape, Action::Clear);
        table.add(0x00..=0x17, State::Escape, Action::Execute, State::Escape);
        table.add(0x19, State::Escape, Action::Execute, State::Escape);
        table.add(0x1C..=0x1F, State::Escape, Action::Execute, State::Escape);
        table.add(0x7F, State::Escape, Action::Ignore, State::Escape);
        table.add(
            0x20..=0x2F,
            State::Escape,
            Action::Collect,
            State::EscapeIntermediate,
        );
        table.add(0x30..=0x4F, State::Escape, Action::Dispatch, State::Ground);
        table.add(0x51..=0x57, State::Escape, Action::Dispatch, State::Ground);
        table.add(0x59, State::Escape, Action::Dispatch, State::Ground);
        table.add(0x5A, State::Escape, Action::Dispatch, State::Ground);
        table.add(0x5C, State::Escape, Action::Dispatch, State::Ground);
        table.add(0x60..=0x7E, State::Escape, Action::Dispatch, State::Ground);
        table.add(0x5B, State::Escape, Action::None, State::CsiEntry);
        table.add(0x5D, State::Escape, Action::None, State::OscData);
        table.add(0x50, State::Escape, Action::None, State::DcsEntry);
        table.add(0x58, State::Escape, Action::None, State::SosData); // SOS
        table.add(0x5E, State::Escape, Action::None, State::PmData); // PM
        table.add(0x5F, State::Escape, Action::None, State::ApcData); // APC

        // State::EscapeIntermediate
        table.add(
            0x00..=0x17,
            State::EscapeIntermediate,
            Action::Execute,
            State::EscapeIntermediate,
        );
        table.add(
            0x19,
            State::EscapeIntermediate,
            Action::Execute,
            State::EscapeIntermediate,
        );
        table.add(
            0x1C..=0x1F,
            State::EscapeIntermediate,
            Action::Execute,
            State::EscapeIntermediate,
        );
        table.add(
            0x20..=0x2F,
            State::EscapeIntermediate,
            Action::Collect,
            State::EscapeIntermediate,
        );
        table.add(
            0x7F,
            State::EscapeIntermediate,
            Action::Ignore,
            State::EscapeIntermediate,
        );
        table.add(
            0x30..=0x7E,
            State::EscapeIntermediate,
            Action::Dispatch,
            State::Ground,
        );

        // State::CsiEntry
        table.add_enter(State::CsiEntry, Action::Clear);
        table.add(
            0x00..=0x17,
            State::CsiEntry,
            Action::Execute,
            State::CsiEntry,
        );
        table.add(0x19, State::CsiEntry, Action::Execute, State::CsiEntry);
        table.add(
            0x1C..=0x1F,
            State::CsiEntry,
            Action::Execute,
            State::CsiEntry,
        );
        table.add(0x7F, State::CsiEntry, Action::Ignore, State::CsiEntry);
        table.add(
            0x20..=0x2F,
            State::CsiEntry,
            Action::Collect,
            State::CsiIntermediate,
        );
        table.add(0x3A, State::CsiEntry, Action::None, State::CsiIgnore);
        table.add(0x30..=0x39, State::CsiEntry, Action::Param, State::CsiParam);
        table.add(0x3B, State::CsiEntry, Action::Param, State::CsiParam);
        table.add(
            0x3C..=0x3F,
            State::CsiEntry,
            Action::Collect,
            State::CsiParam,
        );
        table.add(
            0x40..=0x7E,
            State::CsiEntry,
            Action::Dispatch,
            State::Ground,
        );

        // State::CsiIgnore
        table.add(
            0x00..=0x17,
            State::CsiIgnore,
            Action::Execute,
            State::CsiIgnore,
        );
        table.add(0x19, State::CsiIgnore, Action::Execute, State::CsiIgnore);
        table.add(
            0x1C..=0x1F,
            State::CsiIgnore,
            Action::Execute,
            State::CsiIgnore,
        );
        table.add(
            0x20..=0x3F,
            State::CsiIgnore,
            Action::Ignore,
            State::CsiIgnore,
        );
        table.add(0x7F, State::CsiIgnore, Action::Ignore, State::CsiIgnore);
        table.add(0x40..=0x7E, State::CsiIgnore, Action::None, State::Ground);

        // State::CsiParam
        table.add(
            0x00..=0x17,
            State::CsiParam,
            Action::Execute,
            State::CsiParam,
        );
        table.add(0x19, State::CsiParam, Action::Execute, State::CsiParam);
        table.add(
            0x1C..=0x1F,
            State::CsiParam,
            Action::Execute,
            State::CsiParam,
        );
        table.add(0x30..=0x39, State::CsiParam, Action::Param, State::CsiParam);
        table.add(0x3A, State::CsiParam, Action::Param, State::CsiParam);
        table.add(0x3B, State::CsiParam, Action::Param, State::CsiParam);
        table.add(0x7F, State::CsiParam, Action::Ignore, State::CsiParam);
        table.add(0x3C..=0x3F, State::CsiParam, Action::None, State::CsiIgnore);
        table.add(
            0x20..=0x2F,
            State::CsiParam,
            Action::Collect,
            State::CsiIntermediate,
        );
        table.add(
            0x40..=0x7E,
            State::CsiParam,
            Action::Dispatch,
            State::Ground,
        );

        // State::CsiIntermediate
        table.add(
            0x00..=0x17,
            State::CsiIntermediate,
            Action::Execute,
            State::CsiIntermediate,
        );
        table.add(
            0x19,
            State::CsiIntermediate,
            Action::Execute,
            State::CsiIntermediate,
        );
        table.add(
            0x1C..=0x1F,
            State::CsiIntermediate,
            Action::Execute,
            State::CsiIntermediate,
        );
        table.add(
            0x20..=0x2F,
            State::CsiIntermediate,
            Action::Collect,
            State::CsiIntermediate,
        );
        table.ignore(0x7F, State::CsiIntermediate);
        table.add(
            0x30..=0x3F,
            State::CsiIntermediate,
            Action::None,
            State::CsiIgnore,
        );
        table.add(
            0x40..=0x7E,
            State::CsiIntermediate,
            Action::Dispatch,
            State::Ground,
        );

        // State::DcsEntry
        table.add_enter(State::DcsEntry, Action::Clear);
        table.add(
            0x00..=0x17,
            State::DcsEntry,
            Action::Ignore,
            State::DcsEntry,
        );
        table.add(0x19, State::DcsEntry, Action::Ignore, State::DcsEntry);
        table.add(
            0x1C..=0x1F,
            State::DcsEntry,
            Action::Ignore,
            State::DcsEntry,
        );
        table.add(0x7F, State::DcsEntry, Action::Ignore, State::DcsEntry);
        table.add(0x3A, State::DcsEntry, Action::None, State::DcsIgnore);
        table.add(
            0x20..=0x2F,
            State::DcsEntry,
            Action::Collect,
            State::DcsIntermediate,
        );
        table.add(0x30..=0x39, State::DcsEntry, Action::Param, State::DcsParam);
        table.add(0x3B, State::DcsEntry, Action::Param, State::DcsParam);
        table.add(
            0x3C..=0x3F,
            State::DcsEntry,
            Action::Collect,
            State::DcsParam,
        );
        table.add(0x40..=0x7E, State::DcsEntry, Action::None, State::DcsData);

        // State::DcsIntermediate
        table.add(
            0x00..=0x17,
            State::DcsIntermediate,
            Action::Ignore,
            State::DcsIntermediate,
        );
        table.add(
            0x19,
            State::DcsIntermediate,
            Action::Ignore,
            State::DcsIntermediate,
        );
        table.add(
            0x1C..=0x1F,
            State::DcsIntermediate,
            Action::Ignore,
            State::DcsIntermediate,
        );
        table.add(
            0x20..=0x2F,
            State::DcsIntermediate,
            Action::Collect,
            State::DcsIntermediate,
        );
        table.ignore(0x7F, State::DcsIntermediate);
        table.add(
            0x30..=0x3F,
            State::DcsIntermediate,
            Action::None,
            State::DcsIgnore,
        );
        table.add(
            0x40..=0x7E,
            State::DcsIntermediate,
            Action::None,
            State::DcsData,
        );

        // State::DcsIgnore
        table.add(
            0x00..=0x17,
            State::DcsIgnore,
            Action::Ignore,
            State::DcsIgnore,
        );
        table.add(0x19, State::DcsIgnore, Action::Ignore, State::DcsIgnore);
        table.add(
            0x1C..=0x1F,
            State::DcsIgnore,
            Action::Ignore,
            State::DcsIgnore,
        );
        table.add(
            0x20..=0x7F,
            State::DcsIgnore,
            Action::Ignore,
            State::DcsIgnore,
        );

        // State::DcsParam
        table.add(
            0x00..=0x17,
            State::DcsParam,
            Action::Ignore,
            State::DcsParam,
        );
        table.add(0x19, State::DcsParam, Action::Ignore, State::DcsParam);
        table.add(
            0x1C..=0x1F,
            State::DcsParam,
            Action::Ignore,
            State::DcsParam,
        );
        table.add(0x30..=0x39, State::DcsParam, Action::Param, State::DcsParam);
        table.add(0x3B, State::DcsParam, Action::Param, State::DcsParam);
        table.add(0x7F, State::DcsParam, Action::Ignore, State::DcsParam);
        table.add(0x3A, State::DcsParam, Action::None, State::DcsIgnore);
        table.add(0x3C..=0x3F, State::DcsParam, Action::None, State::DcsIgnore);
        table.add(
            0x20..=0x2F,
            State::DcsParam,
            Action::Collect,
            State::DcsIntermediate,
        );
        table.add(0x40..=0x7E, State::DcsParam, Action::None, State::DcsData);

        // State::DcsData
        table.add(0x00..=0x17, State::DcsData, Action::Record, State::DcsData);
        table.add(0x19, State::DcsData, Action::Record, State::DcsData);
        table.add(0x1C..=0x1F, State::DcsData, Action::Record, State::DcsData);
        table.add(0x20..=0x7E, State::DcsData, Action::Record, State::DcsData);
        table.add(0x7F, State::DcsData, Action::Ignore, State::DcsData);

        // State::OscData
        table.add(0x00..=0x17, State::OscData, Action::Ignore, State::OscData);
        table.add(0x19, State::OscData, Action::Ignore, State::OscData);
        table.add(0x1C..=0x1F, State::OscData, Action::Ignore, State::OscData);
        table.add(0x20..=0x7F, State::OscData, Action::Record, State::OscData);

        // String-type passthrough states that carry no params / intermediates.
        each!(SosData, PmData, ApcData => |state| {
            table.add(0x00..=0x17, state, Action::Ignore, state);
            table.add(0x19, state, Action::Ignore, state);
            table.add(0x1C..=0x1F, state, Action::Ignore, state);
            table.add(0x20..=0x7F, state, Action::Record, state);
        });


        // UTF-8 passthrough for every string-data state. Overrides the C1
        // anywhere rules for 0x80..=0x9F — without this, a continuation byte
        // in that range (e.g. 0x9F in the 🦀 encoding `F0 9F A6 80`) would
        // fire an APC/SOS/etc. anywhere transition and shred the payload.
        // ST (0x9C) is re-bound afterwards so it still terminates the string.
        each!(DcsData, OscData, SosData, PmData, ApcData => |state| {
            table.add(0x80..=0xFF, state, Action::Record, state);
            table.add(0x9C, state, Action::None, State::Ground);
        });

        table
    }

    pub fn empty() -> Self {
        Table {
            inner: [u16::default(); _],
            enter: [Action::None; _],
            exit: [Action::None; _],
        }
    }

    pub fn add_enter(&mut self, state: State, action: Action) {
        self.enter[state as usize] = action;
    }

    pub fn add_exit(&mut self, state: State, action: Action) {
        self.exit[state as usize] = action;
    }

    pub fn ignore<Bytes>(&mut self, bytes: Bytes, state: State) where Self: [const] Transition<Bytes> {
        Transition::add(self, bytes, state, Action::Ignore, state);
    }

    pub fn transition(&self, state: State, byte: u8) -> (State, Action) {
        let index = Self::index(byte, state);
        let value = self.inner[index];
        (
            State::from((value & Self::TRANSITION_STATE_MASK) as u8),
            Action::from((value >> Self::TRANSITION_ACTION_SHIFT) as u8),
        )
    }

    pub fn on_enter(&self, state: State) -> Action {
        self.enter[state as usize]
    }

    pub fn on_exit(&self, state: State) -> Action {
        self.exit[state as usize]
    }

    pub fn index(byte: u8, state: State) -> usize {
        const INDEX_STATE_SHIFT: usize = 8;

        ((state as usize) << INDEX_STATE_SHIFT) | (byte as usize)
    }

    pub fn value(action: Action, next: State) -> u16 {
        const TRANSITION_ACTION_SHIFT: usize = 5;
        const TRANSITION_STATE_MASK: u16 = 0x1F; // 5 bits for State
        ((action as u16) << TRANSITION_ACTION_SHIFT) | (next as u16)
    }
}

impl const Index<usize> for Table {
    type Output = u16;
    fn index(&self, index: usize) -> &Self::Output {
        &self.inner[index]
    }
}
impl const IndexMut<usize> for Table {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.inner[index]
    }
}
impl Default for Table {
    fn default() -> Self {
        Self::default()
    }
}

