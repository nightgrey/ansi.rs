use std::char::EscapeDebug;
use super::*;

#[derive(Debug, Default)]
pub struct Parser {
    pub state: State,
    pub params: ParametersAccumulator,
    pub intermediates: IntermediatesAccumulator,
}

impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn advance(&mut self, handler: &mut dyn Handler, bytes: &[u8]) -> usize {
        // debug_advance(bytes);
        let mut i = 0;

        while i < bytes.len() {
            match self.state {
                State::Ground => {
                    memspan::skip_class! {
                        pub fn skip_ascii_graphic_and_utf8(
                            ranges = [0x21..=0xFF],
                        );
                    }
                    let skipped = skip_ascii_graphic_and_utf8(&bytes[i..]);

                    if skipped > 0 {
                        let start = i;
                        i += skipped;
                        // debug_print(&bytes[start..i], skipped);
                        handler.print(&bytes[start..i]);
                    }

                    if i >= bytes.len() {
                        break;
                    }

                    i += self.advance_byte(handler, bytes[i]);
                }
                State::OscData => {
                    memspan::skip_class! {
                        pub fn skip_osc_data(
                            ranges = [0x20..=0xFF],
                        );
                    }
                    let batched = skip_osc_data(&bytes[i..]);

                    if batched > 0 {
                        let start = i;
                        i += batched;
                        handler.osc_data(&bytes[start..i]);
                    }

                    if i >= bytes.len() {
                        break;
                    }

                    i += self.advance_byte(handler, bytes[i]);
                }
                State::DcsData => {
                    memspan::skip_class! {
                        pub fn skip_dcs_data(
                            ranges = [0x20..=0x7E, 0x80u8..=0xFFu8],
                        );
                    }
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
                }
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

        // debug_transition(
        //     byte,
        //     self.state,
        //     next_state,
        //     action,
        //     self.state.exit(),
        //     next_state.entry(),
        // );

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
                self.params.flush();
                handler.csi(self.params.as_ref(), &self.intermediates, byte as char);
            }

            Action::Dcs => {
                self.params.flush();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::tests::{Record, Recorder};
    use crate::{Intermediates, params};

    // ---- OSC ------------------------------------------------------------
    mod osc {
        use vte::{Params, Perform};
        use super::*;
        use crate::assert_parser;

        /// vte: `parse_osc`. Payload is emitted as a single verbatim run (no
        /// `;`-splitting into params).
        #[test]
        fn parse_osc() {
            assert_eq!(
                Recorder::record(b"\x1b]2;jwilm@jwilm-desk: ~/code/alacritty\x07"),
                vec![
                    Record::Osc,
                    Record::OscData(b"2;jwilm@jwilm-desk: ~/code/alacritty".to_vec()),
                    Record::OscEnd(0x07),
                ],
            );
        }

        /// vte: `parse_empty_osc`.
        #[test]
        fn parse_empty_osc() {
            assert_eq!(
                Recorder::record(b"\x1b]\x07"),
                vec![Record::Osc, Record::OscEnd(0x07)],
            );
        }

        /// vte: `osc_bell_terminated`.
        #[test]
        fn osc_bell_terminated() {
            assert_eq!(
                Recorder::record(b"\x1b]11;ff/00/ff\x07"),
                vec![
                    Record::Osc,
                    Record::OscData(b"11;ff/00/ff".to_vec()),
                    Record::OscEnd(0x07),
                ],
            );
        }

        /// vte: `osc_c0_st_terminated`. ST (`ESC \`) closes the OSC (with the
        /// ESC as the terminating byte) and then dispatches as its own `Esc`.
        #[test]
        fn osc_c0_st_terminated() {
            assert_eq!(
                Recorder::record(b"\x1b]11;ff/00/ff\x1b\\"),
                vec![
                    Record::Osc,
                    Record::OscData(b"11;ff/00/ff".to_vec()),
                    Record::OscEnd(0x1b),
                    Record::Esc(Intermediates::empty(), b'\\'),
                ],
            );
        }
        #[test]
        fn parse_osc_with_utf8_arguments() {
            assert_eq!(
                Recorder::record(&[
                    0x0D, 0x1B, 0x5D, 0x32, 0x3B, 0x65, 0x63, 0x68, 0x6F, 0x20, 0x27, 0xC2, 0xAF,
                    0x5C, 0x5F, 0x28, 0xE3, 0x83, 0x84, 0x29, 0x5F, 0x2F, 0xC2, 0xAF, 0x27, 0x20,
                    0x26, 0x26, 0x20, 0x73, 0x6C, 0x65, 0x65, 0x70, 0x20, 0x31, 0x07,
                ]),
                vec![
                    Record::Execute(b'\r'),
                    Record::Osc,
                    Record::OscData(
                        String::from("2;echo '¯\\_(ツ)_/¯' && sleep 1")
                            .as_bytes()
                            .to_vec()
                    ),
                    Record::OscEnd(0x07),
                ],
            );
        }

        #[test]
        fn osc_containing_string_terminator() {
            const INPUT: &[u8] = b"\x1b]2;\xe6\x9c\xab\x1b\\";

            assert_parser!(INPUT, [
                Record::Osc,
                Record::OscData(b"2;\xe6\x9c\xab".to_vec()),
                Record::OscEnd(0x1b),
                Record::Esc(Intermediates::empty(), b'\\'),
            ]);

        }

        /// vte: `parse_osc_max_params`. This parser has no param cap because it
        /// never splits OSC on `;` — the separators survive verbatim in the
        /// payload run.
        #[test]
        fn osc_semicolons_are_verbatim() {
            assert_eq!(
                Recorder::record(b"\x1b];;;;;;;;;;;;;;;;;\x1b\\"),
                vec![
                    Record::Osc,
                    Record::OscData(b";;;;;;;;;;;;;;;;;".to_vec()),
                    Record::OscEnd(0x1b),
                    Record::Esc(Intermediates::empty(), b'\\'),
                ],
            );
        }

        /// vte: `exceed_max_buffer_size`. There is no fixed OSC buffer here, so
        /// arbitrarily large payloads round-trip without truncation.
        #[test]
        fn large_osc_payload_is_not_truncated() {
            const NUM_BYTES: usize = 4096;
            let mut input = Vec::from(b"\x1b]52;".as_slice());
            input.extend(std::iter::repeat(b'a').take(NUM_BYTES));
            input.push(0x07);

            let events = Recorder::record(&input);
            assert_eq!(
                events,
                vec![
                    Record::Osc,
                    Record::OscData({
                        let mut p = Vec::from(b"52;".as_slice());
                        p.extend(std::iter::repeat(b'a').take(NUM_BYTES));
                        p
                    }),
                    Record::OscEnd(0x07),
                ]
            );
        }
    }

    // ---- CSI ------------------------------------------------------------
    mod csi {
        use super::*;

        /// vte: `parse_csi_params_trailing_semicolon`. A trailing `;` yields an
        /// implicit `0` param — matching vte exactly.
        #[test]
        fn trailing_semicolon() {
            assert_eq!(
                Recorder::record(b"\x1b[4;m"),
                vec![Record::Csi(params![[4], [0]], Intermediates::empty(), 'm')],
            );
        }

        /// vte: `parse_csi_params_leading_semicolon`.
        #[test]
        fn leading_semicolon() {
            assert_eq!(
                Recorder::record(b"\x1b[;4m"),
                vec![Record::Csi(params![[0], [4]], Intermediates::empty(), 'm')],
            );
        }

        /// vte: `parse_long_csi_param`. `i64::MAX + 1` saturates at `u16::MAX`.
        #[test]
        fn long_param_saturates() {
            assert_eq!(
                Recorder::record(b"\x1b[9223372036854775808m"),
                vec![Record::Csi(params![[65535]], Intermediates::empty(), 'm')],
            );
        }

        /// vte: `csi_reset`. An embedded ESC abandons the in-flight CSI and
        /// starts a fresh one.
        #[test]
        fn reset_on_embedded_esc() {
            assert_eq!(
                Recorder::record(b"\x1b[3;1\x1b[?1049h"),
                vec![Record::Csi(params![[1049]], Intermediates::from(b"?"), 'h')],
            );
        }

        /// vte: `csi_subparameters`.
        #[test]
        fn subparameters() {
            assert_eq!(
                Recorder::record(b"\x1b[38:2:255:0:255;1m"),
                vec![Record::Csi(
                    params![[38, 2, 255, 0, 255], [1]],
                    Intermediates::empty(),
                    'm',
                )],
            );
        }

        /// vte: `parse_csi_params_ignore_long_params`. There is no `ignore`
        /// flag; an over-long param list still dispatches once, with the
        /// trailing params dropped at capacity (32).
        #[test]
        fn overflow_dispatches_capped() {
            let mut input = Vec::from(b"\x1b[".as_slice());
            for _ in 0..40 {
                input.extend_from_slice(b"1;");
            }
            input.push(b'm');

            let events = Recorder::record(&input);
            assert_eq!(events.len(), 1);
            match &events[0] {
                Record::Csi(params, intermediates, c) => {
                    assert_eq!(*c, 'm');
                    assert!(intermediates.is_empty());
                    assert_eq!(params.len(), 32);
                }
                other => panic!("expected a single Csi dispatch, got {other:?}"),
            }
        }

        /// Params exactly at capacity dispatch losslessly.
        #[test]
        fn params_within_capacity() {
            assert_eq!(
                Recorder::record(b"\x1b[1;2;3;4;5;6;7;8;9;10;11;12;13;14;15;16m"),
                vec![Record::Csi(
                    params!(
                        [1],
                        [2],
                        [3],
                        [4],
                        [5],
                        [6],
                        [7],
                        [8],
                        [9],
                        [10],
                        [11],
                        [12],
                        [13],
                        [14],
                        [15],
                        [16]
                    ),
                    Intermediates::empty(),
                    'm',
                )],
            );
        }

        /// vte: `params_buffer_filled_with_subparam`. A leading `:` sends CSI
        /// straight to the ignore state, so nothing is dispatched (this parser
        /// has no dispatch-with-ignore path).
        #[test]
        fn leading_colon_is_ignored() {
            assert_eq!(Recorder::record(b"\x1b[::::::::x"), vec![]);
        }

        /// vte: `advance_csi_param`, `0x7F => ()`. DEL inside a param is
        /// dropped; the sequence still dispatches.
        #[test]
        fn del_inside_param_is_ignored() {
            assert_eq!(
                Recorder::record(b"\x1b[1;2\x7fm"),
                vec![Record::Csi(params![[1], [2]], Intermediates::empty(), 'm')],
            );
        }

        /// A private-marker CSI (`CSI ? 25 h`).
        #[test]
        fn private_marker() {
            assert_eq!(
                Recorder::record(b"\x1b[?25h"),
                vec![Record::Csi(params![[25]], Intermediates::from(b"?"), 'h')],
            );
        }

        /// An intermediate byte in a CSI (`CSI 2 SP q`).
        #[test]
        fn intermediate() {
            assert_eq!(
                Recorder::record(b"\x1b[2 q"),
                vec![Record::Csi(params![[2]], Intermediates::from(b" "), 'q')],
            );
        }
    }

    // ---- ESC ------------------------------------------------------------
    mod esc {
        use super::*;

        /// vte: `esc_reset`. An in-flight CSI is abandoned by ESC, and the
        /// following ESC-with-intermediate dispatches cleanly.
        #[test]
        fn reset() {
            assert_eq!(
                Recorder::record(b"\x1b[3;1\x1b(A"),
                vec![Record::Esc(Intermediates::from(b"("), b'A')],
            );
        }

        /// vte: `esc_reset_intermediates`. Intermediates from a completed CSI
        /// do not leak into the following ESC.
        #[test]
        fn reset_intermediates() {
            assert_eq!(
                Recorder::record(b"\x1b[?2004l\x1b#8"),
                vec![
                    Record::Csi(params![[2004]], Intermediates::from(b"?"), 'l'),
                    Record::Esc(Intermediates::from(b"#"), b'8'),
                ],
            );
        }
    }

    // ---- DCS ------------------------------------------------------------
    mod dcs {
        use super::*;

        /// vte: `parse_dcs`. `vte`'s per-byte `put` becomes a single batched
        /// `DcsData` run; ST is reported as `DcsEnd` + a following `Esc`.
        /// (C1 ST `0x9C` is 7-bit-only here, so the ST is spelled `ESC \`.)
        #[test]
        fn parse_dcs() {
            assert_eq!(
                Recorder::record(b"\x1bP0;1|17/ab\x1b\\"),
                vec![
                    Record::Dcs(params![[0], [1]], Intermediates::empty(), '|'),
                    Record::DcsData(b"17/ab".to_vec()),
                    Record::DcsEnd(0x1b),
                    Record::Esc(Intermediates::empty(), b'\\'),
                ],
            );
        }

        /// vte: `dcs_reset`. An in-flight CSI is abandoned, then a DCS with an
        /// intermediate runs to completion.
        #[test]
        fn reset() {
            assert_eq!(
                Recorder::record(b"\x1b[3;1\x1bP1$tx\x1b\\"),
                vec![
                    Record::Dcs(params![[1]], Intermediates::from(b"$"), 't'),
                    Record::DcsData(b"x".to_vec()),
                    Record::DcsEnd(0x1b),
                    Record::Esc(Intermediates::empty(), b'\\'),
                ],
            );
        }

        /// vte: `intermediate_reset_on_dcs_exit`. The DCS intermediate (`=`)
        /// does not leak into the trailing ESC.
        #[test]
        fn intermediate_reset_on_exit() {
            assert_eq!(
                Recorder::record(b"\x1bP=1sZZZ\x1b+\x5c"),
                vec![
                    Record::Dcs(params![[1]], Intermediates::from(b"="), 's'),
                    Record::DcsData(b"ZZZ".to_vec()),
                    Record::DcsEnd(0x1b),
                    Record::Esc(Intermediates::from(b"+"), b'\\'),
                ],
            );
        }
    }

    // ---- Controls -------------------------------------------------------
    mod controls {
        use super::*;

        /// vte: `execute_anywhere`. CAN (`0x18`) and SUB (`0x1A`) execute from
        /// any state.
        #[test]
        fn execute_anywhere() {
            assert_eq!(
                Recorder::record(b"\x18\x1a"),
                vec![Record::Execute(0x18), Record::Execute(0x1a)],
            );
        }

        /// vte: `c1s` (C0 portion). C0 controls execute; surrounding text
        /// prints.
        #[test]
        fn c0_controls_execute() {
            assert_eq!(
                Recorder::record(b"\x00\x1fa"),
                vec![
                    Record::Execute(0x00),
                    Record::Execute(0x1f),
                    Record::Print(b"a".to_vec()),
                ],
            );
        }

        /// vte: `c1s` (C1 portion) — divergence. This is a 7-bit parser: a raw
        /// C1 byte (`0x9B`, "8-bit CSI") is not a control here, it is folded
        /// into the surrounding printable run.
        #[test]
        fn raw_c1_bytes_are_printed() {
            let mut parser = Parser::new();
            let mut recorder = Recorder::new();
            parser.advance(&mut recorder, &[b'x', 0x9b, b'1', b'm']);
            assert_eq!(recorder, [Record::Print(b"x\x9b1m".to_vec())]);
        }
    }

    // ---- "Unicode" --------------------------------------------
    mod unicode {
        use crate::assert_parser;
        use super::*;

        #[test]
        fn unicode() {
            const INPUT: &[u8] = b"\xF0\x9F\x8E\x89_\xF0\x9F\xA6\x80\xF0\x9F\xA6\x80_\xF0\x9F\x8E\x89";

            assert_parser!(INPUT, [
                Record::Print(INPUT.to_vec()),
            ]);
        }

        #[test]
        fn invalid_utf8() {
            const INPUT: &[u8] = b"a\xEF\xBCb";

            assert_parser!(INPUT, [
                Record::Print(INPUT.to_vec()),
            ]);
        }

        #[test]
        fn partial_utf8() {
            const INPUT: &[u8] = b"\xF0\x9F\x9A\x80";

            assert_parser!(INPUT, [
                Record::Print(INPUT.to_vec()),
            ]);
        }

        #[test]
        fn partial_utf8_separating_utf8() {
            // This is different from the `partial_utf8` test since it has a multi-byte UTF8
            // character after the partial UTF8 state, causing a partial byte to be present
            // in the `partial_utf8` buffer after the 2-byte codepoint.

            // "ĸ🎉"
            const INPUT: &[u8] = b"\xC4\xB8\xF0\x9F\x8E\x89";

            let mut recorder = Recorder::default();
            let mut parser = Parser::new();

            parser.advance(&mut recorder, &INPUT[..1]);
            parser.advance(&mut recorder, &INPUT[1..]);

            assert_eq!(recorder, [Record::Print(INPUT[..1].to_vec()), Record::Print(INPUT[1..].to_vec())]);
        }
    }


}
