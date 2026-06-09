use arrayvec::ArrayVec;
use crate::parser::{ByteString, Handler, ParametersBuilder};
use utils::Nested;
use utils_derive::state_machine;
use bilge::prelude::*;

state_machine! {
    #[bitsize(8)]
    #[derive(Copy, Debug, Default)]
    #[derive_const(Clone, PartialEq, Eq)]
    #[repr(u8)]
    enum Action;

    #[bitsize(8)]
    #[derive(Copy, Debug, Default)]
    #[derive_const(Clone, PartialEq, Eq)]
    #[repr(u8)]
    enum State;

    // Anywhere transitions (from any state)
   _ => {
        0x18       => [Action::Execute, State::Ground],
        0x1a       => [Action::Execute, State::Ground],
        0x80..=0x8f => [Action::Execute, State::Ground],
        0x91..=0x97 => [Action::Execute, State::Ground],
        0x99       => [Action::Execute, State::Ground],
        0x9a       => [Action::Execute, State::Ground],
        0x9c       => State::Ground,
        0x1b       => State::Escape,
        0x98       => State::SosPmApcData,
        0x9e       => State::SosPmApcData,
        0x9f       => State::SosPmApcData,
        0x90       => State::DcsEntry,
        0x9d       => State::OscString,
        0x9b       => State::CsiEntry,
    },

    #[default]
    State::Ground =>  {
        0x00..=0x17 => Action::Execute,
        0x19       => Action::Execute,
        0x1c..=0x1f => Action::Execute,
        0x20..=0x7f => Action::Print,
    },

    // State: Escape
    State::Escape => {
        on_entry => Action::Clear,

        0x00..=0x17 => Action::Execute,
        0x19       => Action::Execute,
        0x1c..=0x1f => Action::Execute,
        0x7f       => Action::Ignore,
        0x20..=0x2f => [Action::Intermediate, State::EscapeIntermediate],
        0x30..=0x4f => [Action::EscDispatch, State::Ground],
        0x51..=0x57 => [Action::EscDispatch, State::Ground],
        0x59       => [Action::EscDispatch, State::Ground],
        0x5a       => [Action::EscDispatch, State::Ground],
        0x5c       => [Action::EscDispatch, State::Ground],
        0x60..=0x7e => [Action::EscDispatch, State::Ground],
        0x5b       => State::CsiEntry,
        0x5d       => State::OscString,
        0x50       => State::DcsEntry,
        0x58       => State::SosPmApcData,
        0x5e       => State::SosPmApcData,
        0x5f       => State::SosPmApcData,
    },

    // State: EscapeIntermediate
    State::EscapeIntermediate => {
        0x00..=0x17 => Action::Execute,
        0x19       => Action::Execute,
        0x1c..=0x1f => Action::Execute,
        0x20..=0x2f => Action::Intermediate,
        0x7f       => Action::Ignore,
        0x30..=0x7e => [Action::EscDispatch, State::Ground],
    },

    // State: CsiEntry
    State::CsiEntry => {
        on_entry => Action::Clear,

        0x00..=0x17 => Action::Execute,
        0x19       => Action::Execute,
        0x1c..=0x1f => Action::Execute,
        0x7f       => Action::Ignore,
        0x20..=0x2f => [Action::Intermediate, State::CsiIntermediate],
        0x3a       => State::CsiIgnore,
        0x30..=0x39 => [Action::Param, State::CsiParam],
        0x3b       => [Action::Param, State::CsiParam],
        0x3c..=0x3f => [Action::Intermediate, State::CsiParam],
        0x40..=0x7e => [Action::CsiDispatch, State::Ground],
    },

    // State: CsiIgnore
    State::CsiIgnore => {
        0x00..=0x17 => Action::Execute,
        0x19       => Action::Execute,
        0x1c..=0x1f => Action::Execute,
        0x20..=0x3f => Action::Ignore,
        0x7f       => Action::Ignore,
        0x40..=0x7e => State::Ground,
    },

    // State: CsiParam
    State::CsiParam => {
        0x00..=0x17 => Action::Execute,
        0x19       => Action::Execute,
        0x1c..=0x1f => Action::Execute,
        0x30..=0x39 => Action::Param,
        0x3b       => Action::Param,
        0x7f       => Action::Ignore,
        0x3a       => Action::Param,
        0x3c..=0x3f => State::CsiIgnore,
        0x20..=0x2f => [Action::Intermediate, State::CsiIntermediate],
        0x40..=0x7e => [Action::CsiDispatch, State::Ground],
    },

    // State: CsiIntermediate
    State::CsiIntermediate => {
        0x00..=0x17 => Action::Execute,
        0x19       => Action::Execute,
        0x1c..=0x1f => Action::Execute,
        0x20..=0x2f => Action::Intermediate,
        0x7f       => Action::Ignore,
        0x30..=0x3f => State::CsiIgnore,
        0x40..=0x7e => [Action::CsiDispatch, State::Ground],
    },

    // State: DcsEntry
    State::DcsEntry => {
        on_entry => Action::Clear,

        0x00..=0x17 => Action::Ignore,
        0x19       => Action::Ignore,
        0x1c..=0x1f => Action::Ignore,
        0x7f       => Action::Ignore,
        0x3a       => State::DcsIgnore,
        0x20..=0x2f => [Action::Intermediate, State::DcsIntermediate],
        0x30..=0x39 => [Action::Param, State::DcsParam],
        0x3b       => [Action::Param, State::DcsParam],
        0x3c..=0x3f => [Action::Intermediate, State::DcsParam],
        0x40..=0x7e => State::DcsData,
    },

    // State: DcsIntermediate
    State::DcsIntermediate => {
        0x00..=0x17 => Action::Ignore,
        0x19       => Action::Ignore,
        0x1c..=0x1f => Action::Ignore,
        0x20..=0x2f => Action::Intermediate,
        0x7f       => Action::Ignore,
        0x30..=0x3f => State::DcsIgnore,
        0x40..=0x7e => State::DcsData,
    },

    // State: DcsIgnore
    State::DcsIgnore => {
        0x00..=0x17 => Action::Ignore,
        0x19       => Action::Ignore,
        0x1c..=0x1f => Action::Ignore,
        0x20..=0x7f => Action::Ignore,
    },

    // State: DcsParam
    State::DcsParam => {

        0x00..=0x17 => Action::Ignore,
        0x19       => Action::Ignore,
        0x1c..=0x1f => Action::Ignore,
        0x30..=0x39 => Action::Param,
        0x3b       => Action::Param,
        0x7f       => Action::Ignore,
        0x3a       => Action::Param,
        0x3c..=0x3f => Action::Param,
        0x20..=0x2f => [Action::Intermediate, State::DcsIntermediate],
        0x40..=0x7e => State::DcsData,
    },

    // State: DcsPassthrough
    State::DcsData => {
        on_entry => Action::DcsStart,
        on_exit  => Action::DcsEnd,

        0x00..=0x17 => Action::DcsPut,
        0x19       => Action::DcsPut,
        0x1c..=0x1f => Action::DcsPut,
        0x20..=0x7e => Action::DcsPut,
        0x7f       => Action::Ignore,
        0xa0..=0xff => Action::DcsPut,
    },

    // State: SosPmApcString
    State::SosPmApcData => {
        0x00..=0x17 => Action::Ignore,
        0x19       => Action::Ignore,
        0x1c..=0x1f => Action::Ignore,
        0x20..=0x7f => Action::Ignore,
    },

    // State: OscString
    State::OscString => {
        on_entry => Action::OscStart,
        on_exit  => Action::OscEnd,

        0x00..=0x17 => Action::Ignore,
        0x19       => Action::Ignore,
        0x1c..=0x1f => Action::Ignore,
        0x20..=0x7f => Action::OscPut,
    },
}

impl const Action {
    pub fn is_some(&self) -> bool {
        *self != Action::None
    }

    pub fn is_none(&self) -> bool {
        *self == Action::None
    }
}

impl const State {

    pub fn is_some(&self) -> bool {
        *self != State::None
    }

    pub fn is_none(&self) -> bool {
        *self == State::None
    }
    #[inline(always)]
    pub fn transition(self, byte: u8) -> (Action, Self) {
        transition(self, byte)
    }

    #[inline(always)]
    pub fn exit(self) -> Action {
        exit_action(self)
    }

    #[inline(always)]
    pub fn entry(self) -> Action {
        entry_action(self)
    }

}
#[derive(Debug, Default)]
pub struct Parser {
    pub state: State,

    pub params: ParametersBuilder,
    pub intermediates: ByteString,

    pub utf8: ArrayVec<u8, 4>,
}

impl Parser {
    fn advance(&mut self, handler: &mut impl Handler, bytes: &[u8]) {
        let mut i = 0;

        while i < bytes.len() {
                self.transition(handler, bytes[i]);
                i += 1;
        }
    }

    /// Signal end of input. Any buffered partial UTF-8 codepoint is resolved as
    /// U+FFFD and the parser is reset to [`crate::parser::State::Ground`]. Incomplete control
    /// sequences (a CSI/OSC/DCS cut off mid-stream) are discarded without
    /// dispatch, matching standard VT behavior. Call this when the producer is
    /// done and any dangling bytes should be flushed rather than held for a
    /// future `advance`.
    pub fn flush(&mut self, handler: &mut impl Handler) {
        if !self.utf8.is_empty() {
            handler.print(char::REPLACEMENT_CHARACTER);
        }
        self.state = State::Ground;
        self.clear();
    }

    /// Reset parameter / intermediate / data buffers.
    pub fn clear(&mut self) {
        self.params.clear();
        self.intermediates.clear();
        self.utf8.clear();
    }


    fn transition(&mut self, handler: &mut impl Handler, byte: u8) {
        let (action, next_state) = self.state.transition(byte);
        let prev_state = self.state;

        if next_state.is_some() {
            let exit_action = prev_state.exit();
            if exit_action.is_some() {
                self.action(handler, exit_action, byte);
            }

            if action.is_some() {
                self.action(handler, action, byte);
            }

            let entry_action = next_state.entry();
            if entry_action.is_some() {
                self.action(handler, entry_action, byte);
            }

            self.state = next_state;
        } else {
            self.action(handler, action, byte);
        }
    }

    fn advance_ground(&mut self, handler: &mut impl Handler, bytes: &[u8]) -> usize {
        0
    }

    fn action(&mut self, handler: &mut impl Handler, action: Action, byte: u8) {
        match action {
            Action::None | Action::Ignore => {}

            Action::Clear => self.clear(),

            Action::Print => handler.print(byte as char),
            Action::Execute => handler.execute(byte),

            Action::Intermediate => self.intermediates.push(byte),

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

}


#[cfg(test)]
mod tests {
    use std::fmt::{Debug, Display};
    use derive_more::{Deref, DerefMut};
    use utils::NestedConstructor;
    use crate::params;
    use crate::parser::{ByteStr, Parameters, Params};
    use super::*;

    fn inter(b: &[u8]) -> ByteString {
        ByteString::from(b)
    }


    #[derive(Clone, PartialEq, Eq)]
    struct AnsiChar(pub char);

    impl Debug for AnsiChar {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{} / 0x{:2x}", self.0, self.0 as u8)
        }
    }
    impl Display for AnsiChar {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{} / 0x{:2x}", self.0, self.0 as u8)
        }
    }
    impl From<char> for AnsiChar {
        fn from(c: char) -> Self {
            Self(c)
        }
    }

    impl From<u8> for AnsiChar {
        fn from(b: u8) -> Self {
            Self(b as char)
        }
    }
    impl From<&char> for AnsiChar {
        fn from(c: &char) -> Self {
            Self(*c)
        }
    }

    impl From<&u8> for AnsiChar {
        fn from(b: &u8) -> Self {
            Self(*b as char)
        }
    }

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
                Record::Print(c) => write!(f, "Print({:?})", AnsiChar::from(c)),
                Record::Execute(b) => write!(f, "Execute({:?})", AnsiChar::from(b)),
                Record::Esc(i, b) => write!(f, "Esc({:?}, {:?})", i, AnsiChar::from(b)),
                Record::Csi(p, i, c) => write!(f, "Csi({:?}, {:?}, {:?})", p, i, AnsiChar::from(c)),
                Record::Dcs(p, i, c) => write!(f, "Dcs({:?}, {:?}, {:?})", p, i, AnsiChar::from(c)),
                Record::DcsByte(b) => write!(f, "DcsByte({:?})", AnsiChar::from(b)),
                Record::DcsTermination(b) => write!(f, "DcsTermination({:?})", AnsiChar::from(b)),
                Record::Osc => write!(f, "Osc"),
                Record::OscByte(b) => write!(f, "OscByte({:?})", AnsiChar::from(b)),
                Record::OscTermination(b) => write!(f, "OscTermination({:?})", AnsiChar::from(b)),
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
        fn advance(&mut self, bytes: impl AsRef<[u8]>) -> &mut Recorder {
            self.inner.advance(&mut self.recorder, bytes.as_ref())  ;
            &mut self.recorder
        }

        fn run(bytes: impl AsRef<[u8]>) -> Vec<Record> {
            let mut h = Harness::default();
            h.advance(bytes);

            dbg!(&h.recorder.values);
            h.recorder.values.clone()
        }
    }

    // ---- Ground / print / execute ---------------------------------------

    #[test]
    fn prints_plain_ascii() {
        assert_eq!(
            Harness::run(b"abc"),
            vec![Record::Print('a'), Record::Print('b'), Record::Print('c')],
        );
    }

    #[test]
    fn executes_c0_controls() {
        // BEL, BS, TAB, LF, CR
        assert_eq!(
            Harness::run(b"\x07\x08\x09\x0A\x0D"),
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
            Harness::run(b"a\x07b"),
            vec![Record::Print('a'), Record::Execute(0x07), Record::Print('b')],
        );
    }

    #[test]
    fn ignores_del_in_ground_via_print_path() {
        // 0x7F is in the printable-fast-path range; it should not be executed
        // since DEL traditionally has no visible glyph but most parsers treat
        // it as printable here. We assert current behavior so regressions are
        // visible.
        assert_eq!(Harness::run(b"\x7f"), vec![Record::Print('\x7f')]);
    }

    #[test]
    fn prints_utf8_multibyte() {
        let events = Harness::run("aé東🦀b".as_bytes());
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
            Harness::run(&[b'a', 0xA0, b'b']),
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
        h.advance(&[0xF0, 0x9F]);
        assert!(h.recorder.values.is_empty(), "no output yet");
        h.advance(&[0xA6, 0x80]);
        assert_eq!(h.recorder.values, vec![Record::Print('🦀')]);
    }

    #[test]
    fn partial_utf8_followed_by_esc_emits_replacement() {
        // Partial 2-byte sequence (0xC3) cut off by ESC.
        assert_eq!(
            Harness::run(b"a\xC3\x1B[m"),
            vec![
                Record::Print('a'),
                Record::Print('\u{FFFD}'),
                Record::Csi(Parameters::new(), inter(b""), 'm'),
            ],
        );
    }

    // ---- ESC ------------------------------------------------------------

    #[test]
    fn esc_simple_dispatch() {
        // ESC 7 — DECSC
        assert_eq!(Harness::run(b"\x1B7"), vec![Record::Esc(inter(b""), b'7')],);
    }

    #[test]
    fn esc_with_intermediate() {
        // ESC # 8 — DECALN
        assert_eq!(Harness::run(b"\x1B#8"), vec![Record::Esc(inter(b"#"), b'8')],);
    }

    #[test]
    fn esc_with_two_intermediates() {
        // ESC SP # F (uncommon but legal)
        assert_eq!(
            Harness::run(b"\x1B #F"),
            vec![Record::Esc(inter(b" #"), b'F')],
        );
    }

    #[test]
    fn esc_re_entry_aborts_previous() {
        // ESC inside a CSI sequence should abandon the CSI and start fresh.
        assert_eq!(
            Harness::run(b"\x1B[1;\x1B[2m"),
            vec![Record::Csi(params![[2]], inter(b""), 'm')],
        );
    }

    // ---- CSI ------------------------------------------------------------

    #[test]
    fn csi_no_params() {
        assert_eq!(
            Harness::run(b"\x1B[m"),
            vec![Record::Csi(Parameters::new(), inter(b""), 'm')],
        );
    }

    #[test]
    fn csi_single_param() {
        assert_eq!(
            Harness::run(b"\x1B[1m"),
            vec![Record::Csi(params![[1]], inter(b""), 'm')],
        );
    }

    #[test]
    fn csi_multiple_params() {
        assert_eq!(
            Harness::run(b"\x1B[1;2;3m"),
            vec![Record::Csi(params!([1], [2], [3]), inter(b""), 'm')],
        );
    }

    #[test]
    fn csi_subparams() {
        // 24-bit fg via sub-params: 38:2:255:128:0
        assert_eq!(
            Harness::run(b"\x1B[38:2:255:128:0m"),
            vec![Record::Csi(params![[38, 2, 255, 128, 0]], inter(b""), 'm')],
        );
    }

    #[test]
    fn csi_mixed_subparams_and_params() {
        assert_eq!(
            Harness::run(b"\x1B[1;2:3:4;5m"),
            vec![Record::Csi(params![[1], [2, 3, 4], [5]], inter(b""), 'm')],
        );
    }

    #[test]
    fn csi_empty_leading_param_defaults_to_zero() {
        assert_eq!(
            Harness::run(b"\x1B[;1m"),
            vec![Record::Csi(params![[0], [1]], inter(b""), 'm')],
        );
    }

    #[test]
    fn csi_empty_subparam_defaults_to_zero() {
        // 38:2::255:128:0 — empty colorspace ID should be 0.
        assert_eq!(
            Harness::run(b"\x1B[38:2::255:128:0m"),
            vec![Record::Csi(
                params![[38, 2, 0, 255, 128, 0]],
                inter(b""),
                'm'
            )],
        );
    }

    #[test]
    fn csi_trailing_semicolon_does_not_add_param() {
        assert_eq!(
            Harness::run(b"\x1B[1;m"),
            vec![Record::Csi(params![[1]], inter(b""), 'm')],
        );
    }

    #[test]
    fn csi_double_semicolon_inserts_zero() {
        assert_eq!(
            Harness::run(b"\x1B[1;;2m"),
            vec![Record::Csi(params![[1], [0], [2]], inter(b""), 'm')],
        );
    }

    #[test]
    fn csi_trailing_colon_dispatches_with_zero_subparam() {
        assert_eq!(
            Harness::run(b"\x1B[1::m"),
            vec![Record::Csi(params![[1, 0, 0]], inter(b""), 'm')],
        );
    }

    #[test]
    fn csi_clamps_param_to_max() {
        // 99999 saturates at 16383 (ECMA-48 cap).
        assert_eq!(
            Harness::run(b"\x1B[99999m"),
            vec![Record::Csi(params![[16383]], inter(b""), 'm')],
        );
    }

    #[test]
    fn csi_private_marker() {
        // DECSET — CSI ? 25 h
        assert_eq!(
            Harness::run(b"\x1B[?25h"),
            vec![Record::Csi(params![[25]], inter(b"?"), 'h')],
        );
    }

    #[test]
    fn csi_intermediate() {
        // CSI SP q — DECSCUSR
        assert_eq!(
            Harness::run(b"\x1B[2 q"),
            vec![Record::Csi(params![[2]], inter(b" "), 'q')],
        );
    }

    #[test]
    fn csi_colon_at_entry_enters_ignore() {
        // `[:1m` — leading `:` enters CsiIgnore; nothing dispatches.
        assert_eq!(Harness::run(b"\x1B[:1m"), vec![]);
    }

    #[test]
    fn csi_followed_by_text() {
        assert_eq!(
            Harness::run(b"\x1B[1mhi"),
            vec![
                Record::Csi(params![[1]], inter(b""), 'm'),
                Record::Print('h'),
                Record::Print('i'),
            ],
        );
    }

    #[test]
    fn csi_can_cancels() {
        // CAN inside a CSI returns to Ground without dispatch.
        assert_eq!(
            Harness::run(b"\x1B[1;2\x18m"),
            vec![Record::Execute(0x18), Record::Print('m')],
        );
    }

    #[test]
    fn csi_sub_cancels() {
        // SUB inside a CSI returns to Ground without dispatch.
        assert_eq!(
            Harness::run(b"\x1B[1;2\x1Am"),
            vec![Record::Execute(0x1A), Record::Print('m')],
        );
    }

    // ---- 8-bit C1 introducers ------------------------------------------

    #[test]
    fn c1_csi_starts_csi() {
        // 0x9B is 8-bit CSI.
        assert_eq!(
            Harness::run([0x9B, b'1', b';', b'2', b'm']),
            vec![Record::Csi(params![[1], [2]], inter(b""), 'm')],
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
                Record::Dcs(Parameters::new(), inter(b""), 'q'),
                Record::DcsByte(b'X'),
                Record::DcsTermination(0x9C),
            ],
        );
    }

    // ---- OSC ------------------------------------------------------------

    #[test]
    fn osc_st_terminated() {
        // OSC 0 ; title ST  (ST = ESC \)
        assert_eq!(
            Harness::run(b"\x1B]0;title\x1B\\"),
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
                Record::Esc(inter(b""), b'\\'),
            ],
        );
    }

    #[test]
    fn osc_bel_terminated() {
        // OSC 0 ; title BEL — xterm convention.
        assert_eq!(
            Harness::run(b"\x1B]0;hi\x07"),
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
            Harness::run(b"\x1B]\x07"),
            vec![Record::Osc, Record::OscTermination(0x07),],
        );
    }

    // ---- DCS ------------------------------------------------------------

    #[test]
    fn dcs_basic() {
        // DCS $ q   <data>   ESC \
        assert_eq!(
            Harness::run(b"\x1BP$q q\x1B\\"),
            vec![
                Record::Dcs(Parameters::new(), inter(b"$"), 'q'),
                Record::DcsByte(b' '),
                Record::DcsByte(b'q'),
                Record::DcsTermination(0x1B),
                Record::Esc(inter(b""), b'\\'),
            ],
        );
    }

    #[test]
    fn dcs_with_params() {
        assert_eq!(
            Harness::run(b"\x1BP1;2|data\x1B\\"),
            vec![
                Record::Dcs(params![[1], [2]], inter(b""), '|'),
                Record::DcsByte(b'd'),
                Record::DcsByte(b'a'),
                Record::DcsByte(b't'),
                Record::DcsByte(b'a'),
                Record::DcsTermination(0x1B),
                Record::Esc(inter(b""), b'\\'),
            ],
        );
    }

    #[test]
    fn dcs_with_subparams() {
        assert_eq!(
            Harness::run(b"\x1BP1:2|x\x1B\\"),
            vec![
                Record::Dcs(params![[1, 2]], inter(b""), '|'),
                Record::DcsByte(b'x'),
                Record::DcsTermination(0x1B),
                Record::Esc(inter(b""), b'\\'),
            ],
        );
    }

    #[test]
    fn dcs_can_cancels() {
        // CAN aborts the DCS without termination event.
        assert_eq!(
            Harness::run(b"\x1BPq abc\x18tail"),
            vec![
                Record::Dcs(Parameters::new(), inter(b""), 'q'),
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
            Harness::run(b"\x1B^junk\x1B\\"),
            vec![Record::Esc(inter(b""), b'\\')],
        );
    }

    #[test]
    fn apc_string_is_silently_consumed() {
        assert_eq!(
            Harness::run(b"\x1B_x\x1B\\"),
            vec![Record::Esc(inter(b""), b'\\')],
        );
    }

    // ---- Streaming / chunked input -------------------------------------

    #[test]
    fn csi_split_across_advance_calls() {
        let mut h = Harness::default();
        h.advance(b"\x1B[");
        h.advance(b"38;5;");
        h.advance(b"196m");
        assert_eq!(
            h.recorder.values,
            vec![Record::Csi(params![[38], [5], [196]], inter(b""), 'm')]
        );
    }

    #[test]
    fn osc_split_across_advance_calls() {
        let mut h = Harness::default();
        h.advance(b"\x1B]0;ti");
        h.advance(b"tle\x07");
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
        let events = Harness::run("\x1B[1;2;3m👨🏿\x1B[0m".as_bytes());
        assert_eq!(
            events,
            vec![
                Record::Csi(params![[1], [2], [3]], inter(b""), 'm'),
                // 👨🏿 = man + dark skin tone modifier (two codepoints).
                Record::Print('\u{1F468}'),
                Record::Print('\u{1F3FF}'),
                Record::Csi(params![[0]], inter(b""), 'm'),
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

        let events = Harness::run(seq);
        // Exactly one CSI dispatch, with the trailing params dropped once the
        // builder hits capacity. `NestedRaw<_, 32, 32>` reserves one `starts`
        // slot as a sentinel, so the group cap is 31.
        assert_eq!(events.len(), 1);
        match &events[0] {
            Record::Csi(params, i, c) => {
                assert_eq!(*c, 'm');
                assert_eq!(i, &inter(b""));
                assert_eq!(params.len(), 31);
            }
            other => panic!("expected a single Csi dispatch, got {other:?}"),
        }
    }

    #[test]
    fn many_param_sgr_within_capacity() {
        assert_eq!(
            Harness::run(b"\x1B[1;2;3;4;5;6;7;8;9;10m"),
            vec![Record::Csi(
                params!([1], [2], [3], [4], [5], [6], [7], [8], [9], [10]),
                inter(b""),
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
            Harness::run(b"\x1BXjunk\x1B\\"),
            vec![Record::Esc(inter(b""), b'\\')],
        );
    }

    #[test]
    fn c1_sos_string_is_silently_consumed() {
        // 8-bit SOS introducer 0x98.
        let mut seq = vec![0x98];
        seq.extend_from_slice(b"junk\x1B\\");
        assert_eq!(Harness::run(seq), vec![Record::Esc(inter(b""), b'\\')]);
    }

    #[test]
    fn standalone_st_dispatches_as_esc() {
        // ESC \ in ground is just an ESC dispatch.
        assert_eq!(
            Harness::run(b"\x1B\\"),
            vec![Record::Esc(inter(b""), b'\\')],
        );
    }

    // ---- OSC cancel -----------------------------------------------------

    #[test]
    fn osc_can_cancels() {
        // CAN inside OSC terminates the string and executes the CAN, returning
        // to ground.
        assert_eq!(
            Harness::run(b"\x1B]0;hi\x18"),
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
            Harness::run(&[b'x', 0xBF, b'y']),
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
            Harness::run(&[b'a', 0xC0, 0xAF, b'b']),
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
        h.advance(&[b'a', 0xE6]);
        assert_eq!(h.recorder.values, vec![Record::Print('a')]);
        h.advance(&[0x9D, 0xB1, b'b']);
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
            Harness::run(b"\x1B[1;2\x7fm"),
            vec![Record::Csi(params![[1], [2]], inter(b""), 'm')],
        );
    }

    #[test]
    fn csi_intermediate_only_soft_reset() {
        // CSI ! p — DECSTR (soft reset): intermediate, no params.
        assert_eq!(
            Harness::run(b"\x1B[!p"),
            vec![Record::Csi(Parameters::new(), inter(b"!"), 'p')],
        );
    }

    // ---- OSC / DCS UTF-8 payload (R1) -----------------------------------

    #[test]
    fn osc_payload_preserves_utf8() {
        // OSC 0 ; é ST — the title contains é (C3 A9). High bytes must reach
        // the handler as raw OSC bytes rather than being dropped.
        assert_eq!(
            Harness::run(b"\x1B]0;\xC3\xA9\x07"),
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
            Harness::run(b"\x1BPq\xC3\xA9\x1B\\"),
            vec![
                Record::Dcs(Parameters::new(), inter(b""), 'q'),
                Record::DcsByte(0xC3),
                Record::DcsByte(0xA9),
                Record::DcsTermination(0x1B),
                Record::Esc(inter(b""), b'\\'),
            ],
        );
    }

    // ---- flush (R2) -----------------------------------------------------

    #[test]
    fn flush_emits_replacement_for_partial_utf8() {
        let mut h = Harness::default();
        // First two bytes of 🦀 (F0 9F A6 80) — incomplete.
        h.advance(&[0xF0, 0x9F]);
        assert!(h.recorder.values.is_empty());
        h.inner.flush(&mut h.recorder);
        assert_eq!(h.recorder.values, vec![Record::Print('\u{FFFD}')]);
    }

    #[test]
    fn flush_on_clean_boundary_emits_nothing() {
        let mut h = Harness::default();
        h.advance(b"ab");
        let before = h.recorder.values.len();
        h.inner.flush(&mut h.recorder);
        assert_eq!(h.recorder.values.len(), before);
    }

    #[test]
    fn flush_resets_to_ground() {
        let mut h = Harness::default();
        // Enter an incomplete CSI, then flush: the dangling sequence is
        // discarded and subsequent text parses from ground.
        h.advance(b"\x1B[1;2");
        h.inner.flush(&mut h.recorder);
        h.advance(b"x");
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
            &[b'a', 0xA0, b'b'],             // lone continuation
            &[b'a', 0xC0, 0xAF, b'b'],       // overlong encoding
            &[b'x', 0x9B, b'1', b'm'],       // raw 8-bit C1 CSI
            "tail 🦀".as_bytes(),            // multibyte at the very end
        ];

        for input in corpus {
            let whole = Harness::run(input);
            for at in 0..=input.len() {
                let mut h = Harness::default();
                h.advance(&input[..at]);
                h.advance(&input[at..]);
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
        let whole = Harness::run(input);
        for i in 0..=input.len() {
            for j in i..=input.len() {
                let mut h = Harness::default();
                h.advance(&input[..i]);
                h.advance(&input[i..j]);
                h.advance(&input[j..]);
                assert_eq!(
                    h.recorder.values, whole,
                    "splits at {i},{j} diverged from whole-feed",
                );
            }
        }
    }

    // ---- Batched data-string dispatch (osc_string / dcs_string) ---------

    /// Records the slices handed to the batched data handlers, overriding the
    /// per-byte defaults so we can assert that batching actually happens (the
    /// `Recorder` above only exercises the byte-by-byte fallback).
    #[derive(Default)]
    struct StringBatches {
        osc: Vec<Vec<u8>>,
        dcs: Vec<Vec<u8>>,
    }

    impl Handler for StringBatches {
        fn osc_string(&mut self, bytes: &[u8]) {
            self.osc.push(bytes.to_vec());
        }
        fn dcs_string(&mut self, bytes: &[u8]) {
            self.dcs.push(bytes.to_vec());
        }
    }

    fn batches(bytes: impl AsRef<[u8]>) -> StringBatches {
        let mut parser = crate::parser::Parser::default();
        let mut h = StringBatches::default();
        parser.advance(&mut h, bytes);
        h
    }

    #[test]
    fn osc_data_dispatched_as_one_slice() {
        let h = batches(b"\x1B]0;title\x1B\\");
        assert_eq!(h.osc, vec![b"0;title".to_vec()]);
        assert!(h.dcs.is_empty());
    }

    #[test]
    fn dcs_data_dispatched_as_one_slice() {
        let h = batches(b"\x1BP1;2|data\x1B\\");
        assert_eq!(h.dcs, vec![b"data".to_vec()]);
        assert!(h.osc.is_empty());
    }

    #[test]
    fn osc_high_bytes_batched_with_ascii() {
        // UTF-8 payload (é = C3 A9) is in the 0xa0..=0xff data set, so it batches
        // together with the surrounding ASCII rather than splitting the run.
        let h = batches(b"\x1B]0;\xC3\xA9!\x07");
        assert_eq!(h.osc, vec![b"0;\xC3\xA9!".to_vec()]);
    }

    #[test]
    fn osc_batch_splits_around_ignored_control() {
        // An ignored C0 (BS = 0x08) inside the body splits the batch and is
        // dropped — exactly what the per-byte path does (it emits no osc_byte).
        let h = batches(b"\x1B]0;ab\x08cd\x07");
        assert_eq!(h.osc, vec![b"0;ab".to_vec(), b"cd".to_vec()]);
    }

    #[test]
    fn osc_string_batches_per_advance_chunk() {
        // Chunked input yields one slice per chunk; concatenation matches the
        // whole-feed body, so a buffering handler reconstructs it losslessly.
        let mut parser = crate::parser::Parser::default();
        let mut h = StringBatches::default();
        parser.advance(&mut h, b"\x1B]0;ti");
        parser.advance(&mut h, b"tle\x07");
        assert_eq!(h.osc, vec![b"0;ti".to_vec(), b"tle".to_vec()]);
    }

}