use derive_more::{Deref, DerefMut, Index, IndexMut};
use std::fmt::Debug;
use std::ops;
use std::ops::{Range, RangeInclusive};

// Table values are generated like this:
//
// Index:  State << IndexStateShift | Byte
// Value:  Action << TransitionActionShift | NextState
const TRANSITION_ACTION_SHIFT: usize = 4;
const TRANSITION_STATE_MASK: usize = State::COUNT as usize - 1;
const INDEX_STATE_SHIFT: usize = 8;
const DEFAULT_TABLE_SIZE: usize = State::COUNT * 256;

/// DEC ANSI transition table
///
/// https://vt100.net/emu/dec_ansi_parser
#[derive(Clone, Debug)]
pub struct Table {
    inner: [usize; DEFAULT_TABLE_SIZE],
    entry: [usize; DEFAULT_TABLE_SIZE],
    exit: [usize; DEFAULT_TABLE_SIZE],
}

impl Table {
    pub fn fill(&mut self, action: Action, state: State) {
        let value = (action as usize) << TRANSITION_ACTION_SHIFT | (state as usize);
        self.inner.fill(value);
    }

    pub fn add_entry(&mut self, state: State, action: Action) {
        let idx = ((state as usize) << INDEX_STATE_SHIFT) | 0 as usize;
        let value = (action as usize) << TRANSITION_ACTION_SHIFT | (state as usize);
        self.entry[idx as usize] = value;
    }

    pub fn add(&mut self, value: impl TransitionValue, state: State, action: Action, next: State) {
        value.add(self, state, action, next);
    }

    pub fn add_exit(&mut self, state: State, action: Action) {
        let idx = ((state as usize) << INDEX_STATE_SHIFT) | 0 as usize;
        let value = (action as usize) << TRANSITION_ACTION_SHIFT | (state as usize);
        self.exit[idx as usize] = value;
    }

    pub fn ignore(&mut self, value: impl TransitionValue, state: State) {
        value.add(self, state, Action::Ignore, state);
    }

    pub fn transition(&self, state: State, char: u8) -> (State, Action) {
        let index = ((state as usize) << INDEX_STATE_SHIFT) | (char as usize);
        let value = self.inner[index];
        (
            State::from((value & TRANSITION_STATE_MASK) as u8),
            Action::from((value >> TRANSITION_ACTION_SHIFT) as u8),
        )
    }

    pub fn entry(&self, state: State, char: u8) -> Action {
        let index = ((state as usize) << INDEX_STATE_SHIFT) | (0 as usize);
        let value = self.entry[index];

        Action::from((value >> TRANSITION_ACTION_SHIFT) as u8)
    }

    pub fn exit(&self, state: State, char: u8) -> Action {
        let index = ((state as usize) << INDEX_STATE_SHIFT) | (0 as usize);
        let value = self.exit[index];

        Action::from((value >> TRANSITION_ACTION_SHIFT) as u8)
    }

    pub fn range(range: RangeInclusive<State>) -> impl Iterator<Item = State> {
        ((*range.start() as u8)..=(*range.end() as u8)).map(State::from)
    }
}

impl Table {
    pub fn default() -> Self {
        let mut table = Table {
            inner: [0; DEFAULT_TABLE_SIZE],
            entry: [0; DEFAULT_TABLE_SIZE],
            exit: [0; DEFAULT_TABLE_SIZE],
        };

        table.fill(Action::None, State::Ground);

        for state in Table::range(State::Ground..=State::Utf8) {
            table.add(0x18, state, Action::Execute, State::Ground);
            table.add(0x18, state, Action::Execute, State::Ground);
            table.add(0x80..=0x8f, state, Action::Execute, State::Ground);
            table.add(0x91..=0x97, state, Action::Execute, State::Ground);
            table.add(0x99, state, Action::Execute, State::Ground);
            table.add(0x9a, state, Action::Execute, State::Ground);
            table.add(0x9c, state, Action::None, State::Ground);

            table.add(0x1B, state, Action::None, State::Escape);

            table.add(0x98, state, Action::None, State::Data);
            table.add(0x9E, state, Action::None, State::Data);
            table.add(0x9F, state, Action::None, State::Data);

            table.add(0x90, state, Action::None, State::DcsEntry);
            table.add(0x9D, state, Action::None, State::OscData);
            table.add(0x9B, state, Action::None, State::CsiEntry);

            // Utf8
            // table.add(0xC2..=0xDF, state, Action::Collect, State::Utf8); // UTF8 2 byte sequence
            // table.add(0xE0..=0xEF, state, Action::Collect, State::Utf8); // UTF8 3 byte sequence
            // table.add(0xF0..=0xF4, state, Action::Collect, State::Utf8); // UTF8 4 byte sequence
        }

        // State::Ground
        table.add(0x00..=0x17, State::Ground, Action::Execute, State::Ground);
        table.add(0x19, State::Ground, Action::Execute, State::Ground);
        table.add(0x1C..=0x1F, State::Ground, Action::Execute, State::Ground);
        table.add(0x20..=0x7F, State::Ground, Action::Print, State::Ground);

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
        table.add(0x5b, State::Escape, Action::None, State::CsiEntry);
        table.add(0x5d, State::Escape, Action::None, State::OscData);
        table.add(0x50, State::Escape, Action::None, State::DcsEntry);
        table.add(0x58, State::Escape, Action::Record, State::Data); // SOS
        table.add(0x5e, State::Escape, Action::Record, State::Data); // PM
        table.add(0x5f, State::Escape, Action::Record, State::Data); // APC

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
        table.add(0x3A, State::CsiEntry, Action::Param, State::CsiParam);
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
            State::CsiEntry,
        );
        table.add(0x19, State::CsiIgnore, Action::Execute, State::CsiEntry);
        table.add(
            0x1C..=0x1F,
            State::CsiIgnore,
            Action::Execute,
            State::CsiEntry,
        );
        table.add(0x7F, State::CsiIgnore, Action::Ignore, State::CsiEntry);

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
        table.add(
            0x3C..=0x3F,
            State::CsiParam,
            Action::Ignore,
            State::CsiParam,
        );
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
            Action::Ignore,
            State::CsiIgnore,
        );
        table.add(
            0x40..=0x7E,
            State::CsiIntermediate,
            Action::Dispatch,
            State::Ground,
        );

        // State::DcsEntry
        table.add_entry(State::DcsEntry, Action::DcsStart);
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
        table.add(0x3A, State::DcsEntry, Action::Ignore, State::DcsIgnore);
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

        // Dcs_intermediate
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
            Action::Record,
            State::DcsIgnore,
        );
        table.add(
            0x40..=0x7E,
            State::DcsIntermediate,
            Action::Record,
            State::DcsData,
        );

        // Dcs_ignore
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
            0x20..=0x2F,
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
        table.add(0x3b, State::DcsParam, Action::Param, State::DcsParam);
        table.add(0x7F, State::DcsParam, Action::Ignore, State::DcsParam);
        table.add(0x3a, State::DcsParam, Action::Param, State::DcsIgnore);
        table.add(
            0x3C..=0x3F,
            State::DcsParam,
            Action::Ignore,
            State::DcsIgnore,
        );
        table.add(
            0x20..=0x2F,
            State::DcsParam,
            Action::Collect,
            State::DcsIntermediate,
        );
        table.add(0x40..=0x7E, State::DcsParam, Action::Ignore, State::DcsData);

        // State::DcsData
        table.add_entry(State::DcsData, Action::DcsStart);
        table.add(0x00..=0x17, State::DcsData, Action::Record, State::DcsData);
        table.add(0x19, State::DcsData, Action::Record, State::DcsData);
        table.add(0x1C..=0x1F, State::DcsData, Action::Record, State::DcsData);
        table.add(0x20..=0x7E, State::DcsData, Action::Record, State::DcsData);
        table.add(0x7F, State::DcsData, Action::Record, State::DcsData);
        table.add_exit(State::DcsData, Action::DcsEnd);

        // Data
        table.add(0x00..=0x17, State::Data, Action::Ignore, State::Data);
        table.add(0x19, State::Data, Action::Ignore, State::Data);
        table.add(0x1C..=0x1F, State::Data, Action::Ignore, State::Data);
        table.add(0x20..=0x7F, State::Data, Action::Ignore, State::Data);

        // Osc_string
        table.add_entry(State::OscData, Action::OscStart);
        table.add(0x00..=0x17, State::OscData, Action::Ignore, State::OscData);
        table.add(0x19, State::OscData, Action::Ignore, State::OscData);
        table.add(0x1C..=0x1F, State::OscData, Action::Ignore, State::OscData);
        table.add(0x20..=0x7F, State::OscData, Action::Record, State::OscData);
        table.add_exit(State::OscData, Action::OcsEnd);

        table
    }

    pub fn global_transition(state: State, char: u8) -> (State, Action) {
        let table = Self::global();

        table.transition(state, char)
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

trait TransitionValue {
    #[inline]
    fn add(self, table: &mut Table, state: State, action: Action, next: State);
}

impl TransitionValue for u8 {
    fn add(self, table: &mut Table, state: State, action: Action, next: State) {
        let idx = ((state as usize) << INDEX_STATE_SHIFT) | (self as usize);
        let value = (action as usize) << TRANSITION_ACTION_SHIFT | (next as usize);
        table.inner[idx as usize] = value;
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

    /// Data string phase of an OCS; input is handed to a separate handler until termination
    OscData,

    /// SOS (Start-of-String), PM (Private-Message) or APC (Application Program Command) string data
    Data,
    /// UTF-8 decoder state. Bytes are assembled into complete codepoints before being
    /// emitted; invalid sequences are replaced with U+FFFD and the decoder resumes.
    Utf8,
}

impl State {
    pub const COUNT: usize = Self::Utf8 as usize + 1;
}

impl From<u8> for State {
    fn from(value: u8) -> Self {
        if value > Self::COUNT as u8 {
            panic!("Value is too large")
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
            State::OscData => f.write_str("State::OscData"),
            State::Data => f.write_str("State::Data"),
            State::Utf8 => f.write_str("State::Utf8"),
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Default)]
pub enum Action {
    #[default]
    None,
    /// This action causes the current private flag, intermediate characters, final character and parameters to be forgotten. This occurs on entry to the escape, csi entry and dcs entry states, so that erroneous sequences like CSI 3 ; 1 CSI 2 J are handled correctly.
    Clear,
    /// The private marker or intermediate character should be stored for later use in selecting a control function to be executed when a final character arrives. X3.64 doesn’t place any limit on the number of intermediate characters allowed before a final character, although it doesn’t define any control sequences with more than one. Digital defined escape sequences with two intermediate characters, and control sequences and device control strings with one. If more than two intermediate characters arrive, the parser can just flag this so that the dispatch can be turned into a null operation.
    Collect,
    /// The final character of an escape or CSI sequence has arrived, so determined the control function to be executed from the intermediate character(s) and final character, and execute it. The intermediate characters are available because collect stored them as they arrived.
    Dispatch,
    /// The C0 or C1 control function should be executed, which may have any one of a variety of effects, including changing the cursor position, suspending or resuming communications or changing the shift states in effect. There are no parameters to this action.
    Execute,
    /// The character or control is not processed. No observable difference in the terminal’s state would occur if the character that caused this action was not present in the input stream. (Therefore, this action can only occur within a state.)
    Ignore,
    /// This action collects the characters of a parameter string for a control sequence or device control sequence and builds a list of parameters. The characters processed by this action are the digits 0-9 (codes 30-39) and the semicolon (code 3B). The semicolon separates parameters. There is no limit to the number of characters in a parameter string, although a maximum of 16 parameters need be stored. If more than 16 parameters arrive, all the extra parameters are silently ignored.
    ///
    /// The VT500 Programmer Information is inconsistent regarding the maximum value that a parameter can take. In section 4.3.3.2 of EK-VT520-RM it says that “any parameter greater than 9999 (decimal) is set to 9999 (decimal)”. However, in the description of DECSR (Secure Reset), its parameter is allowed to range from 0 to 16383. Because individual control functions need to make sure that numeric parameters are within specific limits, the supported maximum is not critical, but it must be at least 16383.
    ///
    /// Most control functions support default values for their parameters. The default value for a parameter is given by either leaving the parameter blank, or specifying a value of zero. Judging by previous threads on the newsgroup comp.terminals, this causes some confusion, with the occasional assertion that zero is the default parameter value for control functions. This is not the case: many control functions have a default value of 1, one (GSM) has a default value of 100, and some have no default. However, in all cases the default value is represented by either zero or a blank value.
    ///
    /// In the standard ECMA-48, which can be considered X3.64’s successor², there is a distinction between a parameter with an empty value (representing the default value), and one that has the value zero. There used to be a mode, ZDM (Zero Default Mode), in which the two cases were treated identically, but that is now deprecated in the fifth edition (1991). Although a VT500 parser needs to treat both empty and zero parameters as representing the default, it is worth considering future extensions by distinguishing them internally.
    Param,
    /// This action collects the private prefix of a control sequence.
    Prefix,
    /// This action only occurs in ground state. The current code should be mapped to a glyph according to the character set mappings and shift states in effect, and that glyph should be displayed. 20 (SP) and 7F (DEL) have special behaviour in later VT series, as described in ground.
    Print,

    DcsStart,
    DcsEnd,

    OscStart,
    OcsEnd,

    Record,
}

impl Action {
    pub const COUNT: usize = Self::Record as usize + 1;
}

impl From<u8> for Action {
    fn from(value: u8) -> Self {
        if value > Self::COUNT as u8 {
            panic!("Value is too large")
        }
        unsafe { std::mem::transmute(value) }
    }
}

impl Debug for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::None => f.write_str("Action::Action::None"),
            Action::Clear => f.write_str("Action::Clear"),
            Action::Collect => f.write_str("Action::Collect"),
            Action::Dispatch => f.write_str("Action::Dispatch"),
            Action::Execute => f.write_str("Action::Execute"),
            Action::Ignore => f.write_str("Action::Ignore"),
            Action::Param => f.write_str("Action::Param"),
            Action::Prefix => f.write_str("Action::Prefix"),
            Action::Print => f.write_str("Action::Print"),
            Action::OscStart => f.write_str("Action::OscStart"),
            Action::DcsStart => f.write_str("Action::DcsStart"),
            Action::OcsEnd => f.write_str("Action::OcsEnd"),
            Action::DcsEnd => f.write_str("Action::DcsEnd"),
            Action::Record => f.write_str("Action::Record"),
        }
    }
}
