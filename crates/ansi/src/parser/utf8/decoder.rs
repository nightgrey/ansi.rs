//! Incremental UTF-8 decoder.
//! Based on Christian Hansen's `c-utf8` implementation.
//! https://github.com/chansen/c-utf8

use std::iter::FusedIterator;


use super::dfa::{State, step_decode};

/// One position in a lossy-decoded stream: either a well-formed Unicode scalar
/// or an ill-formed spot that a caller renders as `U+FFFD`.
///
/// Unlike [`Event`], this never carries "incomplete" — a byte that is merely
/// absorbed into an unfinished sequence produces no `Codepoint` at all (an
/// empty [`Chunk`] / a skipped step in [`Chunks`]).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Codepoint {
    /// A complete, well-formed Unicode scalar value.
    Scalar(char),
    /// An ill-formed maximal subpart — render as `U+FFFD`.
    Invalid,
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
}

impl Decoder {
    pub const EMPTY: Self = Self { state: State::ACCEPT, codepoint: 0 };
    /// A decoder in the ground state.
    #[inline]
    pub const fn new() -> Self {
        Decoder { state: State::ACCEPT, codepoint: 0 }
    }

    pub const fn is_empty(&self) -> bool {
        self == &Self::EMPTY
    }

    /// Returns `true` if bytes of an unfinished sequence have been absorbed.
    #[inline]
    pub const fn is_incomplete(&self) -> bool {
        // REJECT is never stored, so non-ACCEPT ⟺ mid-sequence.
        self.state != State::ACCEPT
    }

    /// Feed one byte, yielding the resulting [`Event`].
    ///
    /// If [`Event::Reprocess`] is returned, bytes need to be re-fed by the caller.
    pub fn next(&mut self, byte: u8) -> Event {
        let was_incomplete = self.is_incomplete();
        match self.decode(byte) {
            State::ACCEPT => Event::Complete(self.take()),
            State::REJECT => {
                // The byte triggered rejection. If it was
                // the first byte, it is itself the maximal subpart (length 1).
                // Otherwise the lead byte(s) already consumed form the maximal
                // subpart and the triggering byte belongs to the next sequence.
                self.clear();
                if was_incomplete { Event::Reprocess } else { Event::Invalid }
            }
            // The source ended before the current multi-byte sequence was complete.
            _ => Event::Incomplete,
        }
    }

    /// Feed one byte, yielding a [`Chunk`].
    ///
    /// Codepoints are automatically re-fed if necessary.
    #[inline]
    pub fn advance(&mut self, byte: u8) -> Chunk {
        match self.next(byte) {
            Event::Complete(c) => Chunk::single(Codepoint::Scalar(c)),

            // An invalid byte maps to a single replacement.
            Event::Invalid => Chunk::single(Codepoint::Invalid),

            // The byte was absorbed into an unfinished sequence: nothing to
            // emit yet. (Mirrors `Chunks`, which loops on `Incomplete`.)
            Event::Incomplete => Chunk::EMPTY,

            Event::Reprocess => {
                // The maximal subpart of the prior sequence is ill-formed:
                // emit one replacement, then reprocess this byte from the
                // ground state.
                let outcome = match self.next(byte) {
                    Event::Complete(c) => Some(Codepoint::Scalar(c)),
                    Event::Invalid => Some(Codepoint::Invalid),
                    // The byte begins a fresh (possibly multi-byte) sequence,
                    // so only the replacement is emitted now. A second
                    // `Reprocess` is unreachable from the ground state.
                    Event::Incomplete | Event::Reprocess => None,
                };
                Chunk::invalid_then(outcome)
            }
        }
    }

    /// Feed an iterator of bytes, yielding [`Chunks`].
    pub fn advances<I: Iterator<Item = u8>>(&mut self, bytes: I) -> Chunks<'_, I> {
        Chunks { decoder: self, iter: bytes, pending: None }
    }

    /// Finalizes the decoder state.
    ///
    /// Returns [`Event::Incomplete`] if any pending sequence was not completed, otherwise `None`.
    #[inline]
    pub fn flush(&mut self) -> Option<Event> {
        let pending = self.is_incomplete();
        self.clear();
        pending.then_some(Event::Incomplete)
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

#[derive(Copy, Debug)]
#[derive_const(Clone, PartialEq, Eq)]
pub enum Event {
    /// A complete, well-formed Unicode scalar value.
    Complete(char),
    /// An invalid byte was encountered.
    Invalid,
    /// Byte was consumed, and needs to be re-fed.
    Reprocess,
    /// Byte was consumed, but sequence is not complete yet.
    Incomplete,
}

/// The 0, 1, or 2 [`Codepoint`]s produced by feeding a single byte to
/// [`Decoder::advance`].
///
/// Feeding one byte yields at most two codepoints, and a two-codepoint result
/// is *always* `[Invalid, _]`: the only way to emit two is when the byte ends
/// an ill-formed maximal subpart (one replacement) and then begins a fresh
/// sequence that itself resolves. So the chunk is exactly an optional leading
/// replacement plus the outcome for the byte just fed.
#[derive(Copy, Clone, Debug)]
pub struct Chunk {
    /// A replacement emitted for a preceding ill-formed maximal subpart,
    /// yielded before `outcome`.
    leading_invalid: bool,
    /// The outcome for the byte just fed: a scalar, a replacement, or `None`
    /// when the byte was absorbed into an unfinished sequence.
    outcome: Option<Codepoint>,
}

impl Chunk {
    /// A chunk that yields nothing — the byte was absorbed but produced no
    /// codepoint (an unfinished sequence).
    pub const EMPTY: Self = Self { leading_invalid: false, outcome: None };

    /// A chunk yielding exactly `c`.
    const fn single(c: Codepoint) -> Self {
        Self { leading_invalid: false, outcome: Some(c) }
    }

    /// A chunk yielding a leading [`Codepoint::Invalid`] followed by `outcome`
    /// (which may be empty).
    const fn invalid_then(outcome: Option<Codepoint>) -> Self {
        Self { leading_invalid: true, outcome }
    }
}

impl Iterator for Chunk {
    type Item = Codepoint;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.leading_invalid {
            self.leading_invalid = false;
            return Some(Codepoint::Invalid);
        }
        self.outcome.take()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.leading_invalid as usize + self.outcome.is_some() as usize;
        (n, Some(n))
    }
}
impl ExactSizeIterator for Chunk {}
impl FusedIterator for Chunk {}



#[derive(Debug)]
pub struct Chunks<'d, I> {
    decoder: &'d mut Decoder,
    iter: I,
    pending: Option<u8>,
}

impl<'d, I: Iterator<Item = u8>> Iterator for Chunks<'d, I> {
    type Item = Codepoint;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let byte = match self.pending.take().or_else(|| self.iter.next()) {
                Some(b) => b,
                None => return None, // chunk exhausted — decoder may stay pending, by design
            };
            match self.decoder.next(byte) {
                Event::Complete(c) => return Some(Codepoint::Scalar(c)),
                Event::Incomplete => continue,
                Event::Invalid => return Some(Codepoint::Invalid),
                Event::Reprocess => {
                    self.pending = Some(byte);
                    return Some(Codepoint::Invalid);
                }
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // A run of incomplete bytes yields nothing, so the lower bound is 0.
        // Each input byte yields at most two codepoints (a replacement plus a
        // reprocessed scalar), and a stashed `pending` byte at most one more.
        let pending = self.pending.is_some() as usize;
        (0, self.iter.size_hint().1.map(|n| n * 2 + pending))
    }
}
impl<'d, I: Iterator<Item = u8>> FusedIterator for Chunks<'d, I> {}

#[cfg(test)]
mod tests {
    use super::*;

    /// Decode a whole buffer byte-by-byte, replacing each `Invalid`
    /// with U+FFFD — the reference behaviour of `from_utf8_lossy`.
    fn from_utf8_ours(bytes: &[u8]) -> String {
        let mut decoder = Decoder::new();
        let mut out = String::new();

        for codepoint in decoder.advances(bytes.iter().copied()) {
            match codepoint {
                Codepoint::Scalar(c) => out.push(c),
                Codepoint::Invalid => out.push(char::REPLACEMENT_CHARACTER),
            }
        }

        if let Some(Event::Incomplete) = decoder.flush() {
            out.push(char::REPLACEMENT_CHARACTER);
        }

        out
    }

    macro_rules! assert_decode {
        ($bytes:expr, $expected:expr) => {{
            let bytes = $bytes;
            let expected = $expected;
            let actual = from_utf8_ours(bytes);
            assert_eq!(
                actual,
                expected,
                "Expected {:?} but got {:?} for {:02X?}",
                expected.as_bytes(),
                actual.as_bytes(),
                bytes
            )
        }};
        ($bytes:expr) => {{
            assert_decode!($bytes, String::from_utf8_lossy($bytes));
        }};
    }

    #[test]
    fn ascii() {
       assert_decode!(b"Hello, World!");
    }

    #[test]
    fn multibyte_scalars() {
        // 2-, 3-, 4-byte sequences incl. BMP edges and astral plane.
        let s = "héllo → € ｗｉｄｅ 𝄞 🦀 \u{7FF}\u{800}\u{FFFF}\u{10000}\u{10FFFF}";
        assert_decode!((s.as_bytes()), s);
    }

    #[test]
    fn pending_across_feeds() {
        let mut p = Decoder::default();
        let crab = "🦀".as_bytes(); // F0 9F A6 80

        assert_eq!(p.next(crab[0]), Event::Incomplete);
        assert!(p.is_incomplete());
        assert_eq!(p.next(crab[1]), Event::Incomplete);
        assert_eq!(p.next(crab[2]), Event::Incomplete);
        assert_eq!(p.next(crab[3]), Event::Complete('🦀'));
        assert!(!p.is_incomplete());
    }

    #[test]
    fn unicode_table_3_11_maximal_subparts() {
        // TUS §3.9: 61 F1 80 80 E1 80 C2 62 → a, FFFD, FFFD, FFFD, b
        assert_eq!(
            from_utf8_ours(b"\x61\xF1\x80\x80\xE1\x80\xC2\x62"),
            "a\u{FFFD}\u{FFFD}\u{FFFD}b"
        );
    }

    #[test]
    fn advance_incomplete_yields_nothing() {
        // Feeding bytes of an unfinished sequence must not emit a (spurious)
        // replacement — `advance` yields an empty chunk until completion, then
        // the single scalar. Regression for `advance`/`advances` disagreeing on
        // the `Incomplete` case.
        let mut p = Decoder::default();
        let crab = "🦀".as_bytes(); // F0 9F A6 80

        for &b in &crab[..3] {
            assert_eq!(p.advance(b).count(), 0, "incomplete byte must yield nothing");
            assert!(p.is_incomplete());
        }
        assert_eq!(
            p.advance(crab[3]).collect::<Vec<_>>(),
            vec![Codepoint::Scalar('🦀')]
        );
    }

    #[test]
    fn advance_invalid_then_reprocess() {
        // C3 (2-byte lead) followed by 'A' (not a continuation): the lead is a
        // maximal subpart → one replacement, then 'A' decodes normally.
        let mut p = Decoder::default();
        assert_eq!(p.advance(0xC3).count(), 0);
        assert_eq!(
            p.advance(b'A').collect::<Vec<_>>(),
            vec![Codepoint::Invalid, Codepoint::Scalar('A')]
        );
    }

    #[test]
    fn chunks_all_incomplete_is_empty_with_zero_lower_bound() {
        // Three bytes of a 4-byte sequence produce no codepoints; the iterator
        // must be empty and its size_hint must promise a lower bound of 0.
        let mut p = Decoder::default();
        let mut it = p.advances(b"\xF0\x9F\xA6".iter().copied());
        assert_eq!(it.size_hint().0, 0);
        assert_eq!(it.by_ref().count(), 0);
    }

    #[test]
    fn classic_ill_formed_sequences() {
        // Stray continuation byte.
        assert_decode!((b"\x80"), "\u{FFFD}");
        // Overlong "/" (C0 AF): C0 invalid outright, AF stray.
        assert_decode!((b"\xC0\xAF"), "\u{FFFD}\u{FFFD}");
        // Overlong via E0.
        assert_decode!(b"\xE0\x80\xAF");
        // CESU-8 surrogate half ED A0 80.
        assert_decode!((b"\xED\xA0\x80"), "\u{FFFD}\u{FFFD}\u{FFFD}");
        // Above U+10FFFF.
        assert_decode!(b"\xF4\x90\x80\x80");
        // Never-valid bytes.
        assert_decode!((b"\xFE\xFFok"), "\u{FFFD}\u{FFFD}ok");
    }

    #[test]
    fn truncated_at_eof_flushes_invalid() {
        assert_decode!((b"ok\xE2\x82"), "ok\u{FFFD}"); // half a €
        assert_decode!((b"\xF0\x9F\xA6"), "\u{FFFD}"); // ¾ of a 🦀
    }

    #[test]
    fn reset_discards_pending_silently() {
        let mut p = Decoder::default();
        p.advance(0xE2);
        assert!(p.is_incomplete());
        p.clear();
        assert!(!p.is_incomplete());
        assert_eq!(p.flush(), None);
        assert_eq!(p.advance(b'x').next(), Some(Codepoint::Scalar('x')));
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
            assert_decode!(&buf);
        }
    }

    /// Exhaustive over all 1- and 2-byte inputs; cheap and catches
    /// any table typo at sequence starts and first continuations.
    #[test]
    fn exhaustive_short_inputs() {
        for a in 0..=255u8 {
            let vec = vec![a];
            assert_decode!(&vec);
            for b in 0..=255u8 {
                let vec = vec![a, b];
                assert_decode!(&vec);
            }
        }
    }
}