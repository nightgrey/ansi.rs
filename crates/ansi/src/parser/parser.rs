use super::internals::{InternalParameters, Utf8};
use super::state::*;
use crate::Params;
use arrayvec::ArrayVec;
use maybe::Maybe;

pub trait Handler {
    fn print(&mut self, _char: char) {}

    fn control(&mut self, _byte: u8) {}

    fn esc(&mut self, _intermediates: &[u8], _final_byte: u8) {}

    fn csi(&mut self, _params: &Params, _intermediates: &[u8], _final_char: char) {}

    fn dcs(&mut self, _params: &Params, _intermediates: &[u8], _final_char: char) {}

    fn dcs_byte(&mut self, _byte: u8) {}

    fn dcs_end(&mut self, _byte: u8) {}

    fn osc(&mut self) {}

    fn osc_byte(&mut self, _byte: u8) {}

    fn osc_end(&mut self, _byte: u8) {}

    fn apc(&mut self) {}

    fn apc_byte(&mut self, _byte: u8) {}

    fn apc_end(&mut self, _byte: u8) {}
}

#[derive(Debug, Default)]
pub struct Parser {
    pub state: State,
    pub params: InternalParameters,
    pub intermediates: ArrayVec<u8, 2>,
    pub utf8: Utf8,
}

impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn advanced(handler: &mut dyn Handler, bytes: &[u8]) -> Parser {
        let mut parser = Parser::new();

        parser.advance(handler, bytes);

        parser
    }

    pub fn advance(&mut self, handler: &mut dyn Handler, bytes: &[u8]) -> usize {
        match str::from_utf8(bytes) {
            Ok(s) => eprintln!("{}", s.escape_debug()),
            Err(_e) => eprintln!("{}", String::from_utf8_lossy(bytes)),
        }
        let mut i = 0;

        while i != bytes.len() {
            let byte = bytes[i];
            self.transition(handler, byte);
            i += 1;
        }

        i
    }

    pub fn clear(&mut self) {
        self.params.clear();
        self.intermediates.clear();
        self.utf8.clear()
    }

    pub fn flush(&mut self, handler: &mut dyn Handler) {
        if self.utf8.is_partial() {
            handler.print(char::REPLACEMENT_CHARACTER);
        }
        self.state = State::Ground;
        self.clear();
    }

    #[inline]
    fn transition(&mut self, handler: &mut dyn Handler, byte: u8) {
        pub fn debug_transition(
            byte: u8,
            from: State,
            to: State,
            action: Action,
            exit: Action,
            entry: Action,
        ) {
            // ANSI color codes
            const BOLD: &str = "\x1b[1m";

            const FG_GREY: &str = "\x1b[38;2;150;150;150m";
            const BLUE: &str = "\x1b[34m";
            const GREEN: &str = "\x1b[32m";
            const YELLOW: &str = "\x1b[33m";
            const MAGENTA: &str = "\x1b[35m";
            const RESET: &str = "\x1b[0m";

            let byte_str = format!(
                "{YELLOW}{byte_str:<9}{RESET}",
                byte_str = format!("[{}/{:#0x}]", byte as char, byte)
            );

            let action_str = if action.is_some() {
                format!(" {MAGENTA}@ {action}{RESET}")
            } else {
                String::new()
            };

            if from != to {
                eprintln!(
                    "{byte_str} {BOLD}{BLUE}{to}{RESET}{action_str}{hooks}",
                    hooks = if entry.is_some() || exit.is_some() {
                        format!(
                            " {FG_GREY}+ {}{RESET}",
                            if entry.is_some() && exit.is_some() {
                                format!("{entry}..{exit}")
                            } else if entry.is_some() {
                                format!("{entry}..")
                            } else if exit.is_some() {
                                format!("..{exit}")
                            } else {
                                "..".to_string()
                            }
                        )
                    } else {
                        "".to_string()
                    }
                );
            } else {
                eprintln!("{byte_str} {BLUE}...{RESET}{action_str}");
            }
        }

        let (next_state, action) = self.state.transition(byte);

        debug_transition(
            byte,
            self.state,
            next_state,
            action,
            self.state.exit(),
            next_state.entry(),
        );

        if self.state != next_state {
            self.action(handler, self.state.exit(), byte);
            self.action(handler, action, byte);
            self.action(handler, next_state.entry(), byte);

            self.state = next_state;
        } else {
            self.action(handler, action, byte);
        }
    }

    #[inline(always)]
    fn action(&mut self, handler: &mut dyn Handler, action: Action, byte: u8) {
        match action {
            Action::None | Action::Ignore => {}

            Action::Clear => self.clear(),
            Action::Print => handler.print(byte as char),
            Action::Execute => handler.control(byte),

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
                handler.csi(&self.params, &self.intermediates, byte as char);
            }

            Action::DcsStart => {
                self.params.finish();
                handler.dcs(&self.params, &self.intermediates, byte as char);
            }
            Action::DcsByte => handler.dcs_byte(byte),
            Action::DcsEnd => handler.dcs_end(byte),

            Action::OscStart => handler.osc(),
            Action::OscByte => handler.osc_byte(byte),
            Action::OscEnd => handler.osc_end(byte),

            Action::ApcStart => handler.apc(),
            Action::ApcByte => handler.apc_byte(byte),
            Action::ApcEnd => handler.apc_end(byte),

            Action::FlushInvalid => {
                self.utf8.clear();
                handler.print(char::REPLACEMENT_CHARACTER)
            }
            // Set the bottom continuation byte
            Action::SetUtf8Byte1 => {
                self.utf8.set_byte_1(byte);
                handler.print(self.utf8.as_char());
                self.utf8.clear();
            }
            // Set the 2nd-from-last byte of a two byte sequence
            Action::SetUtf8Byte2Top => {
                self.utf8.set_byte_2_top(byte);
            }
            // Set the 2nd-from-last continuation byte
            Action::SetUtf8Byte2 => {
                self.utf8.set_byte_2(byte);
            }
            // Set the 3rd-from-last byte of a three byte sequence
            Action::SetUtf8Byte3Top => {
                self.utf8.set_byte_3_top(byte);
            }
            // Set the 3rd-from-last continuation byte
            Action::SetUtf8Byte3 => {
                self.utf8.set_byte_3(byte);
            }
            // Set the 4th-from-last continuation byte of a four byte sequence
            Action::SetUtf8Byte4Top => {
                self.utf8.set_byte_4_top(byte);
            }
        }
    }
}

/*
 // if self.state == State::Ground {
            //     let end = memspan::skip_ascii_graphic(&bytes[i..]);
            //
            //     if end > 0 {
            //         handler.print(&bytes[i..i + end]);
            //         i += end;
            //
            //         if i >= bytes.len() {
            //             break;
            //         }
            //     }
            // }
            //
            // if self.state == State::CsiIgnore {
            //     let end = skip_csi_ignore(&bytes[i..]);
            //
            //     if end > 0 {
            //         i += end;
            //
            //         if i >= bytes.len() {
            //             break;
            //         }
            //     }
            // }

*/

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Intermediates;
    use crate::parser::tests::{Record, Recorder};
    use crate::{Parameters, assert_parse, params};

    mod utf8 {
        use super::*;

        #[test]
        fn prints_utf8_multibyte() {
            let events = Recorder::record("aé東🦀b");
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
                Recorder::record([b'a', 0xA0, b'b']),
                vec![
                    Record::Print('a'),
                    Record::Print('\u{FFFD}'),
                    Record::Print('b'),
                ],
            );
        }

        #[test]
        fn partial_utf8_buffered_across_calls() {
            let mut parser = Parser::new();
            let mut recorder = Recorder::new();
            // 🦀 = F0 9F A6 80 (4 bytes). Split 2 / 2.
            parser.advance(&mut recorder, &[0xF0, 0x9F]);
            assert!(recorder.is_empty(), "no output yet");
            parser.advance(&mut recorder, &[0xA6, 0x80]);
            assert_eq!(recorder, vec![Record::Print('🦀')]);
        }

        #[test]
        fn partial_utf8_followed_by_esc_emits_replacement() {
            // Partial 2-byte sequence (0xC3) cut off by ESC.
            assert_eq!(
                Recorder::record(b"a\xC3\x1B[m"),
                vec![
                    Record::Print('a'),
                    Record::Print('\u{FFFD}'),
                    Record::Csi(Parameters::new(), Intermediates::empty(), 'm'),
                ],
            );
        }

        #[test]
        fn two_byte_utf8() {
            assert_parse!("é", [Record::Print('é')]);
            assert_eq!(str::from_utf8(&[195, 169]), Ok("é"));
        }

        #[test]
        fn three_byte_utf8() {
            assert_parse!("東", [Record::Print('東')]);
            assert_eq!(str::from_utf8(&[230, 157, 177]), Ok("東"));
        }

        #[test]
        fn four_byte_utf8() {
            assert_parse!("🦀", [Record::Print('🦀')]);
            assert_eq!(str::from_utf8(&[240, 159, 166, 128]), Ok("🦀"));
        }

        #[test]
        fn advances_partial() {
            let mut parser = Parser::new();
            let mut recorder = Recorder::new();

            parser.advance(&mut recorder, &[0xF0, 0x9F]);
            assert!(recorder.is_empty());
            parser.advance(&mut recorder, &[0xA6, 0x80]);

            assert_eq!(recorder, [Record::Print('🦀')]);

            parser.clear();
            recorder.clear();
            // 東 = E6 9D B1 (3 bytes). Split after the first byte: the partial
            // codepoint is buffered, then completed on the following call.

            parser.advance(&mut recorder, &[b'a', 0xE6]);
            assert_eq!(recorder, vec![Record::Print('a')]);
            parser.advance(&mut recorder, &[0x9D, 0xB1, b'b']);
            assert_eq!(
                recorder,
                [Record::Print('a'), Record::Print('東'), Record::Print('b')],
            );
        }

        #[test]
        fn escape_prints_replacement_when_in_partial_unicode() {
            let mut parser = Parser::new();
            let mut recorder = Recorder::new();

            parser.advance(&mut recorder, b"a\xC3");
            parser.advance(&mut recorder, b"\x1B[m");

            assert_eq!(
                recorder,
                [
                    Record::Print('a'),
                    Record::Print(char::REPLACEMENT_CHARACTER),
                    Record::Csi(Parameters::new(), Intermediates::empty(), 'm'),
                ],
            );
        }

        #[test]
        fn lone_continuation_mid_stream_prints_replacement() {
            // 0xBF is a UTF-8 continuation byte with no leader. (Bytes 0x80..=0x9F
            // are C1 controls and dispatch as Execute instead — see the C1 tests.)
            assert_eq!(
                Recorder::record([b'x', 0xBF, b'y']),
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
            assert_parse!(
                &[b'a', 0xC0, 0xAF, b'b'],
                [
                    Record::Print('a'),
                    Record::Print('\u{FFFD}'),
                    Record::Print('\u{FFFD}'),
                    Record::Print('b')
                ]
            );
        }

        #[test]
        fn flush_prints_replacement_for_partial() {
            let mut parser = Parser::new();
            let mut recorder = Recorder::new();
            // First two bytes of 🦀 (F0 9F A6 80) — incomplete.
            parser.advance(&mut recorder, &[0xF0, 0x9F]);
            assert!(recorder.is_empty());
            parser.flush(&mut recorder);
            assert_eq!(recorder, vec![Record::Print('\u{FFFD}')]);

            recorder.clear();
            parser.flush(&mut recorder);
            assert_eq!(recorder, []);
        }

        // ---- OSC / DCS UTF-8 payload (R1) -----------------------------------

        #[test]
        fn osc_payload_preserves_utf8() {
            // OSC 0 ; é ST — the title contains é (C3 A9). High bytes must reach
            // the handler as raw OSC bytes rather than being dropped.
            assert_eq!(
                Recorder::record(b"\x1B]0;\xC3\xA9\x07"),
                vec![
                    Record::OscStart,
                    Record::OscByte(b'0'),
                    Record::OscByte(b';'),
                    Record::OscByte(0xC3),
                    Record::OscByte(0xA9),
                    Record::OscEnd(0x07),
                ],
            );
        }

        #[test]
        fn dcs_payload_preserves_utf8() {
            // DCS q é ESC \ — DCS data with é (C3 A9).
            assert_eq!(
                Recorder::record(b"\x1BPq\xC3\xA9\x1B\\"),
                vec![
                    Record::Dcs(Parameters::new(), Intermediates::empty(), 'q'),
                    Record::DcsByte(0xC3),
                    Record::DcsByte(0xA9),
                    Record::DcsEnd(0x1B),
                    Record::Esc(Intermediates::empty(), b'\\'),
                ],
            );
        }
    }

    mod ground {
        use super::*;
        #[test]
        fn prints_plain_ascii() {
            assert_parse!(
                "abc",
                Record::Print('a'),
                Record::Print('b'),
                Record::Print('c')
            );
        }

        #[test]
        fn executes_c0_controls() {
            // BEL, BS, TAB, LF, CR
            assert_eq!(
                Recorder::record(b"\x07\x08\x09\x0A\x0D"),
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
                Recorder::record(b"a\x07b"),
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
            assert_eq!(Recorder::record(b"\x7f"), vec![Record::Print('\x7f')]);
        }

        #[test]
        fn high_bytes_are_not_raw_c1_controls() {
            let mut parser = Parser::new();
            let mut recorder = Recorder::new();

            parser.advance(&mut recorder, &[b'x', 0x9B, b'1', b'm']);

            assert_eq!(
                recorder,
                [
                    Record::Print('x'),
                    Record::Print(char::REPLACEMENT_CHARACTER),
                    Record::Print('1'),
                    Record::Print('m'),
                ],
            );
        }
    }

    mod esc {
        use super::*;
        #[test]
        fn esc_simple_dispatch() {
            // ESC 7 — DECSC
            assert_eq!(
                Recorder::record(b"\x1B7"),
                vec![Record::Esc(Intermediates::empty(), b'7')],
            );
        }

        #[test]
        fn esc_with_intermediate() {
            // ESC # 8 — DECALN
            assert_eq!(
                Recorder::record(b"\x1B#8"),
                vec![Record::Esc(Intermediates::from(b"#"), b'8')],
            );
        }

        #[test]
        fn esc_with_two_intermediates() {
            // ESC SP # F (uncommon but legal)
            assert_eq!(
                Recorder::record(b"\x1B #F"),
                vec![Record::Esc(Intermediates::from(b" #"), b'F')],
            );
        }

        #[test]
        fn esc_re_entry_aborts_previous() {
            // ESC inside a CSI sequence should abandon the CSI and start fresh.
            assert_eq!(
                Recorder::record(b"\x1B[1;\x1B[2m"),
                vec![Record::Csi(params![[2]], Intermediates::empty(), 'm')],
            );
        }
    }

    mod csi {
        use super::*;
        #[test]
        fn csi_no_params() {
            assert_eq!(
                Recorder::record(b"\x1B[m"),
                vec![Record::Csi(Parameters::new(), Intermediates::empty(), 'm')],
            );
        }

        #[test]
        fn csi_single_param() {
            assert_eq!(
                Recorder::record(b"\x1B[1m"),
                vec![Record::Csi(params![[1]], Intermediates::empty(), 'm')],
            );
        }

        #[test]
        fn csi_multiple_params() {
            assert_eq!(
                Recorder::record(b"\x1B[1;2;3m"),
                vec![Record::Csi(
                    params!([1], [2], [3]),
                    Intermediates::empty(),
                    'm'
                )],
            );
        }

        #[test]
        fn csi_subparams() {
            // 24-bit fg via sub-params: 38:2:255:128:0
            assert_eq!(
                Recorder::record(b"\x1B[38:2:255:128:0m"),
                vec![Record::Csi(
                    params![[38, 2, 255, 128, 0]],
                    Intermediates::empty(),
                    'm'
                )],
            );
        }

        #[test]
        fn csi_mixed_subparams_and_params() {
            assert_eq!(
                Recorder::record(b"\x1B[1;2:3:4;5m"),
                vec![Record::Csi(
                    params![[1], [2, 3, 4], [5]],
                    Intermediates::empty(),
                    'm'
                )],
            );
        }

        #[test]
        fn csi_empty_leading_param_defaults_to_zero() {
            assert_eq!(
                Recorder::record(b"\x1B[;1m"),
                vec![Record::Csi(params![[0], [1]], Intermediates::empty(), 'm')],
            );
        }

        #[test]
        fn csi_empty_subparam_defaults_to_zero() {
            // 38:2::255:128:0 — empty colorspace ID should be 0.
            assert_eq!(
                Recorder::record(b"\x1B[38:2::255:128:0m"),
                vec![Record::Csi(
                    params![[38, 2, 0, 255, 128, 0]],
                    Intermediates::empty(),
                    'm'
                )],
            );
        }

        #[test]
        fn csi_trailing_semicolon_does_not_add_param() {
            assert_eq!(
                Recorder::record(b"\x1B[1;m"),
                vec![Record::Csi(params![[1]], Intermediates::empty(), 'm')],
            );
        }

        #[test]
        fn csi_double_semicolon_inserts_zero() {
            assert_eq!(
                Recorder::record(b"\x1B[1;;2m"),
                vec![Record::Csi(
                    params![[1], [0], [2]],
                    Intermediates::empty(),
                    'm'
                )],
            );
        }

        #[test]
        fn csi_trailing_colon_dispatches_with_zero_subparam() {
            assert_eq!(
                Recorder::record(b"\x1B[1::m"),
                vec![Record::Csi(params![[1, 0, 0]], Intermediates::empty(), 'm')],
            );
        }

        #[test]
        fn csi_clamps_param_to_max() {
            // 99999 saturates at 16383 (ECMA-48 cap).
            assert_eq!(
                Recorder::record(b"\x1B[99999m"),
                vec![Record::Csi(params![[16383]], Intermediates::empty(), 'm')],
            );
        }

        #[test]
        fn csi_private_marker() {
            // DECSET — CSI ? 25 h
            assert_eq!(
                Recorder::record(b"\x1B[?25h"),
                vec![Record::Csi(params![[25]], Intermediates::from(b"?"), 'h')],
            );
        }

        #[test]
        fn csi_intermediate() {
            // CSI SP q — DECSCUSR
            assert_eq!(
                Recorder::record(b"\x1B[2 q"),
                vec![Record::Csi(params![[2]], Intermediates::from(b" "), 'q')],
            );
        }

        #[test]
        fn csi_colon_at_entry_enters_ignore() {
            // `[:1m` — leading `:` enters CsiIgnore; nothing dispatches.
            assert_eq!(Recorder::record(b"\x1B[:1m"), vec![]);
        }

        #[test]
        fn csi_followed_by_text() {
            assert_eq!(
                Recorder::record(b"\x1B[1mhi"),
                vec![
                    Record::Csi(params![[1]], Intermediates::empty(), 'm'),
                    Record::Print('h'),
                    Record::Print('i'),
                ],
            );
        }

        #[test]
        fn csi_can_cancels() {
            // CAN inside a CSI returns to Ground without dispatch.
            assert_eq!(
                Recorder::record(b"\x1B[1;2\x18m"),
                vec![Record::Execute(0x18), Record::Print('m')],
            );
        }

        #[test]
        fn csi_sub_cancels() {
            // SUB inside a CSI returns to Ground without dispatch.
            assert_eq!(
                Recorder::record(b"\x1B[1;2\x1Am"),
                vec![Record::Execute(0x1A), Record::Print('m')],
            );
        }

        #[test]
        fn csi_ignore_consumes_without_text_dispatch() {
            assert_parse!(b"\x1B[:1mX", [Record::Print('X')]);
        }

        #[test]
        fn csi_intermediate_only_soft_reset() {
            // CSI ! p — DECSTR (soft reset): intermediate, no params.
            assert_eq!(
                Recorder::record(b"\x1B[!p"),
                vec![Record::Csi(
                    Parameters::new(),
                    Intermediates::from(b"!"),
                    'p'
                )],
            );
        }

        // ---- CSI edge cases -------------------------------------------------

        #[test]
        fn del_inside_csi_param_is_ignored() {
            // 0x7F inside a CSI param is ignored; the sequence still dispatches.
            assert_eq!(
                Recorder::record(b"\x1B[1;2\x7fm"),
                vec![Record::Csi(params![[1], [2]], Intermediates::empty(), 'm')],
            );
        }

        #[test]
        fn private_public() {
            assert_parse!(
                "\x1B[?25l\x1B[H\x1B[2J\x1B[?25h",
                [Record::Csi(params![[25]], Intermediates::from(b"?"), 'l')]
            );
        }
    }

    /*mod eight_bit {
    use super::*;
        use super::*;
        #[test]
        fn c1_csi_starts_csi() {
            // 0x9B is 8-bit CSI.
            assert_eq!(
                Harness::run([0x9B, b'1', b';', b'2', b'm']),
                vec![Record::Csi(params![[1], [2]], Intermediates::empty(), 'm')],
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
                    Record::Dcs(Parameters::new(), Intermediates::empty(), 'q'),
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
                vec![Record::Esc(Intermediates::empty(), b'\\')]
            );
        }

    }*/

    mod osc {
        use super::*;
        #[test]
        fn osc_st_terminated() {
            // OSC 0 ; title ST  (ST = ESC \)
            assert_eq!(
                Recorder::record(b"\x1B]0;title\x1B\\"),
                vec![
                    Record::OscStart,
                    Record::OscByte(b'0'),
                    Record::OscByte(b';'),
                    Record::OscByte(b't'),
                    Record::OscByte(b'i'),
                    Record::OscByte(b't'),
                    Record::OscByte(b'l'),
                    Record::OscByte(b'e'),
                    Record::OscEnd(0x1B),
                    Record::Esc(Intermediates::empty(), b'\\'),
                ],
            );
        }

        #[test]
        fn osc_bel_terminated() {
            // OSC 0 ; title BEL — xterm convention.
            assert_eq!(
                Recorder::record(b"\x1B]0;hi\x07"),
                vec![
                    Record::OscStart,
                    Record::OscByte(b'0'),
                    Record::OscByte(b';'),
                    Record::OscByte(b'h'),
                    Record::OscByte(b'i'),
                    Record::OscEnd(0x07),
                ],
            );
        }

        #[test]
        fn osc_can_terminated() {
            // CAN inside OSC terminates the string and executes the CAN, returning
            // to ground.
            assert_eq!(
                Recorder::record(b"\x1B]0;hi\x18"),
                vec![
                    Record::OscStart,
                    Record::OscByte(b'0'),
                    Record::OscByte(b';'),
                    Record::OscByte(b'h'),
                    Record::OscByte(b'i'),
                    Record::OscEnd(0x18),
                    Record::Execute(0x18),
                ],
            );
        }

        #[test]
        fn osc_empty() {
            assert_eq!(
                Recorder::record(b"\x1B]\x07"),
                vec![Record::OscStart, Record::OscEnd(0x07),],
            );
        }

        #[test]
        fn osc_ignored_control_splits_run() {
            // An ignored C0 (BS = 0x08) inside the body splits the batch and is
            // dropped — no osc_byte for it — while the data on either side survives.
            assert_eq!(
                Recorder::record(b"\x1B]0;ab\x08cd\x07"),
                vec![
                    Record::OscStart,
                    Record::OscByte(b'0'),
                    Record::OscByte(b';'),
                    Record::OscByte(b'a'),
                    Record::OscByte(b'b'),
                    Record::OscByte(b'c'),
                    Record::OscByte(b'd'),
                    Record::OscEnd(0x07),
                ],
            );
        }

        #[test]
        fn osc_run_spans_advance_chunks() {
            // Data split across advance calls still reconstructs losslessly.
            let mut parser = Parser::new();
            let mut recorder = Recorder::new();
            parser.advance(&mut recorder, b"\x1B]0;ti");
            parser.advance(&mut recorder, b"tle\x07");
            assert_eq!(
                recorder,
                vec![
                    Record::OscStart,
                    Record::OscByte(b'0'),
                    Record::OscByte(b';'),
                    Record::OscByte(b't'),
                    Record::OscByte(b'i'),
                    Record::OscByte(b't'),
                    Record::OscByte(b'l'),
                    Record::OscByte(b'e'),
                    Record::OscEnd(0x07),
                ],
            );
        }

        #[test]
        fn osc_payloads_preserve_high_bytes() {
            let mut parser = Parser::new();
            let mut recorder = Recorder::new();

            parser.advance(&mut recorder, b"\x1B]0;\xC3\xA9\x07");
            parser.advance(&mut recorder, b"\x1BPq\xC3\xA9\x1B\\");

            assert_eq!(
                recorder,
                [
                    Record::OscStart,
                    Record::OscByte(b'0'),
                    Record::OscByte(b';'),
                    Record::OscByte(0xC3),
                    Record::OscByte(0xA9),
                    Record::OscEnd(0x07),
                    Record::Dcs(Parameters::new(), Intermediates::empty(), 'q'),
                    Record::DcsByte(0xC3),
                    Record::DcsByte(0xA9),
                    Record::DcsEnd(0x1B),
                ],
            );
        }
    }

    // ---- DCS ------------------------------------------------------------
    mod dcs {
        use super::*;
        #[test]
        fn dcs_basic() {
            // DCS $ q   <data>   ESC \
            assert_eq!(
                Recorder::record(b"\x1BP$q q\x1B\\"),
                vec![
                    Record::Dcs(Parameters::new(), Intermediates::from(b"$"), 'q'),
                    Record::DcsByte(b' '),
                    Record::DcsByte(b'q'),
                    Record::DcsEnd(0x1B),
                    Record::Esc(Intermediates::empty(), b'\\'),
                ],
            );
        }

        #[test]
        fn dcs_with_params() {
            assert_eq!(
                Recorder::record(b"\x1BP1;2|data\x1B\\"),
                vec![
                    Record::Dcs(params![[1], [2]], Intermediates::empty(), '|'),
                    Record::DcsByte(b'd'),
                    Record::DcsByte(b'a'),
                    Record::DcsByte(b't'),
                    Record::DcsByte(b'a'),
                    Record::DcsEnd(0x1B),
                    Record::Esc(Intermediates::empty(), b'\\'),
                ],
            );
        }

        #[test]
        fn dcs_with_subparams() {
            assert_eq!(
                Recorder::record(b"\x1BP1:2|x\x1B\\"),
                vec![
                    Record::Dcs(params![[1, 2]], Intermediates::empty(), '|'),
                    Record::DcsByte(b'x'),
                    Record::DcsEnd(0x1B),
                    Record::Esc(Intermediates::empty(), b'\\'),
                ],
            );
        }

        #[test]
        fn dcs_can_cancels() {
            // CAN aborts the DCS without termination event.
            assert_eq!(
                Recorder::record(b"\x1BPq abc\x18tail"),
                vec![
                    Record::Dcs(Parameters::new(), Intermediates::empty(), 'q'),
                    Record::DcsByte(b' '),
                    Record::DcsByte(b'a'),
                    Record::DcsByte(b'b'),
                    Record::DcsByte(b'c'),
                    Record::DcsEnd(0x18),
                    Record::Execute(0x18),
                    Record::Print('t'),
                    Record::Print('a'),
                    Record::Print('i'),
                    Record::Print('l'),
                ],
            );
        }
    }

    mod incremental {
        use super::*;
        #[test]
        fn csi_split_across_advance_calls() {
            let mut parser = Parser::new();
            let mut recorder = Recorder::new();
            parser.advance(&mut recorder, b"\x1B[");
            parser.advance(&mut recorder, b"38;5;");
            parser.advance(&mut recorder, b"196m");
            assert_eq!(
                recorder,
                vec![Record::Csi(
                    params![[38], [5], [196]],
                    Intermediates::empty(),
                    'm'
                )]
            );
        }

        #[test]
        fn osc_split_across_advance_calls() {
            let mut parser = Parser::new();
            let mut recorder = Recorder::new();
            parser.advance(&mut recorder, b"\x1B]0;ti");
            parser.advance(&mut recorder, b"tle\x07");
            assert_eq!(
                recorder,
                vec![
                    Record::OscStart,
                    Record::OscByte(b'0'),
                    Record::OscByte(b';'),
                    Record::OscByte(b't'),
                    Record::OscByte(b'i'),
                    Record::OscByte(b't'),
                    Record::OscByte(b'l'),
                    Record::OscByte(b'e'),
                    Record::OscEnd(0x07),
                ]
            );
        }

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
                let whole = Recorder::record(input);
                for at in 0..=input.len() {
                    let mut parser = Parser::new();
                    let mut recorder = Recorder::new();
                    parser.advance(&mut recorder, &input[..at]);
                    parser.advance(&mut recorder, &input[at..]);
                    assert_eq!(
                        recorder, whole,
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
            let whole = Recorder::record(input);
            for i in 0..=input.len() {
                for j in i..=input.len() {
                    let mut parser = Parser::new();
                    let mut recorder = Recorder::new();
                    parser.advance(&mut recorder, &input[..i]);
                    parser.advance(&mut recorder, &input[i..j]);
                    parser.advance(&mut recorder, &input[j..]);
                    assert_eq!(
                        recorder, whole,
                        "splits at {i},{j} diverged from whole-feed",
                    );
                }
            }
        }
    }

    mod overflow {
        use super::*;
        #[test]
        fn param_overflow_does_not_panic() {
            // A pathologically long parameter list must not panic. It dispatches
            // with the trailing params dropped once capacity is reached.
            let mut seq = Vec::from(b"\x1B[".as_slice());
            for _ in 0..40 {
                seq.extend_from_slice(b"1;");
            }
            seq.push(b'm');

            let events = Recorder::record(seq);
            // Exactly one CSI dispatch, with the trailing params dropped once the
            // builder hits capacity. `NestedRaw<_, 32, 32>` reserves one `starts`
            // slot as a sentinel, so the group cap is 31.
            assert_eq!(events.len(), 1);
            match &events[0] {
                Record::Csi(params, i, c) => {
                    assert_eq!(*c, 'm');
                    assert!(i.is_empty());
                    assert_eq!(params.len(), 31);
                }
                other => panic!("expected a single Csi dispatch, got {other:?}"),
            }
        }

        #[test]
        fn many_param_sgr_within_capacity() {
            assert_eq!(
                Recorder::record(b"\x1B[1;2;3;4;5;6;7;8;9;10m"),
                vec![Record::Csi(
                    params!([1], [2], [3], [4], [5], [6], [7], [8], [9], [10]),
                    Intermediates::empty(),
                    'm'
                )],
            );
        }
    }

    #[test]
    fn sgr_emoji_reset_combination() {
        let events = Recorder::record("\x1B[1;2;3m👨🏿\x1B[0m".as_bytes());
        assert_eq!(
            events,
            vec![
                Record::Csi(params![[1], [2], [3]], Intermediates::empty(), 'm'),
                Record::Print('\u{1F468}'),
                Record::Print('\u{1F3FF}'),
                Record::Csi(params![[0]], Intermediates::empty(), 'm'),
            ],
        );
    }

    mod flush {
        use super::*;
        #[test]
        fn flush_on_clean_boundary_emits_nothing() {
            let mut parser = Parser::new();
            let mut recorder = Recorder::new();
            parser.advance(&mut recorder, b"ab");
            let before = recorder.len();
            parser.flush(&mut recorder);
            assert_eq!(recorder.len(), before);
        }

        #[test]
        fn flush_resets_to_ground() {
            let mut parser = Parser::new();
            let mut recorder = Recorder::new();
            // Enter an incomplete CSI, then flush: the dangling sequence is
            // discarded and subsequent text parses from ground.
            parser.advance(&mut recorder, b"\x1B[1;2");
            parser.flush(&mut recorder);
            parser.advance(&mut recorder, b"x");
            assert_eq!(recorder, vec![Record::Print('x')]);
        }
    }
}
