/*
 * Copyright (c) 2026 Christian Hansen <chansen@cpan.org>
 * <https://github.com/chansen/c-utf8>
 *
 * Ported to Rust with the same MIT license terms.
 */
use std::fmt::{Debug, Formatter};

/// DFA state type
#[derive(Copy, Default)]
#[derive_const(Clone, PartialEq, Eq)]
pub struct State(u64);

impl const State {
    /// Error state (absorbing trap)
    pub const REJECT: Self = Self(0);

    /// Start state / valid sequence boundary
    pub const ACCEPT: Self = Self(6);

    pub const TAIL1: Self = Self(12);
    pub const TAIL2: Self = Self(18);
    pub const TAIL3: Self = Self(24);
    pub const E0: Self = Self(30);
    pub const ED: Self = Self(36);
    pub const F0: Self = Self(42);
    pub const F4: Self = Self(48);

    /// Returns true if the state is ACCEPT
    pub fn is_accept(self) -> bool {
        self.0 == Self::ACCEPT.0
    }

    /// Returns true if the state is REJECT (error)
    pub fn is_reject(self) -> bool {
        self.0 == Self::REJECT.0
    }

    pub fn is_pending(self) -> bool {
        self.0 > 6
    }
}
impl Debug for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self  {
            Self::ACCEPT => f.write_str("State::ACCEPT"),
            Self::REJECT => f.write_str("State::REJECT"),
            Self::TAIL1 => f.write_str("State::TAIL1"),
            Self::TAIL2 => f.write_str("State::TAIL2"),
            Self::TAIL3 => f.write_str("State::TAIL3"),
            Self::E0 => f.write_str("State::E0"),
            Self::ED => f.write_str("State::ED"),
            Self::F0 => f.write_str("State::F0"),
            Self::F4 => f.write_str("State::F4"),
            _ => f.debug_tuple("State").field(&self.0).finish(),
        }
    }
}

// State shift values (multiples of 6)
const S_ERROR: u64 = 0;
const S_ACCEPT: u64 = 6;
const S_TAIL1: u64 = 12;
const S_TAIL2: u64 = 18;
const S_TAIL3: u64 = 24;
const S_E0: u64 = 30;
const S_ED: u64 = 36;
const S_F0: u64 = 42;
const S_F4: u64 = 48;


/// Create a DFA table row from state transitions and a payload mask
macro_rules! dfa_row {
($accept:expr,$error:expr,$tail1:expr,$tail2:expr,$tail3:expr,$e0:expr,$ed:expr,$f0:expr,$f4:expr,$mask:expr) => {
        ($accept << S_ACCEPT)
        | ($error << S_ERROR)
        | ($tail1 << S_TAIL1)
        | ($tail2 << S_TAIL2)
        | ($tail3 << S_TAIL3)
        | ($e0 << S_E0)
        | ($ed << S_ED)
        | ($f0 << S_F0)
        | ($f4 << S_F4)
        | (($mask & 0x7F) << 56)
    };
}

// Error transition shorthand
const ERR: u64 = S_ERROR;

const ASCII_ROW: u64 = dfa_row!(S_ACCEPT,ERR,ERR,ERR,ERR,ERR,ERR,ERR,ERR,0x7F);
const LEAD2_ROW: u64 = dfa_row!(S_TAIL1,ERR,ERR,ERR,ERR,ERR,ERR,ERR,ERR,0x1F);
const LEAD3_ROW: u64 = dfa_row!(S_TAIL2,ERR,ERR,ERR,ERR,ERR,ERR,ERR,ERR,0x0F);
const LEAD4_ROW: u64 = dfa_row!(S_TAIL3,ERR,ERR,ERR,ERR,ERR,ERR,ERR,ERR,0x07);
const ERROR_ROW: u64 = dfa_row!(ERR,ERR,ERR,ERR,ERR,ERR,ERR,ERR,ERR,0x00);

const E0_ROW: u64 = dfa_row!(S_E0,ERR,ERR,ERR,ERR,ERR,ERR,ERR,ERR,0x0F);
const ED_ROW: u64 = dfa_row!(S_ED,ERR,ERR,ERR,ERR,ERR,ERR,ERR,ERR,0x0F);
const F0_ROW: u64 = dfa_row!(S_F0,ERR,ERR,ERR,ERR,ERR,ERR,ERR,ERR,0x07);
const F4_ROW: u64 = dfa_row!(S_F4,ERR,ERR,ERR,ERR,ERR,ERR,ERR,ERR,0x07);

const CONT_80_8F: u64 =  dfa_row!(ERR,ERR,S_ACCEPT,S_TAIL1,S_TAIL2,ERR,    S_TAIL1,ERR,     S_TAIL2,0x3F);
const CONT_90_9F: u64 =  dfa_row!(ERR,ERR,S_ACCEPT,S_TAIL1,S_TAIL2,ERR,    S_TAIL1,S_TAIL2, ERR,0x3F);
const CONT_A0_BF: u64 =  dfa_row!(ERR,ERR,S_ACCEPT,S_TAIL1,S_TAIL2,S_TAIL1,ERR,    S_TAIL2, ERR,0x3F);
/// The DFA transition table (256 entries × u64 = 2 KB)
pub static UTF8_DFA: [u64; 256] = [
    // 00-7F
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    ASCII_ROW,ASCII_ROW,ASCII_ROW,ASCII_ROW,
    // 80-8F
    CONT_80_8F,CONT_80_8F,CONT_80_8F,CONT_80_8F,
    CONT_80_8F,CONT_80_8F,CONT_80_8F,CONT_80_8F,
    CONT_80_8F,CONT_80_8F,CONT_80_8F,CONT_80_8F,
    CONT_80_8F,CONT_80_8F,CONT_80_8F,CONT_80_8F,
    // 90-9F
    CONT_90_9F,CONT_90_9F,CONT_90_9F,CONT_90_9F,
    CONT_90_9F,CONT_90_9F,CONT_90_9F,CONT_90_9F,
    CONT_90_9F,CONT_90_9F,CONT_90_9F,CONT_90_9F,
    CONT_90_9F,CONT_90_9F,CONT_90_9F,CONT_90_9F,
    // A0-BF
    CONT_A0_BF,CONT_A0_BF,CONT_A0_BF,CONT_A0_BF,
    CONT_A0_BF,CONT_A0_BF,CONT_A0_BF,CONT_A0_BF,
    CONT_A0_BF,CONT_A0_BF,CONT_A0_BF,CONT_A0_BF,
    CONT_A0_BF,CONT_A0_BF,CONT_A0_BF,CONT_A0_BF,
    CONT_A0_BF,CONT_A0_BF,CONT_A0_BF,CONT_A0_BF,
    CONT_A0_BF,CONT_A0_BF,CONT_A0_BF,CONT_A0_BF,
    CONT_A0_BF,CONT_A0_BF,CONT_A0_BF,CONT_A0_BF,
    CONT_A0_BF,CONT_A0_BF,CONT_A0_BF,CONT_A0_BF,
    // C0-C1
    ERROR_ROW,ERROR_ROW,
    // C2-DF
    LEAD2_ROW,LEAD2_ROW,LEAD2_ROW,LEAD2_ROW,
    LEAD2_ROW,LEAD2_ROW,LEAD2_ROW,LEAD2_ROW,
    LEAD2_ROW,LEAD2_ROW,LEAD2_ROW,LEAD2_ROW,
    LEAD2_ROW,LEAD2_ROW,LEAD2_ROW,LEAD2_ROW,
    LEAD2_ROW,LEAD2_ROW,LEAD2_ROW,LEAD2_ROW,
    LEAD2_ROW,LEAD2_ROW,LEAD2_ROW,LEAD2_ROW,
    LEAD2_ROW,LEAD2_ROW,LEAD2_ROW,LEAD2_ROW,
    LEAD2_ROW,LEAD2_ROW,
    // E0
    E0_ROW,
    // E1-EC
    LEAD3_ROW,LEAD3_ROW,LEAD3_ROW,LEAD3_ROW,
    LEAD3_ROW,LEAD3_ROW,LEAD3_ROW,LEAD3_ROW,
    LEAD3_ROW,LEAD3_ROW,LEAD3_ROW,LEAD3_ROW,
    // ED
    ED_ROW,
    // EE-EF
    LEAD3_ROW,LEAD3_ROW,
    // F0
    F0_ROW,
    // F1-F3
    LEAD4_ROW,LEAD4_ROW,LEAD4_ROW,
    // F4
    F4_ROW,
    // F5-FF
    ERROR_ROW,ERROR_ROW,ERROR_ROW,ERROR_ROW,
    ERROR_ROW,ERROR_ROW,ERROR_ROW,ERROR_ROW,
    ERROR_ROW,ERROR_ROW,ERROR_ROW,
];

/// Process a single byte through the DFA (validation only)
#[inline]
pub fn step(state: State, byte: u8) -> State {
    State((UTF8_DFA[byte as usize] >> state.0) & 63)
}

/// Process a single byte through the DFA with codepoint decoding
///
/// On each call, accumulates payload bits into `codepoint` left-to-right.
/// When the returned state is `ACCEPT`, `codepoint` contains the complete
/// decoded codepoint. The caller must reset `codepoint` to 0 at the start
/// of each new sequence (whenever the previous step returned `ACCEPT` or `REJECT`).
#[inline]
pub fn step_decode(state: State, byte: u8, codepoint: &mut u32) -> State {
    let row = UTF8_DFA[byte as usize];
    *codepoint = (*codepoint << 6) | (byte as u32 & ((row >> 56) as u32));
    State((row >> state.0) & 63)
}

/// Validate a complete UTF-8 byte sequence
#[inline]
pub fn validate(state: State, bytes: &[u8]) -> State {
    let mut s = state;
    for &byte in bytes {
        s = State((UTF8_DFA[byte as usize] >> (s.0 & 63)) & 63);
    }
    State(s.0 & 63)
}

/// Validate a fixed 16-byte chunk (with loop unrolling hint)
#[inline]
pub fn validate_chunk(state: State, bytes: &[u8; 16]) -> State {
    let mut s = state;
    for i in 0..16 {
        s = State((UTF8_DFA[bytes[i] as usize] >> (s.0 & 63)) & 63);
    }
    State(s.0 & 63)
}

/// Parallel validation of a byte slice using dual-thread scanning
///
/// Splits the input at a valid sequence boundary (midpoint that's not
/// a continuation byte) and validates both halves concurrently.
/// Returns `REJECT` if either half contains invalid UTF-8.
pub fn validate_dual(state: State, bytes: &[u8]) -> State {
    let len = bytes.len();
    let mut mid = len / 2;

    // Find a sequence boundary
    while mid > 0 && (bytes[mid] & 0xC0) == 0x80 {
        mid -= 1;
    }

    let mut s0 = state;
    let mut s1 = State::ACCEPT;

    // Process both halves in parallel
    let end = mid.min(len - mid);
    for (i, j) in (0..end).zip(mid..) {
        s0 = State((UTF8_DFA[bytes[i] as usize] >> (s0.0 & 63)) & 63);
        s1 = State((UTF8_DFA[bytes[j] as usize] >> (s1.0 & 63)) & 63);
    }

    // Process remaining bytes
    for &byte in &bytes[end..mid] {
        s0 = State((UTF8_DFA[byte as usize] >> (s0.0 & 63)) & 63);
    }
    for &byte in &bytes[mid + end..] {
        s1 = State((UTF8_DFA[byte as usize] >> (s1.0 & 63)) & 63);
    }

    if s0.0 & 63 != S_ACCEPT {
        return State::REJECT;
    }
    State(s1.0 & 63)
}

/// Parallel validation using triple-thread scanning
pub fn validate_triple(state: State, bytes: &[u8]) -> State {
    let len = bytes.len();
    let mut m0 = len / 3;
    let mut m1 = len * 2 / 3;

    // Find sequence boundaries
    while m0 > 0 && (bytes[m0] & 0xC0) == 0x80 {
        m0 -= 1;
    }
    while m1 > m0 && (bytes[m1] & 0xC0) == 0x80 {
        m1 -= 1;
    }

    let len0 = m0;
    let len1 = m1 - m0;
    let len2 = len - m1;
    let n = len0.min(len1).min(len2);

    let mut s0 = state;
    let mut s1 = State::ACCEPT;
    let mut s2 = State::ACCEPT;

    // Process all three chunks in parallel
    for i in 0..n {
        s0 = State((UTF8_DFA[bytes[i] as usize] >> (s0.0 & 63)) & 63);
        s1 = State((UTF8_DFA[bytes[m0 + i] as usize] >> (s1.0 & 63)) & 63);
        s2 = State((UTF8_DFA[bytes[m1 + i] as usize] >> (s2.0 & 63)) & 63);
    }

    // Process remaining bytes in each chunk
    for &byte in &bytes[n..len0] {
        s0 = State((UTF8_DFA[byte as usize] >> (s0.0 & 63)) & 63);
    }
    for &byte in &bytes[m0 + n..m1] {
        s1 = State((UTF8_DFA[byte as usize] >> (s1.0 & 63)) & 63);
    }
    for &byte in &bytes[m1 + n..] {
        s2 = State((UTF8_DFA[byte as usize] >> (s2.0 & 63)) & 63);
    }

    if (s0.0 & 63) != S_ACCEPT || (s1.0 & 63) != S_ACCEPT {
        return State::REJECT;
    }
    State(s2.0 & 63)
}

#[cfg(test)]
mod tests {
    use crate::parser::models::utf8::decoder::Decoder;
    use super::*;

    #[test]
    fn test_valid_ascii() {
        let mut state = State::ACCEPT;
        for &byte in b"Hello, World!" {
            state = step(state, byte);
            assert!(!state.is_reject());
        }
        assert!(state.is_accept());
    }

    #[test]
    fn test_valid_utf8() {
        let bytes = "Hello, 世界! 😀".as_bytes();
        let mut state = State::ACCEPT;
        for &byte in bytes {
            state = step(state, byte);
            assert!(!state.is_reject());
        }
        assert!(state.is_accept());
    }

    #[test]
    fn test_decode() {
        let bytes = "A世".as_bytes();
        let mut state = State::ACCEPT;
        let mut codepoint = 0u32;
        let mut codepoints = Vec::new();

        for &byte in bytes {
            state = step_decode(state, byte, &mut codepoint);
            if state.is_accept() {
                codepoints.push(codepoint);
                codepoint = 0;
            }
        }
        assert_eq!(codepoints, vec![0x41, 0x4E16]);
    }

    #[test]
    fn test111() {
        let bytes = "é".as_bytes();
        let mut p = Decoder::default();
        dbg!(bytes.iter().copied().map(|b| p.next(b)).take(4).collect::<Vec<_>>());

    }

    #[test]
    fn test_validate_slice() {
        assert!(validate(State::ACCEPT, b"Hello").is_accept());
        assert!(validate(State::ACCEPT, b"Hello, \xF0\x9F\x98\x80!").is_accept());
    }

    #[test]
    fn test_invalid_utf8() {
        // Invalid continuation byte
        let mut state = State::ACCEPT;
        state = step(state, 0xC2);
        state = step(state, 0x7F); // Should be 0x80-0xBF
        assert!(state.is_reject());

        // Overlong sequence
        state = State::ACCEPT;
        state = step(state, 0xE0);
        state = step(state, 0x9F); // Should be 0xA0-0xBF
        assert!(state.is_reject());
    }

    #[test]
    fn test_validate_dual() {
        let valid = "Hello, World! This is a test of parallel validation.".as_bytes();
        assert!(validate_dual(State::ACCEPT, valid).is_accept());
    }
}