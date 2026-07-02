//! Bytes → [`Event`] decoder, translated from charm's `ultraviolet` decoder
//! (see `parse.reference.go`).
//!
//! NOTE: Starting-point translation — intentionally references symbols that
//! don't all exist yet:
//!
//! - `KeyEvent` fields: `text: Option<String>`, `shifted_key: Option<char>`,
//!   `base_key: Option<char>` (in addition to `key`, `kind`, `meta`)
//! - `KeyKind::{Press, Repeat, Release}`
//! - `Key::{Char(char), F(u8), Kp(u8), Extended, ...}` — see the match arms
//! - `Meta::{Super, Hyper, Meta}` in addition to the existing flags
//! - `Event` variants: `Focus`, `Blur`, `PasteStart`, `PasteEnd`, `Multi`,
//!   `CursorPosition`, `ModeReport`, `KeyboardEnhancements`,
//!   `ModifyOtherKeys`, `ColorScheme`, `ForegroundColor`, `BackgroundColor`,
//!   `CursorColor`, `Clipboard`, `Capability`, `TerminalVersion`,
//!   `KittyGraphics`, `{Primary,Secondary,Tertiary}DeviceAttributes`,
//!   `WindowSize`, `PixelSize`, `CellSize`, `WindowOp`, `Unknown`, `Ignored`
//! - Helpers: `xparse_color`, `base64_decode`, `hex_decode`,
//!   `utf8::first_char`, `grapheme::first_cluster`
//!
//! Omitted from the reference: `UseTerminfo` and everything Windows/Win32
//! (`CSI _` input mode, control-key-state translation).
//! 

// Done — crates/ansi/src/events/parse.rs now holds the full translation (~1,300 lines). The decoding logic — every match arm from the Go reference — is transported; it won't compile yet, by design, since it references the assumed symbols listed in the module doc comment at the top of the file.
// 
// Structure (mirrors the reference):
// - Decoder::decode(&[u8]) -> (usize, Option<Event>) — top-level dispatch on the first byte: lone ESC → Escape key, ESC-prefixed introducers, raw C1 bytes → Ctrl+Alt keys, controls, UTF-8 text. The Alt-prefix recursion (ESC <seq> → re-decode with Meta::Alt) is there too.
// - parse_csi with the full command table: DECRPM, DA1/DA2, kitty flags query reply, DECXCPR, SGR + X10 mouse, modifyOtherKeys, light/dark scheme, focus/blur, the ambiguous CPR-vs-modified-F3 case, legacy ~/^/@ function keys, bracketed paste markers, window ops, and the URxvt $-intermediate retry hack.
// - parse_ss3, parse_osc (colors 10/11/12, clipboard 52), parse_st_terminated (shared by PM/SOS/APC), parse_dcs (XTGETTCAP, XTVersion, DA3), parse_apc (ki
// - Go's packed ansi.Cmd bitfield became a (prefix, intermediate, final) tuple match; the param HasMoreFlag/MissingParam packing became an explicit Param { value, missing, has_more } struct.
// - Wheel buttons map to your Event::Scroll(ScrollEvent) instead of a MouseWheel event; other buttons to Event::Pointer with PointerKind::{Press, Release, Motion}. Kitty's IsRepeat became KeyKind::Repeat.
// - I did not reproduce the Go fallthrough in the kitty alternate-keycodes handling (where the base-key subparam also overwrites the shifted key) — it looks like an upstream quirk/bug; noted in a comment at (0, 2).
// - The seven Unknown{Csi,Osc,...}Event types collapse into unknown_* one-liner helpers over Event::Unknown; Go's ignoredEvent became Event::Ignored.
// 
// Skipped per your notes: UseTerminfo, Win32 input mode (CSI _ arm is a comment), the Windows key-event translator, and the unused color-math utilities (rgbToHSL, isDarkColor, colorToHex).
// 
// The biggest thing to decide when you wire this up for real: decode currently mimics the reference's "buffer ends mid-sequence → return what we have as Unknown" behavior. For the Events<R: Read> iterator you'll want those truncation paths (i >= b.len() returns) to become an Incomplete outcome plus a more flag for the lone-ESC case, as we discussed — they're easy to spot since they're all return (i, Some(unknown(&b[..i]))) right after an end-of-buffer check.

use bitflags::*;
use super::*;

// C0/C1 bytes used by the decoder.
const NUL: u8 = 0x00;
const SOH: u8 = 0x01;
const BEL: u8 = 0x07;
const BS: u8 = 0x08;
const HT: u8 = 0x09;
const CR: u8 = 0x0D;
const CAN: u8 = 0x18;
const SUB: u8 = 0x1A;
const ESC: u8 = 0x1B;
const FS: u8 = 0x1C;
const US: u8 = 0x1F;
const SP: u8 = 0x20;
const DEL: u8 = 0x7F;
const SS3: u8 = 0x8F;
const DCS: u8 = 0x90;
const SOS: u8 = 0x98;
const CSI: u8 = 0x9B;
const ST: u8 = 0x9C;
const OSC: u8 = 0x9D;
const PM: u8 = 0x9E;
const APC: u8 = 0x9F;

bitflags! {
    #[derive(Debug, Default)]
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

pub const MAX_PARAMS: usize = 32;

/// A CSI/DCS numeric parameter. `has_more` marks a parameter that is followed
/// by a `:` subparameter (mirrors the reference decoder's `HasMoreFlag`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Param {
    pub value: u16,
    pub missing: bool,
    pub has_more: bool,
}

impl Param {
    pub const MISSING: Self = Self { value: 0, missing: true, has_more: false };

    /// The parameter's value, or `default` if the parameter was omitted.
    pub fn value_or(self, default: i32) -> i32 {
        if self.missing { default } else { self.value as i32 }
    }
}

/// A borrowed slice of accumulated parameters.
#[derive(Debug, Clone, Copy)]
pub struct Params<'a>(pub &'a [Param]);

impl Params<'_> {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// The parameter at `i`, or `default` if it was omitted. `None` when `i`
    /// is out of range.
    pub fn param(&self, i: usize, default: i32) -> Option<i32> {
        self.0.get(i).map(|p| p.value_or(default))
    }
}

/// Decodes terminal input events from a byte buffer. Terminal input events
/// are typically encoded as Unicode or ASCII characters, control codes, or
/// ANSI escape sequences.
#[derive(Debug, Default)]
pub struct Decoder {
    /// Legacy key encoding flags, see [`Flags`].
    pub flags: Flags,
}

impl Decoder {
    /// Finds the first recognized event sequence and returns its length along
    /// with the decoded event.
    ///
    /// Returns `(0, None)` when the buffer is empty. If a sequence is not
    /// supported, an [`Event::Unknown`] is returned.
    pub fn decode(&mut self, buf: &[u8]) -> (usize, Option<Event>) {
        if buf.is_empty() {
            return (0, None);
        }

        match buf[0] {
            ESC => {
                if buf.len() == 1 {
                    // Escape key
                    return (1, Some(Event::Key(press(Key::Escape))));
                }

                match buf[1] {
                    b'O' => self.parse_ss3(buf), // Esc-prefixed SS3
                    b'P' => self.parse_dcs(buf), // Esc-prefixed DCS
                    b'[' => self.parse_csi(buf), // Esc-prefixed CSI
                    b']' => self.parse_osc(buf), // Esc-prefixed OSC
                    b'_' => self.parse_apc(buf), // Esc-prefixed APC
                    b'^' => self.parse_st_terminated(PM, b'^', None, buf),
                    b'X' => self.parse_st_terminated(SOS, b'X', None, buf),
                    _ => {
                        let (n, event) = self.decode(&buf[1..]);
                        if let Some(Event::Key(mut k)) = event
                            && k.kind == KeyKind::Press
                        {
                            k.text = None;
                            k.meta |= Meta::Alt;
                            return (n + 1, Some(Event::Key(k)));
                        }

                        // Not a key sequence, nor an alt modified key
                        // sequence. In that case, just report a single escape
                        // key.
                        (1, Some(Event::Key(press(Key::Escape))))
                    }
                }
            }
            SS3 => self.parse_ss3(buf),
            DCS => self.parse_dcs(buf),
            CSI => self.parse_csi(buf),
            OSC => self.parse_osc(buf),
            APC => self.parse_apc(buf),
            PM => self.parse_st_terminated(PM, b'^', None, buf),
            SOS => self.parse_st_terminated(SOS, b'X', None, buf),
            b => {
                if b <= US || b == DEL || b == SP {
                    (1, Some(self.parse_control(b)))
                } else if (0x80..=0x9F).contains(&b) {
                    // C1 control code. UTF-8 never starts with a C1 control
                    // code; encode these as Ctrl+Alt+<code - 0x40>.
                    let key = Key::Char((b - 0x40) as char);
                    (1, Some(Event::Key(press_mod(key, Meta::Ctrl | Meta::Alt))))
                } else {
                    self.parse_utf8(buf)
                }
            }
        }
    }

    fn parse_control(&self, b: u8) -> Event {
        let k = match b {
            NUL => {
                if self.flags.contains(Flags::CtrlAt) {
                    press_mod(Key::Char('@'), Meta::Ctrl)
                } else {
                    press_mod(Key::Space, Meta::Ctrl)
                }
            }
            BS => press_mod(Key::Char('h'), Meta::Ctrl),
            HT => {
                if self.flags.contains(Flags::CtrlI) {
                    press_mod(Key::Char('i'), Meta::Ctrl)
                } else {
                    press(Key::Tab)
                }
            }
            CR => {
                if self.flags.contains(Flags::CtrlM) {
                    press_mod(Key::Char('m'), Meta::Ctrl)
                } else {
                    press(Key::Enter)
                }
            }
            ESC => {
                if self.flags.contains(Flags::CtrlOpenBracket) {
                    press_mod(Key::Char('['), Meta::Ctrl)
                } else {
                    press(Key::Escape)
                }
            }
            DEL => {
                if self.flags.contains(Flags::Backspace) {
                    press(Key::Delete)
                } else {
                    press(Key::Backspace)
                }
            }
            SP => {
                let mut k = press(Key::Space);
                k.text = Some(" ".into());
                k
            }
            // Use lower case letters for control codes.
            SOH..=SUB => press_mod(Key::Char((b + 0x60) as char), Meta::Ctrl),
            FS..=US => press_mod(Key::Char((b + 0x40) as char), Meta::Ctrl),
            _ => return unknown(&[b]),
        };
        Event::Key(k)
    }

    fn parse_utf8(&mut self, b: &[u8]) -> (usize, Option<Event>) {
        if b.is_empty() {
            return (0, None);
        }

        let c = b[0];
        if c <= US || c == DEL {
            // Control codes get handled by parse_control.
            return (1, Some(self.parse_control(c)));
        } else if c < DEL {
            // ASCII printable characters
            let code = c as char;
            let mut k = press(Key::Char(code));
            k.text = Some(code.to_string());
            if code.is_ascii_uppercase() {
                // Convert upper case letters to lower case + shift modifier.
                k.key = Key::Char(code.to_ascii_lowercase());
                k.shifted_key = Some(code);
                k.meta |= Meta::Shift;
            }
            return (1, Some(Event::Key(k)));
        }

        let Some((code, _)) = utf8::first_char(b) else {
            return (1, Some(unknown(&b[..1])));
        };

        // Use `Key::Extended` for multi-rune graphemes.
        let cluster = grapheme::first_cluster(b);
        let key = if cluster.chars().count() > 1 {
            Key::Extended
        } else {
            Key::Char(code)
        };

        let mut k = press(key);
        k.text = Some(cluster.to_string());
        (cluster.len(), Some(Event::Key(k)))
    }
}

// ---- Key event helpers ----------------------------------------------------

fn press(key: Key) -> KeyEvent {
    KeyEvent {
        key,
        kind: KeyKind::Press,
        ..Default::default()
    }
}

fn press_mod(key: Key, meta: Meta) -> KeyEvent {
    KeyEvent {
        key,
        kind: KeyKind::Press,
        meta,
        ..Default::default()
    }
}

fn arrow(n: u8) -> Key {
    match n {
        0 => Key::Up,
        1 => Key::Down,
        2 => Key::Right,
        _ => Key::Left,
    }
}

/// Converts an xterm `1 + bitfield` modifier parameter (already decremented
/// by one) into [`Meta`]. Bits: 1 shift, 2 alt, 4 ctrl, 8 meta.
fn from_xterm_mod(m: i32) -> Meta {
    let mut meta = Meta::empty();
    if m & 1 != 0 {
        meta |= Meta::Shift;
    }
    if m & 2 != 0 {
        meta |= Meta::Alt;
    }
    if m & 4 != 0 {
        meta |= Meta::Ctrl;
    }
    if m & 8 != 0 {
        meta |= Meta::Meta;
    }
    meta
}

// ---- Unknown/ignored events -----------------------------------------------
//
// The reference decoder distinguishes Unknown{Csi,Osc,Ss3,Dcs,Pm,Sos,Apc}
// events; collapsed here into `Event::Unknown` carrying the raw bytes.

fn unknown(b: &[u8]) -> Event {
    Event::Unknown(b.to_vec())
}

fn unknown_csi(b: &[u8]) -> Event {
    unknown(b)
}

fn unknown_ss3(b: &[u8]) -> Event {
    unknown(b)
}

fn unknown_osc(b: &[u8]) -> Event {
    unknown(b)
}

fn unknown_dcs(b: &[u8]) -> Event {
    unknown(b)
}

/// A cancelled/aborted sequence: its bytes are consumed, but it produces
/// nothing meaningful.
fn ignored(b: &[u8]) -> Event {
    Event::Ignored(b.to_vec())
}

// ---- CSI --------------------------------------------------------------

impl Decoder {
    fn parse_csi(&mut self, b: &[u8]) -> (usize, Option<Event>) {
        if b.len() == 2 && b[0] == ESC {
            // Shortcut if this is an alt+[ key.
            return (2, Some(Event::Key(press_mod(Key::Char(b[1] as char), Meta::Alt))));
        }

        let mut params = [Param::MISSING; MAX_PARAMS];
        let mut params_len = 0;

        let mut i = 0;
        if b[i] == CSI || b[i] == ESC {
            i += 1;
        }
        if i < b.len() && b[i - 1] == ESC && b[i] == b'[' {
            i += 1;
        }

        // Initial (private marker) byte in the range 0x3C..=0x3F. The param
        // scan below consumes it, since it falls in the 0x30..=0x3F range.
        let mut prefix = 0u8;
        if i < b.len() && (b'<'..=b'?').contains(&b[i]) {
            prefix = b[i];
        }

        // Scan parameter bytes in the range 0x30..=0x3F.
        let mut j = 0;
        while i < b.len() && params_len < params.len() && (0x30..=0x3F).contains(&b[i]) {
            match b[i] {
                b'0'..=b'9' => {
                    let p = &mut params[params_len];
                    p.missing = false;
                    p.value = p.value.saturating_mul(10).saturating_add((b[i] - b'0') as u16);
                }
                b':' | b';' => {
                    params[params_len].has_more = b[i] == b':';
                    params_len += 1;
                    if params_len < params.len() {
                        // Don't overflow the params slice.
                        params[params_len] = Param::MISSING;
                    }
                }
                _ => {}
            }
            i += 1;
            j += 1;
        }

        if j > 0 && params_len < params.len() {
            // Has parameters.
            params_len += 1;
        }

        // Scan intermediate bytes in the range 0x20..=0x2F.
        let mut intermed = 0u8;
        while i < b.len() && (0x20..=0x2F).contains(&b[i]) {
            intermed = b[i];
            i += 1;
        }

        // Scan the final byte in the range 0x40..=0x7E.
        if i >= b.len() || !(0x40..=0x7E).contains(&b[i]) {
            // Special case for URxvt keys: CSI <number> $ is an invalid
            // sequence, but URxvt uses it for shift modified keys.
            if intermed == b'$' && i > 0 && b[i - 1] == b'$' {
                let mut buf = b[..i - 1].to_vec();
                buf.push(b'~');
                let (n, ev) = self.parse_csi(&buf);
                if let Some(Event::Key(mut k)) = ev {
                    k.meta |= Meta::Shift;
                    return (n, Some(Event::Key(k)));
                }
            }
            return (i, Some(unknown(&b[..i])));
        }

        let final_byte = b[i];
        i += 1;

        let pa = Params(&params[..params_len]);

        // (prefix, intermediate, final). Arms yield `None` to fall through to
        // `Event::Unknown` (the reference decoder's `break`).
        let event: Option<Event> = match (prefix, intermed, final_byte) {
            (b'?', b'$', b'y') => 'arm: {
                // Report Mode (DECRPM)
                let Some(mode) = pa.param(0, -1) else { break 'arm None };
                if mode == -1 {
                    break 'arm None;
                }
                let Some(value) = pa.param(1, 0) else { break 'arm None };
                Some(Event::ModeReport {
                    mode: Mode::Dec(mode as u16),
                    value: value as u16,
                })
            }
            (b'?', 0, b'c') => {
                // Primary Device Attributes
                Some(parse_primary_dev_attrs(pa))
            }
            (b'>', 0, b'c') => {
                // Secondary Device Attributes
                Some(parse_secondary_dev_attrs(pa))
            }
            (b'?', 0, b'u') => {
                // Kitty keyboard enhancement flags
                let flags = pa.param(0, -1).unwrap_or(-1);
                Some(Event::KeyboardEnhancements(flags))
            }
            (b'?', 0, b'R') => 'arm: {
                // DECXCPR cursor position report. This report may include a
                // third parameter for the page number, which we don't need.
                let row = pa.param(0, 1).unwrap_or(1);
                let Some(col) = pa.param(1, 1) else { break 'arm None };
                Some(Event::CursorPosition(Point::new(col - 1, row - 1)))
            }
            (b'<', 0, b'm' | b'M') => {
                // SGR mouse
                if pa.len() == 3 {
                    Some(parse_sgr_mouse(final_byte, pa))
                } else {
                    None
                }
            }
            (b'>', 0, b'm') => 'arm: {
                // XTerm modifyOtherKeys report
                let Some(mok) = pa.param(0, 0) else { break 'arm None };
                if mok != 4 {
                    break 'arm None;
                }
                let Some(val) = pa.param(1, -1) else { break 'arm None };
                if val == -1 {
                    break 'arm None;
                }
                Some(Event::ModifyOtherKeys(val as u8))
            }
            (b'?', 0, b'n') => {
                // Light/dark color scheme report (CSI ? 997 ; 1/2 n)
                let report = pa.param(0, -1).unwrap_or(-1);
                let dark_light = pa.param(1, -1).unwrap_or(-1);
                match (report, dark_light) {
                    (997, 1) => Some(Event::ColorScheme(ColorScheme::Dark)),
                    (997, 2) => Some(Event::ColorScheme(ColorScheme::Light)),
                    _ => None,
                }
            }
            (0, 0, b'I') => Some(Event::Focus),
            (0, 0, b'O') => Some(Event::Blur),
            (0, 0, b'R') => 'arm: {
                // Cursor position report OR modified F3.
                if pa.len() == 2 {
                    let Some(row) = pa.param(0, 1) else { break 'arm None };
                    let Some(col) = pa.param(1, 1) else { break 'arm None };
                    let m = Event::CursorPosition(Point::new(col - 1, row - 1));
                    if row == 1 && col - 1 <= 0b1111 {
                        // We cannot differentiate between a cursor position
                        // report and CSI 1 ; <mod> R (modified F3) when the
                        // cursor is on row 1, so report both. For an
                        // unambiguous report, use DECXCPR (CSI ? 6 n) instead.
                        break 'arm Some(Event::Multi(vec![
                            Event::Key(press_mod(Key::F(3), from_xterm_mod(col - 1))),
                            m,
                        ]));
                    }
                    break 'arm Some(m);
                }

                if pa.len() != 0 {
                    break 'arm None;
                }

                // Unmodified F3 key (CSI R)
                csi_func_key(final_byte, pa)
            }
            (0, 0, b'a'..=b'd' | b'A'..=b'F' | b'H' | b'P'..=b'S' | b'Z') => {
                csi_func_key(final_byte, pa)
            }
            (0, 0, b'M') => {
                // X10 mouse encoding: CSI M followed by three raw payload
                // bytes (Cb Cx Cy).
                if i + 3 > b.len() {
                    return (i, Some(unknown_csi(&b[..i])));
                }
                return (i + 3, Some(parse_x10_mouse(&b[i..i + 3])));
            }
            (0, b'$', b'y') => 'arm: {
                // Report Mode (DECRPM) — ANSI mode
                let Some(mode) = pa.param(0, -1) else { break 'arm None };
                if mode == -1 {
                    break 'arm None;
                }
                let Some(value) = pa.param(1, 0) else { break 'arm None };
                Some(Event::ModeReport {
                    mode: Mode::Ansi(mode as u16),
                    value: value as u16,
                })
            }
            (0, 0, b'u') => {
                // Kitty keyboard protocol & CSI u (fixterms)
                if pa.len() == 0 {
                    return (i, Some(unknown_csi(&b[..i])));
                }
                Some(parse_kitty_keyboard(pa))
            }
            // NOTE: CSI _ (Win32 Input Mode) intentionally not supported.
            (0, 0, b'@' | b'^' | b'~') => 'arm: {
                if pa.len() == 0 {
                    break 'arm Some(unknown_csi(&b[..i]));
                }

                let param = pa.param(0, 0).unwrap_or(0);
                if final_byte == b'~' {
                    match param {
                        27 => {
                            // XTerm modifyOtherKeys 2
                            if pa.len() != 3 {
                                break 'arm Some(unknown_csi(&b[..i]));
                            }
                            break 'arm Some(parse_xterm_modify_other_keys(pa));
                        }
                        200 => break 'arm Some(Event::PasteStart),
                        201 => break 'arm Some(Event::PasteEnd),
                        _ => {}
                    }
                }

                let key = match param {
                    1 if self.flags.contains(Flags::Find) => Key::Find,
                    1 => Key::Home,
                    2 => Key::Insert,
                    3 => Key::Delete,
                    4 if self.flags.contains(Flags::Select) => Key::Select,
                    4 => Key::End,
                    5 => Key::PageUp,
                    6 => Key::PageDown,
                    7 => Key::Home,
                    8 => Key::End,
                    11..=15 => Key::F(1 + (param - 11) as u8),
                    17..=21 => Key::F(6 + (param - 17) as u8),
                    23..=26 => Key::F(11 + (param - 23) as u8),
                    28 | 29 => Key::F(15 + (param - 28) as u8),
                    31..=34 => Key::F(17 + (param - 31) as u8),
                    _ => break 'arm None,
                };
                let mut k = press(key);

                // Modifiers
                let m = pa.param(1, -1).unwrap_or(-1);
                if pa.len() > 1 && m != -1 {
                    k.meta |= from_xterm_mod(m - 1);
                }

                // Handle URxvt weird keys.
                match final_byte {
                    // Don't forget to handle Kitty keyboard protocol.
                    b'~' => break 'arm Some(parse_kitty_keyboard_ext(pa, k)),
                    b'^' => k.meta |= Meta::Ctrl,
                    b'@' => k.meta |= Meta::Ctrl | Meta::Shift,
                    _ => {}
                }

                Some(Event::Key(k))
            }
            (0, 0, b't') => 'arm: {
                let Some(param) = pa.param(0, 0) else { break 'arm None };

                match (param, pa.len()) {
                    (4, 3) => {
                        // Report terminal window size in pixels.
                        let height = pa.param(1, 0).unwrap_or(0);
                        let width = pa.param(2, 0).unwrap_or(0);
                        break 'arm Some(Event::PixelSize(Size::new(width, height)));
                    }
                    (6, 3) => {
                        // Report terminal character cell size.
                        let height = pa.param(1, 0).unwrap_or(0);
                        let width = pa.param(2, 0).unwrap_or(0);
                        break 'arm Some(Event::CellSize(Size::new(width, height)));
                    }
                    (8, 3) => {
                        // Report terminal window size in cells.
                        let height = pa.param(1, 0).unwrap_or(0);
                        let width = pa.param(2, 0).unwrap_or(0);
                        break 'arm Some(Event::WindowSize(Size::new(width, height)));
                    }
                    (48, 5) => {
                        // In-band terminal size report.
                        let cell_height = pa.param(1, 0).unwrap_or(0);
                        let cell_width = pa.param(2, 0).unwrap_or(0);
                        let pixel_height = pa.param(3, 0).unwrap_or(0);
                        let pixel_width = pa.param(4, 0).unwrap_or(0);
                        break 'arm Some(Event::Multi(vec![
                            Event::WindowSize(Size::new(cell_width, cell_height)),
                            Event::PixelSize(Size::new(pixel_width, pixel_height)),
                        ]));
                    }
                    _ => {}
                }

                // Any other window operation event.
                let mut args = Vec::new();
                for j in 1..pa.len() {
                    if let Some(val) = pa.param(j, 0) {
                        args.push(val);
                    }
                }
                Some(Event::WindowOp { op: param, args })
            }
            _ => None,
        };

        match event {
            Some(event) => (i, Some(event)),
            None => (i, Some(unknown_csi(&b[..i]))),
        }
    }
}

/// Legacy function-key final bytes shared by several CSI forms (arrows,
/// Home/End, F1-F4, shift+Tab, ...), including the Kitty keyboard protocol
/// extensions for them. `None` falls through to `Event::Unknown`.
fn csi_func_key(cmd: u8, pa: Params) -> Option<Event> {
    let mut k = match cmd {
        b'a'..=b'd' => press_mod(arrow(cmd - b'a'), Meta::Shift),
        b'A'..=b'D' => press(arrow(cmd - b'A')),
        b'E' => press(Key::Begin),
        b'F' => press(Key::End),
        b'H' => press(Key::Home),
        b'P'..=b'S' => press(Key::F(1 + cmd - b'P')),
        b'Z' => press_mod(Key::Tab, Meta::Shift),
        _ => return None,
    };

    let id = pa.param(0, 1).unwrap_or(1);
    let m = pa.param(1, 1).unwrap_or(1);
    if (pa.len() > 2 && !pa.0[1].has_more) || id != 1 {
        return None;
    }
    if pa.len() > 1 && id == 1 && m != -1 {
        // CSI 1 ; <modifiers> <cmd>
        k.meta |= from_xterm_mod(m - 1);
    }

    // Don't forget to handle Kitty keyboard protocol.
    Some(parse_kitty_keyboard_ext(pa, k))
}

// ---- SS3 / OSC / DCS / APC / ST-terminated ------------------------------

impl Decoder {
    /// Parses an SS3 sequence.
    ///
    /// See <https://vt100.net/docs/vt220-rm/chapter4.html#S4.4.4.2>
    fn parse_ss3(&mut self, b: &[u8]) -> (usize, Option<Event>) {
        if b.len() == 2 && b[0] == ESC {
            // Shortcut if this is an alt+O key.
            let c = (b[1] as char).to_ascii_lowercase();
            return (2, Some(Event::Key(press_mod(Key::Char(c), Meta::Shift | Meta::Alt))));
        }

        let mut i = 0;
        if b[i] == SS3 || b[i] == ESC {
            i += 1;
        }
        if i < b.len() && b[i - 1] == ESC && b[i] == b'O' {
            i += 1;
        }

        // Scan numbers from 0-9.
        let mut m = 0i32;
        while i < b.len() && b[i].is_ascii_digit() {
            m = m * 10 + (b[i] - b'0') as i32;
            i += 1;
        }

        // Scan a GL character: a single byte in the range 0x21..=0x7E.
        // See <https://vt100.net/docs/vt220-rm/chapter2.html#S2.3.2>
        if i >= b.len() || !(0x21..=0x7E).contains(&b[i]) {
            return (i, Some(unknown(&b[..i])));
        }

        let gl = b[i];
        i += 1;

        let mut k = match gl {
            b'a'..=b'd' => press_mod(arrow(gl - b'a'), Meta::Ctrl),
            b'A'..=b'D' => press(arrow(gl - b'A')),
            b'E' => press(Key::Begin),
            b'F' => press(Key::End),
            b'H' => press(Key::Home),
            b'P'..=b'S' => press(Key::F(1 + gl - b'P')),
            b'M' => press(Key::KpEnter),
            b'X' => press(Key::KpEqual),
            // VT220 application keypad: j..y
            b'j' => press(Key::KpMultiply),
            b'k' => press(Key::KpPlus),
            b'l' => press(Key::KpComma),
            b'm' => press(Key::KpMinus),
            b'n' => press(Key::KpDecimal),
            b'o' => press(Key::KpDivide),
            b'p'..=b'y' => press(Key::Kp(gl - b'p')),
            _ => return (i, Some(unknown_ss3(&b[..i]))),
        };

        // Handle weird SS3 <modifier> Func.
        if m > 0 {
            k.meta |= from_xterm_mod(m - 1);
        }

        (i, Some(Event::Key(k)))
    }

    fn parse_osc(&mut self, b: &[u8]) -> (usize, Option<Event>) {
        let default_key = |b: &[u8]| Event::Key(press_mod(Key::Char(b[1] as char), Meta::Alt));
        if b.len() == 2 && b[0] == ESC {
            // Shortcut if this is an alt+] key.
            return (2, Some(default_key(b)));
        }

        let mut i = 0;
        if b[i] == OSC || b[i] == ESC {
            i += 1;
        }
        if i < b.len() && b[i - 1] == ESC && b[i] == b']' {
            i += 1;
        }

        // Parse the OSC command number. An OSC sequence is terminated by a
        // BEL, ESC, or ST character.
        let mut cmd: i32 = -1;
        while i < b.len() && b[i].is_ascii_digit() {
            if cmd == -1 {
                cmd = 0;
            } else {
                cmd *= 10;
            }
            cmd += (b[i] - b'0') as i32;
            i += 1;
        }

        let mut start = 0;
        if i < b.len() && b[i] == b';' {
            // Mark the start of the sequence data.
            i += 1;
            start = i;
        }

        // Advance to the end of the sequence.
        while i < b.len() && !matches!(b[i], BEL | ESC | ST | CAN | SUB) {
            i += 1;
        }

        if i >= b.len() {
            return (i, Some(unknown(&b[..i])));
        }

        let end = i; // end of the sequence data
        i += 1;

        // Check the 7-bit ST (string terminator) character.
        match b[i - 1] {
            CAN | SUB => return (i, Some(ignored(&b[..i]))),
            ESC => {
                if i >= b.len() || b[i] != b'\\' {
                    if cmd == -1 || (start == 0 && end == 2) {
                        return (2, Some(default_key(b)));
                    }

                    // No valid ST terminator: this is a cancelled sequence
                    // and should be ignored.
                    return (i, Some(ignored(&b[..i])));
                }
                i += 1;
            }
            _ => {}
        }

        if end <= start {
            return (i, Some(unknown(&b[..i])));
        }

        let data = &b[start..end];
        match cmd {
            10 => return (i, Some(Event::ForegroundColor(xparse_color(data)))),
            11 => return (i, Some(Event::BackgroundColor(xparse_color(data)))),
            12 => return (i, Some(Event::CursorColor(xparse_color(data)))),
            52 => {
                let mut parts = data.splitn(2, |&c| c == b';');
                let (sel, payload) = (parts.next().unwrap_or_default(), parts.next());
                let (Some(&sel), Some(payload)) = (sel.first(), payload) else {
                    return (i, Some(Event::Clipboard {
                        selection: None,
                        content: String::new(),
                    }));
                };

                let content = match base64_decode(payload) {
                    Some(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
                    None => String::from_utf8_lossy(payload).into_owned(),
                };
                return (i, Some(Event::Clipboard {
                    selection: Some(sel as char),
                    content,
                }));
            }
            _ => {}
        }

        (i, Some(unknown_osc(&b[..i])))
    }

    /// Parses a control sequence terminated by an ST character. The 7-bit ST
    /// is spelled `ESC \`.
    fn parse_st_terminated(
        &mut self,
        intro8: u8,
        intro7: u8,
        parse_data: Option<&dyn Fn(&[u8]) -> Option<Event>>,
        b: &[u8],
    ) -> (usize, Option<Event>) {
        let default_key = |b: &[u8]| -> (usize, Option<Event>) {
            match intro8 {
                SOS => {
                    let c = (b[1] as char).to_ascii_lowercase();
                    (2, Some(Event::Key(press_mod(Key::Char(c), Meta::Shift | Meta::Alt))))
                }
                PM | APC => (2, Some(Event::Key(press_mod(Key::Char(b[1] as char), Meta::Alt)))),
                _ => (0, None),
            }
        };

        if b.len() == 2 && b[0] == ESC {
            return default_key(b);
        }

        let mut i = 0;
        if b[i] == intro8 || b[i] == ESC {
            i += 1;
        }
        if i < b.len() && b[i - 1] == ESC && b[i] == intro7 {
            i += 1;
        }

        // Scan the sequence data.
        let start = i;
        while i < b.len() && !matches!(b[i], ESC | ST | CAN | SUB) {
            i += 1;
        }

        if i >= b.len() {
            return (i, Some(unknown(&b[..i])));
        }

        let end = i; // end of the sequence data
        i += 1;

        // Check the 7-bit ST (string terminator) character.
        match b[i - 1] {
            CAN | SUB => return (i, Some(ignored(&b[..i]))),
            ESC => {
                if i >= b.len() || b[i] != b'\\' {
                    if start == end {
                        return default_key(b);
                    }

                    // No valid ST terminator: this is a cancelled sequence
                    // and should be ignored.
                    return (i, Some(ignored(&b[..i])));
                }
                i += 1;
            }
            _ => {}
        }

        if let Some(parse_data) = parse_data
            && let Some(event) = parse_data(&b[start..end])
        {
            return (i, Some(event));
        }

        (i, Some(unknown(&b[..i])))
    }

    fn parse_dcs(&mut self, b: &[u8]) -> (usize, Option<Event>) {
        if b.len() == 2 && b[0] == ESC {
            // Shortcut if this is an alt+P key.
            let c = (b[1] as char).to_ascii_lowercase();
            return (2, Some(Event::Key(press_mod(Key::Char(c), Meta::Shift | Meta::Alt))));
        }

        let mut params = [Param::MISSING; 16];
        let mut params_len = 0;

        // DCS sequences are introduced by DCS (0x90) or ESC P.
        let mut i = 0;
        if b[i] == DCS || b[i] == ESC {
            i += 1;
        }
        if i < b.len() && b[i - 1] == ESC && b[i] == b'P' {
            i += 1;
        }

        // Initial (private marker) byte in the range 0x3C..=0x3F.
        let mut prefix = 0u8;
        if i < b.len() && (b'<'..=b'?').contains(&b[i]) {
            prefix = b[i];
        }

        // Scan parameter bytes in the range 0x30..=0x3F.
        let mut j = 0;
        while i < b.len() && params_len < params.len() && (0x30..=0x3F).contains(&b[i]) {
            match b[i] {
                b'0'..=b'9' => {
                    let p = &mut params[params_len];
                    p.missing = false;
                    p.value = p.value.saturating_mul(10).saturating_add((b[i] - b'0') as u16);
                }
                b':' | b';' => {
                    params[params_len].has_more = b[i] == b':';
                    params_len += 1;
                    if params_len < params.len() {
                        // Don't overflow the params slice.
                        params[params_len] = Param::MISSING;
                    }
                }
                _ => {}
            }
            i += 1;
            j += 1;
        }

        if j > 0 && params_len < params.len() {
            // Has parameters.
            params_len += 1;
        }

        // Scan intermediate bytes in the range 0x20..=0x2F.
        let mut intermed = 0u8;
        while i < b.len() && (0x20..=0x2F).contains(&b[i]) {
            intermed = b[i];
            i += 1;
        }

        // Scan the final byte in the range 0x40..=0x7E.
        if i >= b.len() || !(0x40..=0x7E).contains(&b[i]) {
            return (i, Some(unknown(&b[..i])));
        }

        let final_byte = b[i];
        i += 1;

        let start = i; // start of the sequence data
        while i < b.len() && b[i] != ST && b[i] != ESC {
            i += 1;
        }

        if i >= b.len() {
            return (i, Some(unknown(&b[..i])));
        }

        let end = i; // end of the sequence data
        i += 1;

        // Check the 7-bit ST (string terminator) character.
        if i < b.len() && b[i - 1] == ESC && b[i] == b'\\' {
            i += 1;
        }

        let pa = Params(&params[..params_len]);
        let data = &b[start..end];
        match (prefix, intermed, final_byte) {
            (0, b'+', b'r') => {
                // XTGETTCAP response. Param 1 means a valid response, 0 an
                // invalid one. Some terminals (e.g. kitty) report invalid
                // responses with their queries, i.e. querying "Tc" with
                // "\x1bP+q5463\x1b\\" returns "\x1bP0+r5463\x1b\\"; the specs
                // say invalid responses look like DCS 0 + r ST. Ignore
                // invalid responses and only report valid ones.
                //
                // See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Operating-System-Commands>
                if pa.param(0, 0).unwrap_or(0) == 1 {
                    return (i, Some(parse_termcap(data)));
                }
            }
            (b'>', 0, b'|') => {
                // XTVersion response
                return (i, Some(Event::TerminalVersion(
                    String::from_utf8_lossy(data).into_owned(),
                )));
            }
            (0, b'!', b'|') => {
                // Tertiary Device Attributes
                return (i, Some(parse_tertiary_dev_attrs(data)));
            }
            _ => {}
        }

        (i, Some(unknown_dcs(&b[..i])))
    }

    fn parse_apc(&mut self, b: &[u8]) -> (usize, Option<Event>) {
        if b.len() == 2 && b[0] == ESC {
            // Shortcut if this is an alt+_ key.
            return (2, Some(Event::Key(press_mod(Key::Char(b[1] as char), Meta::Alt))));
        }

        // APC sequences are introduced by APC (0x9F) or ESC _.
        self.parse_st_terminated(APC, b'_', Some(&|data: &[u8]| {
            match data.first() {
                // Kitty Graphics Protocol
                Some(b'G') => {
                    let mut parts = data[1..].splitn(2, |&c| c == b';');
                    let options = parts.next().unwrap_or_default().to_vec();
                    let payload = parts.next().unwrap_or_default().to_vec();
                    Some(Event::KittyGraphics { options, payload })
                }
                _ => None,
            }
        }), b)
    }
}

// ---- Kitty keyboard protocol --------------------------------------------

// Kitty keyboard protocol modifier bits.
const KITTY_SHIFT: i32 = 1 << 0;
const KITTY_ALT: i32 = 1 << 1;
const KITTY_CTRL: i32 = 1 << 2;
const KITTY_SUPER: i32 = 1 << 3;
const KITTY_HYPER: i32 = 1 << 4;
const KITTY_META: i32 = 1 << 5;
const KITTY_CAPS_LOCK: i32 = 1 << 6;
const KITTY_NUM_LOCK: i32 = 1 << 7;

fn from_kitty_mod(m: i32) -> Meta {
    let mut meta = Meta::empty();
    if m & KITTY_SHIFT != 0 {
        meta |= Meta::Shift;
    }
    if m & KITTY_ALT != 0 {
        meta |= Meta::Alt;
    }
    if m & KITTY_CTRL != 0 {
        meta |= Meta::Ctrl;
    }
    if m & KITTY_SUPER != 0 {
        meta |= Meta::Super;
    }
    if m & KITTY_HYPER != 0 {
        meta |= Meta::Hyper;
    }
    if m & KITTY_META != 0 {
        meta |= Meta::Meta;
    }
    if m & KITTY_CAPS_LOCK != 0 {
        meta |= Meta::CapsLock;
    }
    if m & KITTY_NUM_LOCK != 0 {
        meta |= Meta::NumLock;
    }
    meta
}

/// The Kitty keyboard protocol functional key mapping, including the faulty
/// C0 mappings some terminals (WezTerm & friends) produce.
fn kitty_key(code: u32) -> Option<(Key, Meta)> {
    let key = match code {
        0x08 => Key::Backspace,
        0x09 => Key::Tab,
        0x0D => Key::Enter,
        0x1B => Key::Escape,
        0x7F => Key::Backspace,

        // Faulty C0 mappings.
        0x00 => return Some((Key::Space, Meta::Ctrl)),
        c @ (0x01..=0x07 | 0x0A..=0x0C | 0x0E..=0x1A) => {
            return Some((Key::Char((c as u8 + 0x60) as char), Meta::Ctrl));
        }
        c @ 0x1C..=0x1F => return Some((Key::Char((c as u8 + 0x40) as char), Meta::Ctrl)),

        57344 => Key::Escape,
        57345 => Key::Enter,
        57346 => Key::Tab,
        57347 => Key::Backspace,
        57348 => Key::Insert,
        57349 => Key::Delete,
        57350 => Key::Left,
        57351 => Key::Right,
        57352 => Key::Up,
        57353 => Key::Down,
        57354 => Key::PageUp,
        57355 => Key::PageDown,
        57356 => Key::Home,
        57357 => Key::End,
        57358 => Key::CapsLock,
        57359 => Key::ScrollLock,
        57360 => Key::NumLock,
        57361 => Key::PrintScreen,
        57362 => Key::Pause,
        57363 => Key::Menu,
        c @ 57364..=57398 => Key::F((c - 57363) as u8), // F1..=F35
        c @ 57399..=57408 => Key::Kp((c - 57399) as u8), // Kp0..=Kp9
        57409 => Key::KpDecimal,
        57410 => Key::KpDivide,
        57411 => Key::KpMultiply,
        57412 => Key::KpMinus,
        57413 => Key::KpPlus,
        57414 => Key::KpEnter,
        57415 => Key::KpEqual,
        57416 => Key::KpSep,
        57417 => Key::KpLeft,
        57418 => Key::KpRight,
        57419 => Key::KpUp,
        57420 => Key::KpDown,
        57421 => Key::KpPageUp,
        57422 => Key::KpPageDown,
        57423 => Key::KpHome,
        57424 => Key::KpEnd,
        57425 => Key::KpInsert,
        57426 => Key::KpDelete,
        57427 => Key::KpBegin,
        57428 => Key::MediaPlay,
        57429 => Key::MediaPause,
        57430 => Key::MediaPlayPause,
        57431 => Key::MediaReverse,
        57432 => Key::MediaStop,
        57433 => Key::MediaFastForward,
        57434 => Key::MediaRewind,
        57435 => Key::MediaNext,
        57436 => Key::MediaPrev,
        57437 => Key::MediaRecord,
        57438 => Key::LowerVolume,
        57439 => Key::RaiseVolume,
        57440 => Key::Mute,
        57441 => Key::LeftShift,
        57442 => Key::LeftCtrl,
        57443 => Key::LeftAlt,
        57444 => Key::LeftSuper,
        57445 => Key::LeftHyper,
        57446 => Key::LeftMeta,
        57447 => Key::RightShift,
        57448 => Key::RightCtrl,
        57449 => Key::RightAlt,
        57450 => Key::RightSuper,
        57451 => Key::RightHyper,
        57452 => Key::RightMeta,
        57453 => Key::IsoLevel3Shift,
        57454 => Key::IsoLevel5Shift,
        _ => return None,
    };
    Some((key, Meta::empty()))
}

/// Parses a Kitty Keyboard Protocol sequence.
///
/// In `CSI u` (fixterms), this is parsed as:
///
/// ```text
/// CSI codepoint ; modifiers u
/// ```
///
/// The Kitty Keyboard Protocol extends this with optional components that
/// can be enabled progressively:
///
/// ```text
/// CSI unicode-key-code:alternate-key-codes ; modifiers:event-type ; text-as-codepoints u
/// ```
///
/// See <https://sw.kovidgoyal.net/kitty/keyboard-protocol/>
fn parse_kitty_keyboard(pa: Params) -> Event {
    let mut is_release = false;
    let mut k = press(Key::Char(char::REPLACEMENT_CHARACTER));
    let mut text = String::new();

    // Parameters are separated by ';', subparameters by ':'.
    let mut param_idx = 0;
    let mut sub_idx = 0;
    for p in pa.0 {
        // The protocol has 3 optional components.
        match (param_idx, sub_idx) {
            (0, 0) => {
                // CSI u has a default value of 1.
                let code = p.value_or(1) as u32;
                match kitty_key(code) {
                    Some((key, meta)) => {
                        k.key = key;
                        k.meta = meta;
                    }
                    None => {
                        k.key = Key::Char(
                            char::from_u32(code).unwrap_or(char::REPLACEMENT_CHARACTER),
                        );
                    }
                }
            }
            (0, 1) => {
                // Shifted key. We want the shifted key to be the one that is
                // reported: shift+a should produce "A" (with the base key
                // kept alongside).
                if let Some(s) = printable_char(p.value_or(1)) {
                    k.shifted_key = Some(s);
                }
            }
            (0, 2) => {
                // Base key: the standard PC-101 layout codepoint. Useful for
                // an unambiguous key mapping under alternative language
                // layouts. (The reference decoder falls through and also
                // overwrites the shifted key here.)
                if let Some(c) = printable_char(p.value_or(1)) {
                    k.base_key = Some(c);
                }
            }
            (1, 0) => {
                let m = p.value_or(1);
                if m > 1 {
                    k.meta = from_kitty_mod(m - 1);
                    if !(k.meta & !Meta::Shift).is_empty() {
                        // Clear the text if any modifier other than shift is
                        // held.
                        text.clear();
                    }
                }
            }
            (1, 1) => match p.value_or(1) {
                2 => k.kind = KeyKind::Repeat,
                3 => is_release = true,
                _ => {}
            },
            (2, _) => {
                // Associated text as codepoints.
                let code = p.value_or(0) as u32;
                if code != 0
                    && let Some(c) = char::from_u32(code)
                {
                    text.push(c);
                }
            }
            _ => {}
        }

        sub_idx += 1;
        if !p.has_more {
            param_idx += 1;
            sub_idx = 0;
        }
    }

    // Remove lock modifiers from now on, they don't affect the text.
    // (Kitty doesn't support scroll lock.)
    let key_mod = k.meta & !Meta::NumLock;

    let print_mod = key_mod.is_empty()
        || key_mod == Meta::Shift
        || key_mod == Meta::CapsLock
        || key_mod == Meta::Shift | Meta::CapsLock;

    if text.is_empty()
        && print_mod
        && let Some(t) = keypad_text(k.key)
    {
        text.push(t);
    }

    if text.is_empty()
        && print_mod
        && let Key::Char(c) = k.key
        && !c.is_control()
    {
        if key_mod.is_empty() {
            text.push(c);
        } else if let Some(s) = k.shifted_key {
            text.push(s);
        } else if key_mod.contains(Meta::Shift) || key_mod.contains(Meta::CapsLock) {
            text.extend(c.to_uppercase());
        } else {
            text.extend(c.to_lowercase());
        }
    }

    k.text = (!text.is_empty()).then_some(text);
    if is_release {
        k.kind = KeyKind::Release;
    }
    Event::Key(k)
}

/// Parses the Kitty keyboard protocol extensions for non-`CSI u` sequences
/// (`CSI A`, `SS3 A`, `CSI ~`, ...).
fn parse_kitty_keyboard_ext(pa: Params, mut k: KeyEvent) -> Event {
    if pa.len() > 2 // at least 3 parameters
        && pa.0[1].has_more // the second parameter has a ':' subparameter
    {
        // The third parameter is the event type (defaults to 1).
        match pa.param(2, 1).unwrap_or(1) {
            2 => k.kind = KeyKind::Repeat,
            3 => k.kind = KeyKind::Release,
            _ => {}
        }
    }
    Event::Key(k)
}

fn keypad_text(key: Key) -> Option<char> {
    Some(match key {
        Key::Kp(n) => (b'0' + n) as char,
        Key::KpEqual => '=',
        Key::KpMultiply => '*',
        Key::KpPlus => '+',
        Key::KpMinus => '-',
        Key::KpDecimal => '.',
        Key::KpDivide => '/',
        Key::KpSep => ',',
        _ => return None,
    })
}

fn printable_char(code: i32) -> Option<char> {
    let c = char::from_u32(code as u32)?;
    (!c.is_control()).then_some(c)
}

/// XTerm modifyOtherKeys: `CSI 27 ; <modifier> ; <code> ~`
fn parse_xterm_modify_other_keys(pa: Params) -> Event {
    let xmod = pa.param(1, 1).unwrap_or(1);
    let xcode = pa.param(2, 1).unwrap_or(1);
    let meta = from_xterm_mod(xmod - 1);

    let key = match xcode as u32 {
        0x08 => Key::Backspace,
        0x09 => Key::Tab,
        0x0D => Key::Enter,
        0x1B => Key::Escape,
        0x7F => Key::Backspace,
        code => {
            let c = char::from_u32(code).unwrap_or(char::REPLACEMENT_CHARACTER);
            let mut k = press_mod(Key::Char(c), meta);
            if meta.is_empty() || meta == Meta::Shift {
                k.text = Some(c.to_string());
            }
            return Event::Key(k);
        }
    };

    Event::Key(press_mod(key, meta))
}

// ---- Device attributes ---------------------------------------------------

fn parse_primary_dev_attrs(pa: Params) -> Event {
    // Only whole (non-subparameter) params are attributes.
    let attrs = pa.0.iter().filter(|p| !p.has_more).map(|p| p.value_or(0) as u16).collect();
    Event::PrimaryDeviceAttributes(attrs)
}

fn parse_secondary_dev_attrs(pa: Params) -> Event {
    let attrs = pa.0.iter().filter(|p| !p.has_more).map(|p| p.value_or(0) as u16).collect();
    Event::SecondaryDeviceAttributes(attrs)
}

fn parse_tertiary_dev_attrs(data: &[u8]) -> Event {
    // The response is a 4-digit hexadecimal number.
    match hex_decode(data) {
        Some(bytes) => Event::TertiaryDeviceAttributes(bytes),
        None => unknown_dcs(data),
    }
}

// ---- Mouse -----------------------------------------------------------------

/// A raw mouse button as encoded in X10/SGR mouse events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum MouseButton {
    None,
    Left,
    Middle,
    Right,
    WheelUp,
    WheelDown,
    WheelLeft,
    WheelRight,
    Backward,
    Forward,
    Button10,
    Button11,
}

/// Decodes the button bitfield shared by the X10 and SGR encodings.
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Extended-coordinates>
fn parse_mouse_button(b: i32) -> (Meta, MouseButton, bool, bool) {
    // Mouse bit shifts.
    const BIT_SHIFT: i32 = 0b0000_0100;
    const BIT_ALT: i32 = 0b0000_1000;
    const BIT_CTRL: i32 = 0b0001_0000;
    const BIT_MOTION: i32 = 0b0010_0000;
    const BIT_WHEEL: i32 = 0b0100_0000;
    const BIT_ADD: i32 = 0b1000_0000; // additional buttons 8-11
    const BITS_MASK: i32 = 0b0000_0011;

    // Modifiers
    let mut meta = Meta::empty();
    if b & BIT_ALT != 0 {
        meta |= Meta::Alt;
    }
    if b & BIT_CTRL != 0 {
        meta |= Meta::Ctrl;
    }
    if b & BIT_SHIFT != 0 {
        meta |= Meta::Shift;
    }

    let mut is_release = false;
    let btn = if b & BIT_ADD != 0 {
        match b & BITS_MASK {
            0 => MouseButton::Backward,
            1 => MouseButton::Forward,
            2 => MouseButton::Button10,
            _ => MouseButton::Button11,
        }
    } else if b & BIT_WHEEL != 0 {
        match b & BITS_MASK {
            0 => MouseButton::WheelUp,
            1 => MouseButton::WheelDown,
            2 => MouseButton::WheelLeft,
            _ => MouseButton::WheelRight,
        }
    } else {
        match b & BITS_MASK {
            0 => MouseButton::Left,
            1 => MouseButton::Middle,
            2 => MouseButton::Right,
            // X10 reports a button release as 0b0000_0011.
            _ => {
                is_release = true;
                MouseButton::None
            }
        }
    };

    // The motion bit doesn't get reported for wheel events.
    let is_motion = b & BIT_MOTION != 0 && !is_wheel(btn);

    (meta, btn, is_release, is_motion)
}

fn is_wheel(btn: MouseButton) -> bool {
    (MouseButton::WheelUp..=MouseButton::WheelRight).contains(&btn)
}

fn mouse_event(
    btn: MouseButton,
    meta: Meta,
    x: i32,
    y: i32,
    is_release: bool,
    is_motion: bool,
) -> Event {
    let position = Point::new(x, y);

    if is_wheel(btn) {
        // Wheel buttons don't have release events.
        let kind = match btn {
            MouseButton::WheelUp => ScrollKind::Up,
            MouseButton::WheelDown => ScrollKind::Down,
            MouseButton::WheelLeft => ScrollKind::Left,
            _ => ScrollKind::Right,
        };
        return Event::Scroll(ScrollEvent { kind, meta, position });
    }

    // Motion can be reported as a release event in some terminals (Windows
    // Terminal).
    let kind = if is_motion {
        PointerKind::Motion
    } else if is_release {
        PointerKind::Release
    } else {
        PointerKind::Press
    };

    Event::Pointer(PointerEvent {
        button: pointer_button(btn),
        kind,
        meta,
        position,
    })
}

fn pointer_button(btn: MouseButton) -> PointerButton {
    match btn {
        MouseButton::None => PointerButton::None,
        MouseButton::Left => PointerButton::Left,
        MouseButton::Middle => PointerButton::Middle,
        MouseButton::Right => PointerButton::Right,
        MouseButton::Backward => PointerButton::Backward,
        MouseButton::Forward => PointerButton::Forward,
        MouseButton::Button10 => PointerButton::Button10,
        MouseButton::Button11 => PointerButton::Button11,
        // Wheel buttons are reported as scroll events.
        _ => PointerButton::None,
    }
}

/// Parses SGR extended mouse events:
///
/// ```text
/// ESC [ < Cb ; Cx ; Cy (M or m)
/// ```
///
/// where `Cb` is the encoded button, `Cx`/`Cy` the coordinates, and `M`/`m`
/// press/release.
///
/// See <https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Extended-coordinates>
fn parse_sgr_mouse(final_byte: u8, pa: Params) -> Event {
    let x = pa.param(1, 1).unwrap_or(1);
    let y = pa.param(2, 1).unwrap_or(1);
    let release = final_byte == b'm';
    let (meta, btn, _, is_motion) = parse_mouse_button(pa.param(0, 0).unwrap_or(0));

    // (1,1) is the upper left; normalize to (0,0).
    mouse_event(btn, meta, x - 1, y - 1, release, is_motion)
}

const X10_MOUSE_BYTE_OFFSET: i32 = 32;

/// Parses X10-encoded mouse events, the simplest kind. (The last release of
/// X10 was December 1986, by the way.) The original protocol limits the
/// coordinates to 223 (= 255 - 32).
///
/// ```text
/// ESC [M Cb Cx Cy
/// ```
///
/// `v` holds the three raw payload bytes (Cb Cx Cy).
///
/// See <http://www.xfree86.org/current/ctlseqs.html#Mouse%20Tracking>
fn parse_x10_mouse(v: &[u8]) -> Event {
    let mut b = v[0] as i32;
    if b >= X10_MOUSE_BYTE_OFFSET {
        // b < 32 should be impossible, but be defensive.
        b -= X10_MOUSE_BYTE_OFFSET;
    }

    let (meta, btn, is_release, is_motion) = parse_mouse_button(b);

    // (1,1) is the upper left; normalize to (0,0).
    let x = v[1] as i32 - X10_MOUSE_BYTE_OFFSET - 1;
    let y = v[2] as i32 - X10_MOUSE_BYTE_OFFSET - 1;

    mouse_event(btn, meta, x, y, is_release, is_motion)
}

// ---- Termcap ---------------------------------------------------------------

/// Parses an XTGETTCAP response payload: `;`-separated `<hex name>=<hex
/// value>` pairs, decoded into a `name=value;name=value` capability string.
fn parse_termcap(data: &[u8]) -> Event {
    if data.is_empty() {
        return Event::Capability(String::new());
    }

    let mut tc = String::new();
    for s in data.split(|&c| c == b';') {
        let mut parts = s.splitn(2, |&c| c == b'=');
        let Some(name) = parts.next().and_then(hex_decode) else {
            continue;
        };
        if name.is_empty() {
            continue;
        }

        let value = parts.next().and_then(hex_decode).unwrap_or_default();

        if !tc.is_empty() {
            tc.push(';');
        }
        tc.push_str(&String::from_utf8_lossy(&name));
        if !value.is_empty() {
            tc.push('=');
            tc.push_str(&String::from_utf8_lossy(&value));
        }
    }

    Event::Capability(tc)
}
