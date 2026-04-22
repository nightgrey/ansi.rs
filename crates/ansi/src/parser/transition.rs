use super::{Action, State};
use derive_more::{Deref, DerefMut, Index, IndexMut};
use std::ops;
type Uint = usize;

// Table values are generated like this:
//
// Index:  State << IndexStateShift | Byte
// Value:  Action << TransitionActionShift | NextState
const TRANSITION_ACTION_SHIFT: Uint = 4;
const TRANSITION_STATE_MASK: Uint = State::COUNT as Uint - 1;
const INDEX_STATE_SHIFT: usize = 8;
const DEFAULT_TABLE_SIZE: usize = State::COUNT * 256;

/// DEC ANSI transition table
///
/// https://vt100.net/emu/dec_ansi_parser
#[derive(Deref, DerefMut, Index, IndexMut)]
pub struct TransitionTable<const N: usize = DEFAULT_TABLE_SIZE>([Uint; N]);

impl<const N: usize> TransitionTable<N> {
    pub fn new() -> Self {
        Self([0; N])
    }

    pub fn set_default(&mut self, action: Action, state: State) {
        let value = (action as Uint) << TRANSITION_ACTION_SHIFT | (state as Uint);
        self.0.fill(value);
    }

    pub fn add(&mut self, byte: u8, state: State, action: Action, next: State) {
        let idx = ((state as Uint) << INDEX_STATE_SHIFT) | (byte as Uint);
        let value = (action as Uint) << TRANSITION_ACTION_SHIFT | (next as Uint);
        self.0[idx as usize] = value;
    }

    pub fn add_many(&mut self, bytes: &[u8], state: State, action: Action, next: State) {
        bytes.iter().for_each(|&code| {
            self.add(code, state, action, next);
        });
    }

    pub fn add_range(
        &mut self,
        bytes: ops::RangeInclusive<u8>,
        state: State,
        action: Action,
        next: State,
    ) {
        bytes.for_each(|code| {
            self.add(code, state, action, next);
        });
    }

    pub fn transition(&self, state: State, byte: u8) -> (State, Action) {
        let index = ((state as usize) << INDEX_STATE_SHIFT) | (byte as usize);
        let value = self.0[index];
        (
            State::from((value & TRANSITION_STATE_MASK) as u8),
            Action::from((value >> TRANSITION_ACTION_SHIFT) as u8),
        )
    }
}

impl TransitionTable<DEFAULT_TABLE_SIZE> {
    pub fn default() -> Self {
        use Action::*;
        use State::*;

        let mut table = Self::new();
        table.set_default(None, Ground);

        for state in [
            Ground,
            CsiEntry,
            CsiIntermediate,
            CsiParam,
            DcsEntry,
            DcsIntermediate,
            DcsParam,
            DcsData,
            Escape,
            EscapeIntermediate,
            ApcData,
            Utf8,
        ] {
            // Anywhere -> Ground
            table.add_many(&[0x18, 0x1a, 0x99, 0x9a], state, Execute, Ground);
            table.add_range(0x80..=0x8F, state, Execute, Ground);
            table.add_range(0x90..=0x97, state, Execute, Ground);
            table.add(0x9C, state, Execute, Ground);
            // Anywhere -> Escape
            table.add(0x1B, state, Clear, Escape);
            // Anywhere -> SosStringState
            // Anywhere -> ApcStringState
            table.add(0x9F, state, Data, ApcData);
            // Anywhere -> CsiEntry
            table.add(0x9B, state, Clear, CsiEntry);
            // Anywhere -> DcsEntry
            table.add(0x90, state, Clear, DcsEntry);
            // Anywhere -> OscString
            // Anywhere -> Utf8
            table.add_range(0xC2..=0xDF, state, Collect, Utf8); // UTF8 2 byte sequence
            table.add_range(0xE0..=0xEF, state, Collect, Utf8); // UTF8 3 byte sequence
            table.add_range(0xF0..=0xF4, state, Collect, Utf8); // UTF8 4 byte sequence
        }

        // Ground
        table.add_range(0x00..=0x17, Ground, Execute, Ground);
        table.add(0x19, Ground, Execute, Ground);
        table.add_range(0x1C..=0x1F, Ground, Execute, Ground);
        table.add_range(0x20..=0x7E, Ground, Print, Ground);
        table.add(0x7F, Ground, Execute, Ground);

        // EscapeIntermediate
        table.add_range(0x00..=0x17, EscapeIntermediate, Execute, EscapeIntermediate);
        table.add(0x19, EscapeIntermediate, Execute, EscapeIntermediate);
        table.add_range(0x1C..=0x1F, EscapeIntermediate, Execute, EscapeIntermediate);
        table.add_range(0x20..=0x2F, EscapeIntermediate, Collect, EscapeIntermediate);
        table.add(0x7F, EscapeIntermediate, Ignore, EscapeIntermediate);
        // EscapeIntermediate -> Ground
        table.add_range(0x30..=0x7E, EscapeIntermediate, Dispatch, Ground);

        // Escape
        table.add_range(0x00..=0x17, Escape, Execute, Escape);
        table.add(0x19, Escape, Execute, Escape);
        table.add_range(0x1C..=0x1F, Escape, Execute, Escape);
        table.add(0x7F, Escape, Ignore, Escape);
        // Escape -> Ground
        table.add_range(0x30..=0x4F, Escape, Dispatch, Ground);
        table.add_range(0x51..=0x57, Escape, Dispatch, Ground);
        table.add(0x59, Escape, Dispatch, Ground);
        table.add(0x5A, Escape, Dispatch, Ground);
        table.add(0x5C, Escape, Dispatch, Ground);
        table.add_range(0x60..=0x7E, Escape, Dispatch, Ground);
        // Escape -> Escape_intermediate
        table.add_range(0x20..=0x2F, Escape, Collect, EscapeIntermediate);
        // Escape -> Data
        table.add(b'_', Escape, Data, ApcData); // APC
        // Escape -> Dcs_entry
        table.add(b'P', Escape, Clear, DcsEntry);
        // Escape -> Csi_entry
        table.add(b'[', Escape, Clear, CsiEntry);
        // Escape -> Osc_string

        // Dcs_entry
        table.add_range(0x00..=0x07, DcsEntry, Ignore, DcsEntry);
        table.add_range(0x0E..=0x17, DcsEntry, Ignore, DcsEntry);
        table.add(0x19, DcsEntry, Ignore, DcsEntry);
        table.add_range(0x1C..=0x1F, DcsEntry, Ignore, DcsEntry);
        table.add(0x7F, DcsEntry, Ignore, DcsEntry);
        // Dcs_entry -> Dcs_intermediate
        table.add_range(0x20..=0x2F, DcsEntry, Collect, DcsIntermediate);
        // Dcs_entry -> Dcs_param
        table.add_range(0x30..=0x3B, DcsEntry, Param, DcsParam);
        table.add_range(0x3C..=0x3F, DcsEntry, Prefix, DcsParam);
        // Dcs_entry -> Dcs_passthrough
        table.add_range(0x08..=0x0D, DcsEntry, Put, DcsData); // Follows ECMA-48 § 8.3.27
        // XXX: allows passing ESC (not a ECMA-48 standard); this to allow for
        // passthrough of ANSI sequences like in Screen or Tmux passthrough mode.
        table.add(0x1B, DcsEntry, Put, DcsData);
        table.add_range(0x40..=0x7E, DcsEntry, Data, DcsData);

        // Dcs_intermediate
        table.add_range(0x00..=0x17, DcsIntermediate, Ignore, DcsIntermediate);
        table.add(0x19, DcsIntermediate, Ignore, DcsIntermediate);
        table.add_range(0x1C..=0x1F, DcsIntermediate, Ignore, DcsIntermediate);
        table.add_range(0x20..=0x2F, DcsIntermediate, Collect, DcsIntermediate);
        table.add(0x7F, DcsIntermediate, Ignore, DcsIntermediate);
        // Dcs_intermediate -> Dcs_passthrough
        table.add_range(0x30..=0x3F, DcsIntermediate, Data, DcsData);
        table.add_range(0x40..=0x7E, DcsIntermediate, Data, DcsData);

        // Dcs_param
        table.add_range(0x00..=0x17, DcsParam, Ignore, DcsParam);
        table.add(0x19, DcsParam, Ignore, DcsParam);
        table.add_range(0x1C..=0x1F, DcsParam, Ignore, DcsParam);
        table.add_range(0x30..=0x3B, DcsParam, Param, DcsParam);
        table.add(0x7F, DcsParam, Ignore, DcsParam);
        table.add_range(0x3C..=0x3F, DcsParam, Ignore, DcsParam);
        // Dcs_param -> Dcs_intermediate
        table.add_range(0x20..=0x2F, DcsParam, Collect, DcsIntermediate);
        // Dcs_param -> Dcs_passthrough
        table.add_range(0x40..=0x7E, DcsParam, Data, DcsData);

        // Dcs_passthrough
        table.add_range(0x00..=0x17, DcsData, Put, DcsData);
        table.add(0x19, DcsData, Put, DcsData);
        table.add_range(0x1C..=0x1F, DcsData, Put, DcsData);
        table.add_range(0x20..=0x7E, DcsData, Put, DcsData);
        table.add(0x7F, DcsData, Put, DcsData);
        table.add_range(0x80..=0xFF, DcsData, Put, DcsData); // Allow Utf8 characters by extending the printable range from 0x7F to 0xFF
        // ST, CAN, SUB, and ESC terminate the sequence
        table.add(0x1B, DcsData, Dispatch, Escape);
        table.add(0x9C, DcsData, Dispatch, Ground);
        table.add_many(&[0x18, 0x1A], DcsData, Ignore, Ground);

        // Csi_param
        table.add_range(0x00..=0x17, CsiParam, Execute, CsiParam);
        table.add(0x19, CsiParam, Execute, CsiParam);
        table.add_range(0x1C..=0x1F, CsiParam, Execute, CsiParam);
        table.add_range(0x30..=0x3B, CsiParam, Param, CsiParam);
        table.add(0x7F, CsiParam, Ignore, CsiParam);
        table.add_range(0x3C..=0x3F, CsiParam, Ignore, CsiParam);
        // Csi_param -> Ground
        table.add_range(0x40..=0x7E, CsiParam, Dispatch, Ground);
        // Csi_param -> Csi_intermediate
        table.add_range(0x20..=0x2F, CsiParam, Collect, CsiIntermediate);

        // Csi_intermediate
        table.add_range(0x00..=0x17, CsiIntermediate, Execute, CsiIntermediate);
        table.add(0x19, CsiIntermediate, Execute, CsiIntermediate);
        table.add_range(0x1C..=0x1F, CsiIntermediate, Execute, CsiIntermediate);
        table.add_range(0x20..=0x2F, CsiIntermediate, Collect, CsiIntermediate);
        table.add(0x7F, CsiIntermediate, Ignore, CsiIntermediate);
        // Csi_intermediate -> Ground
        table.add_range(0x40..=0x7E, CsiIntermediate, Dispatch, Ground);
        // Csi_intermediate -> Csi_ignore
        table.add_range(0x30..=0x3F, CsiIntermediate, Ignore, Ground);

        // Csi_entry
        table.add_range(0x00..=0x17, CsiEntry, Execute, CsiEntry);
        table.add(0x19, CsiEntry, Execute, CsiEntry);
        table.add_range(0x1C..=0x1F, CsiEntry, Execute, CsiEntry);
        table.add(0x7F, CsiEntry, Ignore, CsiEntry);

        // Csi_entry -> Ground
        table.add_range(0x40..=0x4C, CsiEntry, Dispatch, Ground);
        // Csi_entry -> Ground
        table.add_range(0x4E..=0x7E, CsiEntry, Dispatch, Ground);

        // Csi_xterm
        // table.add_range(0x00..=0xFF, XtermParam, Collect, XtermParam);

        // Csi_entry -> Csi_intermediate
        table.add_range(0x20..=0x2F, CsiEntry, Collect, CsiIntermediate);
        // Csi_entry -> Csi_param
        table.add_range(0x30..=0x3B, CsiEntry, Param, CsiParam);
        table.add_range(0x3C..=0x3F, CsiEntry, Prefix, CsiParam);
        table
    }

    pub fn global() -> &'static TransitionTable {
        use std::sync::OnceLock;
        static TABLE: OnceLock<TransitionTable> = OnceLock::new();
        TABLE.get_or_init(TransitionTable::default)
    }
}

impl Default for TransitionTable<DEFAULT_TABLE_SIZE> {
    fn default() -> Self {
        Self::default()
    }
}
