use super::*;


pub trait Utf8Handler: Handler {
    fn char(&mut self, char: char) {}
}

#[derive(Debug, Default)]
pub struct Utf8Parser {
    inner: Parser,
    utf8: Utf8,
}

impl Utf8Parser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn advance(&mut self, handler: &mut dyn Utf8Handler, bytes: &[u8]) -> usize {
       self.inner.advance(handler, bytes)
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.utf8.clear()
    }

    pub fn flush(&mut self, handler: &mut dyn Utf8Handler) {
        if self.utf8.is_partial() {
            handler.char(char::REPLACEMENT_CHARACTER);
        }
        self.inner.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Intermediates;
    use crate::parser::tests::{Record, Recorder};
    use crate::{Parameters, assert_utf8_parser};

    impl Utf8Handler for Recorder {
        fn char(&mut self, char: char) {
            self.push(Record::Char(char));
        }
    }
    mod utf8 {
        use super::*;

        #[test]
        fn prints_utf8_multibyte() {
            let events = Recorder::record("aé東🦀b");
            assert_eq!(
                events,
                vec![
                    Record::Char('a'),
                    Record::Char('é'),
                    Record::Char('東'),
                    Record::Char('🦀'),
                    Record::Char('b'),
                ],
            );
        }

        #[test]
        fn invalid_utf8_emits_replacement() {
            // Lone continuation byte 0xA0 is invalid.
            assert_eq!(
                Recorder::record([b'a', 0xA0, b'b']),
                vec![
                    Record::Char('a'),
                    Record::Char('\u{FFFD}'),
                    Record::Char('b'),
                ],
            );
        }

        #[test]
        fn partial_utf8_buffered_across_calls() {
            let mut parser = Utf8Parser::new();
            let mut recorder = Recorder::new();
            // 🦀 = F0 9F A6 80 (4 bytes). Split 2 / 2.
            parser.advance(&mut recorder, &[0xF0, 0x9F]);
            assert!(recorder.is_empty(), "no output yet");
            parser.advance(&mut recorder, &[0xA6, 0x80]);
            assert_eq!(recorder, vec![Record::Char('🦀')]);
        }

        #[test]
        fn partial_utf8_followed_by_esc_emits_replacement() {
            // Partial 2-byte sequence (0xC3) cut off by ESC.
            assert_eq!(
                Recorder::record(b"a\xC3\x1B[m"),
                vec![
                    Record::Char('a'),
                    Record::Char('\u{FFFD}'),
                    Record::Csi(Parameters::new(), Intermediates::empty(), 'm'),
                ],
            );
        }

        #[test]
        fn two_byte_utf8() {
            assert_utf8_parser!("é", [Record::Char('é')]);
            assert_eq!(str::from_utf8(&[195, 169]), Ok("é"));
        }

        #[test]
        fn three_byte_utf8() {
            assert_utf8_parser!("東", [Record::Char('東')]);
            assert_eq!(str::from_utf8(&[230, 157, 177]), Ok("東"));
        }

        #[test]
        fn four_byte_utf8() {
            assert_utf8_parser!("🦀", [Record::Char('🦀')]);
            assert_eq!(str::from_utf8(&[240, 159, 166, 128]), Ok("🦀"));
        }

        #[test]
        fn advances_partial() {
            let mut parser = Utf8Parser::new();
            let mut recorder = Recorder::new();

            parser.advance(&mut recorder, &[0xF0, 0x9F]);
            assert!(recorder.is_empty());
            parser.advance(&mut recorder, &[0xA6, 0x80]);

            assert_eq!(recorder, [Record::Char('🦀')]);

            parser.clear();
            recorder.clear();
            // 東 = E6 9D B1 (3 bytes). Split after the first byte: the partial
            // codepoint is buffered, then completed on the following call.

            parser.advance(&mut recorder, &[b'a', 0xE6]);
            assert_eq!(recorder, vec![Record::Char('a')]);
            parser.advance(&mut recorder, &[0x9D, 0xB1, b'b']);
            assert_eq!(
                recorder,
                [Record::Char('a'), Record::Char('東'), Record::Char('b')],
            );
        }

        #[test]
        fn escape_prints_replacement_when_in_partial_unicode() {
            let mut parser = Utf8Parser::new();
            let mut recorder = Recorder::new();

            parser.advance(&mut recorder, b"a\xC3");
            parser.advance(&mut recorder, b"\x1B[m");

            assert_eq!(
                recorder,
                [
                    Record::Char('a'),
                    Record::Char(char::REPLACEMENT_CHARACTER),
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
                    Record::Char('x'),
                    Record::Char('\u{FFFD}'),
                    Record::Char('y'),
                ],
            );
        }

        #[test]
        fn overlong_encoding_emits_replacement() {
            // 0xC0 0xAF is an overlong (illegal) encoding of '/'. Each invalid byte
            // resolves to a replacement char.
            assert_utf8_parser!(
                &[b'a', 0xC0, 0xAF, b'b'],
                [
                    Record::Char('a'),
                    Record::Char('\u{FFFD}'),
                    Record::Char('\u{FFFD}'),
                    Record::Char('b')
                ]
            );
        }

        #[test]
        fn flush_prints_replacement_for_partial() {
            let mut parser = Utf8Parser::new();
            let mut recorder = Recorder::new();
            // First two bytes of 🦀 (F0 9F A6 80) — incomplete.
            parser.advance(&mut recorder, &[0xF0, 0x9F]);
            assert!(recorder.is_empty());
            parser.flush(&mut recorder);
            assert_eq!(recorder, vec![Record::Char('\u{FFFD}')]);

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

    mod flush {
        use super::*;
        #[test]
        fn flush_on_clean_boundary_emits_nothing() {
            let mut parser = Utf8Parser::new();
            let mut recorder = Recorder::new();
            parser.advance(&mut recorder, b"ab");
            let before = recorder.len();
            parser.flush(&mut recorder);
            assert_eq!(recorder.len(), before);
        }

        #[test]
        fn flush_resets_to_ground() {
            let mut parser = Utf8Parser::new();
            let mut recorder = Recorder::new();
            // Enter an incomplete CSI, then flush: the dangling sequence is
            // discarded and subsequent text parses from ground.
            parser.advance(&mut recorder, b"\x1B[1;2");
            parser.flush(&mut recorder);
            parser.advance(&mut recorder, b"x");
            assert_eq!(recorder, vec![Record::Char('x')]);
        }
    }
}
