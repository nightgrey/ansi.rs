use super::*;
use arrayvec::ArrayVec;

#[derive(Debug, Default)]
pub struct Parser {
    pub state: State,
    pub params: InternalParameters,
    pub intermediates: ArrayVec<u8, 2>,
}

impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn advance(&mut self, handler: &mut dyn Handler, bytes: &[u8]) -> usize {
        debug_advance(bytes);
        let mut i = 0;


        while i < bytes.len() {
            match self.state {
                State::Ground => {
                    let skipped = skip_ascii_graphic_and_utf8(&bytes[i..]);

                    if skipped > 0 {
                        let start = i;
                        i += skipped;
                        debug_print(&bytes[start..i], skipped);
                        handler.print(&bytes[start..i]);
                    }

                    if i >= bytes.len() {
                        break;
                    }

                    i += self.advance_byte(handler, bytes[i]);
                },
                State::OscData => {
                    let skipped = skip_osc_string(&bytes[i..]);

                    if skipped > 0 {
                        let start = i;
                        i += skipped;
                        handler.osc_data(&bytes[start..i]);
                    }

                    if i >= bytes.len() {
                        break;
                    }

                    i += self.advance_byte(handler, bytes[i]);
                },
                State::DcsData => {
                    let skipped = skip_dcs_data(&bytes[i..]);

                    if skipped > 0 {
                        let start = i;
                        i += skipped;
                        handler.dcs_data(&bytes[start..i]);
                    }

                    if i >= bytes.len() {
                        break;
                    }

                    i += self.advance_byte(handler, bytes[i]);
                },
                _ => i += self.advance_byte(handler, bytes[i]),
            }
        }
        i
    }

    #[inline(always)]
    fn advance_byte(&mut self, handler: &mut dyn Handler, byte: u8) -> usize {
        self.transition(handler, byte);
        1
    }

    pub fn clear(&mut self) {
        self.params.clear();
        self.intermediates.clear();
    }

    pub fn flush(&mut self) {
        self.state = State::Ground;
        self.clear();
    }

    #[inline]
    fn transition(&mut self, handler: &mut dyn Handler, byte: u8) {
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
            Action::Print => handler.print(&[byte]),
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
                handler.csi(self.params.as_ref(), &self.intermediates, byte as char);
            }

            Action::Dcs => {
                self.params.finish();
                handler.dcs(self.params.as_ref(), &self.intermediates, byte as char);
            }
            Action::DcsData => handler.dcs_data(&[byte]),
            Action::DcsEnd => handler.dcs_end(byte),

            Action::Osc => handler.osc(),
            Action::OscData => handler.osc_data(&[byte]),
            Action::OscEnd => handler.osc_end(byte),

            _ => {}

        }
    }
}

/*

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
    use crate::{Parameters, assert_parser, params};

    mod ground {
        use super::*;
        #[test]
        fn prints_plain_ascii() {
            assert_parser!(
                "abc",
                Record::Print(b"abc".to_vec())
            );
        }

        mod vte {
            use vte::{Params, Parser, Perform};

            #[derive(Default)]
            struct Dispatcher {
                dispatched: Vec<Sequence>,
            }

            #[derive(Debug, PartialEq, Eq)]
            enum Sequence {
                Osc(Vec<Vec<u8>>, bool),
                Csi(Vec<Vec<u16>>, Vec<u8>, bool, char),
                Esc(Vec<u8>, bool, u8),
                DcsHook(Vec<Vec<u16>>, Vec<u8>, bool, char),
                DcsPut(u8),
                Print(char),
                Execute(u8),
                DcsUnhook,
            }

            impl Perform for Dispatcher {
                fn osc_dispatch(&mut self, params: &[&[u8]], bell_terminated: bool) {
                    let params = params.iter().map(|p| p.to_vec()).collect();
                    self.dispatched.push(Sequence::Osc(params, bell_terminated));
                }

                fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], ignore: bool, c: char) {
                    let params = params.iter().map(|subparam| subparam.to_vec()).collect();
                    let intermediates = intermediates.to_vec();
                    self.dispatched.push(Sequence::Csi(params, intermediates, ignore, c));
                }

                fn esc_dispatch(&mut self, intermediates: &[u8], ignore: bool, byte: u8) {
                    let intermediates = intermediates.to_vec();
                    self.dispatched.push(Sequence::Esc(intermediates, ignore, byte));
                }

                fn hook(&mut self, params: &Params, intermediates: &[u8], ignore: bool, c: char) {
                    let params = params.iter().map(|subparam| subparam.to_vec()).collect();
                    let intermediates = intermediates.to_vec();
                    self.dispatched.push(Sequence::DcsHook(params, intermediates, ignore, c));
                }

                fn put(&mut self, byte: u8) {
                    self.dispatched.push(Sequence::DcsPut(byte));
                }

                fn unhook(&mut self) {
                    self.dispatched.push(Sequence::DcsUnhook);
                }

                fn print(&mut self, c: char) {
                    self.dispatched.push(Sequence::Print(c));
                }

                fn execute(&mut self, byte: u8) {
                    self.dispatched.push(Sequence::Execute(byte));
                }
            }
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
                    Record::Print(b"a".to_vec()),
                    Record::Execute(0x07),
                    Record::Print(b"b".to_vec()),
                ],
            );
        }

        #[test]
        fn ignores_del_in_ground_via_print_path() {
            // 0x7F is in the printable-fast-path range; it should not be executed
            // since DEL traditionally has no visible glyph but most parsers treat
            // it as printable here. We assert current behavior so regressions are
            // visible.
            assert_eq!(Recorder::record(b"\x7f"), vec![Record::Print(b"\x7f".to_vec())]);
        }

        #[test]
        fn high_bytes_are_not_raw_c1_controls() {
            let mut parser = Parser::new();
            let mut recorder = Recorder::new();

            parser.advance(&mut recorder, &[b'x', 0x9B, b'1', b'm']);

            assert_eq!(
                recorder,
                [
                    Record::Print(b"x\x9B1m".to_vec()),
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
        use crate::{Param, Params};
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
                    Parameters::from([Param::Main(38), Param::Sub(2), Param::Sub(0), Param::Sub(255), Param::Sub(128), Param::Sub(0)]),
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
            // 99999 saturates at 65535 (ECMA-48 cap).
            assert_eq!(
                Recorder::record(b"\x1B[99999m"),
                vec![Record::Csi(params![[65535]], Intermediates::empty(), 'm')],
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
                    Record::Print(b"hi".to_vec()),
                ],
            );
        }

        #[test]
        fn csi_can_cancels() {
            // CAN inside a CSI returns to Ground without dispatch.
            assert_eq!(
                Recorder::record(b"\x1B[1;2\x18m"),
                vec![Record::Execute(0x18), Record::Print(b"m".to_vec())],
            );
        }

        #[test]
        fn csi_sub_cancels() {
            // SUB inside a CSI returns to Ground without dispatch.
            assert_eq!(
                Recorder::record(b"\x1B[1;2\x1Am"),
                vec![Record::Execute(0x1A), Record::Print(b"m".to_vec())],
            );
        }

        #[test]
        fn csi_ignore_consumes_without_text_dispatch() {
            assert_parser!(b"\x1B[:1mX", [Record::Print(b"X".to_vec())]);
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
                    Record::Osc,
                    Record::OscData(b"0;title".to_vec()),
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
                    Record::Osc,
                    Record::OscData(b"0;hi".to_vec()),
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
                    Record::Osc,
                    Record::OscData(b"0;hi".to_vec()),
                    Record::OscEnd(0x18),
                    Record::Execute(0x18),
                ],
            );
        }

        #[test]
        fn osc_empty() {
            assert_eq!(
                Recorder::record(b"\x1B]\x07"),
                vec![Record::Osc, Record::OscEnd(0x07),],
            );
        }

        #[test]
        fn osc_ignored_control_splits_run() {
            // An ignored C0 (BS = 0x08) inside the body splits the batch and is
            // dropped — no osc_byte for it — while the data on either side survives.
            assert_eq!(
                Recorder::record(b"\x1B]0;ab\x08cd\x07"),
                vec![
                    Record::Osc,
                    Record::OscData(b"0;ab".to_vec()),
                    Record::OscData(b"cd".to_vec()),
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
                    Record::Osc,
                    Record::OscData(b"0;ti".to_vec()),
                    Record::OscData(b"tle".to_vec()),
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
                    Record::Osc,
                    Record::OscData(vec![b'0', b';', 0xC3, 0xA9]),
                    Record::OscEnd(0x07),
                    Record::Dcs(Parameters::new(), Intermediates::empty(), 'q'),
                    Record::DcsData(vec![0xC3, 0xA9]),
                    Record::DcsEnd(0x1B),
                    Record::Esc(Intermediates::empty(), b'\\'),
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
                    Record::DcsData(b" q".to_vec()),
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
                    Record::DcsData(b"data".to_vec()),
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
                    Record::DcsData(b"x".to_vec()),
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
                    Record::DcsData(b" abc".to_vec()),
                    Record::DcsEnd(0x18),
                    Record::Execute(0x18),
                    Record::Print(b"tail".to_vec()),
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
                    Record::Osc,
                    Record::OscData(b"0;ti".to_vec()),
                    Record::OscData(b"tle".to_vec()),
                    Record::OscEnd(0x07),
                ]
            );
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
                    assert_eq!(params.len(), 32);
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
                Record::Print("👨🏿".as_bytes().to_vec()),
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
            parser.flush();
            assert_eq!(recorder.len(), before);
        }

        #[test]
        fn flush_resets_to_ground() {
            let mut parser = Parser::new();
            let mut recorder = Recorder::new();
            // Enter an incomplete CSI, then flush: the dangling sequence is
            // discarded and subsequent text parses from ground.
            parser.advance(&mut recorder, b"\x1B[1;2");
            parser.flush();
            parser.advance(&mut recorder, b"x");
            assert_eq!(recorder, vec![Record::Print(b"x".to_vec())]);
        }
    }
}
