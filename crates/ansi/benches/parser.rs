//! Benchmarks for the VT500 ANSI parser.
//!
//! These measure the parser itself, not the consumer: every run uses a no-op
//! [`Sink`] handler so the only work timed is byte scanning, UTF-8 decoding and
//! state-machine dispatch.
//!
//! Run with: cargo bench -p ansi --bench parser

use ansi::parser::{Handler, ByteStr, Params, Parser};
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;

/// A handler that discards everything. Each method blackboxes its arguments so
/// the optimizer can't elide the dispatch work entirely.
#[derive(Default)]
struct Sink;

impl Handler for Sink {
    fn print(&mut self, ch: char) {
        black_box(ch);
    }
    fn printing(&mut self, s: &str) {
        black_box(s);
    }
    fn execute(&mut self, byte: u8) {
        black_box(byte);
    }
    fn esc(&mut self, intermediates: &ByteStr, final_byte: u8) {
        black_box((intermediates, final_byte));
    }
    fn csi(&mut self, params: Params<'_>, intermediates: &ByteStr, final_byte: char) {
        black_box((params, intermediates, final_byte));
    }
    fn dcs(&mut self, params: Params<'_>, intermediates: &ByteStr, final_char: char) {
        black_box((params, intermediates, final_char));
    }
    fn dcs_byte(&mut self, byte: u8) {
        black_box(byte);
    }
    fn dcs_termination(&mut self, byte: u8) {
        black_box(byte);
    }
    fn osc(&mut self) {}
    fn osc_byte(&mut self, byte: u8) {
        black_box(byte);
    }
    fn osc_termination(&mut self, byte: u8) {
        black_box(byte);
    }
}

/// Feed `input` through a fresh parser repeatedly.
fn drive(input: &[u8]) {
    let mut parser = Parser::default();
    let mut sink = Sink;
    parser.advance(&mut sink, black_box(input));
}

fn bench_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser");

    // Plain ASCII text — exercises the printable fast path.
    let ascii = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. "
        .repeat(256)
        .into_bytes();

    // Heavy SGR stream — many CSI dispatches with params and sub-params.
    let sgr = {
        let mut s = Vec::new();
        for _ in 0..512 {
            s.extend_from_slice(b"\x1B[1;31m\x1B[38;2;200;100;50mX\x1B[0m");
        }
        s
    };

    // Mixed UTF-8 text interleaved with escape sequences.
    let mixed = {
        let mut s = Vec::new();
        for _ in 0..256 {
            s.extend_from_slice("héllo 東京 🦀 ".as_bytes());
            s.extend_from_slice(b"\x1B[1m bold \x1B[0m");
        }
        s
    };

    // DCS / OSC string streams.
    let strings = {
        let mut s = Vec::new();
        for _ in 0..256 {
            s.extend_from_slice(b"\x1B]0;window title here\x07");
            s.extend_from_slice(b"\x1BP1;2|some device control data\x1B\\");
        }
        s
    };

    for (name, data) in [
        ("ascii", &ascii),
        ("sgr", &sgr),
        ("utf8_mixed", &mixed),
        ("dcs_osc", &strings),
    ] {
        group.throughput(Throughput::Bytes(data.len() as u64));
        group.bench_function(name, |b| b.iter(|| drive(data)));
    }

    group.finish();
}

criterion_group!(benches, bench_throughput);
criterion_main!(benches);
