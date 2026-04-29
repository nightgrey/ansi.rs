use std::borrow::Cow;
use std::mem;
use memchr::{memchr, memchr3};
use strum::EnumCount;
use utils_derive::transitions;
use crate::parser::{pack, transition, unpack, Action, State};

transitions!(0, {
    Anywhere {
        0x18       => (Ignore, Ground),
        0x1a       => (Ignore, Ground),
        0x80..=0x8f => (Ignore, Ground),
        0x91..=0x97 => (Ignore, Ground),
        0x99       => (Ignore, Ground),
        0x9a       => (Ignore, Ground),
        0x9c       => (Ignore, Ground),
        0x1b       => (Ignore, Escape),
        0x98       => (Ignore, SosPmApcData),
        0x9e       => (Ignore, SosPmApcData),
        0x9f       => (Ignore, SosPmApcData),
        0x90       => (Ignore, DcsEntry),
        0x9d       => (Ignore, OscData),
        0x9b       => (Ignore, OscData),
    },

    None {

    },


    Ground {
        0x00..=0x17 => Ignore,
        0x19       => Ignore,
        0x1c..=0x1f => Ignore,
        0x20..=0x7f => (Print, Utf8),
        0xC2..=0xDF => (Print, Utf8), // UTF8 2 byte sequence
        0xE0..=0xEF => (Print, Utf8), // UTF8 3 byte sequence
        0xF0..=0xF4 => (Print, Utf8), // UTF8 4 byte sequence
    }

    Utf8 {
        0x80..=0xBF => (Print, Utf8), // Continuation byte
    }

    Escape {
        on_entry  => Ignore,
        0x00..=0x17 => Ignore,
        0x19       => Ignore,
        0x1c..=0x1f => Ignore,
        0x7f       => Ignore,
        0x20..=0x2f => (Ignore, EscapeIntermediate),
        0x30..=0x4f => (Ignore, Ground),
        0x51..=0x57 => (Ignore, Ground),
        0x59       => (Ignore, Ground),
        0x5a       => (Ignore, Ground),
        0x5c       => (Ignore, Ground),
        0x60..=0x7e => (Ignore, Ground),
        0x5b       => (Ignore, CsiEntry),
        0x5d       => (Ignore, OscData),
        0x50       => (Ignore, DcsEntry),
        0x58       => (Ignore, SosPmApcData),
        0x5e       => (Ignore, SosPmApcData),
        0x5f       => (Ignore, SosPmApcData),
    },

    EscapeIntermediate {
    0x00..=0x17 => Ignore,
    0x19       => Ignore,
    0x1c..=0x1f => Ignore,
    0x20..=0x2f => Ignore,
    0x7f       => Ignore,
    0x30..=0x7e => (Ignore, Ground)
    },

    CsiEntry {
        on_entry  => Ignore,
        0x00..=0x17 => Ignore,
        0x19       => Ignore,
        0x1c..=0x1f => Ignore,
        0x7f       => Ignore,
        0x20..=0x2f => (Ignore, CsiIntermediate),
        0x3a       => (Ignore, CsiIgnore),
        0x30..=0x39 => (Ignore, CsiParam),
        0x3b       => (Ignore, CsiParam),
        0x3c..=0x3f => (Ignore, CsiParam),
        0x40..=0x7e => (Ignore, Ground)
    },

    CsiParam {
        0x00..=0x17 => Ignore,
        0x19       => Ignore,
        0x1c..=0x1f => Ignore,
        0x30..=0x39 => Ignore,
        0x3b       => Ignore,
        0x7f       => Ignore,
        0x3c..=0x3f => (Ignore, CsiIgnore),
        0x3a        => Ignore,
        0x20..=0x2f => (Ignore, CsiIntermediate),
        0x40..=0x7e => (Ignore, Ground)
    },

    CsiIntermediate {
        0x00..=0x17 => Ignore,
        0x19       => Ignore,
        0x1c..=0x1f => Ignore,
        0x20..=0x2f => Ignore,
        0x7f       => Ignore,
        0x30..=0x3f => (Ignore, CsiIgnore),
        0x40..=0x7e => (Ignore, Ground),
    },

    CsiIgnore {
        0x00..=0x17 => Ignore,
        0x19       => Ignore,
        0x1c..=0x1f => Ignore,
        0x20..=0x3f => Ignore,
        0x7f       => Ignore,
        0x40..=0x7e => (Ignore, Ground),
    },

    DcsEntry {
        on_entry  => Ignore,
        0x00..=0x17 => Ignore,
        0x19       => Ignore,
        0x1c..=0x1f => Ignore,
        0x7f       => Ignore,
        0x3a       => (Ignore, DcsIgnore),
        0x20..=0x2f => (Ignore, DcsIntermediate),
        0x30..=0x39 => (Ignore, DcsParam),
        0x3b       => (Ignore, DcsParam),
        0x3c..=0x3f => (Ignore, DcsParam),
        0x40..=0x7e => (Ignore, DcsData)
    },


    DcsParam {
        0x00..=0x17 => Ignore,
        0x19       => Ignore,
        0x1c..=0x1f => Ignore,
        0x30..=0x39 => Ignore,
        0x3b       => Ignore,
        0x7f       => Ignore,
        0x3a       => (Ignore, DcsIgnore),
        0x3c..=0x3f => (Ignore, DcsIgnore),
        0x20..=0x2f => (Ignore, DcsIntermediate),
        0x40..=0x7e => (Ignore, DcsData)
    },

    DcsIntermediate {
        0x00..=0x17 => Ignore,
        0x19       => Ignore,
        0x1c..=0x1f => Ignore,
        0x20..=0x2f => Ignore,
        0x7f       => Ignore,
        0x30..=0x3f => (Ignore, DcsIgnore),
        0x40..=0x7e => (Ignore, DcsData)
    },

    DcsData {
        on_entry  => Ignore,
        0x00..=0x17 => Ignore,
        0x19       => Ignore,
        0x1c..=0x1f => Ignore,
        0x20..=0x7e => Ignore,
        0x7f       => Ignore,
        on_exit   => Ignore
    },

    DcsIgnore {
        0x00..=0x17 => Ignore,
        0x19       => Ignore,
        0x1c..=0x1f => Ignore,
        0x20..=0x7f => Ignore,
    },

    OscData {
        on_entry  => Ignore,
        0x00..=0x17 => Ignore,
        0x19       => Ignore,
        0x1c..=0x1f => Ignore,
        0x20..=0x7f => Ignore,
        on_exit   => Ignore
    }


    SosPmApcData {
        0x00..=0x17 => Ignore,
        0x19       => Ignore,
        0x1c..=0x1f => Ignore,
        0x20..=0x7f => Ignore,
    },
});
#[derive(Debug, Default)]
#[repr(transparent)]
pub struct StrippingParser(State);

impl StrippingParser {
    /// Create a new parser in the ground state.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self(State::None)
    }

    /// Reset the parser to the ground state.
    #[inline]
    pub fn reset(&mut self) {
        self.0 = State::None;
    }

    /// Returns the current parser state.
    #[inline]
    #[must_use]
    pub fn state(&self) -> State {
       self.0
    }

    /// Returns `true` if the parser is in the ground state.
    #[inline]
    #[must_use]
    pub const fn is_ground(&self) -> bool {
        self.0 == State::Ground
    }
    /// Returns `true` if the parser is in a passthrough body state
    /// (OSC string, DCS passthrough, or SOS/PM/APC string).
    ///
    /// These states consume arbitrary-length body bytes until a
    /// terminator. Callers can use `memchr` to skip directly to
    /// the terminator instead of feeding each byte individually.
    ///
    /// Branchless: single shift + AND + compare. No conditional
    /// branches — avoids branch predictor pollution in tight loops.
    #[inline]
    #[must_use]
    pub const fn is_passthrough(&self) -> bool {
        matches!(self.0, State::OscData | State::DcsData | State::SosPmApcData)
    }

    /// Feed a single byte through the state machine.
    ///
    /// Returns [`Action::Print`] if the byte is content,
    /// [`Action::Skip`] if it is part of an escape sequence.
    ///
    /// Ground state (the common case) is handled inline.
    /// All other states use a precomputed 15×256 lookup table
    /// (3840 bytes, fits in L1) for branchless transitions.
    #[inline]
    pub fn advance(&mut self, byte: u8) -> Action {
        // Fast path: ground state (~80%+ of bytes).
        if self.0 == State::None {
            if byte == 0x1B {
                self.0 = State::Escape;
                return Action::Ignore;
            }
            return Action::Print;
        }

        let transition = TRANSITIONS[self.0 as usize][byte as usize];
        unsafe {
            let action = mem::transmute::<u8, Action>(transition >> 4);
            let state = mem::transmute::<u8, State>(transition & 0x0F);
            println!("{:?} / 0x{:2x} | {:?} -> {:?} @ {:?}", byte as char, byte, self.0, if state == State::None { self.0 } else { state }, action);

            self.0 = state;
            action
        }
    }
}

pub(crate) fn passthrough_skip(state: State, remaining: &[u8]) -> usize {
    match state {
        // OSC: terminates on BEL(07), ESC(1B), CAN(18), SUB(1A).
        // memchr3 only takes 3 needles; find min of two searches.
        State::OscData => {
            let a = memchr3(0x07, 0x1B, 0x18, remaining);
            let b = memchr(0x1A, remaining);
            match (a, b) {
                (Some(x), Some(y)) => x.min(y),
                (Some(x), None) | (None, Some(x)) => x,
                (None, None) => 0,
            }
        }
        // DCS/String: terminates on ESC(1B), CAN(18), SUB(1A).
        State::DcsData | State::SosPmApcData => {
            memchr3(0x1B, 0x18, 0x1A, remaining).unwrap_or(0)
        }
        _ => 0,
    }
}

/// Strip ANSI escape sequences from a byte slice.
///
/// Returns `Cow::Borrowed` when no allocation is needed:
/// - No ESC bytes → borrowed input
/// - Only trailing escapes → borrowed prefix
/// - Only leading escapes → borrowed suffix
///
/// Returns `Cow::Owned` when escapes are interleaved with content.
#[must_use]
pub fn strip(input: &[u8]) -> Cow<'_, [u8]> {
    let Some(first_esc) = memchr(0x1B, input) else {
        return Cow::Borrowed(input);
    };

    // Speculative: are all bytes from first ESC onward part of escapes?
    let mut parser = StrippingParser::new();
    let mut first_emit = None;
    for (i, &b) in input[first_esc..].iter().enumerate() {
        if parser.advance(b) == Action::Print {
            first_emit = Some(first_esc + i);
            break;
        }
    }

    let Some(emit_pos) = first_emit else {
        return Cow::Borrowed(&input[..first_esc]);
    };

    // Leading escapes only?
    if first_esc == 0 && parser.is_ground() && memchr(0x1B, &input[emit_pos..]).is_none() {
        return Cow::Borrowed(&input[emit_pos..]);
    }

    // Full strip: memchr to skip ground bytes, parser for escapes.
    // Adaptive allocation: start at 80% of input (typical ANSI is
    // 10-30% of bytes). The Vec grows if needed but avoids the
    // full-input over-allocation that inflates RSS.
    let mut output = Vec::with_capacity(input.len() * 4 / 5);
    output.extend_from_slice(&input[..first_esc]);

    let mut remaining = &input[first_esc..];
    while !remaining.is_empty() {
        // memchr skip to next ESC — bulk copy ground bytes.
        let esc_pos = memchr(0x1B, remaining).unwrap_or(remaining.len());
        output.extend_from_slice(&remaining[..esc_pos]);
        remaining = &remaining[esc_pos..];
        if remaining.is_empty() {
            break;
        }

        // Parse the escape sequence.
        let mut p = StrippingParser::new();
        let mut i = 0;
        while i < remaining.len() {
            let action = p.advance(remaining[i]);
            i += 1;
            if action == Action::Print {
                output.push(remaining[i - 1]);
            }
            // After entering a passthrough state (OSC, DCS, SOS/PM/APC),
            // use memchr to skip directly to the terminator instead of
            // feeding each body byte through the state table.
            if p.is_passthrough() {
                i += passthrough_skip(p.state(), &remaining[i..]);
            }
            if p.is_ground() {
                break;
            }
        }
        remaining = &remaining[i..];
    }

    Cow::Owned(output)
}

#[cfg(test)]
mod tests {
    use super::*;


        #[test]
        fn test_strip() {
            let input = b"\x1b[31mHello\x1b[0m World!";
            dbg!(str::from_utf8(&strip(input)));
        }
}