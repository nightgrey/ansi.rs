//! Benchmarks for the VT500 ANSI parser.
//!
//! These measure the parser itself, not the consumer: every run uses a no-op
//! [`Sink`] handler so the only work timed is byte scanning, UTF-8 decoding and
//! state-machine dispatch.
//!
//! Run with: cargo bench -p ansi --bench parser

use criterion::{Criterion, Throughput, criterion_group, criterion_main, BenchmarkId};
use std::hint::black_box;
use std::io::Sink;
use ansi::{parser};
fn parser_throughput(c: &mut Criterion) {
    // … your data definitions (ASCII, SGR, UTF8, DATA) stay the same …

    // Define a type that holds both the name and the function to benchmark.
    struct ParserBench {
        name: &'static str,
        run: fn(&[u8]),
    }

    struct Sink;

    impl parser::Handler for Sink {
        fn print(&mut self, ch: char) {
            black_box(ch);
        }
        fn printing(&mut self, s: &str) {
            black_box(s);
        }
        fn execute(&mut self, byte: u8) {
            black_box(byte);
        }
        fn esc(&mut self, intermediates: &parser::ByteStr, final_byte: u8) {
            black_box((intermediates, final_byte));
        }
        fn csi(&mut self, params: parser::Params<'_>, intermediates: &parser::ByteStr, final_byte: char) {
            black_box((params, intermediates, final_byte));
        }
        fn dcs(&mut self, params: parser::Params<'_>, intermediates: &parser::ByteStr, final_char: char) {
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
    impl vte::Perform for Sink {
        /// Draw a character to the screen and update states.
        fn print(&mut self, _c: char) {
            black_box(_c);
        }

        /// Execute a C0 or C1 control function.
        fn execute(&mut self, _byte: u8) {
            black_box(_byte);
        }

        /// Invoked when a final character arrives in first part of device control
        /// string.
        ///
        /// The control function should be determined from the private marker, final
        /// character, and execute with a parameter list. A handler should be
        /// selected for remaining characters in the string; the handler
        /// function should subsequently be called by `put` for every character in
        /// the control string.
        ///
        /// The `ignore` flag indicates that more than two intermediates arrived and
        /// subsequent characters were ignored.
        fn hook(&mut self, _params: &vte::Params, _intermediates: &[u8], _ignore: bool, _action: char) {
            black_box((_params, _intermediates, _ignore, _action));
        }

        /// Pass bytes as part of a device control string to the handle chosen in
        /// `hook`. C0 controls will also be passed to the handler.
        fn put(&mut self, _byte: u8) {
            black_box(_byte);
        }

        /// Called when a device control string is terminated.
        ///
        /// The previously selected handler should be notified that the DCS has
        /// terminated.
        fn unhook(&mut self) {
        }

        /// Dispatch an operating system command.
        fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {
            black_box((_params, _bell_terminated));
        }

        /// A final character has arrived for a CSI sequence
        ///
        /// The `ignore` flag indicates that either more than two intermediates
        /// arrived or the number of parameters exceeded the maximum supported
        /// length, and subsequent characters were ignored.
        fn csi_dispatch(
            &mut self,
            _params: &vte::Params,
            _intermediates: &[u8],
            _ignore: bool,
            _action: char,
        ) {
            black_box((_params, _intermediates, _ignore, _action));
        }

        /// The final character of an escape sequence has arrived.
        ///
        /// The `ignore` flag indicates that more than two intermediates arrived and
        /// subsequent characters were ignored.
        fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {
            black_box((_intermediates, _ignore, _byte));
        }

        /// Whether the parser should terminate prematurely.
        ///
        /// This can be used in conjunction with
        /// [`Parser::advance_until_terminated`] to terminate the parser after
        /// receiving certain escape sequences like synchronized updates.
        ///
        /// This is checked after every parsed byte, so no expensive computation
        /// should take place in this function.
        #[inline(always)]
        fn terminated(&self) -> bool {
            false
        }
    }

    let parsers: &[ParserBench] = &[
        ParserBench {
            name: "ours",
            run: |data| {
                let mut parser = ansi::parser::Parser::default();
                parser.advance(&mut Sink, black_box(data));
            },
        },
        ParserBench {
            name: "vte",
            run: |data| {
                let mut parser = vte::Parser::new();
                parser.advance(&mut Sink, black_box(data));
            },
        },
        ParserBench {
            name: "vt_push_parser",
            run: |data| {
                let mut parser = vt_push_parser::VTPushParser::new();
                parser.feed_with(black_box(data), &mut |vt_input:vt_push_parser::event::VTEvent<'_>| {
                    black_box(vt_input);
                });
            },
        },
    ];
    // Plain ASCII text — exercises the printable fast path.
    for (input_name, input_data) in [
        ("ascii", &"Lorem ipsum dolor sit amet, consectetur adipiscing elit. "
            .repeat(256)
            .into_bytes()),
        ("sgr", &{
            let mut s = Vec::new();
            for _ in 0..512 {
                s.extend_from_slice(b"\x1B[1;31m\x1B[38;2;200;100;50mX\x1B[0m");
            }
            s
        }),
        ("utf8_mixed", &{
            let mut s = Vec::new();
            for _ in 0..256 {
                s.extend_from_slice("héllo 東京 🦀 ".as_bytes());
                s.extend_from_slice(b"\x1B[1m bold \x1B[0m");
            }
            s
        }),
        ("dcs_osc", &{
            let mut s = Vec::new();
            for _ in 0..256 {
                s.extend_from_slice(b"\x1B]0;window title here\x07");
                s.extend_from_slice(b"\x1BP1;2|some device control data\x1B\\");
            }
            s
        }),
    ] {
        let mut group = c.benchmark_group(input_name);
        group.throughput(Throughput::Bytes(input_data.len() as u64));

        for parser in parsers {
            // Unique ID like "ascii / ours", "ascii / vte", …
            group.bench_with_input(
                BenchmarkId::new(parser.name, input_name),
                input_data,
                |b, data| b.iter(|| (parser.run)(data)),
            );
        }
        group.finish();
    }
}

criterion_group!(benches, parser_throughput);
criterion_main!(benches);
