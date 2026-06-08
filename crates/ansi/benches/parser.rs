//! Throughput benchmarks for the VT500 ANSI parser.
//!
//! Parser construction and fixture generation are excluded from the timed
//! region. The sink keeps callback results observable without putting a
//! `black_box` call on every parsed event.
//! Results include callback dispatch, whose granularity differs by parser.
//!
//! Run with: `cargo bench -p ansi --bench parser`
use ansi::parser;
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use derive_more::{AsRef, Deref};

fn parser_throughput(c: &mut Criterion) {

    for fixture in [
        Fixture::prepare(
            "ascii",
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit."
        ),
        Fixture::prepare(
            "utf8",
            "héllo 東京 🦀 \x1B[1m bold \x1B[0m"
        ),
        Fixture::prepare(
            "dcs-osc",
            b"\x1B]0;window title here\x07\x1BP1;2|some device control data\x1B\\"
        ),
    ] {
        for scenario in [
            Scenario { label: "complete", chunk_size: None },
            Scenario { label: "64b", chunk_size: Some(64) },
            Scenario { label: "4kb", chunk_size: Some(4 * 1024) },
            Scenario { label: "1b", chunk_size: Some(1) },
        ] {
            run(c, &fixture, &scenario);
        }
    }
    
    fn run(
        c: &mut Criterion,
        fixture: &Fixture,
        scenario: &Scenario,
    ) {
        let (
            fixture,
            scenario,
            data,
            chunk_size
        ) = scenario.with(&fixture);

        let mut group = c.benchmark_group(format!("parser/{fixture}/{scenario}", fixture = fixture.label, scenario = scenario.label));
        group.throughput(Throughput::Bytes(data.len() as u64));

        for kind in Benchmark::ALL {
            group.bench_with_input(
                BenchmarkId::from_parameter(kind.name()),
                &fixture,
                |b, data| match kind {
                    Benchmark::Ours => b.iter_batched_ref(
                        || (parser::Parser::default(), Collector::default(), data.chunks(chunk_size)),
                        |(parser, collector, data)| {
                            for chunk in data {
                                parser.advance(collector, black_box(chunk));
                            }
                            black_box((parser, collector));
                        },
                        BatchSize::SmallInput,
                    ),
                    Benchmark::Vte => b.iter_batched_ref(
                        || (vte::Parser::new(), Collector::default(), data.chunks(chunk_size)),
                        |(parser, collector, data)| {
                            for chunk in data {
                                parser.advance(collector, black_box(chunk));
                            }
                            black_box((parser, collector));
                        },
                        BatchSize::SmallInput,
                    ),
                    Benchmark::VtPushParser => b.iter_batched_ref(
                        || (vt_push_parser::VTPushParser::new(), Collector::default(), data.chunks(chunk_size)),
                        |(parser, collector, data)| {
                            use vt_push_parser::event::{VTEvent, EscInvalid};
                            use vt_push_parser::VTPushParser;
                            for chunk in data {
                                parser.feed_with(
                                    black_box(chunk),
                                    &mut |event: vt_push_parser::event::VTEvent<'_>| {
                                        match event {
                                            VTEvent::Raw(s) => {
                                                collector.push(s.len());
                                            }
                                            VTEvent::EscInvalid(esc_invalid) => {
                                                collector.push(match esc_invalid {
                                                    EscInvalid::One(_) => 1,
                                                    EscInvalid::Two(_, _) => 2,
                                                    EscInvalid::Three(_, _, _) => 3,
                                                    EscInvalid::Four(_, _, _, _) => 4,
                                                });
                                            }
                                            VTEvent::Csi(sequence)=> {
                                                collector.push(sequence.params.len() + sequence.intermediates.len());
                                            }
                                            VTEvent::Esc(sequence) => {
                                                collector.push(sequence.intermediates.len() + 1);
                                            }
                                            VTEvent::C0(_) => {
                                                collector.push(1);
                                            }
                                            VTEvent::Ss2(_) => {
                                                collector.push(1);
                                            }
                                            VTEvent::Ss3(_) => {
                                                collector.push(1);
                                            }
                                            VTEvent::DcsStart(dcs) => {
                                                collector.push(dcs.params.len() + dcs.intermediates.len() + 1);

                                            }
                                            VTEvent::DcsData(s) | VTEvent::DcsEnd(s) => {
                                                collector.push(s.len());
                                            }
                                            VTEvent::DcsCancel => {
                                                collector.push(0);
                                            }
                                            VTEvent::OscStart => {
                                                collector.push(0);
                                            }
                                            VTEvent::OscData(s) | VTEvent::OscEnd { data: s, .. } => {
                                                collector.push(s.len());
                                            }
                                            VTEvent::OscCancel => {
                                                collector.push(0);
                                            }
                                        }
                                    },
                                );
                            }
                            black_box((parser, collector));
                        },
                        BatchSize::SmallInput,
                    ),
                },
            );
        }

        group.finish();

    }

    #[derive(Clone, Copy)]
    enum Benchmark {
        Ours,
        Vte,
        VtPushParser,
    }

    impl Benchmark {
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
    struct Collector {
        count: u64,
        payload: u64,
    }

    impl Collector {
        #[inline(always)]
        fn push(&mut self, payload: usize) {
            self.count = self.count.wrapping_add(1);
            self.payload = self.payload.wrapping_add(payload as u64);
        }
    }

    impl parser::Handler for Collector {
        fn print(&mut self, ch: char) {
            self.push(ch.len_utf8());
        }

        fn printing(&mut self, text: &str) {
            self.push(text.len());
        }

        fn execute(&mut self, _byte: u8) {
            self.push(1);
        }

        fn esc(&mut self, intermediates: &parser::ByteStr, _final_byte: u8) {
            self.push(intermediates.len() + 1);
        }

        fn csi(
            &mut self,
            _params: parser::Params<'_>,
            intermediates: &parser::ByteStr,
            final_byte: char,
        ) {
            self.push(intermediates.len() + final_byte.len_utf8());
        }

        fn dcs(
            &mut self,
            _params: parser::Params<'_>,
            intermediates: &parser::ByteStr,
            final_char: char,
        ) {
            self.push(intermediates.len() + final_char.len_utf8());
        }

        fn dcs_byte(&mut self, _byte: u8) {
            self.push(1);
        }

        fn dcs_string(&mut self, bytes: &[u8]) {
            self.push(bytes.len());
        }

        fn dcs_termination(&mut self, _byte: u8) {
            self.push(1);
        }

        fn osc(&mut self) {
            self.push(0);
        }

        fn osc_byte(&mut self, _byte: u8) {
            self.push(1);
        }

        fn osc_string(&mut self, bytes: &[u8]) {
            self.push(bytes.len());
        }

        fn osc_termination(&mut self, _byte: u8) {
            self.push(1);
        }
    }

    impl vte::Perform for Collector {
        fn print(&mut self, ch: char) {
            self.push(ch.len_utf8());
        }

        fn execute(&mut self, _byte: u8) {
            self.push(1);
        }

        fn hook(
            &mut self,
            _params: &vte::Params,
            intermediates: &[u8],
            _ignore: bool,
            action: char,
        ) {
            self.push(intermediates.len() + action.len_utf8());
        }

        fn put(&mut self, _byte: u8) {
            self.push(1);
        }

        fn unhook(&mut self) {
            self.push(0);
        }

        fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
            self.push(params.iter().map(|param| param.len()).sum());
        }

        fn csi_dispatch(
            &mut self,
            _params: &vte::Params,
            intermediates: &[u8],
            _ignore: bool,
            action: char,
        ) {
            self.push(intermediates.len() + action.len_utf8());
        }

        fn esc_dispatch(&mut self, intermediates: &[u8], _ignore: bool, _byte: u8) {
            self.push(intermediates.len() + 1);
        }
    }
}


criterion_group!(benches, parser_throughput);
criterion_main!(benches);


// Harness

#[derive(Clone, Deref, AsRef)]
struct Fixture{
    label: &'static str,
    #[deref]
    #[as_ref]
    data: Vec<u8>,
}

impl Fixture {
    const MIN: usize = 1024 * 1024;

    fn prepare(label: &'static str, bytes: impl AsRef<[u8]>) -> Self {
        let bytes = bytes.as_ref();
        let mut data = Vec::with_capacity(Self::MIN);
        data.copy_from_slice(bytes);

        if data.len() < Self::MIN {
            data.repeat((Self::MIN) / bytes.len());
        }
        Self {
            label,
            data
        }
    }
}

#[derive(Clone, Copy)]
struct Scenario {
    label: &'static str,
    chunk_size: Option<usize>
}

impl Scenario {
    fn chunk_size(&self, fixture: &Fixture) -> usize {
        self.chunk_size.unwrap_or(fixture.len())
    }

    fn with<'a>(&'a self, fixture: &'a Fixture) -> (&'a Fixture, &'a Scenario, &'a [u8], usize)     {
        (fixture, self, &fixture.data, self.chunk_size(fixture))
    }
}

