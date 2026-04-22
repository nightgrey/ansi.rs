use bilge::prelude::*;
use derive_more::From;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum State {
    /// Initial state used to consume characters until an escape-style sequence begins;
    /// GL bytes 0x20-0x7F are printed, C0/C1 controls are executed immediately.
    #[default]
    Ground = 0,

    /// Entered on ESC; cancels any unfinished sequence and starts parsing the next one.
    Escape,
    /// Collect zero or more 0x20–0x2F "intermediate" bytes for a final `esc_dispatch`.
    EscapeIntermediate,

    /// First byte of CSI detected (`ESC [` or 0x9B). Digits, intermediate bytes or the
    /// first character from set `<?=>\)` change state; C0 controls are executed on arrival.
    CsiEntry,
    /// One or more parameter bytes (digits, semicolon) have arrived; 0x3A or second private
    /// marker triggers csi_ignore; intermediate chars after this turn it malformed (→ignore).
    CsiParam,
    /// Exactly one intermediate byte seen; error to see digits while here (→ignore).
    CsiIntermediate,

    /// DCS (`ESC P` or 0x90) seen; same rules as CSI, but C0 bytes are buffered instead of
    /// executed. First byte examined for private marker (0x3C-0x3F) just like CSI entry.
    DcsEntry,
    /// Numeric part of a device control string (same character set rules as CSI_PARAM).
    DcsParam,
    /// Intermediate byte(s) after DCS; digits while here make the string malformed (→ignore).
    DcsIntermediate,
    /// Data string phase of a DCS; input is handed to a separate handler until termination
    /// by ST, CAN, SUB or ESC that cancels the string.
    DcsData,

    /// OCS (`ESC ]` or 0x9D) seen.
    OscEntry,
    /// Numeric part of a OCS string (same character set rules as CSI_PARAM).
    OscParam,
    /// Intermediate byte(s) after OCS that are not parameters.
    OscIntermediate,
    /// Data string phase of an OCS; input is handed to a separate handler until termination
    OscData,

    /// SOS (Start-of-String) string
    SosData,
    /// PM (Private-Message) string data
    PmData,
    /// APC (Application Program Command) string data
    ApcData,

    /// UTF-8 decoder state. Bytes are assembled into complete codepoints before being
    /// emitted; invalid sequences are replaced with U+FFFD and the decoder resumes.
    Utf8,
}
impl From<u8> for State {
    fn from(value: u8) -> Self {
        if value > Self::COUNT as u8 {
            panic!("Value is too large")
        }
        unsafe { std::mem::transmute(value) }
    }
}
impl State {
    pub const COUNT: usize = Self::Utf8 as usize + 1;
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Action {
    #[default]
    None = 0,
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
    /// This action collects characters from the data string of a control sequence like DCS or OSC.
    Put,
    /// SOS/PM/APC/OSC data string
    ///
    /// When the control function OSC (Operating System Command), SOS (Start-of-String), PM (Private-Marker) or APC (Application Program Command) is recognised, this action causes the string to be collected and dispatched. The string is terminated by either ST (String Terminator) or BEL (Bell).
    Data,
}
impl Action {
    pub const COUNT: usize = Self::Data as usize + 1;
}
impl From<u8> for Action {
    fn from(value: u8) -> Self {
        if value > Self::COUNT as u8 {
            panic!("Value is too large")
        }
        unsafe { std::mem::transmute(value) }
    }
}
