use std::fmt;
use std::fmt::Display;
use derive_more::Deref;

/// ANSI & DEC Mode
///
/// Indicates the mode for [`SetMode`], [`ResetMode`], [`RequestMode`] and [`ReportMode`].
///
/// See https://vt100.net/docs/vt510-rm/DECRQM.html#T5-8
/// [DECRQM]: https://vt100.net/docs/vt510-rm/DECRQM.html
/// [DECRPM]: https://vt100.net/docs/vt510-rm/DECRPM.html
/// [SM]: https://vt100.net/docs/vt510-rm/SM.html
/// [RM]: https://vt100.net/docs/vt510-rm/RM.html
#[derive(Copy, Debug, Hash)]
#[derive_const(Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum Mode {
    /// \[ANSI\] \[1\] Guarded Area Transfer Mode (GATM)
    GuardedAreaTransfer,
    /// \[ANSI\] \[2\] Keyboard Action Mode (KAM) is a mode that controls locking of the keyboard.
    /// When the keyboard is locked, it cannot send data to the terminal.
    ///
    /// See https://vt100.net/docs/vt510-rm/KAM.html
    KeyboardAction,
    /// \[ANSI\] \[3\] Control Representation Mode (CRM)
    ControlRepresentation,
    /// \[ANSI\] \[4\] Insert/Replace Mode (IRM) is a mode that determines whether characters are
    /// inserted or replaced when typed.
    ///
    /// When enabled, characters are inserted at the cursor position pushing the
    /// characters to the right. When disabled, characters replace the character at
    /// the cursor position.
    ///
    /// See https://vt100.net/docs/vt510-rm/IRM.html
    InsertReplace,
    /// \[ANSI\] \[5\] Status Report Transfer Mode (SRTM)
    StatusReportTransfer,
    /// \[ANSI\] \[7\] Vertical Editing Mode (VEM)
    VerticalEditing,
    /// \[ANSI\] \[10\] Horizontal Editing Mode (HEM)
    HorizontalEditing,
    /// \[ANSI\] \[11\] Positioning Unit Mode (PUM)
    PositioningUnit,
    /// \[ANSI\] \[12\] Send Receive Mode (SRM) or Local Echo Mode is a mode that determines whether
    /// the terminal echoes characters back to the host. When enabled, the terminal
    /// sends characters to the host as they are typed.
    ///
    /// See https://vt100.net/docs/vt510-rm/SRM.html
    SendReceive,
    /// \[ANSI\] \[13\] Format Effector Action Mode (FEAM)
    FormatEffectorAction,
    /// \[ANSI\] \[14\] Format Effector Transfer Mode (FETM)
    FormatEffectorTransfer,
    /// \[ANSI\] \[15\] Multiple Area Transfer Mode (MATM)
    MultipleAreaTransfer,
    /// \[ANSI\] \[16\] Transfer Termination Mode (TTM)
    TransferTermination,
    /// \[ANSI\] \[17\] Selected Area Transfer Mode (SATM)
    SelectedAreaTransfer,
    /// \[ANSI\] \[18\] Tabulation Stop Mode (TSM)
    TabulationStop,
    /// \[ANSI\] \[19\] Editing Boundary Mode (EBM)
    EditingBoundary,
    /// \[ANSI\] \[20\] Line Feed/New Line Mode (LNM) is a mode that determines whether the terminal
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
    AutomaticNewline,

    /// -------------
    /// DEC MODES   |
    /// -------------

    /// \[DEC\] \[1\] Cursor Keys Mode (DECCKM) is a mode that determines whether the cursor keys
    /// send ANSI cursor sequences or application sequences.
    ///
    /// See https://vt100.net/docs/vt510-rm/DECCKM.html
    ApplicationCursorKeys,
    /// \[DEC\] \[2\] ANSI Mode (DECANM)
    Ansi,
    /// \[DEC\] \[3\] Column Mode (DECCOLM)
    Column132,
    /// \[DEC\] \[4\] Scrolling Mode (DECSCLM)
    Scrolling,
    /// \[DEC\] \[5\] Screen Mode (DECSCNM)
    ReverseVideo,
    /// \[DEC\] \[6\] Origin Mode (DECOM) is a mode that determines whether the cursor moves to the
    /// home position or the margin position.
    ///
    /// See https://vt100.net/docs/vt510-rm/DECOM.html
    OriginMode,
    /// \[DEC\] \[7\] Auto Wrap Mode (DECAWM) is a mode that determines whether the cursor wraps
    /// to the next line when it reaches the right margin.
    ///
    /// See https://vt100.net/docs/vt510-rm/DECAWM.html
    AutoWrapMode,
    /// \[DEC\] \[8\] Auto Repeat Keys Mode (DECARM)
    AutoRepeatKeys,
    /// \[DEC\] \[9\] X10 Mouse Mode is a mode that determines whether the mouse reports on button
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
    MouseX10,
    /// \[DEC\] \[10\] Show Toolbar Mode
    ShowToolbar,
    /// \[DEC\] \[13\] Blinking Cursor Resource Mode
    BlinkingCursorResource,
    /// \[DEC\] \[14\] Blinking Cursor XOR Mode
    BlinkingCursorXOR,
    /// \[DEC\] \[18\] Print Form Feed Mode (DECPFF)
    PrintFormFeed,
    /// \[DEC\] \[19\] Printer Extent Full Mode (DECPEX)
    PrintExtentFull,
    /// \[DEC\] \[25\] Text Cursor Enable Mode (DECTCEM) is a mode that shows/hides the cursor.
    ///
    /// See https://vt100.net/docs/vt510-rm/DECTCEM.html
    TextCursorEnable,
    /// \[DEC\] \[30\] Show Scrollbar Mode
    ShowScrollbar,
    /// \[DEC\] \[34\] Cursor Direction Right to Left Mode (DECRLM)
    CursorDirectionRtl,
    /// \[DEC\] \[35\] Hebrew Keyboard Mapping Mode (DECHEBM)
    HebrewKeyboardMapping,
    /// \[DEC\] \[36\] Hebrew Encoding Mode (DECHEM)
    HebrewEncoding,
    /// \[DEC\] \[42\] National Replacement Character Set Mode (DECNRCM)
    NationalReplacementCharsets,
    /// \[DEC\] \[57\] Greek Keyboard Mapping Mode (DECNAKB)
    GreekKeyboardMapping,
    /// \[DEC\] \[60\] Horizontal Cursor Coupling Mode (DECHCCM)
    HorizontalCursorCoupling,
    /// \[DEC\] \[61\] Vertical Cursor Coupling Mode (DECVCCM)
    VerticalCursorCoupling,
    /// \[DEC\] \[64\] Page Cursor Coupling Mode (DECPCCM)
    PageCursorCoupling,
    /// \[DEC\] \[66\] Numeric Keypad Mode (DECNKM) is a mode that determines whether the keypad
    /// sends application sequences or numeric sequences.
    ///
    /// This works like DECKPAM and DECKPNM, but uses different sequences.
    ///
    /// See https://vt100.net/docs/vt510-rm/DECNKM.html
    ApplicationKeypad,
    /// \[DEC\] \[67\] Backarrow Key Mode (DECBKM) is a mode that determines whether the backspace
    /// key sends a backspace or delete character. Disabled by default.
    ///
    /// See https://vt100.net/docs/vt510-rm/DECBKM.html
    BackarrowSendsBackspace,
    /// \[DEC\] \[68\] Keyboard Usage Mode (DECKBUM)
    KeyboardUsage,
    /// \[DEC\] \[69\] Left Right Margin Mode (DECLRMM) is a mode that determines whether the left
    /// and right margins can be set with DECSLRM.
    ///
    /// See https://vt100.net/docs/vt510-rm/DECLRMM.html
    LeftRightMargin,
    /// \[DEC\] \[73\] Transmit Rate Limiting Mode (DECXRLM)
    TransmitRateLimiting,
    /// \[DEC\] \[80\] Sixel Display Mode
    SixelDisplay,
    /// \[DEC\] \[81\] Key Position Mode (DECKPM)
    KeyPosition,
    /// \[DEC\] \[95\] No Clearing Screen on Column Change Mode (DECNCSM)
    NoClearOnDECCOLM,
    /// \[DEC\] \[96\] Cursor Right to Left Mode (DECRLCM)
    CursorRightToLeft,
    /// \[DEC\] \[97\] CRT Save Mode (DECCRTSM)
    CrtSave,
    /// \[DEC\] \[98\] Auto Resize Mode (DECARSM)
    AutoResize,
    /// \[DEC\] \[99\] Modem Control Mode (DECMCM)
    ModemControl,
    /// \[DEC\] \[100\] Auto Answerback Mode (DECAAM)
    AutoAnswerback,
    /// \[DEC\] \[101\] Conceal Answerback Message Mode (DECCANSM)
    ConcealAnswerbackMessage,
    /// \[DEC\] \[102\] Ignoring Null Mode (DECNULM)
    IgnoringNull,
    /// \[DEC\] \[103\] Half-Duplex Mode (DECHDPXM)
    HalfDuplex,
    /// \[DEC\] \[104\] Secondary Keyboard Language Mode (DECESKM)
    SecondaryKeyboardLanguage,
    /// \[DEC\] \[106\] Overscan Mode (DECOSCNM)
    Overscan,

    // XTerm / private extensions (kept as-is)
    /// \[DEC\] \[1000\] Normal Mouse Mode is a mode that determines whether the mouse reports on
    /// button presses and releases. It will also report modifier keys, wheel
    /// events, and extra buttons.
    ///
    /// See https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Mouse-Tracking
    MouseTracking,
    /// \[DEC\] \[1001\] Highlight Mouse Tracking is a mode that determines whether the mouse reports
    /// on button presses, releases, and highlighted cells.
    ///
    /// See https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Mouse-Tracking
    HighlightMouseTracking,
    /// \[DEC\] \[1002\] Button Event Mouse Tracking is essentially the same as NormalMouseMode,
    /// but it also reports button-motion events when a button is pressed.
    ///
    /// See https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Mouse-Tracking
    ButtonEventMouseTracking,
    /// \[DEC\] \[1003\] Any Event Mouse Tracking is the same as ButtonEventMouseMode, except that
    /// all motion events are reported even if no mouse buttons are pressed.
    ///
    /// See https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Mouse-Tracking
    AnyEventMouseTracking,
    /// \[DEC\] \[1004\] Focus Event Mode is a mode that determines whether the terminal reports focus
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
    FocusTracking,
    /// \[DEC\] \[1005\] UTF-8 Extended Mouse Mode is a mode that changes the mouse tracking encoding
    /// to use UTF-8 parameters.
    ///
    /// See https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Mouse-Tracking
    UTF8ExtendedMouse,
    /// \[DEC\] \[1006\] SGR Extended Mouse Mode is a mode that changes the mouse tracking encoding
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
    SGRMouse,
    /// \[DEC\] \[1007\] Alternate Scroll Mode
    AlternateScroll,
    /// \[DEC\] \[1010\] Scroll TTY Output Mode
    ScrollTtyOutput,
    /// \[DEC\] \[1011\] Scroll Key Mode
    ScrollKey,
    /// \[DEC\] \[1014\] Fast Scroll Mode
    FastScroll,
    /// \[DEC\] \[1015\] URXVT Extended Mouse Mode is a mode that changes the mouse tracking encoding
    /// to use an alternate encoding.
    ///
    /// See https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Mouse-Tracking
    URXVTMouse,
    /// \[DEC\] \[1016\] SGR Pixel Extended Mouse Mode is a mode that changes the mouse tracking
    /// encoding to use SGR parameters with pixel coordinates.
    ///
    /// This is similar to SgrExtMouseMode, but also reports pixel coordinates.
    ///
    /// See https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Mouse-Tracking
    SGRPixelMouse,
    /// \[DEC\] \[1034\] Meta Key Mode
    MetaKey,
    /// \[DEC\] \[1035\] Alt Num Lock Modifiers Mode
    AltNumLockModifiers,
    /// \[DEC\] \[1036\] Meta Sends Escape Mode
    MetaSendsEscape,
    /// \[DEC\] \[1037\] Editing Keypad Delete Mode
    EditingKeypadDelete,
    /// \[DEC\] \[1039\] Alt Sends Escape Mode
    AltSendsEscape,
    /// \[DEC\] \[1040\] Keep Selection Mode
    KeepSelection,
    /// \[DEC\] \[1041\] Select to Clipboard Mode
    SelectToClipboard,
    /// \[DEC\] \[1042\] Bell Is Urgent Mode
    BellIsUrgent,
    /// \[DEC\] \[1043\] Pop on Bell Mode
    PopOnBell,
    /// \[DEC\] \[1044\] Keep Clipboard Mode
    KeepClipboard,
    /// \[DEC\] \[1045\] Extended Reverse Wrap Mode
    ExtendedReverseWrap,
    /// \[DEC\] \[1046\] Alternate Screen Tite Mode
    AlternateScreenTite,
    /// \[DEC\] \[1047\] Alternate Screen Mode is a mode that determines whether the alternate screen
    /// buffer is active. When this mode is enabled, the alternate screen buffer is
    /// cleared.
    ///
    /// See https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-The-Alternate-Screen-Buffer
    AlternateScreen,
    /// \[DEC\] \[1048\] Save Cursor Mode is a mode that saves the cursor position.
    /// This is equivalent to SaveCursor and RestoreCursor.
    ///
    /// See https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-The-Alternate-Screen-Buffer
    SaveCursorMode,
    /// \[DEC\] \[1049\] Alternate Screen Save Cursor Mode is a mode that saves the cursor position as in
    /// SaveCursorMode, switches to the alternate screen buffer as in AltScreenMode,
    /// and clears the screen on switch.
    ///
    /// See https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-The-Alternate-Screen-Buffer
    SaveCursorAndAlternate,
    /// \[DEC\] \[1050\] Terminfo Function Keys Mode
    TerminfoFunctionKeys,
    /// \[DEC\] \[1051\] Sun Function Keys Mode
    SunFunctionKeys,
    /// \[DEC\] \[1052\] HP Function Keys Mode
    HPFunctionKeys,
    /// \[DEC\] \[1053\] SCO Function Keys Mode
    SCOFunctionKeys,
    /// \[DEC\] \[1060\] Legacy Keyboard Mode
    LegacyKeyboard,
    /// \[DEC\] \[1061\] VT220 Keyboard Mode
    VT220Keyboard,
    /// \[DEC\] \[2001\] Readline Mouse 1 Mode
    ReadlineMouse1,
    /// \[DEC\] \[2002\] Readline Mouse 2 Mode
    ReadlineMouse2,
    /// \[DEC\] \[2003\] Readline Mouse 3 Mode
    ReadlineMouse3,
    /// \[DEC\] \[2004\] Bracketed Paste Mode is a mode that determines whether pasted text is
    /// bracketed with escape sequences.
    ///
    /// See:
    /// - https://cirw.in/blog/bracketed-paste
    /// - https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h2-Bracketed-Paste-Mode
    BracketedPaste,
    /// \[DEC\] \[2005\] Readline Char Quoting Mode
    ReadlineCharQuoting,
    /// \[DEC\] \[2006\] Readline Newline Paste Mode
    ReadlineNewlinePaste,
    /// \[DEC\] \[2026\] Synchronized Output Mode
    ///
    /// See:
    /// - https://contour-terminal.org/vt-extensions/synchronized-output/
    /// - https://github.com/contour-terminal/vt-extensions/blob/master/synchronized-output.md
    SynchronizedOutput,
}

#[derive(Copy, Debug)]
#[derive_const(Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum ModeKind {
    Ansi,
    Dec,
}

impl Mode {
    pub fn kind(&self) -> ModeKind {
        match (*self as u8) {
            ..=17 => ModeKind::Ansi,
            _ => ModeKind::Dec,
        }
    }
    
    pub fn is_ansi(&self) -> bool {
        matches!(self.kind(), ModeKind::Ansi)
    }

    pub fn is_dec(&self) -> bool {
        matches!(self.kind(), ModeKind::Dec)
    }
}

impl Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Mode::GuardedAreaTransfer => 1,
            Mode::KeyboardAction => 2,
            Mode::ControlRepresentation => 3,
            Mode::InsertReplace => 4,
            Mode::StatusReportTransfer => 5,
            Mode::VerticalEditing => 7,
            Mode::HorizontalEditing => 10,
            Mode::PositioningUnit => 11,
            Mode::SendReceive => 12,
            Mode::FormatEffectorAction => 13,
            Mode::FormatEffectorTransfer => 14,
            Mode::MultipleAreaTransfer => 15,
            Mode::TransferTermination => 16,
            Mode::SelectedAreaTransfer => 17,
            Mode::TabulationStop => 18,
            Mode::EditingBoundary => 19,
            Mode::AutomaticNewline => 20,
            Mode::ApplicationCursorKeys => 1,
            Mode::Ansi => 2,
            Mode::Column132 => 3,
            Mode::Scrolling => 4,
            Mode::ReverseVideo => 5,
            Mode::OriginMode => 6,
            Mode::AutoWrapMode => 7,
            Mode::AutoRepeatKeys => 8,
            Mode::MouseX10 => 9,
            Mode::ShowToolbar => 10,
            Mode::BlinkingCursorResource => 13,
            Mode::BlinkingCursorXOR => 14,
            Mode::PrintFormFeed => 18,
            Mode::PrintExtentFull => 19,
            Mode::TextCursorEnable => 25,
            Mode::ShowScrollbar => 30,
            Mode::CursorDirectionRtl => 34,
            Mode::HebrewKeyboardMapping => 35,
            Mode::HebrewEncoding => 36,
            Mode::NationalReplacementCharsets => 42,
            Mode::GreekKeyboardMapping => 57,
            Mode::HorizontalCursorCoupling => 60,
            Mode::VerticalCursorCoupling => 61,
            Mode::PageCursorCoupling => 64,
            Mode::ApplicationKeypad => 66,
            Mode::BackarrowSendsBackspace => 67,
            Mode::KeyboardUsage => 68,
            Mode::LeftRightMargin => 69,
            Mode::TransmitRateLimiting => 73,
            Mode::SixelDisplay => 80,
            Mode::KeyPosition => 81,
            Mode::NoClearOnDECCOLM => 95,
            Mode::CursorRightToLeft => 96,
            Mode::CrtSave => 97,
            Mode::AutoResize => 98,
            Mode::ModemControl => 99,
            Mode::AutoAnswerback => 100,
            Mode::ConcealAnswerbackMessage => 101,
            Mode::IgnoringNull => 102,
            Mode::HalfDuplex => 103,
            Mode::SecondaryKeyboardLanguage => 104,
            Mode::Overscan => 106,
            Mode::MouseTracking => 1000,
            Mode::HighlightMouseTracking => 1001,
            Mode::ButtonEventMouseTracking => 1002,
            Mode::AnyEventMouseTracking => 1003,
            Mode::FocusTracking => 1004,
            Mode::UTF8ExtendedMouse => 1005,
            Mode::SGRMouse => 1006,
            Mode::AlternateScroll => 1007,
            Mode::ScrollTtyOutput => 1010,
            Mode::ScrollKey => 1011,
            Mode::FastScroll => 1014,
            Mode::URXVTMouse => 1015,
            Mode::SGRPixelMouse => 1016,
            Mode::MetaKey => 1034,
            Mode::AltNumLockModifiers => 1035,
            Mode::MetaSendsEscape => 1036,
            Mode::EditingKeypadDelete => 1037,
            Mode::AltSendsEscape => 1039,
            Mode::KeepSelection => 1040,
            Mode::SelectToClipboard => 1041,
            Mode::BellIsUrgent => 1042,
            Mode::PopOnBell => 1043,
            Mode::KeepClipboard => 1044,
            Mode::ExtendedReverseWrap => 1045,
            Mode::AlternateScreenTite => 1046,
            Mode::AlternateScreen => 1047,
            Mode::SaveCursorMode => 1048,
            Mode::SaveCursorAndAlternate => 1049,
            Mode::TerminfoFunctionKeys => 1050,
            Mode::SunFunctionKeys => 1051,
            Mode::HPFunctionKeys => 1052,
            Mode::SCOFunctionKeys => 1053,
            Mode::LegacyKeyboard => 1060,
            Mode::VT220Keyboard => 1061,
            Mode::ReadlineMouse1 => 2001,
            Mode::ReadlineMouse2 => 2002,
            Mode::ReadlineMouse3 => 2003,
            Mode::BracketedPaste => 2004,
            Mode::ReadlineCharQuoting => 2005,
            Mode::ReadlineNewlinePaste => 2006,
            Mode::SynchronizedOutput => 2026,
        })
    }
}


/// Mode Setting
///
/// Indicates the mode setting for [`AnsiMode`] and [`DecMode`]s.
#[derive(Default, Copy, Debug, Hash)]
#[derive_const(Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum ModeSetting {
    /// Not recognized
    #[default]
    NotRecognized = 0,
    /// Set
    Set = 1,
    /// Reset
    Reset = 2,
    /// Permanently set
    PermanentlySet = 3,
    /// Permanently reset
    PermanentlyReset = 4,
}

const impl ModeSetting {
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