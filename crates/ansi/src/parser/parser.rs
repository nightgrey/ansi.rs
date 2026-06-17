use maybe::Maybe;
use utils::Nested;
use crate::parser::conditions::is_end_of_csi;
use super::*;
use super::collectors::{Parameters};

pub trait Handler {
    /// Draw a character to the screen and update states.
    fn print(&mut self, _byte: char) {}

    /// Execute a C0 or C1 control function.
    fn execute(&mut self, _byte: u8) {}

    /// The final character of an escape sequence has arrived.
    fn esc(&mut self, _intermediates: &ByteStr, _final_byte: u8) {}

    /// A final character has arrived for a CSI sequence.
    fn csi(&mut self, _params: Params<'_>, _intermediates: &ByteStr, _final_byte: char) {}

    /// Invoked when a final character arrives in first part of device control
    /// string. Subsequent bytes in the control string are delivered via
    /// [`Handler::dcs_byte`], and termination via [`Handler::dcs_end`].
    fn dcs_start(&mut self, _params: Params<'_>, _intermediates: &ByteStr, _final_char: char) {}

    /// A byte of a DCS data string. C0 controls are also passed here.
    fn dcs_byte(&mut self, _byte: u8) {}


    /// The DCS data string has been terminated.
    fn dcs_end(&mut self, _byte: u8) {}

    /// Begin an operating system command. Subsequent body bytes are delivered
    /// via [`Handler::osc_byte`]; termination via [`Handler::osc_end`].
    fn osc_start(&mut self) {}

    /// A byte of OSC data.
    fn osc_byte(&mut self, _byte: u8) {}

    /// The OSC string has been terminated.
    fn osc_end(&mut self, _byte: u8) {}
}

#[derive(Debug, Default)]
pub struct Parser {
    pub state: State,

    pub params: Parameters,
    pub intermediates: ByteString,

    pub utf8: [u8; 4],
    pub utf8_len: usize,
}

impl Parser {
    pub fn advance(&mut self, handler: &mut impl Handler, bytes: &[u8]) -> usize {
        match str::from_utf8(bytes) {
            Ok(str) => {
                println!("[FEED] = {:?}", str);
            },
            Err(error) => {
                let valid_to = error.valid_up_to();
                let valid = str::from_utf8(&bytes[..valid_to]).unwrap();
                let invalid = str::from_utf8(&bytes[valid_to..valid_to + error.error_len().unwrap_or(0)]).unwrap_or("");
                let suffix = str::from_utf8(&bytes[valid_to + error.error_len().unwrap_or(0)..]).unwrap_or("");
                println!("[INVALID FEED] = \"{}[INVALID]{}[/INVALID]{}\"", valid, invalid, suffix);
            }
        }

        let mut i = 0;

        // Handle partial codepoints from previous calls.
        if self.utf8_len > 0 {
            i += self.advance_utf8(handler, bytes);
        }

        while i != bytes.len() {
            println!("{:?} -> State::{:?} = {:?}", i, self.state, bytes[i] as char);
            i += match self.state {
                State::Ground => self.advance_ground(handler, &bytes[i..]),
                State::CsiIgnore => self.advance_csi_ignore(handler, &bytes[i..]),
                _ => self.advance_rest(handler, bytes[i]),
            };
        }

        i
    }

    /// Advance the parser while processing a partial utf8 codepoint.
    #[inline]
    pub(super) fn advance_utf8(&mut self, handler: &mut impl Handler, bytes: &[u8]) -> usize {
        // Try to copy up to 3 more characters, to ensure the codepoint is complete.
        let old_bytes = self.utf8_len;
        let to_copy = bytes.len().min(self.utf8.len() - old_bytes);
        self.utf8[old_bytes..old_bytes + to_copy].copy_from_slice(&bytes[..to_copy]);
        self.utf8_len += to_copy;

        // Parse the unicode character.
        match str::from_utf8(&self.utf8[..self.utf8_len]) {
            // If the entire buffer is valid, use the first character and continue parsing.
            Ok(parsed) => {
                let c = unsafe { parsed.chars().next().unwrap_unchecked() };
                handler.print(c);

                self.utf8_len = 0;
                c.len_utf8() - old_bytes
            },
            Err(err) => {
                // If we have any valid bytes, that means we partially copied another
                // utf8 character into `partial_utf8`. Since we only care about the
                // first character, we just ignore the rest.

                let valid_len = err.valid_up_to();

                if valid_len > 0 {
                    let c = unsafe {
                        let parsed = str::from_utf8_unchecked(&self.utf8[..valid_len]);
                        parsed.chars().next().unwrap_unchecked()
                    };

                    handler.print(c);

                    self.utf8_len = 0;
                    return valid_len - old_bytes;
                }

                match err.error_len() {
                    // If the partial character was also invalid, emit the replacement
                    // character.
                    Some(invalid_len) => {
                        handler.print('�');

                        self.utf8_len = 0;
                        invalid_len - old_bytes
                    },
                    // If the character still isn't complete, wait for more data.
                    None => to_copy,
                }
            },
        }
    }

    #[inline]
    pub(super) fn advance_ground(&mut self, handler: &mut impl Handler, bytes: &[u8]) -> usize {
        // Find the next escape character.
        let bytes_len = bytes.len();
        let chars_len = memchr::memchr(0x1B, bytes).unwrap_or(bytes_len);

        // If the next character is ESC, just process it and short-circuit.
        if chars_len == 0 {
            self.state = State::Escape;
            self.clear();
            return 1;
        }

        match str::from_utf8(&bytes[..chars_len]) {
            Ok(parsed) => {
                Self::dispatch_ground(handler, parsed);
                let mut parsed_len = chars_len;

                // If there's another character, it must be escape so process it directly.
                if parsed_len < bytes_len {
                    self.state = State::Escape;
                    self.clear();
                    parsed_len += 1;
                }

                parsed_len
            },
            // Handle invalid and partial utf8.
            Err(err) => {
                // Dispatch all the valid bytes.
                let valid_len = err.valid_up_to();
                let parsed = unsafe { str::from_utf8_unchecked(&bytes[..valid_len]) };
                Self::dispatch_ground(handler, parsed);

                match err.error_len() {
                    Some(len) => {
                        // Execute C1 escapes or emit replacement character.
                        if len == 1 && bytes[valid_len] <= 0x9F {
                            handler.execute(bytes[valid_len]);
                        } else {
                            handler.print('�');
                        }

                        // Restart processing after the invalid bytes.
                        //
                        // While we could theoretically try to just re-parse
                        // `bytes[valid_bytes + len..plain_chars]`, it's easier
                        // to just skip it and invalid utf8 is pretty rare anyway.
                        valid_len + len
                    },
                    None => {
                        if chars_len < bytes_len {
                            // Process bytes cut off by escape.
                            handler.print('�');
                            self.state = State::Escape;
                            self.clear();
                            chars_len + 1
                        } else {
                            // Process bytes cut off by the buffer end.
                            let extra_bytes = bytes_len - valid_len;
                            let partial_len = self.utf8_len + extra_bytes;
                            self.utf8[self.utf8_len..partial_len]
                                .copy_from_slice(&bytes[valid_len..valid_len + extra_bytes]);
                            self.utf8_len = partial_len;
                            bytes_len
                        }
                    },
                }
            },
        }
    }

    #[inline]
    pub(super) fn advance_csi_ignore(&mut self, handler: &mut impl Handler, bytes: &[u8]) -> usize {
        let mut i = 0;
        loop {
            if i >= bytes.len() {
                return bytes.len();
            }
            if is_end_of_csi(bytes[i]) {
                break;
            }
            i += 1;
        }

        if bytes[i] == 0x1B {
            self.state = State::Escape;
        } else {
            self.state = State::Ground;
        }

        i + 1
    }

    #[inline]
    pub(super) fn advance_rest(&mut self, handler: &mut impl Handler, byte: u8) -> usize {
        self.transition(handler, byte);
        1
    }

    #[inline]
    pub(super) fn transition(&mut self, handler: &mut impl Handler, byte: u8) {
        let prev_state = self.state;
        let (action, next_state) = self.state.transition(byte);

        if next_state != State::None {
            let exit = prev_state.exit();
            if exit.is_some() {
                self.action(handler, exit, byte);
            }

            self.action(handler, action, byte);

            let entry = next_state.entry();
            if entry.is_some() {
                self.action(handler, entry, byte);
            }

            self.state = next_state;
        } else {
            self.action(handler, action, byte);
        }
    }
    /// Reset parameter / intermediate / partial-UTF-8 buffers.
    pub fn clear(&mut self) {
        self.params.clear();
        self.intermediates.clear();
        self.utf8_len = 0;
    }

    /// Signal end of input. Any buffered partial UTF-8 codepoint is resolved as
    /// U+FFFD and the parser is reset to [`State::Ground`]. Incomplete control
    /// sequences (a CSI/OSC/DCS cut off mid-stream) are discarded without
    /// dispatch, matching standard VT behavior.
    pub fn flush(&mut self, handler: &mut impl Handler) {
        if self.utf8_len > 0 {
            handler.print(char::REPLACEMENT_CHARACTER);
        }
        self.state = State::Ground;
        self.clear();
    }

    fn action(&mut self, handler: &mut impl Handler, action: Action, byte: u8) {
        match action {
            Action::None | Action::Ignore => {}

            Action::Clear => self.clear(),

            Action::Print => handler.print(byte as char),
            Action::Execute => handler.execute(byte),

            Action::Collect => self.intermediates.push(byte),

            Action::Param => match byte {
                b'0'..=b'9' => self.params.push_digit(byte),
                b':' => self.params.push_sub(),
                b';' => self.params.push_main(),
                _ => {}
            },

            Action::EscDispatch => handler.esc(self.intermediates.as_ref(), byte),
            Action::CsiDispatch => {
                self.params.finish();
                handler.csi(
                    self.params.as_nested_slice(),
                    self.intermediates.as_ref(),
                    byte as char,
                );
            }

            Action::DcsStart => {
                self.params.finish();
                handler.dcs_start(
                    self.params.as_nested_slice(),
                    self.intermediates.as_ref(),
                    byte as char,
                );
            }
            Action::DcsPut => handler.dcs_byte(byte),
            Action::DcsEnd => handler.dcs_end(byte),

            Action::OscStart => handler.osc_start(),
            Action::OscPut => handler.osc_byte(byte),
            Action::OscEnd => handler.osc_end(byte),
        }
    }

    /// Handle ground dispatch of print/execute for all characters in a string.
    #[inline]
    fn dispatch_ground(handler: &mut impl Handler, text: &str) {
        for c in text.chars() {
            match c {
                '\x00'..='\x1f' | '\u{80}'..='\u{9f}' => handler.execute(c as u8),
                _ => handler.print(c),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params;
    use crate::parser::{ByteStr, Parameters, Params};
    use derive_more::{Deref, DerefMut};
    use std::fmt::{Debug, Display};
    use utils::NestedConstructor;

    #[derive(Clone, PartialEq, Eq)]
    enum Record {
        Print(char),
        Execute(u8),
        Esc(ByteString, u8),
        Csi(Parameters, ByteString, char),
        Dcs(Parameters, ByteString, char),
        DcsByte(u8),
        DcsTermination(u8),
        Osc,
        OscByte(u8),
        OscTermination(u8),
    }
    impl Debug for Record {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Record::Print(c) => write!(f, "Print({} / 0x{:2x})", *c as char, *c as u8),
                Record::Execute(b) => write!(f, "Execute({} / 0x{:2x})", *b as char, b),
                Record::Esc(i, b) => write!(f, "Esc({:?},  {} / 0x{:2x})", i, *b as char, b),
                Record::Csi(p, i, c) => write!(
                    f,
                    "Csi({:?}, {:?}, {} / 0x{:2x})",
                    p, i, *c as char, *c as u8
                ),
                Record::Dcs(p, i, c) => write!(
                    f,
                    "Dcs({:?}, {:?}, {} / 0x{:2x})",
                    p, i, *c as char, *c as u8
                ),
                Record::DcsByte(b) => write!(f, "DcsByte({} / 0x{:2x})", *b as char, *b as u8),
                Record::DcsTermination(b) => {
                    write!(f, "DcsTermination({} / 0x{:2x})", *b as char, *b as u8)
                }
                Record::Osc => write!(f, "Osc"),
                Record::OscByte(b) => write!(f, "OscByte({} / 0x{:2x})", *b as char, *b as u8),
                Record::OscTermination(b) => {
                    write!(f, "OscTermination({} / 0x{:2x})", *b as char, *b as u8)
                }
            }
        }
    }
    #[derive(Debug, Default, DerefMut, Deref)]
    struct Recorder {
        pub values: Vec<Record>,
    }

    impl Handler for Recorder {
        fn print(&mut self, ch: char) {
            self.values.push(Record::Print(ch));
        }
        fn execute(&mut self, byte: u8) {
            self.values.push(Record::Execute(byte));
        }
        fn esc(&mut self, intermediates: &ByteStr, final_byte: u8) {
            self.values
                .push(Record::Esc(ByteString::from(intermediates), final_byte));
        }
        fn csi(&mut self, params: Params, intermediates: &ByteStr, final_byte: char) {
            self.values.push(Record::Csi(
                params.to_nested_vec(),
                intermediates.to_owned(),
                final_byte,
            ));
        }
        fn dcs_start(&mut self, params: Params, intermediates: &ByteStr, final_char: char) {
            self.values.push(Record::Dcs(
                params.to_nested_vec(),
                intermediates.to_owned(),
                final_char,
            ));
        }
        fn dcs_byte(&mut self, byte: u8) {
            self.values.push(Record::DcsByte(byte));
        }
        fn dcs_end(&mut self, byte: u8) {
            self.values.push(Record::DcsTermination(byte));
        }
        fn osc_start(&mut self) {
            self.values.push(Record::Osc);
        }
        fn osc_byte(&mut self, byte: u8) {
            self.values.push(Record::OscByte(byte));
        }
        fn osc_end(&mut self, byte: u8) {
            self.values.push(Record::OscTermination(byte));
        }
    }

    #[derive(Debug, Default, DerefMut, Deref)]
    struct Harness {
        inner: super::Parser,
        #[deref_mut]
        #[deref]
        recorder: Recorder,
    }

    impl Harness {
        fn new(bytes: impl AsRef<[u8]>) -> Vec<Record> {
            let mut h = Harness::default();
            h.feed(bytes);

            h.recorder.values.clone()
        }

        fn feed(&mut self, bytes: impl AsRef<[u8]>) -> &mut Recorder {
            self.inner.advance(&mut self.recorder, bytes.as_ref());
            &mut self.recorder
        }
    }

    // ---- Ground / print / execute ---------------------------------------

    #[test]
    fn prints_plain_ascii() {
        assert_eq!(
            Harness::new(b"abc"),
            vec![Record::Print('a'), Record::Print('b'), Record::Print('c')],
        );
    }

    #[test]
    fn executes_c0_controls() {
        // BEL, BS, TAB, LF, CR
        assert_eq!(
            Harness::new(b"\x07\x08\x09\x0A\x0D"),
            vec![
                Record::Execute(0x07),
                Record::Execute(0x08),
                Record::Execute(0x09),
                Record::Execute(0x0A),
                Record::Execute(0x0D),
            ]
        );
    }

    #[test]
    fn mixes_print_and_execute() {
        assert_eq!(
            Harness::new(b"a\x07b"),
            vec![
                Record::Print('a'),
                Record::Execute(0x07),
                Record::Print('b')
            ],
        );
    }

    #[test]
    fn ignores_del_in_ground_via_print_path() {
        // 0x7F is in the printable-fast-path range; it should not be executed
        // since DEL traditionally has no visible glyph but most parsers treat
        // it as printable here. We assert current behavior so regressions are
        // visible.
        assert_eq!(Harness::new(b"\x7f"), vec![Record::Print('\x7f')]);
    }

    #[test]
    fn prints_utf8_multibyte() {
        let events = Harness::new("aé東🦀b".as_bytes());
        assert_eq!(
            events,
            vec![
                Record::Print('a'),
                Record::Print('é'),
                Record::Print('東'),
                Record::Print('🦀'),
                Record::Print('b'),
            ],
        );
    }

    #[test]
    fn invalid_utf8_emits_replacement() {
        // Lone continuation byte 0xA0 is invalid.
        assert_eq!(
            Harness::new(&[b'a', 0xA0, b'b']),
            vec![
                Record::Print('a'),
                Record::Print('\u{FFFD}'),
                Record::Print('b'),
            ],
        );
    }

    #[test]
    fn partial_utf8_buffered_across_calls() {
        let mut h = Harness::default();
        // 🦀 = F0 9F A6 80 (4 bytes). Split 2 / 2.
        h.feed(&[0xF0, 0x9F]);
        assert!(h.recorder.values.is_empty(), "no output yet");
        h.feed(&[0xA6, 0x80]);
        assert_eq!(h.recorder.values, vec![Record::Print('🦀')]);
    }

    #[test]
    fn partial_utf8_followed_by_esc_emits_replacement() {
        // Partial 2-byte sequence (0xC3) cut off by ESC.
        assert_eq!(
            Harness::new(b"a\xC3\x1B[m"),
            vec![
                Record::Print('a'),
                Record::Print('\u{FFFD}'),
                Record::Csi(Parameters::new(), ByteString::from(b""), 'm'),
            ],
        );
    }

    // ---- ESC ------------------------------------------------------------

    #[test]
    fn esc_simple_dispatch() {
        // ESC 7 — DECSC
        assert_eq!(
            Harness::new(b"\x1B7"),
            vec![Record::Esc(ByteString::from(b""), b'7')],
        );
    }

    #[test]
    fn esc_with_intermediate() {
        // ESC # 8 — DECALN
        assert_eq!(
            Harness::new(b"\x1B#8"),
            vec![Record::Esc(ByteString::from(b"#"), b'8')],
        );
    }

    #[test]
    fn esc_with_two_intermediates() {
        // ESC SP # F (uncommon but legal)
        assert_eq!(
            Harness::new(b"\x1B #F"),
            vec![Record::Esc(ByteString::from(b" #"), b'F')],
        );
    }

    #[test]
    fn esc_re_entry_aborts_previous() {
        // ESC inside a CSI sequence should abandon the CSI and start fresh.
        assert_eq!(
            Harness::new(b"\x1B[1;\x1B[2m"),
            vec![Record::Csi(params![[2]], ByteString::from(b""), 'm')],
        );
    }

    // ---- CSI ------------------------------------------------------------

    #[test]
    fn csi_no_params() {
        assert_eq!(
            Harness::new(b"\x1B[m"),
            vec![Record::Csi(Parameters::new(), ByteString::from(b""), 'm')],
        );
    }

    #[test]
    fn csi_single_param() {
        assert_eq!(
            Harness::new(b"\x1B[1m"),
            vec![Record::Csi(params![[1]], ByteString::from(b""), 'm')],
        );
    }

    #[test]
    fn csi_multiple_params() {
        assert_eq!(
            Harness::new(b"\x1B[1;2;3m"),
            vec![Record::Csi(
                params!([1], [2], [3]),
                ByteString::from(b""),
                'm'
            )],
        );
    }

    #[test]
    fn csi_subparams() {
        // 24-bit fg via sub-params: 38:2:255:128:0
        assert_eq!(
            Harness::new(b"\x1B[38:2:255:128:0m"),
            vec![Record::Csi(
                params![[38, 2, 255, 128, 0]],
                ByteString::from(b""),
                'm'
            )],
        );
    }

    #[test]
    fn csi_mixed_subparams_and_params() {
        assert_eq!(
            Harness::new(b"\x1B[1;2:3:4;5m"),
            vec![Record::Csi(
                params![[1], [2, 3, 4], [5]],
                ByteString::from(b""),
                'm'
            )],
        );
    }

    #[test]
    fn csi_empty_leading_param_defaults_to_zero() {
        assert_eq!(
            Harness::new(b"\x1B[;1m"),
            vec![Record::Csi(params![[0], [1]], ByteString::from(b""), 'm')],
        );
    }

    #[test]
    fn csi_empty_subparam_defaults_to_zero() {
        // 38:2::255:128:0 — empty colorspace ID should be 0.
        assert_eq!(
            Harness::new(b"\x1B[38:2::255:128:0m"),
            vec![Record::Csi(
                params![[38, 2, 0, 255, 128, 0]],
                ByteString::from(b""),
                'm'
            )],
        );
    }

    #[test]
    fn csi_trailing_semicolon_does_not_add_param() {
        assert_eq!(
            Harness::new(b"\x1B[1;m"),
            vec![Record::Csi(params![[1]], ByteString::from(b""), 'm')],
        );
    }

    #[test]
    fn csi_double_semicolon_inserts_zero() {
        assert_eq!(
            Harness::new(b"\x1B[1;;2m"),
            vec![Record::Csi(
                params![[1], [0], [2]],
                ByteString::from(b""),
                'm'
            )],
        );
    }

    #[test]
    fn csi_trailing_colon_dispatches_with_zero_subparam() {
        assert_eq!(
            Harness::new(b"\x1B[1::m"),
            vec![Record::Csi(params![[1, 0, 0]], ByteString::from(b""), 'm')],
        );
    }

    #[test]
    fn csi_clamps_param_to_max() {
        // 99999 saturates at 16383 (ECMA-48 cap).
        assert_eq!(
            Harness::new(b"\x1B[99999m"),
            vec![Record::Csi(params![[16383]], ByteString::from(b""), 'm')],
        );
    }

    #[test]
    fn csi_private_marker() {
        // DECSET — CSI ? 25 h
        assert_eq!(
            Harness::new(b"\x1B[?25h"),
            vec![Record::Csi(params![[25]], ByteString::from(b"?"), 'h')],
        );
    }

    #[test]
    fn csi_intermediate() {
        // CSI SP q — DECSCUSR
        assert_eq!(
            Harness::new(b"\x1B[2 q"),
            vec![Record::Csi(params![[2]], ByteString::from(b" "), 'q')],
        );
    }

    #[test]
    fn csi_colon_at_entry_enters_ignore() {
        // `[:1m` — leading `:` enters CsiIgnore; nothing dispatches.
        assert_eq!(Harness::new(b"\x1B[:1m"), vec![]);
    }

    #[test]
    fn csi_followed_by_text() {
        assert_eq!(
            Harness::new(b"\x1B[1mhi"),
            vec![
                Record::Csi(params![[1]], ByteString::from(b""), 'm'),
                Record::Print('h'),
                Record::Print('i'),
            ],
        );
    }

    #[test]
    fn csi_can_cancels() {
        // CAN inside a CSI returns to Ground without dispatch.
        assert_eq!(
            Harness::new(b"\x1B[1;2\x18m"),
            vec![Record::Execute(0x18), Record::Print('m')],
        );
    }

    #[test]
    fn csi_sub_cancels() {
        // SUB inside a CSI returns to Ground without dispatch.
        assert_eq!(
            Harness::new(b"\x1B[1;2\x1Am"),
            vec![Record::Execute(0x1A), Record::Print('m')],
        );
    }

    /*mod eight_bit {
        use super::*;
        #[test]
        fn c1_csi_starts_csi() {
            // 0x9B is 8-bit CSI.
            assert_eq!(
                Harness::run([0x9B, b'1', b';', b'2', b'm']),
                vec![Record::Csi(params![[1], [2]], ByteString::from(b""), 'm')],
            );
        }

        #[test]
        fn c1_osc_starts_osc() {
            // 0x9D is 8-bit OSC, 0x9C is ST.
            assert_eq!(
                Harness::run([0x9D, b'0', b';', b'h', b'i', 0x9C]),
                vec![
                    Record::Osc,
                    Record::OscByte(b'0'),
                    Record::OscByte(b';'),
                    Record::OscByte(b'h'),
                    Record::OscByte(b'i'),
                    Record::OscTermination(0x9C),
                ],
            );
        }

        #[test]
        fn c1_dcs_starts_dcs() {
            // 0x90 is 8-bit DCS, 0x9C is ST.
            assert_eq!(
                Harness::run([0x90, b'q', b'X', 0x9C]),
                vec![
                    Record::Dcs(Parameters::new(), ByteString::from(b""), 'q'),
                    Record::DcsByte(b'X'),
                    Record::DcsTermination(0x9C),
                ],
            );
        }

        #[test]
        fn c1_sos_string_is_silently_consumed() {
            // 8-bit SOS introducer 0x98.
            let mut seq = vec![0x98];
            seq.extend_from_slice(b"junk\x1B\\");
            assert_eq!(
                Harness::run(seq),
                vec![Record::Esc(ByteString::from(b""), b'\\')]
            );
        }

    }*/
    // ---- OSC ------------------------------------------------------------

    #[test]
    fn osc_st_terminated() {
        // OSC 0 ; title ST  (ST = ESC \)
        assert_eq!(
            Harness::new(b"\x1B]0;title\x1B\\"),
            vec![
                Record::Osc,
                Record::OscByte(b'0'),
                Record::OscByte(b';'),
                Record::OscByte(b't'),
                Record::OscByte(b'i'),
                Record::OscByte(b't'),
                Record::OscByte(b'l'),
                Record::OscByte(b'e'),
                Record::OscTermination(0x1B),
                Record::Esc(ByteString::from(b""), b'\\'),
            ],
        );
    }

    #[test]
    fn osc_bel_terminated() {
        // OSC 0 ; title BEL — xterm convention.
        assert_eq!(
            Harness::new(b"\x1B]0;hi\x07"),
            vec![
                Record::Osc,
                Record::OscByte(b'0'),
                Record::OscByte(b';'),
                Record::OscByte(b'h'),
                Record::OscByte(b'i'),
                Record::OscTermination(0x07),
            ],
        );
    }

    #[test]
    fn osc_empty() {
        assert_eq!(
            Harness::new(b"\x1B]\x07"),
            vec![Record::Osc, Record::OscTermination(0x07),],
        );
    }

    // ---- DCS ------------------------------------------------------------

    #[test]
    fn dcs_basic() {
        // DCS $ q   <data>   ESC \
        assert_eq!(
            Harness::new(b"\x1BP$q q\x1B\\"),
            vec![
                Record::Dcs(Parameters::new(), ByteString::from(b"$"), 'q'),
                Record::DcsByte(b' '),
                Record::DcsByte(b'q'),
                Record::DcsTermination(0x1B),
                Record::Esc(ByteString::from(b""), b'\\'),
            ],
        );
    }

    #[test]
    fn dcs_with_params() {
        assert_eq!(
            Harness::new(b"\x1BP1;2|data\x1B\\"),
            vec![
                Record::Dcs(params![[1], [2]], ByteString::from(b""), '|'),
                Record::DcsByte(b'd'),
                Record::DcsByte(b'a'),
                Record::DcsByte(b't'),
                Record::DcsByte(b'a'),
                Record::DcsTermination(0x1B),
                Record::Esc(ByteString::from(b""), b'\\'),
            ],
        );
    }

    #[test]
    fn dcs_with_subparams() {
        assert_eq!(
            Harness::new(b"\x1BP1:2|x\x1B\\"),
            vec![
                Record::Dcs(params![[1, 2]], ByteString::from(b""), '|'),
                Record::DcsByte(b'x'),
                Record::DcsTermination(0x1B),
                Record::Esc(ByteString::from(b""), b'\\'),
            ],
        );
    }

    #[test]
    fn dcs_can_cancels() {
        // CAN aborts the DCS without termination event.
        assert_eq!(
            Harness::new(b"\x1BPq abc\x18tail"),
            vec![
                Record::Dcs(Parameters::new(), ByteString::from(b""), 'q'),
                Record::DcsByte(b' '),
                Record::DcsByte(b'a'),
                Record::DcsByte(b'b'),
                Record::DcsByte(b'c'),
                Record::DcsTermination(0x18),
                Record::Execute(0x18),
                Record::Print('t'),
                Record::Print('a'),
                Record::Print('i'),
                Record::Print('l'),
            ],
        );
    }

    // ---- SOS / PM / APC -------------------------------------------------

    #[test]
    fn pm_string_is_silently_consumed() {
        // ESC ^ ... ESC \ — PM. Body bytes are ignored, only the trailing
        // ESC \ produces an Esc dispatch.
        assert_eq!(
            Harness::new(b"\x1B^junk\x1B\\"),
            vec![Record::Esc(ByteString::from(b""), b'\\')],
        );
    }

    #[test]
    fn apc_string_is_silently_consumed() {
        assert_eq!(
            Harness::new(b"\x1B_x\x1B\\"),
            vec![Record::Esc(ByteString::from(b""), b'\\')],
        );
    }

    // ---- Streaming / chunked input -------------------------------------

    #[test]
    fn csi_split_across_advance_calls() {
        let mut h = Harness::default();
        h.feed(b"\x1B[");
        h.feed(b"38;5;");
        h.feed(b"196m");
        assert_eq!(
            h.recorder.values,
            vec![Record::Csi(
                params![[38], [5], [196]],
                ByteString::from(b""),
                'm'
            )]
        );
    }

    #[test]
    fn osc_split_across_advance_calls() {
        let mut h = Harness::default();
        h.feed(b"\x1B]0;ti");
        h.feed(b"tle\x07");
        assert_eq!(
            h.recorder.values,
            vec![
                Record::Osc,
                Record::OscByte(b'0'),
                Record::OscByte(b';'),
                Record::OscByte(b't'),
                Record::OscByte(b'i'),
                Record::OscByte(b't'),
                Record::OscByte(b'l'),
                Record::OscByte(b'e'),
                Record::OscTermination(0x07),
            ]
        );
    }

    // ---- Combined real-world snippet -----------------------------------

    #[test]
    fn sgr_emoji_reset_combination() {
        // Mirrors the original smoke test.
        let events = Harness::new("\x1B[1;2;3m👨🏿\x1B[0m".as_bytes());
        assert_eq!(
            events,
            vec![
                Record::Csi(params![[1], [2], [3]], ByteString::from(b""), 'm'),
                // 👨🏿 = man + dark skin tone modifier (two codepoints).
                Record::Print('\u{1F468}'),
                Record::Print('\u{1F3FF}'),
                Record::Csi(params![[0]], ByteString::from(b""), 'm'),
            ],
        );
    }

    // ---- Robustness / overflow -----------------------------------------

    #[test]
    fn param_overflow_does_not_panic() {
        // A pathologically long parameter list must not panic. It dispatches
        // with the trailing params dropped once capacity is reached.
        let mut seq = Vec::from(b"\x1B[".as_slice());
        for _ in 0..40 {
            seq.extend_from_slice(b"1;");
        }
        seq.push(b'm');

        let events = Harness::new(seq);
        // Exactly one CSI dispatch, with the trailing params dropped once the
        // builder hits capacity. `NestedRaw<_, 32, 32>` reserves one `starts`
        // slot as a sentinel, so the group cap is 31.
        assert_eq!(events.len(), 1);
        match &events[0] {
            Record::Csi(params, i, c) => {
                assert_eq!(*c, 'm');
                assert_eq!(i, &ByteString::from(b""));
                assert_eq!(params.len(), 31);
            }
            other => panic!("expected a single Csi dispatch, got {other:?}"),
        }
    }

    #[test]
    fn many_param_sgr_within_capacity() {
        assert_eq!(
            Harness::new(b"\x1B[1;2;3;4;5;6;7;8;9;10m"),
            vec![Record::Csi(
                params!([1], [2], [3], [4], [5], [6], [7], [8], [9], [10]),
                ByteString::from(b""),
                'm'
            )],
        );
    }

    // ---- SOS / standalone ST -------------------------------------------

    #[test]
    fn sos_string_is_silently_consumed() {
        // ESC X ... ESC \ — SOS. Body bytes are ignored; only the trailing
        // ESC \ produces an Esc dispatch (parallels PM/APC).
        assert_eq!(
            Harness::new(b"\x1BXjunk\x1B\\"),
            vec![Record::Esc(ByteString::from(b""), b'\\')],
        );
    }

    #[test]
    fn standalone_st_dispatches_as_esc() {
        // ESC \ in ground is just an ESC dispatch.
        assert_eq!(
            Harness::new(b"\x1B\\"),
            vec![Record::Esc(ByteString::from(b""), b'\\')],
        );
    }

    // ---- OSC cancel -----------------------------------------------------

    #[test]
    fn osc_can_cancels() {
        // CAN inside OSC terminates the string and executes the CAN, returning
        // to ground.
        assert_eq!(
            Harness::new(b"\x1B]0;hi\x18"),
            vec![
                Record::Osc,
                Record::OscByte(b'0'),
                Record::OscByte(b';'),
                Record::OscByte(b'h'),
                Record::OscByte(b'i'),
                Record::OscTermination(0x18),
                Record::Execute(0x18),
            ],
        );
    }

    // ---- Malformed UTF-8 ------------------------------------------------

    #[test]
    fn lone_continuation_mid_stream_emits_replacement() {
        // 0xBF is a UTF-8 continuation byte with no leader. (Bytes 0x80..=0x9F
        // are C1 controls and dispatch as Execute instead — see the C1 tests.)
        assert_eq!(
            Harness::new(&[b'x', 0xBF, b'y']),
            vec![
                Record::Print('x'),
                Record::Print('\u{FFFD}'),
                Record::Print('y'),
            ],
        );
    }

    #[test]
    fn overlong_encoding_emits_replacement() {
        // 0xC0 0xAF is an overlong (illegal) encoding of '/'. Each invalid byte
        // resolves to a replacement char.
        assert_eq!(
            Harness::new(&[b'a', 0xC0, 0xAF, b'b']),
            vec![
                Record::Print('a'),
                Record::Print('\u{FFFD}'),
                Record::Print('\u{FFFD}'),
                Record::Print('b'),
            ],
        );
    }

    #[test]
    fn truncated_multibyte_resolved_on_next_advance() {
        // 東 = E6 9D B1 (3 bytes). Split after the first byte: the partial
        // codepoint is buffered, then completed on the following call.
        let mut h = Harness::default();
        h.feed(&[b'a', 0xE6]);
        assert_eq!(h.recorder.values, vec![Record::Print('a')]);
        h.feed(&[0x9D, 0xB1, b'b']);
        assert_eq!(
            h.recorder.values,
            vec![Record::Print('a'), Record::Print('東'), Record::Print('b')],
        );
    }

    // ---- CSI edge cases -------------------------------------------------

    #[test]
    fn del_inside_csi_param_is_ignored() {
        // 0x7F inside a CSI param is ignored; the sequence still dispatches.
        assert_eq!(
            Harness::new(b"\x1B[1;2\x7fm"),
            vec![Record::Csi(params![[1], [2]], ByteString::from(b""), 'm')],
        );
    }

    #[test]
    fn csi_intermediate_only_soft_reset() {
        // CSI ! p — DECSTR (soft reset): intermediate, no params.
        assert_eq!(
            Harness::new(b"\x1B[!p"),
            vec![Record::Csi(Parameters::new(), ByteString::from(b"!"), 'p')],
        );
    }

    // ---- OSC / DCS UTF-8 payload (R1) -----------------------------------

    #[test]
    fn osc_payload_preserves_utf8() {
        // OSC 0 ; é ST — the title contains é (C3 A9). High bytes must reach
        // the handler as raw OSC bytes rather than being dropped.
        assert_eq!(
            Harness::new(b"\x1B]0;\xC3\xA9\x07"),
            vec![
                Record::Osc,
                Record::OscByte(b'0'),
                Record::OscByte(b';'),
                Record::OscByte(0xC3),
                Record::OscByte(0xA9),
                Record::OscTermination(0x07),
            ],
        );
    }

    #[test]
    fn dcs_payload_preserves_utf8() {
        // DCS q é ESC \ — DCS data with é (C3 A9).
        assert_eq!(
            Harness::new(b"\x1BPq\xC3\xA9\x1B\\"),
            vec![
                Record::Dcs(Parameters::new(), ByteString::from(b""), 'q'),
                Record::DcsByte(0xC3),
                Record::DcsByte(0xA9),
                Record::DcsTermination(0x1B),
                Record::Esc(ByteString::from(b""), b'\\'),
            ],
        );
    }

    // ---- flush (R2) -----------------------------------------------------

    #[test]
    fn flush_emits_replacement_for_partial_utf8() {
        let mut h = Harness::default();
        // First two bytes of 🦀 (F0 9F A6 80) — incomplete.
        h.feed(&[0xF0, 0x9F]);
        assert!(h.recorder.values.is_empty());
        h.inner.flush(&mut h.recorder);
        assert_eq!(h.recorder.values, vec![Record::Print('\u{FFFD}')]);
    }

    #[test]
    fn flush_on_clean_boundary_emits_nothing() {
        let mut h = Harness::default();
        h.feed(b"ab");
        let before = h.recorder.values.len();
        h.inner.flush(&mut h.recorder);
        assert_eq!(h.recorder.values.len(), before);
    }

    #[test]
    fn flush_resets_to_ground() {
        let mut h = Harness::default();
        // Enter an incomplete CSI, then flush: the dangling sequence is
        // discarded and subsequent text parses from ground.
        h.feed(b"\x1B[1;2");
        h.inner.flush(&mut h.recorder);
        h.feed(b"x");
        assert_eq!(h.recorder.values, vec![Record::Print('x')]);
    }

    // ---- Chunk-split invariance (R3) ------------------------------------

    /// Feeding a byte stream in any number of chunks must produce the same
    /// events as feeding it whole. This is the core guarantee of a streaming
    /// parser and exercises the partial-UTF-8 buffering paths exhaustively.
    #[test]
    fn split_invariance() {
        let corpus: &[&[u8]] = &[
            b"Hello, world!",
            "héllo 東京 🦀 mix".as_bytes(),
            b"\x1B[1;31m\x1B[38;2;200;100;50mX\x1B[0m",
            b"\x1B[38:2:255:128:0m",
            b"\x1B]0;window \xC3\xA9 title\x07",
            b"\x1BP1;2|device \xF0\x9F\xA6\x80 data\x1B\\",
            b"abc\x07def\x1B7ghi",
            &[b'a', 0xA0, b'b'],       // lone continuation
            &[b'a', 0xC0, 0xAF, b'b'], // overlong encoding
            &[b'x', 0x9B, b'1', b'm'], // raw 8-bit C1 CSI
            "tail 🦀".as_bytes(),      // multibyte at the very end
        ];

        for input in corpus {
            let whole = Harness::new(input);
            for at in 0..=input.len() {
                let mut h = Harness::default();
                h.feed(&input[..at]);
                h.feed(&input[at..]);
                assert_eq!(
                    h.recorder.values, whole,
                    "split at {at} of {input:?} diverged from whole-feed",
                );
            }
        }
    }

    /// A few two-point splits, to cover a partial codepoint straddling more
    /// than one chunk boundary.
    #[test]
    fn split_invariance_two_points() {
        let input = "a東🦀b".as_bytes();
        let whole = Harness::new(input);
        for i in 0..=input.len() {
            for j in i..=input.len() {
                let mut h = Harness::default();
                h.feed(&input[..i]);
                h.feed(&input[i..j]);
                h.feed(&input[j..]);
                assert_eq!(
                    h.recorder.values, whole,
                    "splits at {i},{j} diverged from whole-feed",
                );
            }
        }
    }

    // ---- OSC data batching (perf path, observed per-byte) ---------------
    //
    // The parser batches contiguous OSC/DCS data into a single `osc_run` /
    // `dcs_run` call for throughput. The default `Handler` impl fans those out
    // to `osc_byte` / `dcs_byte`, so a per-byte recorder still observes one
    // event per data byte — these tests pin that fan-out equivalence.

    #[test]
    fn osc_high_bytes_pass_through_run() {
        // UTF-8 payload (é = C3 A9) is in the 0xa0..=0xff data set, so it batches
        // together with the surrounding ASCII; the fan-out still yields one byte
        // each, high bytes included.
        assert_eq!(
            Harness::new(b"\x1B]0;\xC3\xA9!\x07"),
            vec![
                Record::Osc,
                Record::OscByte(b'0'),
                Record::OscByte(b';'),
                Record::OscByte(0xC3),
                Record::OscByte(0xA9),
                Record::OscByte(b'!'),
                Record::OscTermination(0x07),
            ],
        );
    }

    #[test]
    fn osc_ignored_control_splits_run() {
        // An ignored C0 (BS = 0x08) inside the body splits the batch and is
        // dropped — no osc_byte for it — while the data on either side survives.
        assert_eq!(
            Harness::new(b"\x1B]0;ab\x08cd\x07"),
            vec![
                Record::Osc,
                Record::OscByte(b'0'),
                Record::OscByte(b';'),
                Record::OscByte(b'a'),
                Record::OscByte(b'b'),
                Record::OscByte(b'c'),
                Record::OscByte(b'd'),
                Record::OscTermination(0x07),
            ],
        );
    }

    #[test]
    fn osc_run_spans_advance_chunks() {
        // Data split across advance calls still reconstructs losslessly.
        let mut h = Harness::default();
        h.feed(b"\x1B]0;ti");
        h.feed(b"tle\x07");
        assert_eq!(
            h.recorder.values,
            vec![
                Record::Osc,
                Record::OscByte(b'0'),
                Record::OscByte(b';'),
                Record::OscByte(b't'),
                Record::OscByte(b'i'),
                Record::OscByte(b't'),
                Record::OscByte(b'l'),
                Record::OscByte(b'e'),
                Record::OscTermination(0x07),
            ],
        );
    }
}
