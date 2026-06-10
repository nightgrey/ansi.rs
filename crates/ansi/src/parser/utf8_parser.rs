//! Incremental, table-driven (DFA) UTF-8 decoder.
//!
//! Based on Björn Höhrmann's "Flexible and Economical UTF-8 Decoder"
//! (<https://bjoern.hoehrmann.de/utf-8/decoder/dfa/>), extended with
//! the Unicode-recommended *maximal subpart* error policy
//! (TUS §3.9, "U+FFFD Substitution of Maximal Subparts"). A byte
//! stream therefore decodes to exactly the same scalar/replacement
//! sequence as [`String::from_utf8_lossy`] / the WHATWG decoder —
//! just one byte at a time, with no buffering and no allocation.
//!
//! The parser is a plain 5-byte value (`Copy`, `Default`), uses only
//! `core`, and is designed to be embedded in a larger byte-level
//! state machine such as an ANSI/VT parser.
//!
//! # Model
//!
//! Feeding a byte with [`Parser::advance`] yields zero, one, or
//! two [`Event`]s:
//!
//! * **zero** — the byte was absorbed; a sequence is now *pending*
//!   (check with [`Parser::is_pending`]).
//! * **one** — a completed scalar ([`Event::Char`]) or a malformed
//!   maximal subpart ([`Event::Invalid`], i.e. one `U+FFFD` if you
//!   are doing replacement).
//! * **two** — a byte both *terminated* an ill-formed sequence and
//!   *started* something new. E.g. in `F1 80 80 41`, the `41` closes
//!   the truncated 4-byte sequence (`Invalid`) and is itself ASCII
//!   (`Char('A')`). The re-processing is handled internally; callers
//!   never re-feed bytes.
//!
//! At end of input (or wherever the embedding parser wants to cut a
//! sequence short, e.g. on `ESC`), call [`Parser::flush`]: a
//! dangling partial sequence is reported as one final `Invalid`.
//!
//! # Example
//!
//! ```
//! use cratee::{Event, Parser};
//!
//! let mut p = Parser::default();
//! let mut out = String::new();
//!
//! for &byte in b"a\xF0\x9F\xA6\x80b\x80" {
//!     for event in p.advance(byte) {
//!         match event {
//!             Event::Char(c) => out.push(c),
//!             Event::Invalid => out.push(char::REPLACEMENT_CHARACTER),
//!         }
//!     }
//! }
//! if p.flush() == Some(Event::Invalid) {
//!     out.push(char::REPLACEMENT_CHARACTER);
//! }
//!
//! assert_eq!(out, "a🦀b\u{FFFD}");
//! ```

use std::ptr::NonNull;
use derive_more::IntoIterator;
use maybe::Maybe;

/// A decoding status.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Status {
    /// Byte was consumed, but scalar is not complete yet.
    Pending,
    /// A complete, well-formed Unicode scalar value.
    Char(char),
    /// A maximal subpart of an ill-formed sequence. Emit one
    /// `U+FFFD` per `Invalid` to get `from_utf8_lossy` semantics.
    Invalid,
    /// Byte was not consumed and should be re-fed to continue.
    Reprocess,
}

/// A decoding event.
#[derive(Copy, Debug, Hash)]
#[derive_const(Clone, PartialEq, Eq)]
pub enum Event {
    /// A complete, well-formed Unicode scalar value.
    Char(char),
    /// A maximal subpart of an ill-formed sequence. Emit one
    /// `U+FFFD` per `Invalid` to get `from_utf8_lossy` semantics.
    Invalid,
}

/// Iterator over the 0–2 events produced by feeding a single byte.
///
/// Returned by value, fixed-size, allocation-free.
#[derive(Copy, Debug)]
#[derive_const(Clone, PartialEq, Eq)]
pub struct Events([Option<Event>; 2]);

impl const Events {
    const NONE: Self = Self([None, None]);

    pub fn is_none(&self) -> bool {
        self == &Self::NONE
    }

    pub fn is_pending(&self) -> bool {
        self.is_none()
    }

    #[inline]
    fn none() -> Self {
        Self::NONE
    }
    #[inline]
    fn single(a: Event) -> Self {
        Events([Some(a), None])
    }
    #[inline]
    fn multiple(a: Event, b: Event) -> Self {
        Events([Some(a), Some(b)])
    }
}

impl Iterator for Events {
    type Item = Event;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let head = self.0[0].take();
        self.0.swap(0, 1);
        head
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.0.iter().flatten().count();
        (n, Some(n))
    }
}

impl ExactSizeIterator for Events {}
impl core::iter::FusedIterator for Events {}

// ---------------------------------------------------------------------------
// DFA tables (Höhrmann)
//
// Every byte maps to one of 12 character classes; the transition table
// is indexed by `state + class`. States are pre-multiplied by 12 so the
// hot path is exactly two loads and an add.
// ---------------------------------------------------------------------------

const ACCEPT: u8 = 0; // start / "scalar completed"
const REJECT: u8 = 12; // ill-formed

#[rustfmt::skip]
static CLASSES: [u8; 256] = [
    // 0x00..=0x7F: ASCII ............................................. class 0
    0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,  0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,  0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,  0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,  0,0,0,0,0,0,0,0, 0,0,0,0,0,0,0,0,
    // 0x80..=0x8F: continuation ...................................... class 1
    1,1,1,1,1,1,1,1, 1,1,1,1,1,1,1,1,
    // 0x90..=0x9F: continuation (distinct for F0/F4 range checks) ..... class 9
    9,9,9,9,9,9,9,9, 9,9,9,9,9,9,9,9,
    // 0xA0..=0xBF: continuation (distinct for E0/ED range checks) ..... class 7
    7,7,7,7,7,7,7,7, 7,7,7,7,7,7,7,7,  7,7,7,7,7,7,7,7, 7,7,7,7,7,7,7,7,
    // 0xC0..=0xC1: overlong lead (always invalid) ..................... class 8
    // 0xC2..=0xDF: 2-byte lead ........................................ class 2
    8,8,2,2,2,2,2,2, 2,2,2,2,2,2,2,2,  2,2,2,2,2,2,2,2, 2,2,2,2,2,2,2,2,
    // 0xE0: 3-byte lead, A0..BF only (10) | 0xE1..=0xEC: 3-byte (3)
    // 0xED: 3-byte, 80..9F only — excludes surrogates (4) | 0xEE..=0xEF (3)
    10,3,3,3,3,3,3,3, 3,3,3,3,3,4,3,3,
    // 0xF0: 4-byte, 90..BF (11) | 0xF1..=0xF3 (6) | 0xF4: 80..8F (5)
    // 0xF5..=0xFF: invalid (8)
    11,6,6,6,5,8,8,8, 8,8,8,8,8,8,8,8,
];

#[rustfmt::skip]
static TRANSITIONS: [u8; 108] = [
    // class:  0   1   2   3   4   5   6   7   8   9  10  11
    /* s0  */  0, 12, 24, 36, 60, 96, 84, 12, 12, 12, 48, 72, // ACCEPT
    /* s12 */ 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, // REJECT
    /* s24 */ 12,  0, 12, 12, 12, 12, 12,  0, 12,  0, 12, 12, // 1 continuation left
    /* s36 */ 12, 24, 12, 12, 12, 12, 12, 24, 12, 24, 12, 12, // 2 continuations left
    /* s48 */ 12, 12, 12, 12, 12, 12, 12, 24, 12, 12, 12, 12, // after E0 (A0..BF)
    /* s60 */ 12, 24, 12, 12, 12, 12, 12, 12, 12, 24, 12, 12, // after ED (80..9F)
    /* s72 */ 12, 12, 12, 12, 12, 12, 12, 36, 12, 36, 12, 12, // after F0 (90..BF)
    /* s84 */ 12, 36, 12, 12, 12, 12, 12, 36, 12, 36, 12, 12, // after F1..F3
    /* s96 */ 12, 36, 12, 12, 12, 12, 12, 12, 12, 12, 12, 12, // after F4 (80..8F)
];

/// Incremental UTF-8 parser.
///
/// `Default` is the ground state. The whole parser is 5 bytes and
/// `Copy`, so an embedding state machine can snapshot or reset it
/// freely.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Parser {
    state: u8,
    codepoint: u32,
}

impl Parser {
    /// A parser in the ground state.
    #[inline]
    pub const fn new() -> Self {
        Parser { state: ACCEPT, codepoint: 0 }
    }

    /// `true` if bytes of an unfinished sequence have been absorbed.
    #[inline]
    pub const fn is_pending(&self) -> bool {
        self.state != ACCEPT
    }

    /// Drop any pending state and return to ground.
    ///
    /// Unlike [`flush`](Self::flush) this silently discards a partial
    /// sequence — useful when an outer parser preempts decoding.
    #[inline]
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Signal end of input. Returns `Some(Event::Invalid)` if a
    /// partial sequence was pending (it can never be completed),
    /// `None` otherwise. The parser is back in the ground state
    /// afterwards either way.
    #[inline]
    pub fn flush(&mut self) -> Option<Event> {
        let pending = self.is_pending();
        self.reset();
        pending.then_some(Event::Invalid)
    }

    /// Advance, yielding an [`Events`] state.
    ///
    /// An empty [`Events`] means the byte was absorbed into a pending
    /// sequence.
    ///
    /// Note: Invalid bytes never need to be re-fed by the caller. Maximal-subpart processing
    /// happens internally.
    #[inline]
    pub fn advance(&mut self, byte: u8) -> Events {
        let was_pending = self.is_pending();

        match self.transition(byte) {
            ACCEPT => Events::single(Event::Char(self.take())),
            REJECT if !was_pending => {
                // The byte is invalid on its own (stray continuation,
                // C0/C1, F5..FF). Consumed; one maximal subpart.
                self.reset();
                Events::single(Event::Invalid)
            }
            REJECT => {
                // The bytes absorbed so far form a maximal subpart;
                // `byte` is *not* part of it. Report the error, then
                // re-run `byte` from the ground state.
                self.reset();
                match self.transition(byte) {
                    ACCEPT => Events::multiple(Event::Invalid, Event::Char(self.take())),
                    REJECT => {
                        self.reset();
                        Events::multiple(Event::Invalid, Event::Invalid)
                    }
                    _ => Events::single(Event::Invalid), // byte starts a new sequence
                }
            }
            _ => Events::none(), // mid-sequence, pending
        }
    }

    /// Feed one byte, yielding the resulting [`Status`].
    ///
    /// If [`Status::Unhandled`] is returned, bytes need to be re-fed by the caller.
    #[inline]
    pub fn advance_one(&mut self, byte: u8) -> Status {
        let was_pending = self.is_pending();

        match self.transition(byte) {
            ACCEPT => Status::Char(self.take()),
            REJECT => {
                self.reset();
                if !was_pending {
                    Status::Invalid
                } else {
                    Status::Reprocess
                }
            }
            _ => Status::Pending,
        }
    }

    /// One raw DFA transition: classify, fold into the codepoint
    /// accumulator, advance the state. Two table loads, no branches
    /// beyond the accumulator select.
    #[inline(always)]
    fn transition(&mut self, byte: u8) -> u8 {
        let class = CLASSES[byte as usize];
        self.codepoint = if self.state == ACCEPT {
            // Lead byte: mask off the length prefix. `0xFF >> class`
            // happens to be the right mask for every lead class.
            (0xFFu32 >> class) & u32::from(byte)
        } else {
            // Continuation byte: shift in its low 6 bits.
            (self.codepoint << 6) | u32::from(byte & 0x3F)
        };
        self.state = TRANSITIONS[usize::from(self.state + class)];
        self.state
    }

    /// Extract the completed scalar. Only called when the DFA is in
    /// ACCEPT after consuming ≥1 byte.
    #[inline(always)]
    fn take(&self) -> char {
        debug_assert!(char::from_u32(self.codepoint).is_some());
        // SAFETY: the DFA accepts exactly the well-formed UTF-8
        // sequences of TUS Table 3-7; surrogates (via the ED state)
        // and values above U+10FFFF (via the F4 state and F5..FF
        // class) are unreachable, so `codepoint` is a scalar value.
        unsafe { char::from_u32_unchecked(self.codepoint) }
    }
}

// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Decode a whole buffer byte-by-byte, replacing each `Invalid`
    /// with U+FFFD — the reference behaviour of `from_utf8_lossy`.
    fn lossy(bytes: &[u8]) -> String {
        let mut parser = Parser::default();
        let mut out = String::new();
        for &byte in bytes {
            for event in parser.advance(byte) {
                match event {
                    Event::Char(c) => out.push(c),
                    Event::Invalid => out.push(char::REPLACEMENT_CHARACTER),
                }
            }
        }
        if parser.flush() == Some(Event::Invalid) {
            out.push(char::REPLACEMENT_CHARACTER);
        }
        out
    }

    macro_rules! assert_lossy {
        ($bytes:expr) => {{
            let bytes = $bytes;
            assert_eq!(
                lossy(bytes),
                String::from_utf8_lossy(bytes),
                "diverged from std on {:02X?}",
                bytes
            )
        }};
    }

    #[test]
    fn ascii() {
        assert_eq!(lossy(b"hello, world"), "hello, world");
    }

    #[test]
    fn multibyte_scalars() {
        // 2-, 3-, 4-byte sequences incl. BMP edges and astral plane.
        let s = "héllo → € ｗｉｄｅ 𝄞 🦀 \u{7FF}\u{800}\u{FFFF}\u{10000}\u{10FFFF}";
        assert_eq!(lossy(s.as_bytes()), s);
    }

    #[test]
    fn pending_across_feeds() {
        let mut p = Parser::default();
        let crab = "🦀".as_bytes(); // F0 9F A6 80

        assert_eq!(p.advance(crab[0]).next(), None);
        assert!(p.is_pending());
        assert_eq!(p.advance(crab[1]).next(), None);
        assert_eq!(p.advance(crab[2]).next(), None);
        assert_eq!(p.advance(crab[3]).next(), Some(Event::Char('🦀')));
        assert!(!p.is_pending());
    }

    #[test]
    fn unicode_table_3_11_maximal_subparts() {
        // TUS §3.9: 61 F1 80 80 E1 80 C2 62 → a, FFFD, FFFD, FFFD, b
        assert_eq!(
            lossy(b"\x61\xF1\x80\x80\xE1\x80\xC2\x62"),
            "a\u{FFFD}\u{FFFD}\u{FFFD}b"
        );
    }

    #[test]
    fn invalid_then_char_in_one_advance() {
        let mut p = Parser::default();
        assert_eq!(p.advance(0xC3).next(), None); // pending 2-byte seq
        let mut steps = p.advance(b'A'); // not a continuation
        assert_eq!(steps.len(), 2);
        assert_eq!(steps.next(), Some(Event::Invalid));
        assert_eq!(steps.next(), Some(Event::Char('A')));
        assert_eq!(steps.next(), None);
    }

    #[test]
    fn classic_ill_formed_sequences() {
        // Stray continuation byte.
        assert_eq!(lossy(b"\x80"), "\u{FFFD}");
        // Overlong "/" (C0 AF): C0 invalid outright, AF stray.
        assert_eq!(lossy(b"\xC0\xAF"), "\u{FFFD}\u{FFFD}");
        // Overlong via E0.
        assert_lossy!(b"\xE0\x80\xAF");
        // CESU-8 surrogate half ED A0 80.
        assert_eq!(lossy(b"\xED\xA0\x80"), "\u{FFFD}\u{FFFD}\u{FFFD}");
        // Above U+10FFFF.
        assert_lossy!(b"\xF4\x90\x80\x80");
        // Never-valid bytes.
        assert_eq!(lossy(b"\xFE\xFFok"), "\u{FFFD}\u{FFFD}ok");
    }

    #[test]
    fn truncated_at_eof_flushes_invalid() {
        assert_eq!(lossy(b"ok\xE2\x82"), "ok\u{FFFD}"); // half a €
        assert_eq!(lossy(b"\xF0\x9F\xA6"), "\u{FFFD}"); // ¾ of a 🦀
    }

    #[test]
    fn reset_discards_pending_silently() {
        let mut p = Parser::default();
        p.advance(0xE2);
        assert!(p.is_pending());
        p.reset();
        assert!(!p.is_pending());
        assert_eq!(p.flush(), None);
        assert_eq!(p.advance(b'x').next(), Some(Event::Char('x')));
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