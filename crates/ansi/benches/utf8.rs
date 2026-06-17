//! Throughput comparison of incremental, lossy UTF-8 decoders.
//!
//! The decoder is the hot inner loop of the parser's ground path: every byte of
//! printable text flows through it before the parser can hand a run to the
//! handler. These benches probe it from three angles, each across a spread of
//! corpora (pure ASCII, Western text, CJK+emoji, mixed-with-garbage, and
//! adversarial), so the happy path and the lossy path are both visible:
//!
//! * [`fold`]   — raw decode throughput: fold each scalar (and `U+FFFD` per
//!                ill-formed maximal subpart) into an accumulator. No
//!                allocation; pure DFA/branch cost.
//! * [`lossy`]  — materialize a lossy `String`, which is what a real consumer
//!                (a line buffer, a clipboard, a grapheme shaper) ultimately
//!                needs. Here `std`'s `from_utf8_lossy` gets to use its SIMD
//!                ASCII fast-path and zero-copy borrow — the honest yardstick
//!                for the parser's text path.
//! * [`stream`] — feed the corpus in fixed 4 KiB chunks through one decoder
//!                instance, so codepoints straddle chunk boundaries. This is the
//!                terminal-reading-from-a-pty case, where an incremental decoder
//!                carries the partial across reads for free and a whole-buffer
//!                API (`from_utf8_lossy`) cannot be used at all without manual
//!                boundary bookkeeping.
//!
//! Folding/materialization keeps the work observable so the optimizer can't
//! elide the decode, and (within a section) makes the variants directly
//! comparable — they all produce the same lossy code-point sequence.
//!
//! Run with: `cargo bench -p ansi --bench utf8`
//! Filter, e.g.: `cargo bench -p ansi --bench utf8 -- lossy`
use std::hint::black_box;

use divan::Bencher;
use divan::counter::BytesCount;

fn main() {
    divan::main();
}

/// The corpora axis. Each variant names a representative byte mix; `Debug` is
/// what divan prints as the per-argument label.
#[derive(Clone, Copy, Debug)]
enum Corpus {
    /// Pure 7-bit ASCII — the SIMD fast-path's home turf, and the bulk of real
    /// terminal traffic.
    Ascii,
    /// Western prose: mostly ASCII with occasional 2-byte accented scalars.
    Latin,
    /// Dense 3-byte CJK plus 4-byte emoji — valid, but never ASCII.
    Cjk,
    /// ASCII + 2/3/4-byte scalars + a couple of ill-formed bytes (the lossy
    /// branch fires, but rarely).
    Mixed,
    /// Adversarial: ill-formed bytes interleaved throughout (overlongs,
    /// surrogates, out-of-range leads, stray continuations).
    Invalid,
}

impl Corpus {
    /// All corpora, in the order benches iterate them.
    const ALL: &'static [Corpus] = &[
        Corpus::Ascii,
        Corpus::Latin,
        Corpus::Cjk,
        Corpus::Mixed,
        Corpus::Invalid,
    ];

    /// The unrepeated sample for this corpus.
    const fn sample(self) -> &'static [u8] {
        match self {
            Corpus::Ascii => b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. ",
            Corpus::Latin => "Voilà! Un café crème à la française — détour épatant, naïveté. ".as_bytes(),
            Corpus::Cjk => "東京タワーから見た夜景。 한국어 텍스트. 你好，世界！ 🦀🌍✨🚀 ".as_bytes(),
            Corpus::Mixed => {
                b"Lorem ipsum \xC3\xA9 \xE6\x9D\xB1\xE4\xBA\xAC \xF0\x9F\xA6\x80 \xC0 here.\n"
            }
            Corpus::Invalid => {
                b"ok \xC0\xC1 text \xF5\x80\x80 more \xED\xA0\x80 mid \xE0\x80\xAF and \xFE\xFF\x80 done "
            }
        }
    }

    /// The sample repeated to a cache-warm working set (~16 KiB), so each timed
    /// call amortizes per-call overhead and the GB/s figures are comparable
    /// across corpora regardless of sample length.
    fn bytes(self) -> Vec<u8> {
        const TARGET: usize = 16 * 1024;
        let sample = self.sample();
        sample.repeat(TARGET.div_ceil(sample.len()))
    }
}

// ===========================================================================
// Section A — raw decode throughput (fold each codepoint, no allocation).
// ===========================================================================
mod fold {
    use super::*;

    /// `std`'s lossy chunking decoder (`<[u8]>::utf8_chunks`).
    #[divan::bench(args = Corpus::ALL)]
    fn std(bencher: Bencher, corpus: Corpus) {
        let data = corpus.bytes();
        bencher.counter(BytesCount::of_slice(&data)).bench(|| {
            let mut acc: u64 = 0;
            for chunk in data.utf8_chunks() {
                for c in chunk.valid().chars() {
                    acc = acc.wrapping_add(c as u64);
                }
                if !chunk.invalid().is_empty() {
                    acc = acc.wrapping_add(0xFFFD);
                }
            }
            acc
        });
    }

    /// The `utf8parse` crate (push-based, VT-style `Receiver`).
    #[divan::bench(args = Corpus::ALL)]
    fn utf8parse(bencher: Bencher, corpus: Corpus) {
        use utf8parse::{Parser, Receiver};

        struct Sink(u64);
        impl Receiver for Sink {
            fn codepoint(&mut self, c: char) {
                self.0 = self.0.wrapping_add(c as u64);
            }
            fn invalid_sequence(&mut self) {
                self.0 = self.0.wrapping_add(0xFFFD);
            }
        }

        let data = corpus.bytes();
        bencher.counter(BytesCount::of_slice(&data)).bench(|| {
            let mut parser = Parser::new();
            let mut sink = Sink(0);
            for &byte in &data {
                parser.advance(&mut sink, byte);
            }
            sink.0
        });
    }

    /// The `utf8-zero` crate (push-based `LossyDecoder` taking `&str` chunks).
    #[divan::bench(args = Corpus::ALL)]
    fn utf8_zero(bencher: Bencher, corpus: Corpus) {
        use utf8_zero::LossyDecoder;

        let data = corpus.bytes();
        bencher.counter(BytesCount::of_slice(&data)).bench(|| {
            let mut acc: u64 = 0;
            {
                let mut decoder = LossyDecoder::new(|s: &str| {
                    for c in s.chars() {
                        acc = acc.wrapping_add(c as u64);
                    }
                });
                decoder.feed(&data);
                // `LossyDecoder` flushes any trailing partial as U+FFFD on drop.
            }
            acc
        });
    }
}

// ===========================================================================
// Section B — materialize a lossy `String` (the consumer's real work).
//
// This is where `std::from_utf8_lossy` earns its keep: on valid input it
// SIMD-validates and borrows (no copy), so it models the cost a parser would
// pay to hand off validated text. Our per-byte DFA has to justify itself
// against that, especially on the ASCII corpus.
// ===========================================================================
mod lossy {
    use super::*;

    /// `std::String::from_utf8_lossy` — SIMD validate, borrow on valid input,
    /// allocate-and-patch only when ill-formed bytes appear.
    #[divan::bench(args = Corpus::ALL)]
    fn std(bencher: Bencher, corpus: Corpus) {
        let data = corpus.bytes();
        bencher.counter(BytesCount::of_slice(&data)).bench(|| {
            let cow = String::from_utf8_lossy(&data);
            black_box(&cow);
            cow.len() as u64
        });
    }

    /// `utf8-zero` is natively run/`&str`-oriented: append each chunk straight
    /// into the output string.
    #[divan::bench(args = Corpus::ALL)]
    fn utf8_zero(bencher: Bencher, corpus: Corpus) {
        use utf8_zero::LossyDecoder;

        let data = corpus.bytes();
        bencher.counter(BytesCount::of_slice(&data)).bench(|| {
            let mut out = String::with_capacity(data.len());
            {
                let mut decoder = LossyDecoder::new(|s: &str| out.push_str(s));
                decoder.feed(&data);
            }
            black_box(&out);
            out.len() as u64
        });
    }
}

// ===========================================================================
// Section C — streaming: feed the corpus in fixed 4 KiB chunks through one
// decoder instance, so scalars straddle chunk boundaries (the pty-read case).
//
// `from_utf8_lossy` is absent here on purpose: it has no streaming form, so a
// caller must reassemble boundaries by hand — exactly the bookkeeping these
// incremental decoders make disappear.
// ===========================================================================
mod stream {
    use super::*;

    const CHUNK: usize = 4 * 1024;

    /// `utf8parse` is byte-at-a-time, so chunk boundaries are inherently free;
    /// included as the streaming-native baseline.
    #[divan::bench(args = Corpus::ALL)]
    fn utf8parse(bencher: Bencher, corpus: Corpus) {
        use utf8parse::{Parser, Receiver};

        struct Sink(u64);
        impl Receiver for Sink {
            fn codepoint(&mut self, c: char) {
                self.0 = self.0.wrapping_add(c as u64);
            }
            fn invalid_sequence(&mut self) {
                self.0 = self.0.wrapping_add(0xFFFD);
            }
        }

        let data = corpus.bytes();
        bencher.counter(BytesCount::of_slice(&data)).bench(|| {
            let mut parser = Parser::new();
            let mut sink = Sink(0);
            for chunk in data.chunks(CHUNK) {
                for &byte in chunk {
                    parser.advance(&mut sink, byte);
                }
            }
            sink.0
        });
    }

    /// `utf8-zero`'s `LossyDecoder` carries the partial across `feed` calls.
    #[divan::bench(args = Corpus::ALL)]
    fn utf8_zero(bencher: Bencher, corpus: Corpus) {
        use utf8_zero::LossyDecoder;

        let data = corpus.bytes();
        bencher.counter(BytesCount::of_slice(&data)).bench(|| {
            let mut acc: u64 = 0;
            {
                let mut decoder = LossyDecoder::new(|s: &str| {
                    for c in s.chars() {
                        acc = acc.wrapping_add(c as u64);
                    }
                });
                for chunk in data.chunks(CHUNK) {
                    decoder.feed(chunk);
                }
            }
            acc
        });
    }
}
