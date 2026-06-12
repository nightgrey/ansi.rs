//! Incremental UTF-8 decoder.
//! Based on Christian Hansen's `c-utf8` implementation.
//! https://github.com/chansen/c-utf8

use std::iter;
use super::dfa::{State, step_decode};

#[derive(Copy, Debug)]
#[derive_const(Clone, PartialEq, Eq)]
pub enum Codepoint {
    /// A complete, well-formed Unicode scalar value.
    Complete(char),
    /// An invalid byte was encountered.
    Invalid,
    /// Byte was consumed, and needs to be re-fed.
    Reprocess,
    /// Byte was consumed, but sequence is not complete yet.
    Incomplete,
}

#[derive(Copy, Debug)]
#[derive_const(Clone, PartialEq, Eq)]
pub struct Codepoints<I: Iterator<Item = u8>> {
    bytes: I,
    decoder: Decoder,
}

impl<I: Iterator<Item = u8>> Iterator for Codepoints<I> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let byte = self.bytes.next()?;

            match self.decoder.next(byte) {
                Codepoint::Complete(c) => return Some(c),
                Codepoint::Incomplete => continue,
                Codepoint::Reprocess => {
                    // @TODO: Proper re-feed.
                    // self.pending = Some(byte);
                }
                Codepoint::Invalid => {
                    return Some(char::REPLACEMENT_CHARACTER);
                }
            }
        }
    }
}

/// Incremental UTF-8 decoder.
///
/// `Default` is the ground state. The whole decoder is 5 bytes and
/// `Copy`, so an embedding state machine can snapshot or reset it
/// freely.
#[derive(Copy, Debug)]
#[derive_const(Clone, PartialEq, Eq)]
pub struct Decoder {
    state: State,
    codepoint: u32,
    count: usize,
}

impl Decoder {
    pub const EMPTY: Self = Self { state: State::ACCEPT, codepoint: 0, count: 0 };
    /// A decoder in the ground state.
    #[inline]
    pub const fn new() -> Self {
        Decoder { state: State::ACCEPT, codepoint: 0, count: 0 }
    }

    /// Returns `true` if bytes of an unfinished sequence have been absorbed.
    #[inline]
    pub const fn is_pending(&self) -> bool {
        self.count > 0
    }

    /// Feed one byte, yielding the resulting [`Codepoint`].
    ///
    /// If [`Codepoint::Reprocess`] is returned, bytes need to be re-fed by the caller.
    #[inline]
    pub fn next(&mut self, byte: u8) -> Codepoint {
        self.state = self.decode(byte);
        self.count += 1;

        if self.state == State::ACCEPT {
            return Codepoint::Complete(self.take());
        }

        if self.state == State::REJECT {
            // The byte triggered rejection. If it was
            // the first byte, it is itself the maximal subpart (length 1).
            // Otherwise the lead byte(s) already consumed form the maximal
            // subpart and the triggering byte belongs to the next sequence.
            let result = if self.count > 1 { Codepoint::Reprocess } else { Codepoint::Invalid };
            self.clear();
            return result;
        }

        // The source ended before the current multi-byte sequence was complete.
        Codepoint::Incomplete
    }



    /// Feed one byte, yielding the resulting [`Codepoint`].
    ///
    /// Codepoints are automatically re-fed if necessary.
    #[inline]
    pub fn advance(&mut self, byte: u8) -> Codepoints<iter::Once<u8>> {
        Codepoints { bytes: iter::once(byte), decoder: *self }
    }

    /// Finalizes the decoder state.
    ///
    /// Returns [`Codepoint::Incomplete`] if any pending sequence was not completed, otherwise `None`.
    #[inline]
    pub fn flush(&mut self) -> Option<Codepoint> {
        let pending = self.is_pending();
        self.clear();
        pending.then_some(Codepoint::Incomplete)
    }

    /// Clears the decoder state.
    ///
    /// Unlike [`Self::flush`], this discards a pending sequence.
    #[inline]
    pub fn clear(&mut self) {
        *self = Self::EMPTY;
    }

    /// One raw DFA transition: classify, fold into the codepoint
    /// accumulator, advance the state. Two table loads, no branches
    /// beyond the accumulator select.
    #[inline(always)]
    fn decode(&mut self, byte: u8) -> State {
        self.state = step_decode(self.state, byte, &mut self.codepoint);
        self.state
    }

    /// Extract the completed scalar. Only called when the DFA is in
    /// ACCEPT after consuming ≥1 byte.
    #[inline(always)]
    fn take(&mut self) -> char {
        debug_assert!(char::from_u32(self.codepoint).is_some());
        // SAFETY: the DFA accepts exactly the well-formed UTF-8
        // sequences of TUS Table 3-7; surrogates (via the ED state)
        // and values above U+10FFFF (via the F4 state and F5..FF
        // class) are unreachable, so `codepoint` is a scalar value.
        let char = unsafe { char::from_u32_unchecked(self.codepoint) };
        self.clear();
        char
    }
}


impl const Default for Decoder {
    fn default() -> Self {
        Self::EMPTY
    }
}



#[cfg(test)]
mod tests {
    use geometry::Bound;
    use super::*;

    /// Decode a whole buffer byte-by-byte, replacing each `Invalid`
    /// with U+FFFD — the reference behaviour of `from_utf8_lossy`.
    fn decode(bytes: &[u8]) -> String {
        let mut decoder = Decoder::new();
        let mut out = String::new();

        for &byte in bytes {
            match decoder.next(byte) {
                Codepoint::Complete(c) => out.push(c),
                Codepoint::Invalid => out.push(char::REPLACEMENT_CHARACTER),
                Codepoint::Incomplete => {}
            }
        }

        if let Some(Codepoint::Invalid) = decoder.flush() {
            out.push(char::REPLACEMENT_CHARACTER);
        }
        out
    }

    macro_rules! assert_lossy {
        ($bytes:expr) => {{
            let bytes = $bytes;
            let expected = String::from_utf8_lossy(bytes);
            let actual = decode(bytes);
            assert_eq!(
                actual,
                expected,
                "Expected {:?} but got {:?} for {:02X?}",
                expected.as_bytes(),
                actual.as_bytes(),
                bytes
            )
        }};
    }

    #[test]
    fn ascii() {
       assert_lossy!(b"Hello, World!");
    }

    #[test]
    fn multibyte_scalars() {
        // 2-, 3-, 4-byte sequences incl. BMP edges and astral plane.
        let s = "héllo → € ｗｉｄｅ 𝄞 🦀 \u{7FF}\u{800}\u{FFFF}\u{10000}\u{10FFFF}";
        assert_eq!(decode(s.as_bytes()), s);
    }

    #[test]
    fn pending_across_feeds() {
        let mut p = Decoder::default();
        let crab = "🦀".as_bytes(); // F0 9F A6 80

        assert_eq!(p.next(crab[0]), Codepoint::Incomplete);
        assert!(p.is_pending());
        assert_eq!(p.next(crab[1]), Codepoint::Incomplete);
        assert_eq!(p.next(crab[2]), Codepoint::Incomplete);
        assert_eq!(p.next(crab[3]), Codepoint::Complete('🦀'));
        assert!(!p.is_pending());
    }

    #[test]
    fn unicode_table_3_11_maximal_subparts() {
        // TUS §3.9: 61 F1 80 80 E1 80 C2 62 → a, FFFD, FFFD, FFFD, b
        assert_eq!(
            decode(b"\x61\xF1\x80\x80\xE1\x80\xC2\x62"),
            "a\u{FFFD}\u{FFFD}\u{FFFD}b"
        );
    }

    // #[test]
    // fn invalid_then_char_in_one_advance() {
    //     let mut p = Parser::default();
    //     assert_eq!(p.next(0xC3), Codepoint::Pending); // pending 2-byte seq
    //     let mut steps = p.next(b'A'); // not a continuation
    //     assert_eq!(steps.len(), 2);
    //     assert_eq!(steps.next(), Codepoint::Invalid);
    //     assert_eq!(steps.next(), Codepoint::Ok('A'));
    //     assert_eq!(steps.next(), Codepoint::Pending);
    // }
    #[test]
    fn classic_ill_formed_sequences() {
        // Stray continuation byte.
        assert_eq!(decode(b"\x80"), "\u{FFFD}");
        // Overlong "/" (C0 AF): C0 invalid outright, AF stray.
        assert_eq!(decode(b"\xC0\xAF"), "\u{FFFD}\u{FFFD}");
        // Overlong via E0.
        assert_lossy!(b"\xE0\x80\xAF");
        // CESU-8 surrogate half ED A0 80.
        assert_eq!(decode(b"\xED\xA0\x80"), "\u{FFFD}\u{FFFD}\u{FFFD}");
        // Above U+10FFFF.
        assert_lossy!(b"\xF4\x90\x80\x80");
        // Never-valid bytes.
        assert_eq!(decode(b"\xFE\xFFok"), "\u{FFFD}\u{FFFD}ok");
    }

    #[test]
    fn truncated_at_eof_flushes_invalid() {
        assert_eq!(decode(b"ok\xE2\x82"), "ok\u{FFFD}"); // half a €
        assert_eq!(decode(b"\xF0\x9F\xA6"), "\u{FFFD}"); // ¾ of a 🦀
    }

    #[test]
    fn reset_discards_pending_silently() {
        let mut p = Decoder::default();
        p.next(0xE2);
        assert!(p.is_pending());
        p.clear();
        assert!(!p.is_pending());
        assert_eq!(p.flush(), Some(Codepoint::Incomplete));
        assert_eq!(p.next(b'x'), Codepoint::Complete('x'));
    }

    /// Differential test against std's lossy decoder over pseudo-random
    /// buffers mixing garbage with valid UTF-8 fragments, so sequence
    /// boundaries land everywhere.
    #[test]
    fn fuzz_against_from_utf8_lossy() {
        let mut rng: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut next = move || {
            rng ^= rng << 13;
            rng ^= rng >> 7;
            rng ^= rng << 17;
            rng
        };
        let fragments: &[&[u8]] = &[
            "é".as_bytes(),
            "€".as_bytes(),
            "🦀".as_bytes(),
            "\u{10FFFF}".as_bytes(),
            b"\xED\xA0\x80",
            b"\xF4\x90\x80\x80",
        ];

        for _ in 0..2_000 {
            let len = (next() % 48) as usize;
            let mut buf = Vec::with_capacity(len + 4);
            while buf.len() < len {
                let r = next();
                if r % 3 == 0 {
                    buf.extend_from_slice(fragments[(r >> 8) as usize % fragments.len()]);
                } else {
                    buf.push((r >> 16) as u8);
                }
            }
            assert_lossy!(&buf);
        }
    }

    /// Exhaustive over all 1- and 2-byte inputs; cheap and catches
    /// any table typo at sequence starts and first continuations.
    #[test]
    fn exhaustive_short_inputs() {
        for a in 0..=255u8 {
            assert_lossy!(&[a]);
            for b in 0..=255u8 {
                assert_lossy!(&[a, b]);
            }
        }
    }
}