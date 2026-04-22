use std::fmt::Debug;
use std::ops::RangeInclusive;

// Table values are encoded as:
//
//   inner index:   (state << INDEX_STATE_SHIFT) | byte
//   inner value:   (action << TRANSITION_ACTION_SHIFT) | next_state
//
// `State` uses 5 bits (17 states); `Action` takes the remaining bits.
//
// "No transition" is encoded as `next_state == current_state`, so each row
// is initialised to point back at itself with `Action::None`. Real
// transitions are detected with `next_state != prev_state`, mirroring the
// `if(new_state)` check in the C reference.
const TRANSITION_ACTION_SHIFT: usize = 5;
const TRANSITION_STATE_MASK: usize = 0x1F;
const INDEX_STATE_SHIFT: usize = 8;
const DEFAULT_TABLE_SIZE: usize = State::COUNT * 256;

/// DEC ANSI transition table
///
/// https://vt100.net/emu/dec_ansi_parser
#[derive(Clone, Debug)]
pub struct Table {
    inner: [usize; DEFAULT_TABLE_SIZE],
    entry: [Action; State::COUNT],
    exit: [Action; State::COUNT],
}

impl Table {
    pub fn add(&mut self, value: impl TransitionValue, state: State, action: Action, next: State) {
        value.add(self, state, action, next);
    }

    pub fn add_entry(&mut self, state: State, action: Action) {
        self.entry[state as usize] = action;
    }

    pub fn add_exit(&mut self, state: State, action: Action) {
        self.exit[state as usize] = action;
    }

    pub fn ignore(&mut self, value: impl TransitionValue, state: State) {
        value.add(self, state, Action::Ignore, state);
    }

    pub fn transition(&self, state: State, byte: u8) -> (State, Action) {
        let index = ((state as usize) << INDEX_STATE_SHIFT) | (byte as usize);
        let value = self.inner[index];
        (
            State::from((value & TRANSITION_STATE_MASK) as u8),
            Action::from((value >> TRANSITION_ACTION_SHIFT) as u8),
        )
    }

    pub fn entry(&self, state: State) -> Action {
        self.entry[state as usize]
    }

    pub fn exit(&self, state: State) -> Action {
        self.exit[state as usize]
    }

    pub fn range(range: RangeInclusive<State>) -> impl Iterator<Item = State> {
        ((*range.start() as u8)..=(*range.end() as u8)).map(State::from)
    }
}

impl Table {
    pub fn default() -> Self {
        let mut table = Table {
            inner: [0; DEFAULT_TABLE_SIZE],
            entry: [Action::None; State::COUNT],
            exit: [Action::None; State::COUNT],
        };

        // Initialise every row so unset bytes mean "stay in current state,
        // no action". This makes `next_state != prev_state` a faithful
        // proxy for the C reference's `if(new_state)` check.
        for state in Table::range(State::Ground..=State::Utf8) {
            for byte in 0..=255 {
                table.add(byte, state, Action::None, state);
            }
        }

        // "Anywhere" transitions — apply to every state.
        for state in Table::range(State::Ground..=State::Utf8) {
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
        }

        // State::Ground
        table.add(0x00..=0x17, State::Ground, Action::Execute, State::Ground);
        table.add(0x19, State::Ground, Action::Execute, State::Ground);
        table.add(0x1C..=0x1F, State::Ground, Action::Execute, State::Ground);
        table.add(0x20..=0x7F, State::Ground, Action::Print, State::Ground);
        table.add(0xC2..=0xF4, State::Ground, Action::Utf8Collect, State::Utf8);

        // State::Utf8 — accumulate continuation bytes. `Utf8Collect` manually
        // resets to Ground once the codepoint is complete (same-state
        // transition, so no exit/entry fires). The exit action is Clear so
        // that an abort via anywhere rules (ESC, CAN, SUB, …) drops the
        // half-decoded bytes instead of leaving them in `self.utf8`.
        table.add(0x80..=0xBF, State::Utf8, Action::Utf8Collect, State::Utf8);
        table.add_exit(State::Utf8, Action::Clear);

        // State::Escape
        table.add_entry(State::Escape, Action::Clear);
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
        table.add_entry(State::CsiEntry, Action::Clear);
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
        table.add_entry(State::DcsEntry, Action::Clear);
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
        table.add_entry(State::DcsData, Action::DataStart);
        table.add(0x00..=0x17, State::DcsData, Action::Record, State::DcsData);
        table.add(0x19, State::DcsData, Action::Record, State::DcsData);
        table.add(0x1C..=0x1F, State::DcsData, Action::Record, State::DcsData);
        table.add(0x20..=0x7E, State::DcsData, Action::Record, State::DcsData);
        table.add(0x7F, State::DcsData, Action::Ignore, State::DcsData);
        table.add_exit(State::DcsData, Action::DataEnd);

        // State::OscData
        table.add_entry(State::OscData, Action::DataStart);
        table.add(0x00..=0x17, State::OscData, Action::Ignore, State::OscData);
        table.add(0x19, State::OscData, Action::Ignore, State::OscData);
        table.add(0x1C..=0x1F, State::OscData, Action::Ignore, State::OscData);
        table.add(0x20..=0x7F, State::OscData, Action::Record, State::OscData);
        table.add_exit(State::OscData, Action::DataEnd);

        // String-type passthrough states that carry no params / intermediates.
        for state in [State::SosData, State::PmData, State::ApcData] {
            table.add_entry(state, Action::DataStart);
            table.add(0x00..=0x17, state, Action::Ignore, state);
            table.add(0x19, state, Action::Ignore, state);
            table.add(0x1C..=0x1F, state, Action::Ignore, state);
            table.add(0x20..=0x7F, state, Action::Record, state);
            table.add_exit(state, Action::DataEnd);
        }

        // UTF-8 passthrough for every string-data state. Overrides the C1
        // anywhere rules for 0x80..=0x9F — without this, a continuation byte
        // in that range (e.g. 0x9F in the 🦀 encoding `F0 9F A6 80`) would
        // fire an APC/SOS/etc. anywhere transition and shred the payload.
        // ST (0x9C) is re-bound afterwards so it still terminates the string.
        for state in [
            State::DcsData,
            State::OscData,
            State::SosData,
            State::PmData,
            State::ApcData,
        ] {
            table.add(0x80..=0xFF, state, Action::Record, state);
            table.add(0x9C, state, Action::None, State::Ground);
        }

        table
    }

    pub fn global_transition(state: State, byte: u8) -> (State, Action) {
        Self::global().transition(state, byte)
    }

    pub fn global() -> &'static Table {
        use std::sync::OnceLock;
        static TABLE: OnceLock<Table> = OnceLock::new();
        TABLE.get_or_init(Table::default)
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::default()
    }
}

pub trait TransitionValue {
    fn add(self, table: &mut Table, state: State, action: Action, next: State);
}

impl TransitionValue for u8 {
    fn add(self, table: &mut Table, state: State, action: Action, next: State) {
        let idx = ((state as usize) << INDEX_STATE_SHIFT) | (self as usize);
        let value = (action as usize) << TRANSITION_ACTION_SHIFT | (next as usize);
        table.inner[idx] = value;
    }
}

impl<const T: usize> TransitionValue for &[u8; T] {
    fn add(self, table: &mut Table, state: State, action: Action, next: State) {
        self.iter().for_each(|&code| {
            table.add(code, state, action, next);
        });
    }
}

impl TransitionValue for RangeInclusive<u8> {
    fn add(self, table: &mut Table, state: State, action: Action, next: State) {
        self.for_each(|code| {
            table.add(code, state, action, next);
        });
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Default)]
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
    SosData,
    /// PM (Private-Message) string data.
    PmData,
    /// APC (Application Program Command) string data.
    ApcData,

    /// UTF-8 decoder state. Bytes are assembled into complete codepoints before being
    /// emitted; invalid sequences are replaced with U+FFFD and the decoder resumes.
    Utf8,
}

impl State {
    pub const COUNT: usize = Self::Utf8 as usize + 1;
}

impl From<u8> for State {
    fn from(value: u8) -> Self {
        if value as usize >= Self::COUNT {
            panic!("State value {value} out of range")
        }
        unsafe { std::mem::transmute(value) }
    }
}

impl Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Ground => f.write_str("State::Ground"),
            State::Escape => f.write_str("State::Escape"),
            State::EscapeIntermediate => f.write_str("State::EscapeIntermediate"),
            State::CsiEntry => f.write_str("State::CsiEntry"),
            State::CsiIgnore => f.write_str("State::CsiIgnore"),
            State::CsiParam => f.write_str("State::CsiParam"),
            State::CsiIntermediate => f.write_str("State::CsiIntermediate"),
            State::DcsEntry => f.write_str("State::DcsEntry"),
            State::DcsIgnore => f.write_str("State::DcsIgnore"),
            State::DcsIntermediate => f.write_str("State::DcsIntermediate"),
            State::DcsParam => f.write_str("State::DcsParam"),
            State::DcsData => f.write_str("State::DcsData"),
            State::OscData => f.write_str("State::OscData"),
            State::SosData => f.write_str("State::SosData"),
            State::PmData => f.write_str("State::PmData"),
            State::ApcData => f.write_str("State::ApcData"),
            State::Utf8 => f.write_str("State::Utf8"),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Default)]
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
    /// Collect a private prefix of a control sequence.
    Prefix,
    /// In `Ground`, map the code to a glyph and display it.
    Print,

    /// A string-type data phase (DCS / OSC / SOS / PM / APC) has begun.
    /// Fired on entry to the corresponding data state; the handler is
    /// selected from `self.state` at `DataEnd` time.
    DataStart,
    /// A string-type data phase has ended — dispatched to the matching
    /// handler based on the state being left (exit actions fire before
    /// `self.state` is updated, so it still reads as `prev_state`).
    DataEnd,

    /// Append the current byte to the data buffer.
    Record,

    /// Accumulate a UTF-8 byte into `self.utf8`. Emits the codepoint and
    /// returns to `Ground` once enough continuation bytes have arrived.
    Utf8Collect,
}

impl Action {
    pub const COUNT: usize = Self::Utf8Collect as usize + 1;
}

impl From<u8> for Action {
    fn from(value: u8) -> Self {
        if value as usize >= Self::COUNT {
            panic!("Action value {value} out of range")
        }
        unsafe { std::mem::transmute(value) }
    }
}

impl Debug for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::None => f.write_str("Action::None"),
            Action::Clear => f.write_str("Action::Clear"),
            Action::Collect => f.write_str("Action::Collect"),
            Action::Dispatch => f.write_str("Action::Dispatch"),
            Action::Execute => f.write_str("Action::Execute"),
            Action::Ignore => f.write_str("Action::Ignore"),
            Action::Param => f.write_str("Action::Param"),
            Action::Prefix => f.write_str("Action::Prefix"),
            Action::Print => f.write_str("Action::Print"),
            Action::DataStart => f.write_str("Action::DataStart"),
            Action::DataEnd => f.write_str("Action::DataEnd"),
            Action::Record => f.write_str("Action::Record"),
            Action::Utf8Collect => f.write_str("Action::Utf8Collect"),
        }
    }
}
