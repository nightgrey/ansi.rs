use bitflags::*;
use super::*;

bitflags! {
    pub struct Flags: u32 {
    /// When this flag is set, the driver will treat both Ctrl+Space and Ctrl+@
    /// as the same key sequence.
    ///
    /// Historically, the ANSI specs generate NUL (0x00) on both the Ctrl+Space
    /// and Ctrl+@ key sequences. This flag allows the driver to treat both as
    /// the same key sequence.
    const CtrlAt = 1 << 0;

    /// When this flag is set, the driver will treat the Tab key and Ctrl+I as
    /// the same key sequence.
    ///
    /// Historically, the ANSI specs generate HT (0x09) on both the Tab key and
    /// Ctrl+I. This flag allows the driver to treat both as the same key
    /// sequence.
    const CtrlI = 1 << 1;

    /// When this flag is set, the driver will treat the Enter key and Ctrl+M as
    /// the same key sequence.
    ///
    /// Historically, the ANSI specs generate CR (0x0D) on both the Enter key
    /// and Ctrl+M. This flag allows the driver to treat both as the same key.
    const CtrlM = 1 << 2;

    /// When this flag is set, the driver will treat Escape and Ctrl+[ as
    /// the same key sequence.
    ///
    /// Historically, the ANSI specs generate ESC (0x1B) on both the Escape key
    /// and Ctrl+[. This flag allows the driver to treat both as the same key
    /// sequence.
    const CtrlOpenBracket = 1 << 3;

    /// When this flag is set, the driver will send a BS (0x08 byte) character
    /// instead of a DEL (0x7F byte) character when the Backspace key is
    /// pressed.
    ///
    /// The VT100 terminal has both a Backspace and a Delete key. The VT220
    /// terminal dropped the Backspace key and replaced it with the Delete key.
    /// Both terminals send a DEL character when the Delete key is pressed.
    /// Modern terminals and PCs later readded the Delete key but used a
    /// different key sequence, and the Backspace key was standardized to send a
    /// DEL character.
    const Backspace = 1 << 4;

    /// When this flag is set, the driver will recognize the Find key instead of
    /// treating it as a Home key.
    ///
    /// The Find key was part of the VT220 keyboard, and is no longer used in
    /// modern day PCs.
    const Find = 1 << 5;

    /// When this flag is set, the driver will recognize the Select key instead
    /// of treating it as a End key.
    ///
    /// The Symbol key was part of the VT220 keyboard, and is no longer used in
    /// modern day PCs.
    const Select = 1 << 6;

    /// When this flag is set, the driver will preserve function keys (F13-F63)
    /// as symbols.
    ///
    /// Since these keys are not part of today's standard 20th century keyboard,
    /// we treat them as F1-F12 modifier keys i.e. ctrl/shift/alt + Fn combos.
    /// Key definitions come from Terminfo, this flag is only useful when
    /// FlagTerminfo is not set.
    const FKeys = 1 << 7;
    }
}


// EventDecoder decodes terminal input events from a byte buffer. Terminal
// input events are typically encoded as Unicode or ASCII characters, control
// codes, or ANSI escape sequences.
struct Decoder {
    flags: Flags,
    lastCks: Meta,
}