//! Throughput benchmarks for the VT500 ANSI parser.
//!
//! Parser construction and fixture generation are excluded from the timed
//! region. The sink keeps callback results observable without putting a
//! `black_box` call on every parsed event.
//!
//! Run with: `cargo bench -p ansi --bench parser`
use ansi::parser;
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use bytesize::ByteSize;


fn parser_throughput(c: &mut Criterion) {

    #[derive(Clone, Copy)]
    enum ParserKind {
        Ours,
        Vte,
        VtPushParser,
    }

    impl ParserKind {
        const ALL: [Self; 3] = [Self::Ours, Self::Vte, Self::VtPushParser];

        const fn name(self) -> &'static str {
            match self {
                Self::Ours => "ours",
                Self::Vte => "vte",
                Self::VtPushParser => "vt_push_parser",
            }
        }
    }

    #[derive(Default)]
    struct Sink {
        events: u64,
        payload: u64,
    }

    impl Sink {
        #[inline(always)]
        fn event(&mut self, payload: usize) {
            self.events = self.events.wrapping_add(1);
            self.payload = self.payload.wrapping_add(payload as u64);
        }
    }

    impl parser::Handler for Sink {
        fn print(&mut self, ch: char) {
            self.event(ch.len_utf8());
        }

        fn printing(&mut self, text: &str) {
            self.event(text.len());
        }

        fn execute(&mut self, _byte: u8) {
            self.event(1);
        }

        fn esc(&mut self, intermediates: &parser::ByteStr, _final_byte: u8) {
            self.event(intermediates.len() + 1);
        }

        fn csi(
            &mut self,
            _params: parser::Params<'_>,
            intermediates: &parser::ByteStr,
            final_byte: char,
        ) {
            self.event(intermediates.len() + final_byte.len_utf8());
        }

        fn dcs(
            &mut self,
            _params: parser::Params<'_>,
            intermediates: &parser::ByteStr,
            final_char: char,
        ) {
            self.event(intermediates.len() + final_char.len_utf8());
        }

        fn dcs_byte(&mut self, _byte: u8) {
            self.event(1);
        }

        fn dcs_string(&mut self, bytes: &[u8]) {
            self.event(bytes.len());
        }

        fn dcs_termination(&mut self, _byte: u8) {
            self.event(1);
        }

        fn osc(&mut self) {
            self.event(0);
        }

        fn osc_byte(&mut self, _byte: u8) {
            self.event(1);
        }

        fn osc_string(&mut self, bytes: &[u8]) {
            self.event(bytes.len());
        }

        fn osc_termination(&mut self, _byte: u8) {
            self.event(1);
        }
    }

    impl vte::Perform for Sink {
        fn print(&mut self, ch: char) {
            self.event(ch.len_utf8());
        }

        fn execute(&mut self, _byte: u8) {
            self.event(1);
        }

        fn hook(&mut self, _params: &vte::Params, intermediates: &[u8], _ignore: bool, action: char) {
            self.event(intermediates.len() + action.len_utf8());
        }

        fn put(&mut self, _byte: u8) {
            self.event(1);
        }

        fn unhook(&mut self) {
            self.event(0);
        }

        fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
            self.event(params.iter().map(|param| param.len()).sum());
        }

        fn csi_dispatch(
            &mut self,
            _params: &vte::Params,
            intermediates: &[u8],
            _ignore: bool,
            action: char,
        ) {
            self.event(intermediates.len() + action.len_utf8());
        }

        fn esc_dispatch(&mut self, intermediates: &[u8], _ignore: bool, _byte: u8) {
            self.event(intermediates.len() + 1);
        }
    }


    fn bench_input(c: &mut Criterion, name: &str, data: &[u8], chunk_size: Option<usize>) {
        let scenario = if let Some(chunk_size) = chunk_size {
            &format!("chunked/{}", ByteSize::kb(chunk_size as u64).display().si_short())
        } else {
            "whole"
        };

        let mut group = c.benchmark_group(format!("{scenario}/{name}"));
        group.throughput(Throughput::Bytes(data.len() as u64));

        let chunk_size = chunk_size.unwrap_or(data.len());

        for kind in ParserKind::ALL {
            group.bench_with_input(
                BenchmarkId::from_parameter(kind.name()),
                &data,
                |b, data| match kind {
                    ParserKind::Ours => b.iter_batched_ref(
                        || (parser::Parser::default(), Sink::default()),
                        |(parser, sink)| {
                            for chunk in data.chunks(chunk_size) {
                                parser.advance(sink, black_box(chunk));
                            }
                            black_box((parser, sink));
                        },
                        BatchSize::SmallInput,
                    ),
                    ParserKind::Vte => b.iter_batched_ref(
                        || (vte::Parser::new(), Sink::default()),
                        |(parser, sink)| {
                            for chunk in data.chunks(chunk_size) {
                                parser.advance(sink, black_box(chunk));
                            }
                            black_box((parser, sink));
                        },
                        BatchSize::SmallInput,
                    ),
                    ParserKind::VtPushParser => b.iter_batched_ref(
                        || (vt_push_parser::VTPushParser::new(), Sink::default()),
                        |(parser, sink)| {
                            for chunk in data.chunks(chunk_size) {
                                parser.feed_with(
                                    black_box(chunk),
                                    &mut |_: vt_push_parser::event::VTEvent<'_>| sink.event(0),
                                );
                            }
                            black_box((parser, sink));
                        },
                        BatchSize::SmallInput,
                    ),
                },
            );
        }

        group.finish();
    }


    let inputs = [
        (
            "ascii",
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit. "
                .repeat(256)
                .into_bytes(),
        ),
        ("sgr", {
            let mut data = Vec::new();
            for _ in 0..512 {
                data.extend_from_slice(b"\x1B[1;31m\x1B[38;2;200;100;50mX\x1B[0m");
            }
            data
        }),
        ("utf8_mixed", {
            let mut data = Vec::new();
            for _ in 0..256 {
                data.extend_from_slice("héllo 東京 🦀 ".as_bytes());
                data.extend_from_slice(b"\x1B[1m bold \x1B[0m");
            }
            data
        }),
        ("dcs_osc", {
            let mut data = Vec::new();
            for _ in 0..256 {
                data.extend_from_slice(b"\x1B]0;window title here\x07");
                data.extend_from_slice(b"\x1BP1;2|some device control data\x1B\\");
            }
            data
        }),
    ];

    const SCENARIOS: &'static [Option<usize>] = &[
         None,
        Some(ByteSize::kb(4).as_u64() as usize),
        Some(ByteSize::mb(1).as_u64() as usize),
    ];

    for (name, data) in &inputs {
        for &scenario in SCENARIOS {
            bench_input(c, name, data, scenario);
        }
    }
}

criterion_group!(benches, parser_throughput);
criterion_main!(benches);
