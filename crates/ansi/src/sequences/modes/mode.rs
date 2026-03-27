use crate::{ResetMode, SetMode};
use std::fmt;
use std::fmt::{Display, Write};

/// Mode
///
/// Indicates the mode for [`SetMode`], [`ResetMode`], [`RequestMode`] and [`ReportMode`].
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum Mode {
    Ansi(AnsiMode),
    Dec(DecMode),
}

/// Mode Setting
///
/// Indicates the mode setting for [`AnsiMode`] and [`DecMode`]s.
#[derive(Default, Copy, Clone, Eq, PartialEq, Debug, Hash)]
#[repr(u8)]
pub enum ModeSetting {
    /// Mode not recognized
    #[default]
    NotRecognized = 0,
    /// Mode is set
    Set = 1,
    /// Mode is reset (not set)
    Reset = 2,
    /// Permanently set
    PermanentlySet = 3,
    /// Permanently reset
    PermanentlyReset = 4,
}

impl ModeSetting {
    pub fn set(&mut self) {
        *self = Self::Set;
    }

    pub fn reset(&mut self) {
        *self = Self::Reset;
    }

    pub fn reset_permanently(&mut self) {
        *self = Self::PermanentlyReset;
    }

    pub fn set_permanently(&mut self) {
        *self = Self::PermanentlySet;
    }

    pub fn is_not_recognized(&self) -> bool {
        matches!(self, Self::NotRecognized)
    }

    pub fn is_set(&self) -> bool {
        matches!(self, Self::Set | Self::PermanentlySet)
    }

    pub fn is_permanently_set(&self) -> bool {
        matches!(self, Self::PermanentlySet)
    }

    pub fn is_reset(&self) -> bool {
        matches!(self, Self::Reset | Self::PermanentlyReset)
    }

    pub fn is_permanently_reset(&self) -> bool {
        matches!(self, Self::PermanentlyReset)
    }
}

impl Display for ModeSetting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self as u8)
    }
}
/// ANSI modes
///
/// Indicates the mode for [`SetMode`], [`ResetMode`], [`RequestMode`] and [`ReportMode`].
///
/// See https://vt100.net/docs/vt510-rm/DECRQM.html#T5-7
///
/// [DECRQM]: https://vt100.net/docs/vt510-rm/DECRQM.html
/// [DECRPM]: https://vt100.net/docs/vt510-rm/DECRPM.html
/// [SM]: https://vt100.net/docs/vt510-rm/SM.html
/// [RM]: https://vt100.net/docs/vt510-rm/RM.html
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
#[repr(u16)]
pub enum AnsiMode {
    /// (1) Guarded Area Transfer Mode (GATM)
    GuardedAreaTransfer = 1,
    /// (2) Keyboard Action Mode (KAM) is a mode that controls locking of the keyboard.
    /// When the keyboard is locked, it cannot send data to the terminal.
    ///
    /// See https://vt100.net/docs/vt510-rm/KAM.html
    KeyboardAction = 2,
    /// (3) Control Representation Mode (CRM)
    ControlRepresentation = 3,
    /// (4) Insert/Replace Mode (IRM) is a mode that determines whether characters are
    /// inserted or replaced when typed.
    ///
    /// When enabled, characters are inserted at the cursor position pushing the
    /// characters to the right. When disabled, characters replace the character at
    /// the cursor position.
    ///
    /// See https://vt100.net/docs/vt510-rm/IRM.html
    InsertReplace = 4,
    /// (5) Status Report Transfer Mode (SRTM)
    StatusReportTransfer = 5,
    /// (7) Vertical Editing Mode (VEM)
    VerticalEditing = 7,
    /// (10) Horizontal Editing Mode (HEM)
    HorizontalEditing = 10,
    /// (11) Positioning Unit Mode (PUM)
    PositioningUnit = 11,
    /// (12) Send Receive Mode (SRM) or Local Echo Mode is a mode that determines whether
    /// the terminal echoes characters back to the host. When enabled, the terminal
    /// sends characters to the host as they are typed.
    ///
    /// See https://vt100.net/docs/vt510-rm/SRM.html
    SendReceive = 12,
    /// (13) Format Effector Action Mode (FEAM)
    FormatEffectorAction = 13,
    /// (14) Format Effector Transfer Mode (FETM)
    FormatEffectorTransfer = 14,
    /// (15) Multiple Area Transfer Mode (MATM)
    MultipleAreaTransfer = 15,
    /// (16) Transfer Termination Mode (TTM)
    TransferTermination = 16,
    /// (17) Selected Area Transfer Mode (SATM)
    SelectedAreaTransfer = 17,
    /// (18) Tabulation Stop Mode (TSM)
    TabulationStop = 18,
    /// (19) Editing Boundary Mode (EBM)
    EditingBoundary = 19,
    /// (20) Line Feed/New Line Mode (LNM) is a mode that determines whether the terminal
    /// interprets the line feed character as a new line.
    ///
    /// When enabled, the terminal interprets the line feed character as a new line.
    /// When disabled, the terminal interprets the line feed character as a line feed.
    ///
    /// A new line moves the cursor to the first position of the next line.
    /// A line feed moves the cursor down one line without changing the column
    /// scrolling the screen if necessary.
    ///
    /// See https://vt100.net/docs/vt510-rm/LNM.html
    AutomaticNewline = 20,
}

impl From<AnsiMode> for Mode {
    fn from(mode: AnsiMode) -> Self {
        Mode::Ansi(mode)
    }
}

impl Display for AnsiMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self as u16)
    }
}
/// DEC modes
///
/// Indicates the mode for [`SetMode`], [`ResetMode`], [`RequestMode`] and [`ReportMode`].
///
/// See https://vt100.net/docs/vt510-rm/DECRQM.html#T5-8
///
/// [DECRQM]: https://vt100.net/docs/vt510-rm/DECRQM.html
/// [DECRPM]: https://vt100.net/docs/vt510-rm/DECRPM.html
/// [SM]: https://vt100.net/docs/vt510-rm/SM.html
/// [RM]: https://vt100.net/docs/vt510-rm/RM.html
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
#[repr(u16)]
pub enum DecMode {
    /// (1) Cursor Keys Mode (DECCKM) is a mode that determines whether the cursor keys
    /// send ANSI cursor sequences or application sequences.
    ///
    /// See https://vt100.net/docs/vt510-rm/DECCKM.html
    ApplicationCursorKeys = 1,
    /// (2) ANSI Mode (DECANM)
    Ansi = 2,
    /// (3) Column Mode (DECCOLM)
    Column132 = 3,
    /// (4) Scrolling Mode (DECSCLM)
    Scrolling = 4,
    /// (5) Screen Mode (DECSCNM)
    ReverseVideo = 5,
    /// (6) Origin Mode (DECOM) is a mode that determines whether the cursor moves to the
    /// home position or the margin position.
    ///
    /// See https://vt100.net/docs/vt510-rm/DECOM.html
    OriginMode = 6,
    /// (7) Auto Wrap Mode (DECAWM) is a mode that determines whether the cursor wraps
    /// to the next line when it reaches the right margin.
    ///
    /// See https://vt100.net/docs/vt510-rm/DECAWM.html
    AutoWrapMode = 7,
    /// (8) Auto Repeat Keys Mode (DECARM)
    AutoRepeatKeys = 8,
    /// (9) X10 Mouse Mode is a mode that determines whether the mouse reports on button
    /// presses.
    ///
    /// The terminal responds with the following encoding:
    ///
    /// ```text
    /// CSI M CbCxCy
    /// ```
    ///
    /// Where Cb is the button-1, where it can be 1, 2, or 3.
    /// Cx and Cy are the x and y coordinates of the mouse event.
    ///
    /// See https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Mouse-Tracking
    MouseX10 = 9,
    /// (10) Show Toolbar Mode
    ShowToolbar = 10,
    /// (13) Blinking Cursor Resource Mode
    BlinkingCursorResource = 13,
    /// (14) Blinking Cursor XOR Mode
    BlinkingCursorXOR = 14,
    /// (18) Print Form Feed Mode (DECPFF)
    PrintFormFeed = 18,
    /// (19) Printer Extent Full Mode (DECPEX)
    PrintExtentFull = 19,
    /// (25) Text Cursor Enable Mode (DECTCEM) is a mode that shows/hides the cursor.
    ///
    /// See https://vt100.net/docs/vt510-rm/DECTCEM.html
    TextCursorEnable = 25,
    /// (30) Show Scrollbar Mode
    ShowScrollbar = 30,
    /// (34) Cursor Direction Right to Left Mode (DECRLM)
    CursorDirectionRtl = 34,
    /// (35) Hebrew Keyboard Mapping Mode (DECHEBM)
    HebrewKeyboardMapping = 35,
    /// (36) Hebrew Encoding Mode (DECHEM)
    HebrewEncoding = 36,
    /// (42) National Replacement Character Set Mode (DECNRCM)
    NationalReplacementCharsets = 42,
    /// (57) Greek Keyboard Mapping Mode (DECNAKB)
    GreekKeyboardMapping = 57,
    /// (60) Horizontal Cursor Coupling Mode (DECHCCM)
    HorizontalCursorCoupling = 60,
    /// (61) Vertical Cursor Coupling Mode (DECVCCM)
    VerticalCursorCoupling = 61,
    /// (64) Page Cursor Coupling Mode (DECPCCM)
    PageCursorCoupling = 64,
    /// (66) Numeric Keypad Mode (DECNKM) is a mode that determines whether the keypad
    /// sends application sequences or numeric sequences.
    ///
    /// This works like DECKPAM and DECKPNM, but uses different sequences.
    ///
    /// See https://vt100.net/docs/vt510-rm/DECNKM.html
    ApplicationKeypad = 66,
    /// (67) Backarrow Key Mode (DECBKM) is a mode that determines whether the backspace
    /// key sends a backspace or delete character. Disabled by default.
    ///
    /// See https://vt100.net/docs/vt510-rm/DECBKM.html
    BackarrowSendsBackspace = 67,
    /// (68) Keyboard Usage Mode (DECKBUM)
    KeyboardUsage = 68,
    /// (69) Left Right Margin Mode (DECLRMM) is a mode that determines whether the left
    /// and right margins can be set with DECSLRM.
    ///
    /// See https://vt100.net/docs/vt510-rm/DECLRMM.html
    LeftRightMargin = 69,
    /// (73) Transmit Rate Limiting Mode (DECXRLM)
    TransmitRateLimiting = 73,
    /// (80) Sixel Display Mode
    SixelDisplay = 80,
    /// (81) Key Position Mode (DECKPM)
    KeyPosition = 81,
    /// (95) No Clearing Screen on Column Change Mode (DECNCSM)
    NoClearOnDECCOLM = 95,
    /// (96) Cursor Right to Left Mode (DECRLCM)
    CursorRightToLeft = 96,
    /// (97) CRT Save Mode (DECCRTSM)
    CrtSave = 97,
    /// (98) Auto Resize Mode (DECARSM)
    AutoResize = 98,
    /// (99) Modem Control Mode (DECMCM)
    ModemControl = 99,
    /// (100) Auto Answerback Mode (DECAAM)
    AutoAnswerback = 100,
    /// (101) Conceal Answerback Message Mode (DECCANSM)
    ConcealAnswerbackMessage = 101,
    /// (102) Ignoring Null Mode (DECNULM)
    IgnoringNull = 102,
    /// (103) Half-Duplex Mode (DECHDPXM)
    HalfDuplex = 103,
    /// (104) Secondary Keyboard Language Mode (DECESKM)
    SecondaryKeyboardLanguage = 104,
    /// (106) Overscan Mode (DECOSCNM)
    Overscan = 106,

    // XTerm / private extensions (kept as-is)
    /// (1000) Normal Mouse Mode is a mode that determines whether the mouse reports on
    /// button presses and releases. It will also report modifier keys, wheel
    /// events, and extra buttons.
    ///
    /// See https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Mouse-Tracking
    MouseTracking = 1000,
    /// (1001) Highlight Mouse Tracking is a mode that determines whether the mouse reports
    /// on button presses, releases, and highlighted cells.
    ///
    /// See https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Mouse-Tracking
    HighlightMouseTracking = 1001,
    /// (1002) Button Event Mouse Tracking is essentially the same as NormalMouseMode,
    /// but it also reports button-motion events when a button is pressed.
    ///
    /// See https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Mouse-Tracking
    ButtonEventMouseTracking = 1002,
    /// (1003) Any Event Mouse Tracking is the same as ButtonEventMouseMode, except that
    /// all motion events are reported even if no mouse buttons are pressed.
    ///
    /// See https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Mouse-Tracking
    AnyEventMouseTracking = 1003,
    /// (1004) Focus Event Mode is a mode that determines whether the terminal reports focus
    /// and blur events.
    ///
    /// The terminal sends the following encoding:
    ///
    /// ```text
    /// CSI I // Focus In
    /// CSI O // Focus Out
    /// ```
    ///
    /// See https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Focus-Tracking
    FocusTracking = 1004,
    /// (1005) UTF-8 Extended Mouse Mode is a mode that changes the mouse tracking encoding
    /// to use UTF-8 parameters.
    ///
    /// See https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Mouse-Tracking
    UTF8ExtendedMouse = 1005,
    /// (1006) SGR Extended Mouse Mode is a mode that changes the mouse tracking encoding
    /// to use SGR parameters.
    ///
    /// The terminal responds with the following encoding:
    ///
    /// ```text
    /// CSI < Cb ; Cx ; Cy M
    /// ```
    ///
    /// Where Cb is the same as NormalMouseMode, and Cx and Cy are the x and y.
    ///
    /// See https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Mouse-Tracking
    SGRMouse = 1006,
    /// (1007) Alternate Scroll Mode
    AlternateScroll = 1007,
    /// (1010) Scroll TTY Output Mode
    ScrollTtyOutput = 1010,
    /// (1011) Scroll Key Mode
    ScrollKey = 1011,
    /// (1014) Fast Scroll Mode
    FastScroll = 1014,
    /// (1015) URXVT Extended Mouse Mode is a mode that changes the mouse tracking encoding
    /// to use an alternate encoding.
    ///
    /// See https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Mouse-Tracking
    URXVTMouse = 1015,
    /// (1016) SGR Pixel Extended Mouse Mode is a mode that changes the mouse tracking
    /// encoding to use SGR parameters with pixel coordinates.
    ///
    /// This is similar to SgrExtMouseMode, but also reports pixel coordinates.
    ///
    /// See https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Mouse-Tracking
    SGRPixelMouse = 1016,
    /// (1034) Meta Key Mode
    MetaKey = 1034,
    /// (1035) Alt Num Lock Modifiers Mode
    AltNumLockModifiers = 1035,
    /// (1036) Meta Sends Escape Mode
    MetaSendsEscape = 1036,
    /// (1037) Editing Keypad Delete Mode
    EditingKeypadDelete = 1037,
    /// (1039) Alt Sends Escape Mode
    AltSendsEscape = 1039,
    /// (1040) Keep Selection Mode
    KeepSelection = 1040,
    /// (1041) Select to Clipboard Mode
    SelectToClipboard = 1041,
    /// (1042) Bell Is Urgent Mode
    BellIsUrgent = 1042,
    /// (1043) Pop on Bell Mode
    PopOnBell = 1043,
    /// (1044) Keep Clipboard Mode
    KeepClipboard = 1044,
    /// (1045) Extended Reverse Wrap Mode
    ExtendedReverseWrap = 1045,
    /// (1046) Alternate Screen Tite Mode
    AlternateScreenTite = 1046,
    /// (1047) Alternate Screen Mode is a mode that determines whether the alternate screen
    /// buffer is active. When this mode is enabled, the alternate screen buffer is
    /// cleared.
    ///
    /// See https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-The-Alternate-Screen-Buffer
    AlternateScreen = 1047,
    /// (1048) Save Cursor Mode is a mode that saves the cursor position.
    /// This is equivalent to SaveCursor and RestoreCursor.
    ///
    /// See https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-The-Alternate-Screen-Buffer
    SaveCursorMode = 1048,
    /// (1049) Alternate Screen Save Cursor Mode is a mode that saves the cursor position as in
    /// SaveCursorMode, switches to the alternate screen buffer as in AltScreenMode,
    /// and clears the screen on switch.
    ///
    /// See https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-The-Alternate-Screen-Buffer
    SaveCursorAndAlternate = 1049,
    /// (1050) Terminfo Function Keys Mode
    TerminfoFunctionKeys = 1050,
    /// (1051) Sun Function Keys Mode
    SunFunctionKeys = 1051,
    /// (1052) HP Function Keys Mode
    HPFunctionKeys = 1052,
    /// (1053) SCO Function Keys Mode
    SCOFunctionKeys = 1053,
    /// (1060) Legacy Keyboard Mode
    LegacyKeyboard = 1060,
    /// (1061) VT220 Keyboard Mode
    VT220Keyboard = 1061,
    /// (2001) Readline Mouse 1 Mode
    ReadlineMouse1 = 2001,
    /// (2002) Readline Mouse 2 Mode
    ReadlineMouse2 = 2002,
    /// (2003) Readline Mouse 3 Mode
    ReadlineMouse3 = 2003,
    /// (2004) Bracketed Paste Mode is a mode that determines whether pasted text is
    /// bracketed with escape sequences.
    ///
    /// See:
    /// - https://cirw.in/blog/bracketed-paste
    /// - https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Bracketed-Paste-Mode
    BracketedPaste = 2004,
    /// (2005) Readline Char Quoting Mode
    ReadlineCharQuoting = 2005,
    /// (2006) Readline Newline Paste Mode
    ReadlineNewlinePaste = 2006,
    /// (2026) Synchronized Output Mode
    ///
    /// See:
    /// - https://contour-terminal.org/vt-extensions/synchronized-output/
    /// - https://github.com/contour-terminal/vt-extensions/blob/master/synchronized-output.md
    SynchronizedOutput = 2026,
}

impl From<DecMode> for Mode {
    fn from(mode: DecMode) -> Self {
        Mode::Dec(mode)
    }
}

impl Display for DecMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self as u16)
    }
}
