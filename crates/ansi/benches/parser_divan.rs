#![feature(const_trait_impl)]
//! Throughput benchmarks for the VT500 ANSI parser – divan port.
//!
//! Parser construction and fixture generation are excluded from the timed
//! region. The sink keeps callback results observable without putting a
//! `black_box` call on every parsed event.
//!
//! Run with: `cargo bench -p ansi --bench parser_divan` (divan integration)
use ansi::parser;
use divan::Bencher;
use std::hint::black_box;

#[derive(Clone, Copy, Debug)]
enum Parser {
    Own,
    Vte,
    VtPushParser,
}

#[cfg(test)]
mod benches {
    use super::*;
    
    const CHUNK_SIZES: &[Option<usize>] = &[None, Some(4 * 1024)];

    fn bench_parser(bencher: Bencher, target: Parser, data: &[u8], chunk_size: Option<usize>) {
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
            fn csi(&mut self, _params: parser::Params<'_>, intermediates: &parser::ByteStr, final_byte: char) {
                self.push(intermediates.len() + final_byte.len_utf8());
            }
            fn dcs_start(&mut self, _params: parser::Params<'_>, intermediates: &parser::ByteStr, final_char: char) {
                self.push(intermediates.len() + final_char.len_utf8());
            }
            fn dcs_byte(&mut self, _byte: u8) {
                self.push(1);
            }
            fn dcs_string(&mut self, bytes: &[u8]) {
                self.push(bytes.len());
            }
            fn dcs_end(&mut self, _byte: u8) {
                self.push(1);
            }
            fn osc_start(&mut self) {
                self.push(0);
            }
            fn osc_byte(&mut self, _byte: u8) {
                self.push(1);
            }
            fn osc_string(&mut self, bytes: &[u8]) {
                self.push(bytes.len());
            }
            fn osc_end(&mut self, _byte: u8) {
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
            fn hook(&mut self, _params: &vte::Params, intermediates: &[u8], _ignore: bool, action: char) {
                self.push(intermediates.len() + action.len_utf8());
            }
            fn put(&mut self, _byte: u8) {
                self.push(1);
            }
            fn unhook(&mut self) {
                self.push(0);
            }
            fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
                self.push(params.iter().map(|p| p.len()).sum());
            }
            fn csi_dispatch(&mut self, _params: &vte::Params, intermediates: &[u8], _ignore: bool, action: char) {
                self.push(intermediates.len() + action.len_utf8());
            }
            fn esc_dispatch(&mut self, intermediates: &[u8], _ignore: bool, _byte: u8) {
                self.push(intermediates.len() + 1);
            }
        }
        
        let data = if data.len() < 1024 * 1024 {
            data.repeat(1024 * 1024 / data.len())
        } else {
            data.to_vec()
        };
        
        let chunk_size = chunk_size.unwrap_or(data.len());
        match target {
            Parser::Own => {
                bencher
                    .with_inputs(|| (vte::Parser::new(), Collector::default(), data.chunks(chunk_size)))
                    .bench_values(|(parser, collector, data)| {
                        let mut parser = vte::Parser::new();
                        let mut collector = Collector::default();
                        for chunk in data {
                            parser.advance(&mut collector, black_box(chunk));
                        }
                        black_box((parser, collector));
                    })
            }
            Parser::Vte => {
                bencher
                    .with_inputs(|| (vte::Parser::new(), Collector::default(), data.chunks(chunk_size)))
                    .bench_values(|(parser, collector, data)| {
                        let mut parser = vte::Parser::new();
                        let mut collector = Collector::default();
                        for chunk in data {
                            parser.advance(&mut collector, black_box(chunk));
                        }
                        black_box((parser, collector));
                    });
            }
            Parser::VtPushParser => {
                bencher
                    .with_inputs(|| {
                        (vt_push_parser::VTPushParser::new(), Collector::default(), data.chunks(chunk_size))
                    })
                    .bench_values(|(parser, collector, data)| {
                        use vt_push_parser::event::{VTEvent, EscInvalid};

                        let mut parser = vt_push_parser::VTPushParser::new();
                        let mut collector = Collector::default();
                        for chunk in data {
                            parser.feed_with(
                                black_box(chunk),
                                &mut |event: vt_push_parser::event::VTEvent<'_>| {
                                    match event {
                                        vt_push_parser::event::VTEvent::Raw(s) => {
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
                                        VTEvent::Csi(sequence) => {
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
                    })
            }
        }
    }

    macro_rules! bench {
        ($name:ident, $data:expr) => {
            mod $name {
                use super::*;

                const DATA: &[u8] = $data;
                #[divan::bench(args = CHUNK_SIZES)]
                fn our(bencher: divan::Bencher, chunk_size: Option<usize>) {
                    super::bench_parser(bencher, Parser::Own, DATA, chunk_size);
                }

                #[divan::bench(args = CHUNK_SIZES)]
                fn vte(bencher: divan::Bencher, chunk_size: Option<usize>) {
                    const TARGET: Parser = Parser::Vte;
                    const CHUNK_SIZE: Option<usize> = None;
                    super::bench_parser(bencher, Parser::Vte, DATA, chunk_size);
                }

                #[divan::bench(args = CHUNK_SIZES)]
                fn vt_push_parser(bencher: divan::Bencher, chunk_size: Option<usize>) {
                    super::bench_parser(bencher, Parser::VtPushParser, DATA, chunk_size);
                }
            }
        };
    }

    bench!(sgr, b"\x1B[1m bold \x1B[0m");
    bench!(ascii, b"Lorem ipsum dolor sit amet, consectetur adipiscing elit.");
    bench!(utf8, "héllo 東京 🦀 \x1B[1m bold \x1B[0m".as_bytes());
    bench!(dcs_osc, b"\x1B]0;window title here\x07\x1BP1;2|some device control data\x1B\\");

}

fn main() {
    // Run `math::add` and `math::div` benchmarks:
    divan::main();
}