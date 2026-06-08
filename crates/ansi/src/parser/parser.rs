use super::*;
use arrayvec::ArrayVec;
use derive_more::{Deref, DerefMut};
use utils::{Nested, NestedMut, NestedRaw, TryNestedMut};

#[derive(Debug, Default)]
pub struct Parser {
    pub state: State,

    pub params: ParametersBuilder,
    pub intermediates: ByteString,

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
                }
            }
        }
    }

    /// Signal end of input. Any buffered partial UTF-8 codepoint is resolved as
    /// U+FFFD and the parser is reset to [`State::Ground`]. Incomplete control
    /// sequences (a CSI/OSC/DCS cut off mid-stream) are discarded without
    /// dispatch, matching standard VT behavior. Call this when the producer is
    /// done and any dangling bytes should be flushed rather than held for a
    /// future `advance`.
    pub fn flush(&mut self, handler: &mut impl Handler) {
        if !self.utf8.is_empty() {
            handler.printable(char::REPLACEMENT_CHARACTER);
        }
        self.state = State::Ground;
        self.clear();
    }

    #[inline]
    fn advance_byte(&mut self, handler: &mut impl Handler, byte: u8) {
        let prev_state = self.state;
        let (action, next_state) = transition(self.state, byte);

        if next_state != State::None {
            let exit_action = exit(prev_state);
            let entry_action = entry(next_state);

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
    /// Continue assembling a partial UTF-8 codepoint buffered from a previous
    /// call. The buffer always holds a valid incomplete prefix (1..=3 bytes
    /// starting with a multibyte lead), so we only ever pull enough new bytes to
    /// finish that single codepoint — never bytes belonging to the next one.
    /// Returns the number of bytes from `bytes` consumed.
    fn advance_utf8(&mut self, handler: &mut impl Handler, bytes: &[u8]) -> usize {
        let old_bytes = self.utf8.len();
        // Bytes still needed to finish the buffered codepoint.
        let need = utf8_width(self.utf8[0]).saturating_sub(old_bytes);
        let take = bytes.len().min(need);
        // old_bytes + take <= utf8_width(..) <= 4, so this always fits.
        debug_assert!(old_bytes + take <= self.utf8.capacity());
        let _ = self.utf8.try_extend_from_slice(&bytes[..take]);

        match str::from_utf8(&self.utf8) {
            // Buffer is now a complete codepoint: dispatch it and clear.
            Ok(parsed) => {
                let c = unsafe { parsed.chars().next().unwrap_unchecked() };
                handler.printable(c);
                self.utf8.clear();
                take
            }
            // Couldn't finish: either the continuation bytes are malformed, or
            // we still ran out of input.
            Err(err) => match err.error_len() {
                // Malformed sequence — emit one replacement and drop the
                // buffered lead. Any new bytes we copied are left for the main
                // loop to reprocess (`valid_up_to` is 0 here, so consume none).
                Some(_) => {
                    handler.printable('\u{FFFD}');
                    let consumed = err.valid_up_to().saturating_sub(old_bytes);
                    self.utf8.clear();
                    consumed
                }
                // Still incomplete — keep the buffered prefix for the next call.
                None => take,
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
        let num_chars = memchr::memchr(0x1B, bytes).unwrap_or(num_bytes);

        // ESC is the very first byte: short-circuit to Escape state.
        if num_chars == 0 {
            self.state = State::Escape;
            self.clear();
            return 1;
        }

        let chars = &bytes[..num_chars];
        if chars.is_ascii() {
            let processed = Self::dispatch_ground_ascii(handler, chars);
            if processed < num_chars {
                return processed;
            }

            if num_chars < num_bytes {
                self.state = State::Escape;
                self.clear();
                return num_chars + 1;
            }

            return num_chars;
        }

        match str::from_utf8(chars) {
            Ok(str) => {
                let consumed = Self::dispatch_ground_chars(handler, str);

                // If we stopped early on a C1 char encoded as utf8, hand the
                // bytes after `consumed` back to the state machine unless the
                // C1 char is first, in which case consume it now to guarantee
                // progress.
                if consumed < num_chars {
                    if consumed > 0 {
                        return consumed;
                    }

                    let c = unsafe { str.chars().next().unwrap_unchecked() };
                    self.advance_byte(handler, c as u8);
                    return c.len_utf8();
                }

                let mut processed = num_chars;
                if processed < num_bytes {
                    // Next byte must be ESC — process directly.
                    self.state = State::Escape;
                    self.clear();
                    processed += 1;
                }
                processed
            }
            Err(err) => {
                let valid_bytes = err.valid_up_to();
                let parsed = unsafe { str::from_utf8_unchecked(&bytes[..valid_bytes]) };
                let dispatched = Self::dispatch_ground_chars(handler, parsed);

                // Stopped early inside the valid prefix: bail out so the state
                // machine can take over with the C1 char unless it is first,
                // in which case consume it now to guarantee progress.
                if dispatched < valid_bytes {
                    if dispatched > 0 {
                        return dispatched;
                    }

                    let c = unsafe { parsed.chars().next().unwrap_unchecked() };
                    self.advance_byte(handler, c as u8);
                    return c.len_utf8();
                }

                match err.error_len() {
                    Some(len) => {
                        let bad = bytes[valid_bytes];
                        // Raw 8-bit C1 controls (0x80..=0x9F) are handled by
                        // the state machine via Anywhere transitions.
                        if len == 1 && (0x80..=0x9F).contains(&bad) {
                            if valid_bytes > 0 {
                                return valid_bytes;
                            }

                            self.advance_byte(handler, bad);
                            return len;
                        }
                        // Otherwise the bytes are genuinely malformed.
                        handler.printable('\u{FFFD}');
                        valid_bytes + len
                    }
                    None => {
                        if num_chars < num_bytes {
                            // The partial codepoint is followed by ESC — drop
                            // it and start the escape sequence.
                            handler.printable('\u{FFFD}');
                            self.state = State::Escape;
                            self.clear();
                            num_chars + 1
                        } else {
                            // Buffer the partial codepoint for the next call.
                            // A partial UTF-8 prefix is at most 3 bytes, so this
                            // always fits the capacity-4 buffer.
                            debug_assert!(num_bytes - valid_bytes <= self.utf8.capacity());
                            let _ = self.utf8.try_extend_from_slice(&bytes[valid_bytes..]);
                            num_bytes
                        }
                    }
                }
            }
        }
    }

    /// Walk the chars of a validated str, dispatching printable runs in a
    /// single [`Handler::printables`] call and C0 controls via
    /// [`Handler::execute`]. Stops at the first C1 control char (0x80..=0x9F)
    /// without consuming it, returning the byte offset where dispatch stopped.
    #[inline]
    fn dispatch_ground_chars(handler: &mut impl Handler, parsed: &str) -> usize {
        let mut run_start = 0;
        let mut i = 0;
        for c in parsed.chars() {
            match c {
                '\u{80}'..='\u{9F}' => {
                    if run_start < i {
                        handler.printables(&parsed[run_start..i]);
                    }
                    return i;
                }
                '\x00'..='\x1F' => {
                    if run_start < i {
                        handler.printables(&parsed[run_start..i]);
                    }
                    handler.execute(c as u8);
                    run_start = i + c.len_utf8();
                }
                _ => {}
            }
            i += c.len_utf8();
        }

        if run_start < i {
            handler.printables(&parsed[run_start..i]);
        }
        i
    }

    /// Walk ASCII bytes without UTF-8 char decoding, batching printable runs
    /// into [`Handler::printables`] calls.
    #[inline]
    fn dispatch_ground_ascii(handler: &mut impl Handler, bytes: &[u8]) -> usize {
        let mut run_start = 0;
        for (i, &byte) in bytes.iter().enumerate() {
            if byte < 0x20 {
                if run_start < i {
                    // SAFETY: the caller validated `bytes` is ASCII, so every
                    // sub-slice is valid UTF-8.
                    handler.printables(unsafe { str::from_utf8_unchecked(&bytes[run_start..i]) });
                }
                handler.execute(byte);
                run_start = i + 1;
            }
        }

        if run_start < bytes.len() {
            // SAFETY: ASCII, see above.
            handler.printables(unsafe { str::from_utf8_unchecked(&bytes[run_start..]) });
        }
        bytes.len()
    }

    #[inline]
    fn action(&mut self, handler: &mut impl Handler, action: Action, byte: u8) {
        match action {
            Action::None | Action::Ignore | Action::_Unused => {}

            Action::Clear => self.clear(),

            Action::Print => handler.printable(byte as char),
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

            Action::DcsDispatch => {
                self.params.finish();
                handler.dcs(
                    self.params.as_nested_slice(),
                    self.intermediates.as_ref(),
                    byte as char,
                );
            }
            Action::DcsByte => handler.dcs_byte(byte),
            Action::DcsTermination => handler.dcs_termination(byte),

            Action::OscDispatch => handler.osc(),
            Action::OscByte => handler.osc_byte(byte),
            Action::OscTermination => handler.osc_termination(byte),
        }
    }

    /// Reset parameter / intermediate / data buffers.
    pub fn clear(&mut self) {
        self.params.clear();
        self.intermediates.clear();
        self.utf8.clear();
    }
}

/// Total byte length of a UTF-8 codepoint given its leading byte. Buffered
/// partials always start with a multibyte lead (`0xC2..=0xF4`); anything else
/// is treated as a single byte.
#[inline]
const fn utf8_width(lead: u8) -> usize {
    match lead {
        0xC0..=0xDF => 2,
        0xE0..=0xEF => 3,
        0xF0..=0xF7 => 4,
        _ => 1,
    }
}

#[derive(Deref, DerefMut, Debug, Default, Clone)]
pub struct ParametersBuilder {
    #[deref]
    #[deref_mut]
    inner: NestedRaw<u16, 32, 32>,
    current: Option<u16>,
    group_active: bool,
    last_separator: Option<ParameterSeparator>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParameterSeparator {
    Main,
    Sub,
}

impl ParametersBuilder {
    /// ECMA-48 caps parameters at 16383.
    const MAX: u16 = 16383;

    /// Accumulate an ASCII digit into the current sub-parameter value.
    pub fn push_digit(&mut self, digit: u8) {
        self.current = Some(
            self.current
                .unwrap_or(0)
                .saturating_mul(10)
                .saturating_add((digit - b'0') as u16)
                .min(Self::MAX),
        );
    }

    /// Append current parameter as a sub-parameter (`:` separator).
    /// Empty sub-parameters default to 0 to mirror ECMA-48 — `1::3` means `[1, 0, 3]`.
    pub fn push_sub(&mut self) {
        let val = self.current.take().unwrap_or(0);
        self.push_sub_value(val);
        self.last_separator = Some(ParameterSeparator::Sub);
    }

    /// Append current parameter as a main parameter (`;` separator).
    /// An empty leading param defaults to 0 — `;1m` means `[[0], [1]]`.
    pub fn push_main(&mut self) {
        match self.current.take() {
            Some(val) => self.push_value(val),
            None if matches!(self.last_separator, Some(ParameterSeparator::Sub)) => {
                self.push_sub_value(0);
            }
            // Drop on overflow: the CSI/DCS still dispatches with the capped
            // params rather than panicking.
            None => {
                let _ = self.inner.try_push_one(0);
            }
        }

        self.group_active = false;
        self.last_separator = Some(ParameterSeparator::Main);
    }

    /// Finalize the in-flight param at dispatch time.
    pub fn finish(&mut self) {
        match self.current.take() {
            Some(val) => self.push_value(val),
            None if matches!(self.last_separator, Some(ParameterSeparator::Sub)) => {
                self.push_sub_value(0);
            }
            None => {}
        }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.current = None;
        self.group_active = false;
        self.last_separator = None;
    }

    // The push helpers drop values that exceed capacity (`NestedError::Overflow`)
    // instead of panicking. A pathologically long parameter list still
    // dispatches, just with the trailing params capped — mirroring the
    // reference's cap-and-continue behavior, safely.
    fn push_value(&mut self, val: u16) {
        if self.group_active {
            self.push_sub_value(val);
        } else {
            let _ = self.inner.try_push_one(val);
        }
    }

    fn push_sub_value(&mut self, val: u16) {
        if self.group_active {
            let _ = self.inner.try_extend_one(val);
        } else {
            // Only mark the group active if the value actually landed, so
            // `group_active` bookkeeping stays consistent on overflow.
            if self.inner.try_push_one(val).is_ok() {
                self.group_active = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params;
    use std::fmt::{Debug, Display};
    use utils::NestedConstructor;

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
        Esc(ByteString, u8),
        Csi(Parameters, ByteString, char),
        Dcs(Parameters, ByteString, char),
        DcsByte(u8),
        DcsTermination(u8),
        Osc,
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
                Value::Osc => write!(f, "Osc"),
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
        fn printable(&mut self, ch: char) {
            self.values.push(Value::Print(ch));
        }
        fn execute(&mut self, byte: u8) {
            self.values.push(Value::Execute(byte));
        }
        fn esc(&mut self, intermediates: &ByteStr, final_byte: u8) {
            self.values
                .push(Value::Esc(ByteString::from(intermediates), final_byte));
        }
        fn csi(&mut self, params: Params, intermediates: &ByteStr, final_byte: char) {
            self.values.push(Value::Csi(
                params.to_nested_vec(),
                intermediates.to_owned(),
                final_byte,
            ));
        }
        fn dcs(&mut self, params: Params, intermediates: &ByteStr, final_char: char) {
            self.values.push(Value::Dcs(
                params.to_nested_vec(),
                intermediates.to_owned(),
                final_char,
            ));
        }
        fn dcs_byte(&mut self, byte: u8) {
            self.values.push(Value::DcsByte(byte));
        }
        fn dcs_termination(&mut self, byte: u8) {
            self.values.push(Value::DcsTermination(byte));
        }
        fn osc(&mut self) {
            self.values.push(Value::Osc);
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
    fn inter(b: &[u8]) -> ByteString {
        ByteString::from(b)
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
        assert_eq!(Harness::run(b"\x1B7"), vec![Value::Esc(inter(b""), b'7')],);
    }

    #[test]
    fn esc_with_intermediate() {
        // ESC # 8 — DECALN
        assert_eq!(Harness::run(b"\x1B#8"), vec![Value::Esc(inter(b"#"), b'8')],);
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
            vec![Value::Csi(params!([1], [2], [3]), inter(b""), 'm')],
        );
    }

    #[test]
    fn csi_subparams() {
        // 24-bit fg via sub-params: 38:2:255:128:0
        assert_eq!(
            Harness::run(b"\x1B[38:2:255:128:0m"),
            vec![Value::Csi(params![[38, 2, 255, 128, 0]], inter(b""), 'm')],
        );
    }

    #[test]
    fn csi_mixed_subparams_and_params() {
        assert_eq!(
            Harness::run(b"\x1B[1;2:3:4;5m"),
            vec![Value::Csi(params![[1], [2, 3, 4], [5]], inter(b""), 'm')],
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
            vec![Value::Csi(params![[1], [0], [2]], inter(b""), 'm')],
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
            vec![Value::Csi(params![[1], [2]], inter(b""), 'm')],
        );
    }

    #[test]
    fn c1_osc_starts_osc() {
        // 0x9D is 8-bit OSC, 0x9C is ST.
        assert_eq!(
            Harness::run([0x9D, b'0', b';', b'h', b'i', 0x9C]),
            vec![
                Value::Osc,
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
                Value::Osc,
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
                Value::Osc,
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
            vec![Value::Osc, Value::OscTermination(0x07),],
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
            vec![Value::Csi(params![[38], [5], [196]], inter(b""), 'm')]
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
                Value::Osc,
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
                Value::Csi(params![[1], [2], [3]], inter(b""), 'm'),
                // 👨🏿 = man + dark skin tone modifier (two codepoints).
                Value::Print('\u{1F468}'),
                Value::Print('\u{1F3FF}'),
                Value::Csi(params![[0]], inter(b""), 'm'),
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
            Value::Csi(params, i, c) => {
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
            vec![Value::Csi(
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
            vec![Value::Esc(inter(b""), b'\\')],
        );
    }

    #[test]
    fn c1_sos_string_is_silently_consumed() {
        // 8-bit SOS introducer 0x98.
        let mut seq = vec![0x98];
        seq.extend_from_slice(b"junk\x1B\\");
        assert_eq!(Harness::run(seq), vec![Value::Esc(inter(b""), b'\\')]);
    }

    #[test]
    fn standalone_st_dispatches_as_esc() {
        // ESC \ in ground is just an ESC dispatch.
        assert_eq!(
            Harness::run(b"\x1B\\"),
            vec![Value::Esc(inter(b""), b'\\')],
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
                Value::Osc,
                Value::OscByte(b'0'),
                Value::OscByte(b';'),
                Value::OscByte(b'h'),
                Value::OscByte(b'i'),
                Value::OscTermination(0x18),
                Value::Execute(0x18),
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
                Value::Print('x'),
                Value::Print('\u{FFFD}'),
                Value::Print('y'),
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
                Value::Print('a'),
                Value::Print('\u{FFFD}'),
                Value::Print('\u{FFFD}'),
                Value::Print('b'),
            ],
        );
    }

    #[test]
    fn truncated_multibyte_resolved_on_next_advance() {
        // 東 = E6 9D B1 (3 bytes). Split after the first byte: the partial
        // codepoint is buffered, then completed on the following call.
        let mut h = Harness::default();
        h.advance(&[b'a', 0xE6]);
        assert_eq!(h.recorder.values, vec![Value::Print('a')]);
        h.advance(&[0x9D, 0xB1, b'b']);
        assert_eq!(
            h.recorder.values,
            vec![Value::Print('a'), Value::Print('東'), Value::Print('b')],
        );
    }

    // ---- CSI edge cases -------------------------------------------------

    #[test]
    fn del_inside_csi_param_is_ignored() {
        // 0x7F inside a CSI param is ignored; the sequence still dispatches.
        assert_eq!(
            Harness::run(b"\x1B[1;2\x7fm"),
            vec![Value::Csi(params![[1], [2]], inter(b""), 'm')],
        );
    }

    #[test]
    fn csi_intermediate_only_soft_reset() {
        // CSI ! p — DECSTR (soft reset): intermediate, no params.
        assert_eq!(
            Harness::run(b"\x1B[!p"),
            vec![Value::Csi(Parameters::new(), inter(b"!"), 'p')],
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
                Value::Osc,
                Value::OscByte(b'0'),
                Value::OscByte(b';'),
                Value::OscByte(0xC3),
                Value::OscByte(0xA9),
                Value::OscTermination(0x07),
            ],
        );
    }

    #[test]
    fn dcs_payload_preserves_utf8() {
        // DCS q é ESC \ — DCS data with é (C3 A9).
        assert_eq!(
            Harness::run(b"\x1BPq\xC3\xA9\x1B\\"),
            vec![
                Value::Dcs(Parameters::new(), inter(b""), 'q'),
                Value::DcsByte(0xC3),
                Value::DcsByte(0xA9),
                Value::DcsTermination(0x1B),
                Value::Esc(inter(b""), b'\\'),
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
        assert_eq!(h.recorder.values, vec![Value::Print('\u{FFFD}')]);
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
        assert_eq!(h.recorder.values, vec![Value::Print('x')]);
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
}
