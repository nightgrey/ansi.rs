// https://github.com/charmbracelet/ultraviolet/blob/main/decoder.go

use std::io::BorrowedCursor;
use derive_more::Deref;
use crate::Params;
use crate::parser::{Handler, Parser, State};

#[derive(Debug, Default, Deref)]
pub struct InputParser(Parser);

impl InputParser {
    pub fn advance(&mut self, handler: &mut dyn Handler, bytes: &[u8]) -> usize {
        if bytes.len() == 1 && bytes[0] == b'\x1b' {
            handler.esc(&[], b'\x1b');
            return 1;
        }

        self.0.advance(handler, bytes)
    }
}

impl Handler for InputHandler {

    fn csi(&mut self, params: &Params, intermediates: &[u8], final_char: char) {

    }
}

#[test]
fn qwe() {
    std::io::Read::read(&[])

}
//
//
//
// fn decode(p: &mut EventDecoder, buf: &[u8]) -> (usize, Option<Event>) {
//     if buf.is_empty() {
//         return (0, None);
//     }
//
//     match buf[0] {
//         ansi::ESC => {
//             if buf.len() == 1 {
//                 // Escape key
//                 return (1, Some(KeyPressEvent { code: KeyEscape, ..Default::default() }));
//             }
//
//             match buf[1] {
//                 b'O' => { // Esc-prefixed SS3
//                     return p.parse_ss3(buf);
//                 }
//                 b'P' => { // Esc-prefixed DCS
//                     return p.parse_dcs(buf);
//                 }
//                 b'[' => { // Esc-prefixed CSI
//                     return p.parse_csi(buf);
//                 }
//                 b']' => { // Esc-prefixed OSC
//                     return p.parse_osc(buf);
//                 }
//                 b'_' => { // Esc-prefixed APC
//                     return p.parse_apc(buf);
//                 }
//                 b'^' => { // Esc-prefixed PM
//                     return p.parse_st_terminated(ansi::PM, b'^', None)(buf);
//                 }
//                 b'X' => { // Esc-prefixed SOS
//                     return p.parse_st_terminated(ansi::SOS, b'X', None)(buf);
//                 }
//                 _ => {
//                     let (n, e) = p.decode(&buf[1..]);
//                     if let Some(e) = e {
//                         if let KeyPressEvent(mut k) = e {
//                             k.text = String::new();
//                             k.mod_ |= ModAlt;
//                             return (n + 1, Some(KeyPressEvent(k)));
//                         }
//                     }
//
//                     // Not a key sequence, nor an alt modified key sequence. In that
//                     // case, just report a single escape key.
//                     return (1, Some(KeyPressEvent { code: KeyEscape, ..Default::default() }));
//                 }
//             }
//         },
//         ansi::SS3 => {
//             return p.parse_ss3(buf);
//         },
//         ansi::DCS => {
//             return p.parse_dcs(buf);
//         },
//         ansi::CSI => {
//             return p.parse_csi(buf);
//         },
//         ansi::OSC => {
//             return p.parse_osc(buf);
//         },
//         ansi::APC => {
//             return p.parse_apc(buf);
//         },
//         ansi::PM => {
//             return p.parse_st_terminated(ansi::PM, b'^', None)(buf);
//         },
//         ansi::SOS => {
//             return p.parse_st_terminated(ansi::SOS, b'X', None)(buf);
//         },
//         ..US | ansi::DEL | ansi::SP => {
//             return (1, Some(p.parse_control(b)));
//         },
//         ansi::PAD | ansi::PAD..=ansi::APC => {
//             // C1 control code
//             // UTF-8 never starts with a C1 control code
//             // Encode these as Ctrl+Alt+<code - 0x40>
//             let code = char::from_u32(b as u32 - 0x40).unwrap_or('\u{FFFD}');
//             return (1, Some(KeyPressEvent { code, mod_: ModCtrl | ModAlt, ..Default::default() }));
//         }
//         _ => {
//             return p.parse_utf8(buf);
//         }
//     }
// }
//
// fn parse_csi(p: &mut EventDecoder, b: &[u8]) -> (usize, Option<Event>) {
//     if b.len() == 2 && b[0] == ansi::ESC {
//         // short cut if this is an alt+[ key
//         return (2, Some(KeyPressEvent { code: b[1] as char, mod_: ModAlt, ..Default::default() }));
//     }
//
//     let mut cmd: ansi::Cmd = 0;
//     let mut params = [ansi::Param::default(); parser::MAX_PARAMS_SIZE];
//     let mut params_len: usize = 0;
//
//     let mut i: usize = 0;
//     if b[i] == ansi::CSI || b[i] == ansi::ESC {
//         i += 1;
//     }
//     if i < b.len() && b[i - 1] == ansi::ESC && b[i] == b'[' {
//         i += 1;
//     }
//
//     // Initial CSI byte
//     if i < b.len() && b[i] >= b'<' && b[i] <= b'?' {
//         cmd |= ansi::Cmd::from(b[i]) << parser::PREFIX_SHIFT;
//     }
//
//     // Scan parameter bytes in the range 0x30-0x3F
//     let mut j: usize = 0;
//     while i < b.len() && params_len < params.len() && b[i] >= 0x30 && b[i] <= 0x3F {
//         if b[i] >= b'0' && b[i] <= b'9' {
//             if params[params_len] == parser::MISSING_PARAM {
//                 params[params_len] = 0;
//             }
//             params[params_len] *= 10;
//             params[params_len] += ansi::Param::from(b[i]) - ansi::Param::from(b'0');
//         }
//         if b[i] == b':' {
//             params[params_len] |= parser::HAS_MORE_FLAG;
//         }
//         if b[i] == b';' || b[i] == b':' {
//             params_len += 1;
//             if params_len < params.len() {
//                 // Don't overflow the params slice
//                 params[params_len] = parser::MISSING_PARAM;
//             }
//         }
//         i += 1;
//         j += 1;
//     }
//
//     if j > 0 && params_len < params.len() {
//         // has parameters
//         params_len += 1;
//     }
//
//     // Scan intermediate bytes in the range 0x20-0x2F
//     let mut intermed: u8 = 0;
//     while i < b.len() && b[i] >= 0x20 && b[i] <= 0x2F {
//         intermed = b[i];
//         i += 1;
//     }
//
//     // Set the intermediate byte
//     cmd |= ansi::Cmd::from(intermed) << parser::INTERMED_SHIFT;
//
//     // Scan final byte in the range 0x40-0x7E
//     if i >= b.len() || b[i] < 0x40 || b[i] > 0x7E {
//         // Special case for URxvt keys
//         // CSI <number> $ is an invalid sequence, but URxvt uses it for
//         // shift modified keys.
//         if intermed == b'$' && b[i - 1] == b'$' {
//             let mut buf = b[..i - 1].to_vec();
//             buf.push(b'~');
//             let (n, ev) = p.parse_csi(&buf);
//             if let Some(KeyPressEvent(mut k)) = ev {
//                 k.mod_ |= ModShift;
//                 return (n, Some(k));
//             }
//         }
//         return (i, Some(UnknownEvent(b[..i].to_vec())));
//     }
//
//     // Add the final byte
//     cmd |= ansi::Cmd::from(b[i]);
//     i += 1;
//
//     let pa = ansi::Params(&params[..params_len]);
//     match cmd {
//         cmd if cmd == b'y' as ansi::Cmd | (b'?' as ansi::Cmd) << parser::PREFIX_SHIFT | (b'$' as ansi::Cmd) << parser::INTERMED_SHIFT => {
//             // Report Mode (DECRPM)
//             let (mode, _, ok) = pa.param(0, -1);
//             if !ok || mode == -1 {
//                 return (i, None); // break
//             }
//             let (value, _, ok) = pa.param(1, 0);
//             if !ok {
//                 return (i, None); // break
//             }
//             return (i, Some(ModeReportEvent { mode: ansi::DECMode::from(mode), value: ansi::ModeSetting::from(value) }));
//         }
//         cmd if cmd == b'c' as ansi::Cmd | (b'?' as ansi::Cmd) << parser::PREFIX_SHIFT => {
//             // Primary Device Attributes
//             return (i, parse_primary_dev_attrs(pa));
//         }
//         cmd if cmd == b'c' as ansi::Cmd | (b'>' as ansi::Cmd) << parser::PREFIX_SHIFT => {
//             // Secondary Device Attributes
//             return (i, parse_secondary_dev_attrs(pa));
//         }
//         cmd if cmd == b'u' as ansi::Cmd | (b'?' as ansi::Cmd) << parser::PREFIX_SHIFT => {
//             // Kitty keyboard flags
//             let (flags, _, _) = pa.param(0, -1);
//             return (i, Some(KeyboardEnhancementsEvent { flags }));
//         }
//         cmd if cmd == b'R' as ansi::Cmd | (b'?' as ansi::Cmd) << parser::PREFIX_SHIFT => {
//             // This report may return a third parameter representing the page
//             // number, but we don't really need it.
//             let (row, _, _) = pa.param(0, 1);
//             let (col, _, ok) = pa.param(1, 1);
//             if !ok {
//                 return (i, None); // break
//             }
//             return (i, Some(CursorPositionEvent { y: row - 1, x: col - 1 }));
//         }
//         cmd if cmd == b'm' as ansi::Cmd | (b'<' as ansi::Cmd) << parser::PREFIX_SHIFT
//             || cmd == b'M' as ansi::Cmd | (b'<' as ansi::Cmd) << parser::PREFIX_SHIFT => {
//             // Handle SGR mouse
//             if params_len == 3 {
//                 return (i, parse_sgr_mouse_event(cmd, pa));
//             }
//         }
//         cmd if cmd == b'm' as ansi::Cmd | (b'>' as ansi::Cmd) << parser::PREFIX_SHIFT => {
//             // XTerm modifyOtherKeys
//             let (mok, _, ok) = pa.param(0, 0);
//             if !ok || mok != 4 {
//                 return (i, None); // break
//             }
//             let (val, _, ok) = pa.param(1, -1);
//             if !ok || val == -1 {
//                 return (i, None); // break
//             }
//             return (i, Some(ModifyOtherKeysEvent { val }));
//         }
//         cmd if cmd == b'n' as ansi::Cmd | (b'?' as ansi::Cmd) << parser::PREFIX_SHIFT => {
//             let (report, _, _) = pa.param(0, -1);
//             let (dark_light, _, _) = pa.param(1, -1);
//             match report {
//                 997 => { // [ansi.LightDarkReport]
//                     match dark_light {
//                         1 => return (i, Some(DarkColorSchemeEvent {})),
//                         2 => return (i, Some(LightColorSchemeEvent {})),
//                         _ => {}
//                     }
//                 }
//                 _ => {}
//             }
//         }
//         cmd if cmd == b'I' as ansi::Cmd => {
//             return (i, Some(FocusEvent {}));
//         }
//         cmd if cmd == b'O' as ansi::Cmd => {
//             return (i, Some(BlurEvent {}));
//         }
//         cmd if cmd == b'R' as ansi::Cmd => {
//             // Cursor position report OR modified F3
//             let (row, _, rok) = pa.param(0, 1);
//             let (col, _, cok) = pa.param(1, 1);
//             if params_len == 2 && rok && cok {
//                 let m = CursorPositionEvent { y: row - 1, x: col - 1 };
//                 if row == 1 && col - 1 <= (ModMeta | ModShift | ModAlt | ModCtrl) as i32 {
//                     // XXX: We cannot differentiate between cursor position report and
//                     // CSI 1 ; <mod> R (which is modified F3) when the cursor is at the
//                     // row 1. In this case, we report both messages.
//                     //
//                     // For a non ambiguous cursor position report, use
//                     // [ansi.RequestExtendedCursorPosition] (DECXCPR) instead.
//                     return (i, Some(MultiEvent {
//                         events: vec![
//                             Event::KeyPress(KeyPressEvent { code: KeyF3, mod_: KeyMod(col - 1), ..Default::default() }),
//                             Event::CursorPosition(m),
//                         ],
//                     }));
//                 }
//
//                 return (i, Some(m));
//             }
//
//             if params_len != 0 {
//                 return (i, None); // break
//             }
//
//             // Unmodified key F3 (CSI R)
//             // fallthrough
//         }
//         cmd if matches!(cmd, b'a' | b'b' | b'c' | b'd' | b'A' | b'B' | b'C' | b'D' | b'E' | b'F' | b'H' | b'P' | b'Q' | b'S' | b'Z') => {
//             let mut k = KeyPressEvent::default();
//             let cmd_byte = cmd as u8;
//             match cmd_byte {
//                 b'a' | b'b' | b'c' | b'd' => {
//                     k = KeyPressEvent { code: KeyUp + (cmd_byte - b'a') as u32, mod_: ModShift, ..Default::default() };
//                 }
//                 b'A' | b'B' | b'C' | b'D' => {
//                     k = KeyPressEvent { code: KeyUp + (cmd_byte - b'A') as u32, ..Default::default() };
//                 }
//                 b'E' => {
//                     k = KeyPressEvent { code: KeyBegin, ..Default::default() };
//                 }
//                 b'F' => {
//                     k = KeyPressEvent { code: KeyEnd, ..Default::default() };
//                 }
//                 b'H' => {
//                     k = KeyPressEvent { code: KeyHome, ..Default::default() };
//                 }
//                 b'P' | b'Q' | b'R' | b'S' => {
//                     k = KeyPressEvent { code: KeyF1 + (cmd_byte - b'P') as u32, ..Default::default() };
//                 }
//                 b'Z' => {
//                     k = KeyPressEvent { code: KeyTab, mod_: ModShift, ..Default::default() };
//                 }
//                 _ => {}
//             }
//             let (id, _, _) = pa.param(0, 1);
//             let (mod_, _, _) = pa.param(1, 1);
//             if params_len > 2 && !pa[1].has_more() || id != 1 {
//                 return (i, None); // break
//             }
//             if params_len > 1 && id == 1 && mod_ != -1 {
//                 // CSI 1 ; <modifiers> A
//                 k.mod_ |= key_mod(mod_ - 1);
//             }
//             // Don't forget to handle Kitty keyboard protocol
//             return (i, parse_kitty_keyboard_ext(pa, k));
//         }
//         cmd if cmd == b'M' as ansi::Cmd => {
//             // Handle X10 mouse
//             if i + 3 > b.len() {
//                 return (i, Some(UnknownCsiEvent(b[..i].to_vec())));
//             }
//             return (i + 3, parse_x10_mouse_event(&b[..i + 3]));
//         }
//         cmd if cmd == b'y' as ansi::Cmd | (b'$' as ansi::Cmd) << parser::INTERMED_SHIFT => {
//             // Report Mode (DECRPM)
//             let (mode, _, ok) = pa.param(0, -1);
//             if !ok || mode == -1 {
//                 return (i, None); // break
//             }
//             let (val, _, ok) = pa.param(1, 0);
//             if !ok {
//                 return (i, None); // break
//             }
//             return (i, Some(ModeReportEvent { mode: ansi::ANSIMode::from(mode), value: ansi::ModeSetting::from(val) }));
//         }
//         cmd if cmd == b'u' as ansi::Cmd => {
//             // Kitty keyboard protocol & CSI u (fixterms)
//             if params_len == 0 {
//                 return (i, Some(UnknownCsiEvent(b[..i].to_vec())));
//             }
//             return (i, parse_kitty_keyboard(pa));
//         }
//         cmd if cmd == b'_' as ansi::Cmd => {
//             // Win32 Input Mode
//             if params_len != 6 {
//                 return (i, Some(UnknownCsiEvent(b[..i].to_vec())));
//             }
//
//             let (vk, _, _) = pa.param(0, 0);
//             let (sc, _, _) = pa.param(1, 0);
//             let (uc, _, _) = pa.param(2, 0);
//             let (kd, _, _) = pa.param(3, 0);
//             let (cs, _, _) = pa.param(4, 0);
//             let (rc, _, _) = pa.param(5, 0);
//             let event = p.parse_win32_input_key_event(
//                 vk as u16,          // Vk wVirtualKeyCode
//                 sc as u16,          // Sc wVirtualScanCode
//                 char::from_u32(uc as u32).unwrap_or('\u{FFFD}'),            // Uc UnicodeChar
//                 kd == 1,            // Kd bKeyDown
//                 cs as u32,          // Cs dwControlKeyState
//                 std::cmp::max(1, rc as u16), // Rc wRepeatCount
//             );
//
//             return (i, event);
//         }
//         cmd if matches!(cmd, b'@' | b'^' | b'~') => {
//             if params_len == 0 {
//                 return (i, Some(UnknownCsiEvent(b[..i].to_vec())));
//             }
//
//             let (param, _, _) = pa.param(0, 0);
//             let cmd_byte = cmd as u8;
//             match cmd_byte {
//                 b'~' => {
//                     match param {
//                         27 => {
//                             // XTerm modifyOtherKeys 2
//                             if params_len != 3 {
//                                 return (i, Some(UnknownCsiEvent(b[..i].to_vec())));
//                             }
//                             return (i, parse_xterm_modify_other_keys(pa));
//                         }
//                         200 => {
//                             // bracketed-paste start
//                             return (i, Some(PasteStartEvent {}));
//                         }
//                         201 => {
//                             // bracketed-paste end
//                             return (i, Some(PasteEndEvent {}));
//                         }
//                         _ => {}
//                     }
//                 }
//                 _ => {}
//             }
//
//             match param {
//                 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8
//                 | 11 | 12 | 13 | 14 | 15
//                 | 17 | 18 | 19 | 20 | 21
//                 | 23 | 24 | 25 | 26
//                 | 28 | 29 | 31 | 32 | 33 | 34 => {
//                     let mut k = KeyPressEvent::default();
//                     match param {
//                         1 => {
//                             if p.legacy & FLAG_FIND != 0 {
//                                 k = KeyPressEvent { code: KeyFind, ..Default::default() };
//                             } else {
//                                 k = KeyPressEvent { code: KeyHome, ..Default::default() };
//                             }
//                         }
//                         2 => {
//                             k = KeyPressEvent { code: KeyInsert, ..Default::default() };
//                         }
//                         3 => {
//                             k = KeyPressEvent { code: KeyDelete, ..Default::default() };
//                         }
//                         4 => {
//                             if p.legacy & FLAG_SELECT != 0 {
//                                 k = KeyPressEvent { code: KeySelect, ..Default::default() };
//                             } else {
//                                 k = KeyPressEvent { code: KeyEnd, ..Default::default() };
//                             }
//                         }
//                         5 => {
//                             k = KeyPressEvent { code: KeyPgUp, ..Default::default() };
//                         }
//                         6 => {
//                             k = KeyPressEvent { code: KeyPgDown, ..Default::default() };
//                         }
//                         7 => {
//                             k = KeyPressEvent { code: KeyHome, ..Default::default() };
//                         }
//                         8 => {
//                             k = KeyPressEvent { code: KeyEnd, ..Default::default() };
//                         }
//                         11 | 12 | 13 | 14 | 15 => {
//                             k = KeyPressEvent { code: KeyF1 + (param - 11) as u32, ..Default::default() };
//                         }
//                         17 | 18 | 19 | 20 | 21 => {
//                             k = KeyPressEvent { code: KeyF6 + (param - 17) as u32, ..Default::default() };
//                         }
//                         23 | 24 | 25 | 26 => {
//                             k = KeyPressEvent { code: KeyF11 + (param - 23) as u32, ..Default::default() };
//                         }
//                         28 | 29 => {
//                             k = KeyPressEvent { code: KeyF15 + (param - 28) as u32, ..Default::default() };
//                         }
//                         31 | 32 | 33 | 34 => {
//                             k = KeyPressEvent { code: KeyF17 + (param - 31) as u32, ..Default::default() };
//                         }
//                         _ => {}
//                     }
//
//                     // modifiers
//                     let (mod_, _, _) = pa.param(1, -1);
//                     if params_len > 1 && mod_ != -1 {
//                         k.mod_ |= key_mod(mod_ - 1);
//                     }
//
//                     // Handle URxvt weird keys
//                     match cmd_byte {
//                         b'~' => {
//                             // Don't forget to handle Kitty keyboard protocol
//                             return (i, parse_kitty_keyboard_ext(pa, k));
//                         }
//                         b'^' => {
//                             k.mod_ |= ModCtrl;
//                         }
//                         b'@' => {
//                             k.mod_ |= ModCtrl | ModShift;
//                         }
//                         _ => {}
//                     }
//
//                     return (i, Some(k));
//                 }
//                 _ => {}
//             }
//         }
//         cmd if cmd == b't' as ansi::Cmd => {
//             let (param, _, ok) = pa.param(0, 0);
//             if !ok {
//                 return (i, None); // break
//             }
//
//             match param {
//                 4 => { // Report Terminal window size in pixels.
//                     if params_len == 3 {
//                         let (height, _, h_ok) = pa.param(1, 0);
//                         let (width, _, w_ok) = pa.param(2, 0);
//                         if !h_ok || !w_ok {
//                             return (i, None); // break
//                         }
//                         return (i, Some(PixelSizeEvent { width, height }));
//                     }
//                 }
//                 6 => { // Report Terminal character cell size.
//                     if params_len == 3 {
//                         let (height, _, h_ok) = pa.param(1, 0);
//                         let (width, _, w_ok) = pa.param(2, 0);
//                         if !h_ok || !w_ok {
//                             return (i, None); // break
//                         }
//                         return (i, Some(CellSizeEvent { width, height }));
//                     }
//                 }
//                 8 => { // Report Terminal Window size in cells.
//                     if params_len == 3 {
//                         let (height, _, h_ok) = pa.param(1, 0);
//                         let (width, _, w_ok) = pa.param(2, 0);
//                         if !h_ok || !w_ok {
//                             return (i, None); // break
//                         }
//                         return (i, Some(WindowSizeEvent { width, height }));
//                     }
//                 }
//                 48 => { // In band terminal size report.
//                     if params_len == 5 {
//                         let (cell_height, _, ch_ok) = pa.param(1, 0);
//                         let (cell_width, _, cw_ok) = pa.param(2, 0);
//                         let (pixel_height, _, ph_ok) = pa.param(3, 0);
//                         let (pixel_width, _, pw_ok) = pa.param(4, 0);
//                         if !ch_ok || !cw_ok || !ph_ok || !pw_ok {
//                             return (i, None); // break
//                         }
//                         return (i, Some(MultiEvent {
//                             events: vec![
//                                 Event::WindowSize(WindowSizeEvent { width: cell_width, height: cell_height }),
//                                 Event::PixelSize(PixelSizeEvent { width: pixel_width, height: pixel_height }),
//                             ],
//                         }));
//                     }
//                 }
//                 _ => {}
//             }
//
//             // Any other window operation event.
//
//             let mut winop = WindowOpEvent::default();
//             winop.op = param;
//             for j in 1..params_len {
//                 let (val, _, ok) = pa.param(j, 0);
//                 if ok {
//                     winop.args.push(val);
//                 }
//             }
//
//             return (i, Some(winop));
//         }
//         _ => {}
//     }
//     return (i, Some(UnknownCsiEvent(b[..i].to_vec())));
// }
//
// // parseSs3 parses a SS3 sequence.
// // See https://vt100.net/docs/vt220-rm/chapter4.html#S4.4.4.2
// fn parse_ss3(p: &mut EventDecoder, b: &[u8]) -> (usize, Option<Event>) {
//     if b.len() == 2 && b[0] == ansi::ESC {
//         // short cut if this is an alt+O key
//         return (2, Some(KeyPressEvent { code: (b[1] as char).to_ascii_lowercase(), mod_: ModShift | ModAlt, ..Default::default() }));
//     }
//
//     let mut i: usize = 0;
//     if b[i] == ansi::SS3 || b[i] == ansi::ESC {
//         i += 1;
//     }
//     if i < b.len() && b[i - 1] == ansi::ESC && b[i] == b'O' {
//         i += 1;
//     }
//
//     // Scan numbers from 0-9
//     let mut mod_ = 0;
//     while i < b.len() && b[i] >= b'0' && b[i] <= b'9' {
//         mod_ *= 10;
//         mod_ += (b[i] - b'0') as i32;
//         i += 1;
//     }
//
//     // Scan a GL character
//     // A GL character is a single byte in the range 0x21-0x7E
//     // See https://vt100.net/docs/vt220-rm/chapter2.html#S2.3.2
//     if i >= b.len() || b[i] < 0x21 || b[i] > 0x7E {
//         return (i, Some(UnknownEvent(b[..i].to_vec())));
//     }
//
//     // GL character(s)
//     let gl = b[i];
//     i += 1;
//
//     let mut k = KeyPressEvent::default();
//     match gl {
//         b'a' | b'b' | b'c' | b'd' => {
//             k = KeyPressEvent { code: KeyUp + (gl - b'a') as u32, mod_: ModCtrl, ..Default::default() };
//         }
//         b'A' | b'B' | b'C' | b'D' => {
//             k = KeyPressEvent { code: KeyUp + (gl - b'A') as u32, ..Default::default() };
//         }
//         b'E' => {
//             k = KeyPressEvent { code: KeyBegin, ..Default::default() };
//         }
//         b'F' => {
//             k = KeyPressEvent { code: KeyEnd, ..Default::default() };
//         }
//         b'H' => {
//             k = KeyPressEvent { code: KeyHome, ..Default::default() };
//         }
//         b'P' | b'Q' | b'R' | b'S' => {
//             k = KeyPressEvent { code: KeyF1 + (gl - b'P') as u32, ..Default::default() };
//         }
//         b'M' => {
//             k = KeyPressEvent { code: KeyKpEnter, ..Default::default() };
//         }
//         b'X' => {
//             k = KeyPressEvent { code: KeyKpEqual, ..Default::default() };
//         }
//         b'j' | b'k' | b'l' | b'm' | b'n' | b'o' | b'p' | b'q' | b'r' | b's' | b't' | b'u' | b'v' | b'w' | b'x' | b'y' => {
//             k = KeyPressEvent { code: KeyKpMultiply + (gl - b'j') as u32, ..Default::default() };
//         }
//         _ => {
//             return (i, Some(UnknownSs3Event(b[..i].to_vec())));
//         }
//     }
//
//     // Handle weird SS3 <modifier> Func
//     if mod_ > 0 {
//         k.mod_ |= key_mod(mod_ - 1);
//     }
//
//     return (i, Some(k));
// }
//
//
// fn parse_osc(p: &mut EventDecoder, b: &[u8]) -> (usize, Option<Event>) {
//     let default_key = || KeyPressEvent { code: b[1] as char, mod_: ModAlt, ..Default::default() };
//     if b.len() == 2 && b[0] == ansi::ESC {
//         // short cut if this is an alt+] key
//         return (2, Some(default_key()));
//     }
//
//     let mut i: usize = 0;
//     if b[i] == ansi::OSC || b[i] == ansi::ESC {
//         i += 1;
//     }
//     if i < b.len() && b[i - 1] == ansi::ESC && b[i] == b']' {
//         i += 1;
//     }
//
//     // Parse OSC command
//     // An OSC sequence is terminated by a BEL, ESC, or ST character
//     let mut start: usize = 0;
//     let mut end: usize = 0;
//     let mut cmd: i32 = -1;
//     while i < b.len() && b[i] >= b'0' && b[i] <= b'9' {
//         if cmd == -1 {
//             cmd = 0;
//         } else {
//             cmd *= 10;
//         }
//         cmd += (b[i] - b'0') as i32;
//         i += 1;
//     }
//
//     if i < b.len() && b[i] == b';' {
//         // mark the start of the sequence data
//         i += 1;
//         start = i;
//     }
//
//     while i < b.len() {
//         // advance to the end of the sequence
//         if [ansi::BEL, ansi::ESC, ansi::ST, ansi::CAN, ansi::SUB].contains(&b[i]) {
//             break;
//         }
//         i += 1;
//     }
//
//     if i >= b.len() {
//         return (i, Some(UnknownEvent(b[..i].to_vec())));
//     }
//
//     end = i; // end of the sequence data
//     i += 1;
//
//     // Check 7-bit ST (string terminator) character
//     match b[i - 1] {
//         ansi::CAN | ansi::SUB => {
//             return (i, Some(ignored_event(&b[..i])));
//         }
//         ansi::ESC => {
//             if i >= b.len() || b[i] != b'\\' {
//                 if cmd == -1 || (start == 0 && end == 2) {
//                     return (2, Some(default_key()));
//                 }
//
//                 // If we don't have a valid ST terminator, then this is a
//                 // cancelled sequence and should be ignored.
//                 return (i, Some(ignored_event(&b[..i])));
//             }
//
//             i += 1;
//         }
//         _ => {}
//     }
//
//     if end <= start {
//         return (i, Some(UnknownEvent(b[..i].to_vec())));
//     }
//
//     let data = String::from_utf8_lossy(&b[start..end]).to_string();
//     match cmd {
//         10 => {
//             return (i, Some(ForegroundColorEvent { color: ansi::x_parse_color(&data) }));
//         }
//         11 => {
//             return (i, Some(BackgroundColorEvent { color: ansi::x_parse_color(&data) }));
//         }
//         12 => {
//             return (i, Some(CursorColorEvent { color: ansi::x_parse_color(&data) }));
//         }
//         52 => {
//             let parts: Vec<&str> = data.split(';').collect();
//             if parts.len() != 2 || parts[0].len() < 1 {
//                 return (i, Some(ClipboardEvent::default()));
//             }
//
//             let b64 = parts[1];
//             let bts = base64::engine::general_purpose::STANDARD.decode(b64);
//             match bts {
//                 Ok(bts) => {
//                     let sel = ClipboardSelection::from(parts[0].as_bytes()[0]); //nolint:unconvert
//                     return (i, Some(ClipboardEvent { selection: sel, content: String::from_utf8_lossy(&bts).to_string() }));
//                 }
//                 Err(_) => {
//                     return (i, Some(ClipboardEvent { content: parts[1].to_string(), ..Default::default() }));
//                 }
//             }
//         }
//         _ => {}
//     }
//
//     return (i, Some(UnknownOscEvent(b[..i].to_vec())));
// }
//
// // parseStTerminated parses a control sequence that gets terminated by a ST character.
// fn parse_st_terminated(
//     intro8: u8,
//     intro7: u8,
//     fn_: Option<Box<dyn Fn(&[u8]) -> Option<Event>>>,
// ) -> Box<dyn Fn(&[u8]) -> (usize, Option<Event>)> {
//     let default_key = move |b: &[u8]| -> (usize, Option<Event>) {
//         match intro8 {
//             ansi::SOS => {
//                 return (2, Some(KeyPressEvent { code: (b[1] as char).to_ascii_lowercase(), mod_: ModShift | ModAlt, ..Default::default() }));
//             }
//             ansi::PM | ansi::APC => {
//                 return (2, Some(KeyPressEvent { code: b[1] as char, mod_: ModAlt, ..Default::default() }));
//             }
//             _ => {
//                 return (0, None);
//             }
//         }
//     };
//
//     Box::new(move |b: &[u8]| -> (usize, Option<Event>) {
//         if b.len() == 2 && b[0] == ansi::ESC {
//             return default_key(b);
//         }
//
//         let mut i: usize = 0;
//         if b[i] == intro8 || b[i] == ansi::ESC {
//             i += 1;
//         }
//         if i < b.len() && b[i - 1] == ansi::ESC && b[i] == intro7 {
//             i += 1;
//         }
//
//         // Scan control sequence
//         // Most common control sequence is terminated by a ST character
//         // ST is a 7-bit string terminator character is (ESC \)
//         let start = i;
//         while i < b.len() {
//             if [ansi::ESC, ansi::ST, ansi::CAN, ansi::SUB].contains(&b[i]) {
//                 break;
//             }
//             i += 1;
//         }
//
//         if i >= b.len() {
//             return (i, Some(UnknownEvent(b[..i].to_vec())));
//         }
//
//         let end = i; // end of the sequence data
//         i += 1;
//
//         // Check 7-bit ST (string terminator) character
//         match b[i - 1] {
//             ansi::CAN | ansi::SUB => {
//                 return (i, Some(ignored_event(&b[..i])));
//             }
//             ansi::ESC => {
//                 if i >= b.len() || b[i] != b'\\' {
//                     if start == end {
//                         return default_key(b);
//                     }
//
//                     // If we don't have a valid ST terminator, then this is a
//                     // cancelled sequence and should be ignored.
//                     return (i, Some(ignored_event(&b[..i])));
//                 }
//
//                 i += 1;
//             }
//             _ => {}
//         }
//
//         // Call the function to parse the sequence and return the result
//         if let Some(fn_) = &fn_ {
//             if let Some(e) = fn_(&b[start..end]) {
//                 return (i, Some(e));
//             }
//         }
//
//         match intro8 {
//             ansi::PM => {
//                 return (i, Some(UnknownPmEvent(b[..i].to_vec())));
//             }
//             ansi::SOS => {
//                 return (i, Some(UnknownSosEvent(b[..i].to_vec())));
//             }
//             ansi::APC => {
//                 return (i, Some(UnknownApcEvent(b[..i].to_vec())));
//             }
//             _ => {
//                 return (i, Some(UnknownEvent(b[..i].to_vec())));
//             }
//         }
//     })
// }
//
// fn parse_dcs(p: &mut EventDecoder, b: &[u8]) -> (usize, Option<Event>) {
//     if b.len() == 2 && b[0] == ansi::ESC {
//         // short cut if this is an alt+P key
//         return (2, Some(KeyPressEvent { code: (b[1] as char).to_ascii_lowercase(), mod_: ModShift | ModAlt, ..Default::default() }));
//     }
//
//     let mut params = [ansi::Param::default(); 16];
//     let mut params_len: usize = 0;
//     let mut cmd: ansi::Cmd = 0;
//
//     // DCS sequences are introduced by DCS (0x90) or ESC P (0x1b 0x50)
//     let mut i: usize = 0;
//     if b[i] == ansi::DCS || b[i] == ansi::ESC {
//         i += 1;
//     }
//     if i < b.len() && b[i - 1] == ansi::ESC && b[i] == b'P' {
//         i += 1;
//     }
//
//     // initial DCS byte
//     if i < b.len() && b[i] >= b'<' && b[i] <= b'?' {
//         cmd |= ansi::Cmd::from(b[i]) << parser::PREFIX_SHIFT;
//     }
//
//     // Scan parameter bytes in the range 0x30-0x3F
//     let mut j: usize = 0;
//     while i < b.len() && params_len < params.len() && b[i] >= 0x30 && b[i] <= 0x3F {
//         if b[i] >= b'0' && b[i] <= b'9' {
//             if params[params_len] == parser::MISSING_PARAM {
//                 params[params_len] = 0;
//             }
//             params[params_len] *= 10;
//             params[params_len] += ansi::Param::from(b[i]) - ansi::Param::from(b'0');
//         }
//         if b[i] == b':' {
//             params[params_len] |= parser::HAS_MORE_FLAG;
//         }
//         if b[i] == b';' || b[i] == b':' {
//             params_len += 1;
//             if params_len < params.len() {
//                 // Don't overflow the params slice
//                 params[params_len] = parser::MISSING_PARAM;
//             }
//         }
//         i += 1;
//         j += 1;
//     }
//
//     if j > 0 && params_len < params.len() {
//         // has parameters
//         params_len += 1;
//     }
//
//     // Scan intermediate bytes in the range 0x20-0x2F
//     let mut intermed: u8 = 0;
//     let mut _j2: usize = 0;
//     while i < b.len() && b[i] >= 0x20 && b[i] <= 0x2F {
//         intermed = b[i];
//         i += 1;
//         _j2 += 1;
//     }
//
//     // set intermediate byte
//     cmd |= ansi::Cmd::from(intermed) << parser::INTERMED_SHIFT;
//
//     // Scan final byte in the range 0x40-0x7E
//     if i >= b.len() || b[i] < 0x40 || b[i] > 0x7E {
//         return (i, Some(UnknownEvent(b[..i].to_vec())));
//     }
//
//     // Add the final byte
//     cmd |= ansi::Cmd::from(b[i]);
//     i += 1;
//
//     let start = i; // start of the sequence data
//     while i < b.len() {
//         if b[i] == ansi::ST || b[i] == ansi::ESC {
//             break;
//         }
//         i += 1;
//     }
//
//     if i >= b.len() {
//         return (i, Some(UnknownEvent(b[..i].to_vec())));
//     }
//
//     let end = i; // end of the sequence data
//     i += 1;
//
//     // Check 7-bit ST (string terminator) character
//     if i < b.len() && b[i - 1] == ansi::ESC && b[i] == b'\\' {
//         i += 1;
//     }
//
//     let pa = ansi::Params(&params[..params_len]);
//     match cmd {
//         cmd if cmd == b'r' as ansi::Cmd | (b'+' as ansi::Cmd) << parser::INTERMED_SHIFT => {
//             // XTGETTCAP responses
//             let (param, _, _) = pa.param(0, 0);
//             match param {
//                 1 => { // 1 means valid response, 0 means invalid response
//                     let tc = parse_termcap(&b[start..end]);
//                     // XXX: some terminals like KiTTY report invalid responses with
//                     // their queries i.e. sending a query for "Tc" using "\x1bP+q5463\x1b\\"
//                     // returns "\x1bP0+r5463\x1b\\".
//                     // The specs says that invalid responses should be in the form of
//                     // DCS 0 + r ST "\x1bP0+r\x1b\\"
//                     // We ignore invalid responses and only send valid ones to the program.
//                     //
//                     // See: https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Operating-System-Commands
//                     return (i, tc);
//                 }
//                 _ => {}
//             }
//         }
//         cmd if cmd == b'|' as ansi::Cmd | (b'>' as ansi::Cmd) << parser::PREFIX_SHIFT => {
//             // XTVersion response
//             return (i, Some(TerminalVersionEvent { version: String::from_utf8_lossy(&b[start..end]).to_string() }));
//         }
//         cmd if cmd == b'|' as ansi::Cmd | (b'!' as ansi::Cmd) << parser::INTERMED_SHIFT => {
//             // Teritary Device Attributes
//             return (i, parse_tertiary_dev_attrs(&b[start..end]));
//         }
//         _ => {}
//     }
//
//     return (i, Some(UnknownDcsEvent(b[..i].to_vec())));
// }
//
// fn parse_apc(p: &mut EventDecoder, b: &[u8]) -> (usize, Option<Event>) {
//     if b.len() == 2 && b[0] == ansi::ESC {
//         // short cut if this is an alt+_ key
//         return (2, Some(KeyPressEvent { code: b[1] as char, mod_: ModAlt, ..Default::default() }));
//     }
//
//     // APC sequences are introduced by APC (0x9f) or ESC _ (0x1b 0x5f)
//     p.parse_st_terminated(ansi::APC, b'_', Some(Box::new(|b: &[u8]| -> Option<Event> {
//         if b.is_empty() {
//             return None;
//         }
//
//         match b[0] {
//             b'G' => { // Kitty Graphics Protocol
//                 let mut g = KittyGraphicsEvent::default();
//                 let parts: Vec<&[u8]> = b[1..].split(|&c| c == b';').collect();
//                 g.options.unmarshal_text(parts[0]); //nolint:errcheck,gosec
//                 if parts.len() > 1 {
//                     g.payload = parts[1].to_vec();
//                 }
//                 return Some(Event::KittyGraphics(g));
//             }
//             _ => {}
//         }
//
//         None
//     })))(b)
// }
//
// fn parse_utf8(p: &mut EventDecoder, b: &[u8]) -> (usize, Option<Event>) {
//     if b.is_empty() {
//         return (0, None);
//     }
//
//     let c = b[0];
//     if c <= ansi::US || c == ansi::DEL {
//         // Control codes get handled by parseControl
//         return (1, Some(p.parse_control(c)));
//     } else if c > ansi::US && c < ansi::DEL {
//         // ASCII printable characters
//         let code = c as char;
//         let mut k = KeyPressEvent { code, text: code.to_string(), ..Default::default() };
//         if code.is_ascii_uppercase() {
//             // Convert upper case letters to lower case + shift modifier
//             k.code = code.to_ascii_lowercase();
//             k.shifted_code = Some(code);
//             k.mod_ |= ModShift;
//         }
//
//         return (1, Some(k));
//     }
//
//     let (code, _) = std::str::from_utf8(b).unwrap_or("").chars().next()
//         .map(|c| (c, 0))
//         .unwrap_or(('\u{FFFD}', 0));
//     let (code, _) = if code == '\u{FFFD}' && b.len() >= 1 {
//         // Check for RuneError
//         if std::str::from_utf8(b).is_err() {
//             ('\u{FFFD}', 0)
//         } else {
//             (code, 0)
//         }
//     } else {
//         (code, 0)
//     };
//     // Simplified: use the first grapheme cluster
//     let cluster = b; // In practice, use uniseg::FirstGraphemeCluster
//     let text = String::from_utf8_lossy(cluster).to_string();
//     let mut code = code;
//     for (i, _) in text.char_indices() {
//         if i > 0 {
//             // Use [KeyExtended] for multi-rune graphemes
//             code = KeyExtended;
//             break;
//         }
//     }
//
//     (cluster.len(), Some(KeyPressEvent { code, text, ..Default::default() }))
// }
//
// fn parse_control(p: &EventDecoder, b: u8) -> Event {
//     match b {
//         ansi::NUL => {
//             if p.legacy & FLAG_CTRL_AT != 0 {
//                 return KeyPressEvent { code: '@', mod_: ModCtrl, ..Default::default() }.into();
//             }
//             return KeyPressEvent { code: KeySpace, mod_: ModCtrl, ..Default::default() }.into();
//         }
//         ansi::BS => {
//             return KeyPressEvent { code: 'h', mod_: ModCtrl, ..Default::default() }.into();
//         }
//         ansi::HT => {
//             if p.legacy & FLAG_CTRL_I != 0 {
//                 return KeyPressEvent { code: 'i', mod_: ModCtrl, ..Default::default() }.into();
//             }
//             return KeyPressEvent { code: KeyTab, ..Default::default() }.into();
//         }
//         ansi::CR => {
//             if p.legacy & FLAG_CTRL_M != 0 {
//                 return KeyPressEvent { code: 'm', mod_: ModCtrl, ..Default::default() }.into();
//             }
//             return KeyPressEvent { code: KeyEnter, ..Default::default() }.into();
//         }
//         ansi::ESC => {
//             if p.legacy & FLAG_CTRL_OPEN_BRACKET != 0 {
//                 return KeyPressEvent { code: '[', mod_: ModCtrl, ..Default::default() }.into();
//             }
//             return KeyPressEvent { code: KeyEscape, ..Default::default() }.into();
//         }
//         ansi::DEL => {
//             if p.legacy & FLAG_BACKSPACE != 0 {
//                 return KeyPressEvent { code: KeyDelete, ..Default::default() }.into();
//             }
//             return KeyPressEvent { code: KeyBackspace, ..Default::default() }.into();
//         }
//         ansi::SP => {
//             return KeyPressEvent { code: KeySpace, text: " ".to_string(), ..Default::default() }.into();
//         }
//         _ => {
//             if b >= ansi::SOH && b <= ansi::SUB {
//                 // Use lower case letters for control codes
//                 let code = char::from_u32(b as u32 + 0x60).unwrap_or('\u{FFFD}');
//                 return KeyPressEvent { code, mod_: ModCtrl, ..Default::default() }.into();
//             } else if b >= ansi::FS && b <= ansi::US {
//                 let code = char::from_u32(b as u32 + 0x40).unwrap_or('\u{FFFD}');
//                 return KeyPressEvent { code, mod_: ModCtrl, ..Default::default() }.into();
//             }
//             return UnknownEvent(vec![b]).into();
//         }
//     }
// }
//
// fn parse_xterm_modify_other_keys(params: ansi::Params) -> Option<Event> {
//     // XTerm modify other keys starts with ESC [ 27 ; <modifier> ; <code> ~
//     let (xmod, _, _) = params.param(1, 1);
//     let (xrune, _, _) = params.param(2, 1);
//     let mod_ = key_mod(xmod - 1);
//     let r = char::from_u32(xrune as u32).unwrap_or('\u{FFFD}');
//
//     match r as u32 {
//         x if x == ansi::BS as u32 => {
//             return Some(KeyPressEvent { mod_, code: KeyBackspace, ..Default::default() }.into());
//         }
//         x if x == ansi::HT as u32 => {
//             return Some(KeyPressEvent { mod_, code: KeyTab, ..Default::default() }.into());
//         }
//         x if x == ansi::CR as u32 => {
//             return Some(KeyPressEvent { mod_, code: KeyEnter, ..Default::default() }.into());
//         }
//         x if x == ansi::ESC as u32 => {
//             return Some(KeyPressEvent { mod_, code: KeyEscape, ..Default::default() }.into());
//         }
//         x if x == ansi::DEL as u32 => {
//             return Some(KeyPressEvent { mod_, code: KeyBackspace, ..Default::default() }.into());
//         }
//         _ => {}
//     }
//
//     // CSI 27 ; <modifier> ; <code> ~ keys defined in XTerm modifyOtherKeys
//     let mut k = KeyPressEvent { code: r, mod_, ..Default::default() };
//     if k.mod_ <= ModShift {
//         k.text = r.to_string();
//     }
//
//     Some(k.into())
// }
//
// // Kitty Clipboard Control Sequences.
// static mut KITTY_KEY_MAP: std::sync::LazyLock<std::collections::HashMap<i32, Key>> = std::sync::LazyLock::new(|| {
//     let mut m = std::collections::HashMap::new();
//     m.insert(ansi::BS as i32, Key { code: KeyBackspace, ..Default::default() });
//     m.insert(ansi::HT as i32, Key { code: KeyTab, ..Default::default() });
//     m.insert(ansi::CR as i32, Key { code: KeyEnter, ..Default::default() });
//     m.insert(ansi::ESC as i32, Key { code: KeyEscape, ..Default::default() });
//     m.insert(ansi::DEL as i32, Key { code: KeyBackspace, ..Default::default() });
//
//     m.insert(57344, Key { code: KeyEscape, ..Default::default() });
//     m.insert(57345, Key { code: KeyEnter, ..Default::default() });
//     m.insert(57346, Key { code: KeyTab, ..Default::default() });
//     m.insert(57347, Key { code: KeyBackspace, ..Default::default() });
//     m.insert(57348, Key { code: KeyInsert, ..Default::default() });
//     m.insert(57349, Key { code: KeyDelete, ..Default::default() });
//     m.insert(57350, Key { code: KeyLeft, ..Default::default() });
//     m.insert(57351, Key { code: KeyRight, ..Default::default() });
//     m.insert(57352, Key { code: KeyUp, ..Default::default() });
//     m.insert(57353, Key { code: KeyDown, ..Default::default() });
//     m.insert(57354, Key { code: KeyPgUp, ..Default::default() });
//     m.insert(57355, Key { code: KeyPgDown, ..Default::default() });
//     m.insert(57356, Key { code: KeyHome, ..Default::default() });
//     m.insert(57357, Key { code: KeyEnd, ..Default::default() });
//     m.insert(57358, Key { code: KeyCapsLock, ..Default::default() });
//     m.insert(57359, Key { code: KeyScrollLock, ..Default::default() });
//     m.insert(57360, Key { code: KeyNumLock, ..Default::default() });
//     m.insert(57361, Key { code: KeyPrintScreen, ..Default::default() });
//     m.insert(57362, Key { code: KeyPause, ..Default::default() });
//     m.insert(57363, Key { code: KeyMenu, ..Default::default() });
//     m.insert(57364, Key { code: KeyF1, ..Default::default() });
//     m.insert(57365, Key { code: KeyF2, ..Default::default() });
//     m.insert(57366, Key { code: KeyF3, ..Default::default() });
//     m.insert(57367, Key { code: KeyF4, ..Default::default() });
//     m.insert(57368, Key { code: KeyF5, ..Default::default() });
//     m.insert(57369, Key { code: KeyF6, ..Default::default() });
//     m.insert(57370, Key { code: KeyF7, ..Default::default() });
//     m.insert(57371, Key { code: KeyF8, ..Default::default() });
//     m.insert(57372, Key { code: KeyF9, ..Default::default() });
//     m.insert(57373, Key { code: KeyF10, ..Default::default() });
//     m.insert(57374, Key { code: KeyF11, ..Default::default() });
//     m.insert(57375, Key { code: KeyF12, ..Default::default() });
//     m.insert(57376, Key { code: KeyF13, ..Default::default() });
//     m.insert(57377, Key { code: KeyF14, ..Default::default() });
//     m.insert(57378, Key { code: KeyF15, ..Default::default() });
//     m.insert(57379, Key { code: KeyF16, ..Default::default() });
//     m.insert(57380, Key { code: KeyF17, ..Default::default() });
//     m.insert(57381, Key { code: KeyF18, ..Default::default() });
//     m.insert(57382, Key { code: KeyF19, ..Default::default() });
//     m.insert(57383, Key { code: KeyF20, ..Default::default() });
//     m.insert(57384, Key { code: KeyF21, ..Default::default() });
//     m.insert(57385, Key { code: KeyF22, ..Default::default() });
//     m.insert(57386, Key { code: KeyF23, ..Default::default() });
//     m.insert(57387, Key { code: KeyF24, ..Default::default() });
//     m.insert(57388, Key { code: KeyF25, ..Default::default() });
//     m.insert(57389, Key { code: KeyF26, ..Default::default() });
//     m.insert(57390, Key { code: KeyF27, ..Default::default() });
//     m.insert(57391, Key { code: KeyF28, ..Default::default() });
//     m.insert(57392, Key { code: KeyF29, ..Default::default() });
//     m.insert(57393, Key { code: KeyF30, ..Default::default() });
//     m.insert(57394, Key { code: KeyF31, ..Default::default() });
//     m.insert(57395, Key { code: KeyF32, ..Default::default() });
//     m.insert(57396, Key { code: KeyF33, ..Default::default() });
//     m.insert(57397, Key { code: KeyF34, ..Default::default() });
//     m.insert(57398, Key { code: KeyF35, ..Default::default() });
//     m.insert(57399, Key { code: KeyKp0, ..Default::default() });
//     m.insert(57400, Key { code: KeyKp1, ..Default::default() });
//     m.insert(57401, Key { code: KeyKp2, ..Default::default() });
//     m.insert(57402, Key { code: KeyKp3, ..Default::default() });
//     m.insert(57403, Key { code: KeyKp4, ..Default::default() });
//     m.insert(57404, Key { code: KeyKp5, ..Default::default() });
//     m.insert(57405, Key { code: KeyKp6, ..Default::default() });
//     m.insert(57406, Key { code: KeyKp7, ..Default::default() });
//     m.insert(57407, Key { code: KeyKp8, ..Default::default() });
//     m.insert(57408, Key { code: KeyKp9, ..Default::default() });
//     m.insert(57409, Key { code: KeyKpDecimal, ..Default::default() });
//     m.insert(57410, Key { code: KeyKpDivide, ..Default::default() });
//     m.insert(57411, Key { code: KeyKpMultiply, ..Default::default() });
//     m.insert(57412, Key { code: KeyKpMinus, ..Default::default() });
//     m.insert(57413, Key { code: KeyKpPlus, ..Default::default() });
//     m.insert(57414, Key { code: KeyKpEnter, ..Default::default() });
//     m.insert(57415, Key { code: KeyKpEqual, ..Default::default() });
//     m.insert(57416, Key { code: KeyKpSep, ..Default::default() });
//     m.insert(57417, Key { code: KeyKpLeft, ..Default::default() });
//     m.insert(57418, Key { code: KeyKpRight, ..Default::default() });
//     m.insert(57419, Key { code: KeyKpUp, ..Default::default() });
//     m.insert(57420, Key { code: KeyKpDown, ..Default::default() });
//     m.insert(57421, Key { code: KeyKpPgUp, ..Default::default() });
//     m.insert(57422, Key { code: KeyKpPgDown, ..Default::default() });
//     m.insert(57423, Key { code: KeyKpHome, ..Default::default() });
//     m.insert(57424, Key { code: KeyKpEnd, ..Default::default() });
//     m.insert(57425, Key { code: KeyKpInsert, ..Default::default() });
//     m.insert(57426, Key { code: KeyKpDelete, ..Default::default() });
//     m.insert(57427, Key { code: KeyKpBegin, ..Default::default() });
//     m.insert(57428, Key { code: KeyMediaPlay, ..Default::default() });
//     m.insert(57429, Key { code: KeyMediaPause, ..Default::default() });
//     m.insert(57430, Key { code: KeyMediaPlayPause, ..Default::default() });
//     m.insert(57431, Key { code: KeyMediaReverse, ..Default::default() });
//     m.insert(57432, Key { code: KeyMediaStop, ..Default::default() });
//     m.insert(57433, Key { code: KeyMediaFastForward, ..Default::default() });
//     m.insert(57434, Key { code: KeyMediaRewind, ..Default::default() });
//     m.insert(57435, Key { code: KeyMediaNext, ..Default::default() });
//     m.insert(57436, Key { code: KeyMediaPrev, ..Default::default() });
//     m.insert(57437, Key { code: KeyMediaRecord, ..Default::default() });
//     m.insert(57438, Key { code: KeyLowerVol, ..Default::default() });
//     m.insert(57439, Key { code: KeyRaiseVol, ..Default::default() });
//     m.insert(57440, Key { code: KeyMute, ..Default::default() });
//     m.insert(57441, Key { code: KeyLeftShift, ..Default::default() });
//     m.insert(57442, Key { code: KeyLeftCtrl, ..Default::default() });
//     m.insert(57443, Key { code: KeyLeftAlt, ..Default::default() });
//     m.insert(57444, Key { code: KeyLeftSuper, ..Default::default() });
//     m.insert(57445, Key { code: KeyLeftHyper, ..Default::default() });
//     m.insert(57446, Key { code: KeyLeftMeta, ..Default::default() });
//     m.insert(57447, Key { code: KeyRightShift, ..Default::default() });
//     m.insert(57448, Key { code: KeyRightCtrl, ..Default::default() });
//     m.insert(57449, Key { code: KeyRightAlt, ..Default::default() });
//     m.insert(57450, Key { code: KeyRightSuper, ..Default::default() });
//     m.insert(57451, Key { code: KeyRightHyper, ..Default::default() });
//     m.insert(57452, Key { code: KeyRightMeta, ..Default::default() });
//     m.insert(57453, Key { code: KeyIsoLevel3Shift, ..Default::default() });
//     m.insert(57454, Key { code: KeyIsoLevel5Shift, ..Default::default() });
//
//     // These are some faulty C0 mappings some terminals such as WezTerm have
//     // and doesn't follow the specs.
//     m.insert(ansi::NUL as i32, Key { code: KeySpace, mod_: ModCtrl, ..Default::default() });
//     for i in ansi::SOH as i32..=ansi::SUB as i32 {
//         if !m.contains_key(&i) {
//             m.insert(i, Key { code: char::from_u32((i + 0x60) as u32).unwrap_or('\u{FFFD}'), mod_: ModCtrl, ..Default::default() });
//         }
//     }
//     for i in ansi::FS as i32..=ansi::US as i32 {
//         if !m.contains_key(&i) {
//             m.insert(i, Key { code: char::from_u32((i + 0x40) as u32).unwrap_or('\u{FFFD}'), mod_: ModCtrl, ..Default::default() });
//         }
//     }
//
//     m
// });
//
// const KITTY_SHIFT: i32 = 1 << 0;
// const KITTY_ALT: i32 = 1 << 1;
// const KITTY_CTRL: i32 = 1 << 2;
// const KITTY_SUPER: i32 = 1 << 3;
// const KITTY_HYPER: i32 = 1 << 4;
// const KITTY_META: i32 = 1 << 5;
// const KITTY_CAPS_LOCK: i32 = 1 << 6;
// const KITTY_NUM_LOCK: i32 = 1 << 7;
//
// fn from_kitty_mod(mod_: i32) -> KeyMod {
//     let mut m: KeyMod = 0;
//     if mod_ & KITTY_SHIFT != 0 {
//         m |= ModShift;
//     }
//     if mod_ & KITTY_ALT != 0 {
//         m |= ModAlt;
//     }
//     if mod_ & KITTY_CTRL != 0 {
//         m |= ModCtrl;
//     }
//     if mod_ & KITTY_SUPER != 0 {
//         m |= ModSuper;
//     }
//     if mod_ & KITTY_HYPER != 0 {
//         m |= ModHyper;
//     }
//     if mod_ & KITTY_META != 0 {
//         m |= ModMeta;
//     }
//     if mod_ & KITTY_CAPS_LOCK != 0 {
//         m |= ModCapsLock;
//     }
//     if mod_ & KITTY_NUM_LOCK != 0 {
//         m |= ModNumLock;
//     }
//     return m;
// }
//
// // parseKittyKeyboard parses a Kitty Keyboard Protocol sequence.
// //
// // In `CSI u`, this is parsed as:
// //
// //	CSI codepoint ; modifiers u
// //	codepoint: ASCII Dec value
// //
// // The Kitty Keyboard Protocol extends this with optional components that can be
// // enabled progressively. The full sequence is parsed as:
// //
// //	CSI unicode-key-code:alternate-key-codes ; modifiers:event-type ; text-as-codepoints u
// //
// // See https://sw.kovidgoyal.net/kitty/keyboard-protocol/
// fn parse_kitty_keyboard(params: ansi::Params) -> Option<Event> {
//     let mut is_release = false;
//     let mut key = Key::default();
//
//     // The index of parameters separated by semicolons ';'. Sub parameters are
//     // separated by colons ':'.
//     let mut param_idx: usize = 0;
//     let mut sud_idx: usize = 0; // The sub parameter index
//     for p in params.iter() {
//         // Kitty Keyboard Protocol has 3 optional components.
//         match param_idx {
//             0 => {
//                 match sud_idx {
//                     0 => {
//                         let code = p.param(1); // CSI u has a default value of 1
//                         let found_key = KITTY_KEY_MAP.get(&code);
//                         if let Some(k) = found_key {
//                             key = *k;
//                         } else {
//                             let r = char::from_u32(code as u32).unwrap_or('\u{FFFD}');
//                             if !r.is_ascii() || r == '\u{FFFD}' {
//                                 // Check if valid rune
//                                 if char::from_u32(code as u32).is_none() {
//                                     key.code = '\u{FFFD}';
//                                 } else {
//                                     key.code = r;
//                                 }
//                             } else {
//                                 key.code = r;
//                             }
//                         }
//                     }
//                     2 => {
//                         // shifted key + base key
//                         let b = char::from_u32(p.param(1) as u32).unwrap_or('\u{FFFD}');
//                         if b.is_ascii_graphic() || b.is_alphanumeric() { // unicode.IsPrint
//                             // XXX: When alternate key reporting is enabled, the protocol
//                             // can return 3 things, the unicode codepoint of the key,
//                             // the shifted codepoint of the key, and the standard
//                             // PC-101 key layout codepoint.
//                             // This is useful to create an unambiguous mapping of keys
//                             // when using a different language layout.
//                             key.base_code = Some(b);
//                         }
//                         // fallthrough
//                         // shifted key
//                         let s = char::from_u32(p.param(1) as u32).unwrap_or('\u{FFFD}');
//                         if s.is_ascii_graphic() || s.is_alphanumeric() {
//                             // XXX: We swap keys here because we want the shifted key
//                             // to be the Rune that is returned by the event.
//                             // For example, shift+a should produce "A" not "a".
//                             // In such a case, we set AltRune to the original key "a"
//                             // and Rune to "A".
//                             key.shifted_code = Some(s);
//                         }
//                     }
//                     1 => {
//                         // shifted key
//                         let s = char::from_u32(p.param(1) as u32).unwrap_or('\u{FFFD}');
//                         if s.is_ascii_graphic() || s.is_alphanumeric() {
//                             // XXX: We swap keys here because we want the shifted key
//                             // to be the Rune that is returned by the event.
//                             // For example, shift+a should produce "A" not "a".
//                             // In such a case, we set AltRune to the original key "a"
//                             // and Rune to "A".
//                             key.shifted_code = Some(s);
//                         }
//                     }
//                     _ => {}
//                 }
//             }
//             1 => {
//                 match sud_idx {
//                     0 => {
//                         let mod_ = p.param(1);
//                         if mod_ > 1 {
//                             key.mod_ = from_kitty_mod(mod_ - 1);
//                             if key.mod_ > ModShift {
//                                 // XXX: We need to clear the text if we have a modifier key
//                                 // other than a [ModShift] key.
//                                 key.text = String::new();
//                             }
//                         }
//                     }
//                     1 => {
//                         match p.param(1) {
//                             2 => {
//                                 key.is_repeat = true;
//                             }
//                             3 => {
//                                 is_release = true;
//                             }
//                             _ => {}
//                         }
//                     }
//                     2 => {}
//                     _ => {}
//                 }
//             }
//             2 => {
//                 let code = p.param(0);
//                 if code != 0 {
//                     key.text.push(char::from_u32(code as u32).unwrap_or('\u{FFFD}'));
//                 }
//             }
//             _ => {}
//         }
//
//         sud_idx += 1;
//         if !p.has_more() {
//             param_idx += 1;
//             sud_idx = 0;
//         }
//     }
//
//     let mut key_mod = key.mod_;
//
//     // Remove these lock modifiers from now on since they don't affect the text.
//     key_mod &= !ModNumLock;
//     // keyMod &^= ModScrollLock // Kitty doesn't support scroll lock
//
//     let print_mod = key_mod <= ModShift || key_mod == ModCapsLock || key_mod == (ModShift | ModCapsLock);
//     let print_key_pad = key.code as u32 >= KeyKpEqual && key.code as u32 <= KeyKpSep;
//     if key.text.is_empty() && print_key_pad && print_mod {
//         let code_u32 = key.code as u32;
//         if code_u32 >= KeyKp0 && code_u32 <= KeyKp9 {
//             key.text = char::from_u32('0' as u32 + code_u32 - KeyKp0).unwrap().to_string();
//         } else if code_u32 == KeyKpEqual {
//             key.text = "=".to_string();
//         } else if code_u32 == KeyKpMultiply {
//             key.text = "*".to_string();
//         } else if code_u32 == KeyKpPlus {
//             key.text = "+".to_string();
//         } else if code_u32 == KeyKpMinus {
//             key.text = "-".to_string();
//         } else if code_u32 == KeyKpDecimal {
//             key.text = ".".to_string();
//         } else if code_u32 == KeyKpDivide {
//             key.text = "/".to_string();
//         } else if code_u32 == KeyKpSep {
//             key.text = ",".to_string();
//         }
//     }
//
//     if key.text.is_empty() && key.code.is_ascii_graphic() && print_mod {
//         if key_mod == 0 {
//             key.text = key.code.to_string();
//         } else {
//             let desired_case = if key_mod.contains(ModShift) || key_mod.contains(ModCapsLock) {
//                 |c: char| c.to_ascii_uppercase()
//             } else {
//                 |c: char| c.to_ascii_lowercase()
//             };
//             if let Some(shifted) = key.shifted_code {
//                 key.text = shifted.to_string();
//             } else {
//                 key.text = desired_case(key.code).to_string();
//             }
//         }
//     }
//
//     if is_release {
//         return Some(Event::KeyRelease(KeyReleaseEvent(key)));
//     }
//
//     return Some(Event::KeyPress(KeyPressEvent(key)));
// }
//
// // parseKittyKeyboardExt parses a Kitty Keyboard Protocol sequence extensions
// // for non CSI u sequences. This includes things like CSI A, SS3 A and others,
// // and CSI ~.
// fn parse_kitty_keyboard_ext(params: ansi::Params, k: KeyPressEvent) -> Option<Event> {
//     // Handle Kitty keyboard protocol
//     if params.len() > 2 && // We have at least 3 parameters
//         params[1].has_more() { // The second parameter is a subparameter (separated by a ":")
//         match params[2].param(1) { // The third parameter is the event type (defaults to 1)
//             2 => {
//                 let mut k = k;
//                 k.is_repeat = true;
//                 return Some(k.into());
//             }
//             3 => {
//                 return Some(KeyReleaseEvent(k).into());
//             }
//             _ => {}
//         }
//     }
//     return Some(k.into());
// }
//
// fn parse_primary_dev_attrs(params: ansi::Params) -> Option<Event> {
//     // Primary Device Attributes
//     let mut da1 = Vec::with_capacity(params.len());
//     for p in params.iter() {
//         if !p.has_more() {
//             da1.push(p.param(0));
//         }
//     }
//     return Some(PrimaryDeviceAttributesEvent(da1).into());
// }
//
// fn parse_secondary_dev_attrs(params: ansi::Params) -> Option<Event> {
//     // Secondary Device Attributes
//     let mut da2 = Vec::with_capacity(params.len());
//     for p in params.iter() {
//         if !p.has_more() {
//             da2.push(p.param(0));
//         }
//     }
//     return Some(SecondaryDeviceAttributesEvent(da2).into());
// }
//
// fn parse_tertiary_dev_attrs(b: &[u8]) -> Option<Event> {
//     // Tertiary Device Attributes
//     // The response is a 4-digit hexadecimal number.
//     let bts = hex::decode(b);
//     match bts {
//         Ok(bts) => {
//             return Some(TertiaryDeviceAttributesEvent(bts).into());
//         }
//         Err(_) => {
//             let mut buf = b"\x1bP!|".to_vec();
//             buf.extend_from_slice(b);
//             buf.extend_from_slice(b"\x1b\\");
//             return Some(UnknownDcsEvent(buf).into());
//         }
//     }
// }
//
// // Parse SGR-encoded mouse events; SGR extended mouse events. SGR mouse events
// // look like:
// //
// //	ESC [ < Cb ; Cx ; Cy (M or m)
// //
// // where:
// //
// //	Cb is the encoded button code
// //	Cx is the x-coordinate of the mouse
// //	Cy is the y-coordinate of the mouse
// //	M is for button press, m is for button release
// //
// // https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Extended-coordinates
// fn parse_sgr_mouse_event(cmd: ansi::Cmd, params: ansi::Params) -> Option<Event> {
//     let mut x;
//     let (_, _, ok) = params.param(1, 1);
//     if !ok {
//         x = 1;
//     } else {
//         x = params.param(1, 1).0;
//     }
//     let mut y;
//     let (_, _, ok) = params.param(2, 1);
//     if !ok {
//         y = 1;
//     } else {
//         y = params.param(2, 1).0;
//     }
//     let release = cmd.final_byte() == b'm';
//     let (b, _, _) = params.param(0, 0);
//     let (mod_, btn, _, is_motion) = parse_mouse_button(b);
//
//     // (1,1) is the upper left. We subtract 1 to normalize it to (0,0).
//     x -= 1;
//     y -= 1;
//
//     let m = Mouse { x, y, button: btn, mod_ };
//
//     // Wheel buttons don't have release events
//     // Motion can be reported as a release event in some terminals (Windows Terminal)
//     if is_wheel(m.button) {
//         return Some(MouseWheelEvent(m).into());
//     } else if !is_motion && release {
//         return Some(MouseReleaseEvent(m).into());
//     } else if is_motion {
//         return Some(MouseMotionEvent(m).into());
//     }
//     return Some(MouseClickEvent(m).into());
// }
//
// const X10_MOUSE_BYTE_OFFSET: i32 = 32;
//
// // Parse X10-encoded mouse events; the simplest kind. The last release of X10
// // was December 1986, by the way. The original X10 mouse protocol limits the Cx
// // and Cy coordinates to 223 (=255-032).
// //
// // X10 mouse events look like:
// //
// //	ESC [M Cb Cx Cy
// //
// // See: http://www.xfree86.org/current/ctlseqs.html#Mouse%20Tracking
// fn parse_x10_mouse_event(buf: &[u8]) -> Option<Event> {
//     let v = &buf[3..6];
//     let mut b = v[0] as i32;
//     if b >= X10_MOUSE_BYTE_OFFSET {
//         // XXX: b < 32 should be impossible, but we're being defensive.
//         b -= X10_MOUSE_BYTE_OFFSET;
//     }
//
//     let (mod_, btn, is_release, is_motion) = parse_mouse_button(b);
//
//     // (1,1) is the upper left. We subtract 1 to normalize it to (0,0).
//     let x = v[1] as i32 - X10_MOUSE_BYTE_OFFSET - 1;
//     let y = v[2] as i32 - X10_MOUSE_BYTE_OFFSET - 1;
//
//     let m = Mouse { x, y, button: btn, mod_ };
//     if is_wheel(m.button) {
//         return Some(MouseWheelEvent(m).into());
//     } else if is_motion {
//         return Some(MouseMotionEvent(m).into());
//     } else if is_release {
//         return Some(MouseReleaseEvent(m).into());
//     }
//     return Some(MouseClickEvent(m).into());
// }
//
// // See: https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Extended-coordinates
// fn parse_mouse_button(b: i32) -> (KeyMod, MouseButton, bool, bool) {
//     // mouse bit shifts
//     const BIT_SHIFT: i32 = 0b0000_0100;
//     const BIT_ALT: i32 = 0b0000_1000;
//     const BIT_CTRL: i32 = 0b0001_0000;
//     const BIT_MOTION: i32 = 0b0010_0000;
//     const BIT_WHEEL: i32 = 0b0100_0000;
//     const BIT_ADD: i32 = 0b1000_0000; // additional buttons 8-11
//
//     const BITS_MASK: i32 = 0b0000_0011;
//
//     let mut mod_: KeyMod = 0;
//     let mut btn: MouseButton;
//     let mut is_release: bool = false;
//     let mut is_motion: bool = false;
//
//     // Modifiers
//     if b & BIT_ALT != 0 {
//         mod_ |= ModAlt;
//     }
//     if b & BIT_CTRL != 0 {
//         mod_ |= ModCtrl;
//     }
//     if b & BIT_SHIFT != 0 {
//         mod_ |= ModShift;
//     }
//
//     if b & BIT_ADD != 0 {
//         btn = MouseBackward + MouseButton::from(b & BITS_MASK);
//     } else if b & BIT_WHEEL != 0 {
//         btn = MouseWheelUp + MouseButton::from(b & BITS_MASK);
//     } else {
//         btn = MouseLeft + MouseButton::from(b & BITS_MASK);
//         // X10 reports a button release as 0b0000_0011 (3)
//         if b & BITS_MASK == BITS_MASK {
//             btn = MouseNone;
//             is_release = true;
//         }
//     }
//
//     // Motion bit doesn't get reported for wheel events.
//     if b & BIT_MOTION != 0 && !is_wheel(btn) {
//         is_motion = true;
//     }
//
//     return (mod_, btn, is_release, is_motion);
// }
//
// // isWheel returns true if the mouse event is a wheel event.
// fn is_wheel(btn: MouseButton) -> bool {
//     return btn >= MouseWheelUp && btn <= MouseWheelRight;
// }