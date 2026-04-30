use std::mem;
use arrayvec::ArrayVec;
use derive_more::{Deref, DerefMut};
use utils::NestedRaw;
use super::*;
use super::raw_parameters::*;
pub trait Handler {
    /// Draw a character to the screen and update states.
    fn print(&mut self, byte: char) {}

    /// Execute a C0 or C1 control function.
    fn execute(&mut self, byte: u8) {}

    /// The final character of an escape sequence has arrived.
    fn esc(&mut self, intermediates: &Inter, final_byte: u8) {}

    /// A final character has arrived for a CSI sequence.
    fn csi(
        &mut self,
        params: Params<'_>,
        intermediates: &Inter,
        final_byte: char,
    ) {
    }

    /// Invoked when a final character arrives in first part of device control
    /// string. Subsequent bytes in the control string are delivered via
    /// [`Handler::dcs_byte`], and termination via [`Handler::dcs_termination`].
    fn dcs(&mut self, params: Params<'_>, intermediates: &Inter, final_char: char) {}

    /// A byte of a DCS data string. C0 controls are also passed here.
    fn dcs_byte(&mut self, byte: u8) {}

    /// The DCS data string has been terminated.
    fn dcs_termination(&mut self, byte: u8) {}

    /// Begin an operating system command. Subsequent body bytes are delivered
    /// via [`Handler::osc_byte`]; termination via [`Handler::osc_termination`].
    fn osc(&mut self, params: Params<'_>) {}

    /// A byte of OSC data.
    fn osc_byte(&mut self, byte: u8) {}

    /// The OSC string has been terminated.
    fn osc_termination(&mut self, byte: u8) {}
}

#[derive(Debug, Default)]
pub struct Parser {
    pub state: State,

    pub params: ParametersBuilder,
    pub intermediates: Intermediates,

    pub data: DataString,
    pub utf8: ArrayVec<u8, 4>,
}

impl Parser {
    pub fn advance(&mut self, handler: &mut impl Handler, bytes: impl AsRef<[u8]>) {
        let mut i = 0;
        let bytes = bytes.as_ref();

        // Handle partial codepoints from previous calls to `advance`.
        if !self.utf8.is_empty() {
            i += self.advance_utf8(handler, bytes);
        }

        while i < bytes.len() {
            match self.state {
                State::Ground => i += self.advance_ground(handler, &bytes[i..]),
                _ => {
                    let byte = bytes[i];
                    self.advance_byte(handler, byte);
                    i += 1;
                },
            }
        }
    }

    #[inline]
    fn advance_byte(&mut self, handler: &mut impl Handler, byte: u8) {
        let prev_state = self.state;
            let (action, next_state) = transition(self.state, byte);
        println!("{:?} / 0x{:2x} | {:?} -> {:?} @ {:?}", byte as char, byte, prev_state, if next_state == State::None { prev_state } else { next_state }, action);

        if next_state != State::None {
            let exit_action = exit(prev_state);
            let entry_action = entry(next_state);

            println!("Exit {:?} <-> Entry {:?}", exit_action, entry_action);
            if exit_action != Action::None {
                self.action(handler, exit_action, byte);
            }

            if action != Action::None {
                self.action(handler, action, byte);
            }

            if entry_action != Action::None {
                self.action(handler, entry_action, byte);
            }

            self.state = next_state;
        } else {
            self.action(handler, action, byte);
        }
    }
    /// Continue assembling a partial UTF-8 codepoint from a previous call.
    /// Returns the number of bytes from `bytes` consumed.
    fn advance_utf8(&mut self, handler: &mut impl Handler, bytes: &[u8]) -> usize {
        let old_bytes = self.utf8.len();
        // Try to fill the utf8 buffer (capacity 4) from the new input.
        let to_copy = bytes.len().min(self.utf8.capacity() - old_bytes);
        // SAFETY: bounds enforced by `to_copy` clamp above.
        self.utf8.try_extend_from_slice(&bytes[..to_copy]).expect("utf8 buffer fit");

        match str::from_utf8(&self.utf8) {
            // Buffer is now valid utf8: dispatch the first char and clear.
            Ok(parsed) => {
                let c = unsafe { parsed.chars().next().unwrap_unchecked() };
                handler.print(c);

                let total = c.len_utf8();
                self.utf8.clear();
                total - old_bytes
            },
            Err(err) => {
                let valid_bytes = err.valid_up_to();
                if valid_bytes > 0 {
                    // The buffer contains a complete leading char plus extra
                    // bytes. Dispatch the leading char and treat the extras as
                    // a fresh sequence — but since we can't easily know how
                    // many of `bytes` they belong to, we conservatively just
                    // re-buffer them on the next iteration. The simplest path:
                    // dispatch leading and return the bytes that were taken
                    // from the new input only.
                    let c = unsafe {
                        let parsed = str::from_utf8_unchecked(&self.utf8[..valid_bytes]);
                        parsed.chars().next().unwrap_unchecked()
                    };
                    handler.print(c);

                    let consumed = valid_bytes - old_bytes;
                    self.utf8.clear();
                    return consumed;
                }

                match err.error_len() {
                    // The leading bytes form an invalid sequence — emit
                    // replacement char and skip the bad bytes.
                    Some(invalid_len) => {
                        handler.print('\u{FFFD}');
                        let consumed = invalid_len - old_bytes;
                        self.utf8.clear();
                        consumed
                    },
                    // Still incomplete — keep what we have, return how many
                    // we copied from `bytes`.
                    None => to_copy,
                }
            },
        }
    }

    /// Process bytes while in `Ground` state. Returns the number of bytes
    /// consumed; the caller is responsible for resuming from where this left
    /// off.
    fn advance_ground(&mut self, handler: &mut impl Handler, bytes: &[u8]) -> usize {
        // Find the next byte that needs special handling: ESC (0x1B) only —
        // C0 controls inside the printable run are dispatched as we walk the
        // chars; C1 introducer bytes appear as invalid utf8 and are handled
        // in the error branch below.
        let num_bytes = bytes.len();
        let plain_chars = memchr::memchr(0x1B, bytes).unwrap_or(num_bytes);

        // ESC is the very first byte: short-circuit to Escape state.
        if plain_chars == 0 {
            self.state = State::Escape;
            self.clear();
            return 1;
        }

        match str::from_utf8(&bytes[..plain_chars]) {
            Ok(parsed) => {
                let consumed = Self::dispatch_ground_chars(handler, parsed);

                // If we stopped early on a C1 char encoded as utf8, hand the
                // bytes after `consumed` back to the state machine.
                if consumed < plain_chars {
                    return consumed;
                }

                let mut processed = plain_chars;
                if processed < num_bytes {
                    // Next byte must be ESC — process directly.
                    self.state = State::Escape;
                    self.clear();
                    processed += 1;
                }
                processed
            },
            Err(err) => {
                let valid_bytes = err.valid_up_to();
                let parsed = unsafe { str::from_utf8_unchecked(&bytes[..valid_bytes]) };
                let dispatched = Self::dispatch_ground_chars(handler, parsed);

                // Stopped early inside the valid prefix: bail out so the state
                // machine can take over with the C1 char.
                if dispatched < valid_bytes {
                    return dispatched;
                }

                match err.error_len() {
                    Some(len) => {
                        let bad = bytes[valid_bytes];
                        // Raw 8-bit C1 controls (0x80..=0x9F) are handled by
                        // the state machine via Anywhere transitions — bail
                        // out so the next iteration runs `advance_byte`.
                        if len == 1 && (0x80..=0x9F).contains(&bad) {
                            return valid_bytes;
                        }
                        // Otherwise the bytes are genuinely malformed.
                        handler.print('\u{FFFD}');
                        valid_bytes + len
                    },
                    None => {
                        if plain_chars < num_bytes {
                            // The partial codepoint is followed by ESC — drop
                            // it and start the escape sequence.
                            handler.print('\u{FFFD}');
                            self.state = State::Escape;
                            self.clear();
                            plain_chars + 1
                        } else {
                            // Buffer the partial codepoint for the next call.
                            let extra = num_bytes - valid_bytes;
                            // utf8 buffer has capacity 4; partial UTF-8 is at
                            // most 3 bytes, so this always fits.
                            self.utf8
                                .try_extend_from_slice(&bytes[valid_bytes..valid_bytes + extra])
                                .expect("partial utf8 buffer fit");
                            num_bytes
                        }
                    },
                }
            },
        }
    }

    /// Walk the chars of a validated str, dispatching prints and C0 executes.
    /// Stops at the first C1 control char (0x80..=0x9F) without consuming it,
    /// returning the byte offset where dispatch stopped.
    fn dispatch_ground_chars(handler: &mut impl Handler, parsed: &str) -> usize {
        let mut consumed = 0;
        for c in parsed.chars() {
            match c {
                '\u{80}'..='\u{9F}' => return consumed,
                '\x00'..='\x1F' => handler.execute(c as u8),
                _ => handler.print(c),
            }
            consumed += c.len_utf8();
        }
        consumed
    }

    #[inline]
    fn action(&mut self, handler: &mut impl Handler, action: Action, byte: u8) {
        match action {
            Action::None | Action::Ignore | Action::_Unused => {}

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
                    self.params.as_params(),
                    self.intermediates.as_ref(),
                    byte as char,
                );
            }

            Action::DcsDispatch => {
                self.params.finish();
                handler.dcs(
                    self.params.as_params(),
                    self.intermediates.as_ref(),
                    byte as char,
                );
            }
            Action::DcsByte => handler.dcs_byte(byte),
            Action::DcsTermination => handler.dcs_termination(byte),

            Action::OscDispatch => handler.osc(self.params.as_params()),
            Action::OscByte => handler.osc_byte(byte),
            Action::OscTermination => handler.osc_termination(byte),
        }
    }

    /// Reset parameter / intermediate / data buffers.
    pub fn clear(&mut self) {
        self.params.clear();
        self.intermediates.clear();
        self.data.clear();
        self.utf8.clear();
    }

}

#[derive(Deref, DerefMut, Debug, Default, Clone)]
pub struct ParametersBuilder {
    #[deref]
    #[deref_mut]
    inner: NestedRaw<u16, 16, 8>,
    current: Option<u16>,
}

impl ParametersBuilder {
    /// ECMA-48 caps parameters at 16383.
    const MAX: u16 = 16383;

    /// Accumulate an ASCII digit into the current sub-parameter value.
    pub fn push_digit(&mut self, digit: u8) {
        self.current = Some(self
            .current
            .unwrap_or(0)
            .saturating_mul(10)
            .saturating_add((digit - b'0') as u16)
            .min(Self::MAX));
    }

    /// Append current parameter as a sub-parameter (`:` separator).
    /// Empty sub-parameters default to 0 to mirror ECMA-48 — `1::3` means `[1, 0, 3]`.
    pub fn push_sub(&mut self) {
        dbg!("sub", self.current);
        self.inner.extend_one(self.current.take().unwrap_or(0));
    }

    /// Append current parameter as a main parameter (`;` separator).
    /// An empty leading param defaults to 0 — `;1m` means `[[0], [1]]`.
    pub fn push_main(&mut self) {
        dbg!("main", self.current);
        self.inner.push_one(self.current.take().unwrap_or(0));
    }

    /// Finalize the in-flight param at dispatch time. Only commits if there's an unfinished value.
    pub fn finish(&mut self) {
        if self.current.is_some() {
            self.push_sub();
        }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.current = None;
    }
}


#[cfg(test)]
mod tests {
    use std::fmt::{Debug, Display};
    use crate::params;
    use super::*;


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
    enum Value {
        Print(char),
        Execute(u8),
        Esc(Intermediates, u8),
        Csi(Parameters, Intermediates, char),
        Dcs(Parameters, Intermediates, char),
        DcsByte(u8),
        DcsTermination(u8),
        Osc(Parameters),
        OscByte(u8),
        OscTermination(u8),
    }
    impl Debug for Value {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Value::Print(c) => write!(f, "Print({:?})", AnsiChar::from(c)),
                Value::Execute(b) => write!(f, "Execute({:?})", AnsiChar::from(b)),
                Value::Esc(i, b) => write!(f, "Esc({:?}, {:?})", i, AnsiChar::from(b)),
                Value::Csi(p, i, c) => write!(f, "Csi({:?}, {:?}, {:?})", p, i, AnsiChar::from(c)),
                Value::Dcs(p, i, c) => write!(f, "Dcs({:?}, {:?}, {:?})", p, i, AnsiChar::from(c)),
                Value::DcsByte(b) => write!(f, "DcsByte({:?})", AnsiChar::from(b)),
                Value::DcsTermination(b) => write!(f, "DcsTermination({:?})", AnsiChar::from(b)),
                Value::Osc(p) => write!(f, "Osc({:?})", p),
                Value::OscByte(b) => write!(f, "OscByte({:?})", AnsiChar::from(b)),
                Value::OscTermination(b) => write!(f, "OscTermination({:?})", AnsiChar::from(b)),
            }
        }
    }
    #[derive(Debug, Default, DerefMut, Deref)]
    struct Recorder {
        pub values: Vec<Value>,
    }

    impl Handler for Recorder {
        fn print(&mut self, ch: char) {
            self.values.push(Value::Print(ch));
        }
        fn execute(&mut self, byte: u8) {
            self.values.push(Value::Execute(byte));
        }
        fn esc(&mut self, intermediates: &Inter, final_byte: u8) {
            self.values.push(Value::Esc(Intermediates::from(intermediates), final_byte));
        }
        fn csi(&mut self, params: Params, intermediates: &Inter, final_byte: char) {
            self.values.push(Value::Csi(params.to_nested_vec(), intermediates.to_owned(), final_byte));
        }
        fn dcs(&mut self, params: Params, intermediates: &Inter, final_char: char) {
            self.values.push(Value::Dcs(params.to_nested_vec(), intermediates.to_owned(), final_char));
        }
        fn dcs_byte(&mut self, byte: u8) {
            self.values.push(Value::DcsByte(byte));
        }
        fn dcs_termination(&mut self, byte: u8) {
            self.values.push(Value::DcsTermination(byte));
        }
        fn osc(&mut self, params: Params) {
            self.values.push(Value::Osc(params.to_nested_vec()));
        }
        fn osc_byte(&mut self, byte: u8) {
            self.values.push(Value::OscByte(byte));
        }
        fn osc_termination(&mut self, byte: u8) {
            self.values.push(Value::OscTermination(byte));
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
            self.inner.advance(&mut self.recorder, bytes);
            &mut self.recorder
        }

        fn run(bytes: impl AsRef<[u8]>) -> Vec<Value> {
            let mut h = Harness::default();
            h.advance(bytes);
            h.recorder.values.clone()
        }
    }
    fn inter(b: &[u8]) -> Intermediates {
        Intermediates::from(b)
    }

    // ---- Ground / print / execute ---------------------------------------

    #[test]
    fn prints_plain_ascii() {
        assert_eq!(
            Harness::run("abc"),
            vec![Value::Print('a'), Value::Print('b'), Value::Print('c')],
        );
    }

    #[test]
    fn executes_c0_controls() {
        // BEL, BS, TAB, LF, CR
        assert_eq!(
            Harness::run(b"\x07\x08\x09\x0A\x0D"),
            vec![
                Value::Execute(0x07),
                Value::Execute(0x08),
                Value::Execute(0x09),
                Value::Execute(0x0A),
                Value::Execute(0x0D),
            ]
        );
    }

    #[test]
    fn mixes_print_and_execute() {
        assert_eq!(
            Harness::run(b"a\x07b"),
            vec![Value::Print('a'), Value::Execute(0x07), Value::Print('b')],
        );
    }

    #[test]
    fn ignores_del_in_ground_via_print_path() {
        // 0x7F is in the printable-fast-path range; it should not be executed
        // since DEL traditionally has no visible glyph but most parsers treat
        // it as printable here. We assert current behavior so regressions are
        // visible.
        assert_eq!(Harness::run(b"\x7f"), vec![Value::Print('\x7f')]);
    }

    #[test]
    fn prints_utf8_multibyte() {
        let events = Harness::run("aé東🦀b".as_bytes());
        assert_eq!(
            events,
            vec![
                Value::Print('a'),
                Value::Print('é'),
                Value::Print('東'),
                Value::Print('🦀'),
                Value::Print('b'),
            ],
        );
    }

    #[test]
    fn invalid_utf8_emits_replacement() {
        // Lone continuation byte 0xA0 is invalid.
        assert_eq!(
            Harness::run(&[b'a', 0xA0, b'b']),
            vec![
                Value::Print('a'),
                Value::Print('\u{FFFD}'),
                Value::Print('b'),
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
        assert_eq!(h.recorder.values, vec![Value::Print('🦀')]);
    }

    #[test]
    fn partial_utf8_followed_by_esc_emits_replacement() {
        // Partial 2-byte sequence (0xC3) cut off by ESC.
        assert_eq!(
            Harness::run(b"a\xC3\x1B[m"),
            vec![
                Value::Print('a'),
                Value::Print('\u{FFFD}'),
                Value::Csi(Parameters::new(), inter(b""), 'm'),
            ],
        );
    }

    // ---- ESC ------------------------------------------------------------

    #[test]
    fn esc_simple_dispatch() {
        // ESC 7 — DECSC
        assert_eq!(
            Harness::run(b"\x1B7"),
            vec![Value::Esc(inter(b""), b'7')],
        );
    }

    #[test]
    fn esc_with_intermediate() {
        // ESC # 8 — DECALN
        assert_eq!(
            Harness::run(b"\x1B#8"),
            vec![Value::Esc(inter(b"#"), b'8')],
        );
    }

    #[test]
    fn esc_with_two_intermediates() {
        // ESC SP # F (uncommon but legal)
        assert_eq!(
            Harness::run(b"\x1B #F"),
            vec![Value::Esc(inter(b" #"), b'F')],
        );
    }

    #[test]
    fn esc_re_entry_aborts_previous() {
        // ESC inside a CSI sequence should abandon the CSI and start fresh.
        assert_eq!(
            Harness::run(b"\x1B[1;\x1B[2m"),
            vec![Value::Csi(params![[2]], inter(b""), 'm')],
        );
    }

    // ---- CSI ------------------------------------------------------------

    #[test]
    fn csi_no_params() {
        assert_eq!(
            Harness::run(b"\x1B[m"),
            vec![Value::Csi(Parameters::new(), inter(b""), 'm')],
        );
    }

    #[test]
    fn csi_single_param() {
        assert_eq!(
            Harness::run(b"\x1B[1m"),
            vec![Value::Csi(params![[1]], inter(b""), 'm')],
        );
    }

    #[test]
    fn csi_multiple_params() {
        assert_eq!(
            Harness::run(b"\x1B[1;2;3m"),
            vec![Value::Csi(
                params!([1], [2], [3]),
                inter(b""),
                'm'
            )],
        );
    }

    #[test]
    fn csi_subparams() {
        // 24-bit fg via sub-params: 38:2:255:128:0
        assert_eq!(
            Harness::run(b"\x1B[38:2:255:128:0m"),
            vec![Value::Csi(
                params![[38, 2, 255, 128, 0]],
                inter(b""),
                'm'
            )],
        );
    }

    #[test]
    fn csi_mixed_subparams_and_params() {
        assert_eq!(
            Harness::run(b"\x1B[1;2:3:4;5m"),
            vec![Value::Csi(
                params![[1], [2, 3, 4], [5]],
                inter(b""),
                'm'
            )],
        );
    }

    #[test]
    fn csi_empty_leading_param_defaults_to_zero() {
        assert_eq!(
            Harness::run(b"\x1B[;1m"),
            vec![Value::Csi(params![[0], [1]], inter(b""), 'm')],
        );
    }

    #[test]
    fn csi_empty_subparam_defaults_to_zero() {
        // 38:2::255:128:0 — empty colorspace ID should be 0.
        assert_eq!(
            Harness::run(b"\x1B[38:2::255:128:0m"),
            vec![Value::Csi(
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
            vec![Value::Csi(params![[1]], inter(b""), 'm')],
        );
    }

    #[test]
    fn csi_double_semicolon_inserts_zero() {
        assert_eq!(
            Harness::run(b"\x1B[1;;2m"),
            vec![Value::Csi(
                params![[1], [0], [2]],
                inter(b""),
                'm'
            )],
        );
    }

    #[test]
    fn csi_trailing_colon_dispatches_with_zero_subparam() {
        assert_eq!(
            Harness::run(b"\x1B[1::m"),
            vec![Value::Csi(params![[1, 0, 0]], inter(b""), 'm')],
        );
    }

    #[test]
    fn csi_clamps_param_to_max() {
        // 99999 saturates at 16383 (ECMA-48 cap).
        assert_eq!(
            Harness::run(b"\x1B[99999m"),
            vec![Value::Csi(params![[16383]], inter(b""), 'm')],
        );
    }

    #[test]
    fn csi_private_marker() {
        // DECSET — CSI ? 25 h
        assert_eq!(
            Harness::run(b"\x1B[?25h"),
            vec![Value::Csi(params![[25]], inter(b"?"), 'h')],
        );
    }

    #[test]
    fn csi_intermediate() {
        // CSI SP q — DECSCUSR
        assert_eq!(
            Harness::run(b"\x1B[2 q"),
            vec![Value::Csi(params![[2]], inter(b" "), 'q')],
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
                Value::Csi(params![[1]], inter(b""), 'm'),
                Value::Print('h'),
                Value::Print('i'),
            ],
        );
    }

    #[test]
    fn csi_can_cancels() {
        // CAN inside a CSI returns to Ground without dispatch.
        assert_eq!(
            Harness::run(b"\x1B[1;2\x18m"),
            vec![Value::Execute(0x18), Value::Print('m')],
        );
    }

    #[test]
    fn csi_sub_cancels() {
        // SUB inside a CSI returns to Ground without dispatch.
        assert_eq!(
            Harness::run(b"\x1B[1;2\x1Am"),
            vec![Value::Execute(0x1A), Value::Print('m')],
        );
    }

    // ---- 8-bit C1 introducers ------------------------------------------

    #[test]
    fn c1_csi_starts_csi() {
        // 0x9B is 8-bit CSI.
        assert_eq!(
            Harness::run([0x9B, b'1', b';', b'2', b'm']),
            vec![Value::Csi(
                params![[1], [2]],
                inter(b""),
                'm'
            )],
        );
    }

    #[test]
    fn c1_osc_starts_osc() {
        // 0x9D is 8-bit OSC, 0x9C is ST.
        assert_eq!(
            Harness::run([0x9D, b'0', b';', b'h', b'i', 0x9C]),
            vec![
                Value::Osc(Parameters::new()),
                Value::OscByte(b'0'),
                Value::OscByte(b';'),
                Value::OscByte(b'h'),
                Value::OscByte(b'i'),
                Value::OscTermination(0x9C),
            ],
        );
    }

    #[test]
    fn c1_dcs_starts_dcs() {
        // 0x90 is 8-bit DCS, 0x9C is ST.
        assert_eq!(
            Harness::run([0x90, b'q', b'X', 0x9C]),
            vec![
                Value::Dcs(Parameters::new(), inter(b""), 'q'),
                Value::DcsByte(b'X'),
                Value::DcsTermination(0x9C),
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
                Value::Osc(Parameters::new()),
                Value::OscByte(b'0'),
                Value::OscByte(b';'),
                Value::OscByte(b't'),
                Value::OscByte(b'i'),
                Value::OscByte(b't'),
                Value::OscByte(b'l'),
                Value::OscByte(b'e'),
                Value::OscTermination(0x1B),
                Value::Esc(inter(b""), b'\\'),
            ],
        );
    }

    #[test]
    fn osc_bel_terminated() {
        // OSC 0 ; title BEL — xterm convention.
        assert_eq!(
            Harness::run(b"\x1B]0;hi\x07"),
            vec![
                Value::Osc(Parameters::new()),
                Value::OscByte(b'0'),
                Value::OscByte(b';'),
                Value::OscByte(b'h'),
                Value::OscByte(b'i'),
                Value::OscTermination(0x07),
            ],
        );
    }

    #[test]
    fn osc_empty() {
        assert_eq!(
            Harness::run(b"\x1B]\x07"),
            vec![
                Value::Osc(Parameters::new()),
                Value::OscTermination(0x07),
            ],
        );
    }

    // ---- DCS ------------------------------------------------------------

    #[test]
    fn dcs_basic() {
        // DCS $ q   <data>   ESC \
        assert_eq!(
            Harness::run(b"\x1BP$q q\x1B\\"),
            vec![
                Value::Dcs(Parameters::new(), inter(b"$"), 'q'),
                Value::DcsByte(b' '),
                Value::DcsByte(b'q'),
                Value::DcsTermination(0x1B),
                Value::Esc(inter(b""), b'\\'),
            ],
        );
    }

    #[test]
    fn dcs_with_params() {
        assert_eq!(
            Harness::run(b"\x1BP1;2|data\x1B\\"),
            vec![
                Value::Dcs(params![[1], [2]], inter(b""), '|'),
                Value::DcsByte(b'd'),
                Value::DcsByte(b'a'),
                Value::DcsByte(b't'),
                Value::DcsByte(b'a'),
                Value::DcsTermination(0x1B),
                Value::Esc(inter(b""), b'\\'),
            ],
        );
    }

    #[test]
    fn dcs_with_subparams() {
        assert_eq!(
            Harness::run(b"\x1BP1:2|x\x1B\\"),
            vec![
                Value::Dcs(params![[1, 2]], inter(b""), '|'),
                Value::DcsByte(b'x'),
                Value::DcsTermination(0x1B),
                Value::Esc(inter(b""), b'\\'),
            ],
        );
    }

    #[test]
    fn dcs_can_cancels() {
        // CAN aborts the DCS without termination event.
        assert_eq!(
            Harness::run(b"\x1BPq abc\x18tail"),
            vec![
                Value::Dcs(Parameters::new(), inter(b""), 'q'),
                Value::DcsByte(b' '),
                Value::DcsByte(b'a'),
                Value::DcsByte(b'b'),
                Value::DcsByte(b'c'),
                Value::DcsTermination(0x18),
                Value::Execute(0x18),
                Value::Print('t'),
                Value::Print('a'),
                Value::Print('i'),
                Value::Print('l'),
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
            vec![Value::Esc(inter(b""), b'\\')],
        );
    }

    #[test]
    fn apc_string_is_silently_consumed() {
        assert_eq!(
            Harness::run(b"\x1B_x\x1B\\"),
            vec![Value::Esc(inter(b""), b'\\')],
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
            vec![Value::Csi(
                params![[38], [5], [196]],
                inter(b""),
                'm'
            )]
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
                Value::Osc(Parameters::new()),
                Value::OscByte(b'0'),
                Value::OscByte(b';'),
                Value::OscByte(b't'),
                Value::OscByte(b'i'),
                Value::OscByte(b't'),
                Value::OscByte(b'l'),
                Value::OscByte(b'e'),
                Value::OscTermination(0x07),
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
                Value::Csi(
                    params![[1], [2], [3]],
                    inter(b""),
                    'm'
                ),
                // 👨🏿 = man + dark skin tone modifier (two codepoints).
                Value::Print('\u{1F468}'),
                Value::Print('\u{1F3FF}'),
                Value::Csi(params![[0]], inter(b""), 'm'),
            ],
        );
    }



    #[test]
    fn qwe() {
        let a = Parameters::<8, 8>::new();
        dbg!(a);

    }
}
