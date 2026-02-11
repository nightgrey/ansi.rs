use ansi::Style;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use sigil::buffer_next::*;

// ── Test data ──────────────────────────────────────────────────────────

/// Typical TUI content: status bars, borders, text.
const ASCII_LINE: &str = " File  Edit  View  Help                              [main] ln 42, col 7 ";

/// CJK line (3-byte inline graphemes).
const CJK_LINE: &str = "中文文本测试内容中文文本测试内容中文文本测试内容中文文";

/// Extended graphemes (emoji ZWJ sequences, >4 bytes each).
const EMOJI_SEQUENCES: &[&str] = &[
    "👨\u{200D}👩\u{200D}👧\u{200D}👦", // family
    "👩\u{200D}💻",                       // woman technologist
    "🏳\u{FE0F}\u{200D}🌈",              // rainbow flag
    "👨\u{200D}🔬",                       // man scientist
    "🧑\u{200D}🤝\u{200D}🧑",           // people holding hands
];

const DEFAULT_STYLE: Style = Style {
    fg: 0x00_FF_FF_FF, // white
    bg: 0x00_00_00_00, // black
    attrs: 0,
};

const HIGHLIGHT_STYLE: Style = Style {
    fg: 0x00_00_00_00,
    bg: 0x00_FF_FF_00,
    attrs: 1, // bold
};

// ── Benchmark helpers ──────────────────────────────────────────────────

fn terminal_sizes() -> Vec<(usize, usize, &'static str)> {
    vec![
        (80, 24, "80x24"),
        (120, 40, "120x40"),
        (200, 60, "200x60"),
        (320, 100, "320x100"),
    ]
}

// ── Benchmarks ─────────────────────────────────────────────────────────

fn bench_buffer_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_create");

    for (w, h, label) in terminal_sizes() {
        group.bench_with_input(BenchmarkId::new("new", label), &(w, h), |b, &(w, h)| {
            b.iter(|| Buffer::new(black_box(w), black_box(h)));
        });
    }

    group.finish();
}

fn bench_buffer_clear(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_clear");

    for (w, h, label) in terminal_sizes() {
        // Clear a buffer that's pure ASCII (no pool pressure).
        group.bench_with_input(
            BenchmarkId::new("ascii_only", label),
            &(w, h),
            |b, &(w, h)| {
                let mut buf = Buffer::new(w, h);
                // Fill with ASCII.
                for y in 0..h {
                    buf.put_line(0, y, &ASCII_LINE[..w.min(ASCII_LINE.len())], DEFAULT_STYLE);
                }
                b.iter(|| {
                    buf.clear();
                    // Refill so next iteration has something to clear.
                    for y in 0..h {
                        buf.put_line(0, y, &ASCII_LINE[..w.min(ASCII_LINE.len())], DEFAULT_STYLE);
                    }
                });
            },
        );

        // Clear a buffer with extended graphemes (pool release pressure).
        group.bench_with_input(
            BenchmarkId::new("with_emoji", label),
            &(w, h),
            |b, &(w, h)| {
                let mut buf = Buffer::with_capacity(w, h, 4096);
                let fill = |buf: &mut Buffer| {
                    for y in 0..h {
                        for x in 0..w {
                            if x % 20 == 0 && x + 1 < w {
                                let emoji = EMOJI_SEQUENCES[y % EMOJI_SEQUENCES.len()];
                                buf.put_str(x, y, emoji, DEFAULT_STYLE);
                            } else {
                                buf.put_char(x, y, ' ', DEFAULT_STYLE);
                            }
                        }
                    }
                };
                fill(&mut buf);
                b.iter(|| {
                    buf.clear();
                    fill(&mut buf);
                });
            },
        );
    }

    group.finish();
}

fn bench_put_char_ascii(c: &mut Criterion) {
    let mut group = c.benchmark_group("put_char_ascii");

    for (w, h, label) in terminal_sizes() {
        group.bench_with_input(BenchmarkId::new("fill", label), &(w, h), |b, &(w, h)| {
            let mut buf = Buffer::new(w, h);
            b.iter(|| {
                for y in 0..h {
                    for x in 0..w {
                        buf.put_char(x, y, 'A', DEFAULT_STYLE);
                    }
                }
                black_box(&buf);
            });
        });
    }

    group.finish();
}

fn bench_put_char_cjk(c: &mut Criterion) {
    let mut group = c.benchmark_group("put_char_cjk");

    let cjk_chars: Vec<char> = CJK_LINE.chars().collect();

    for (w, h, label) in terminal_sizes() {
        group.bench_with_input(BenchmarkId::new("fill", label), &(w, h), |b, &(w, h)| {
            let mut buf = Buffer::new(w, h);
            b.iter(|| {
                for y in 0..h {
                    for x in 0..w {
                        buf.put_char(x, y, cjk_chars[x % cjk_chars.len()], DEFAULT_STYLE);
                    }
                }
                black_box(&buf);
            });
        });
    }

    group.finish();
}

fn bench_put_str_extended(c: &mut Criterion) {
    let mut group = c.benchmark_group("put_str_extended");

    for (w, h, label) in terminal_sizes() {
        group.bench_with_input(BenchmarkId::new("fill", label), &(w, h), |b, &(w, h)| {
            let mut buf = Buffer::with_capacity(w, h, 64 * 1024);
            b.iter(|| {
                // Must clear to avoid unbounded pool growth.
                buf.clear();
                buf.inner.resize(w * h, Cell::EMPTY);

                for y in 0..h {
                    for x in 0..w {
                        let emoji = EMOJI_SEQUENCES[(x + y) % EMOJI_SEQUENCES.len()];
                        buf.put_str(x, y, emoji, DEFAULT_STYLE);
                    }
                }
                black_box(&buf.pool.used());
            });
        });
    }

    group.finish();
}

fn bench_put_line(c: &mut Criterion) {
    let mut group = c.benchmark_group("put_line");

    for (w, h, label) in terminal_sizes() {
        // ASCII line — the overwhelmingly common case.
        group.bench_with_input(
            BenchmarkId::new("ascii", label),
            &(w, h),
            |b, &(w, h)| {
                let mut buf = Buffer::new(w, h);
                let line = &ASCII_LINE[..w.min(ASCII_LINE.len())];
                b.iter(|| {
                    for y in 0..h {
                        buf.put_line(0, y, line, DEFAULT_STYLE);
                    }
                    black_box(&buf);
                });
            },
        );
    }

    group.finish();
}

fn bench_overwrite_cells(c: &mut Criterion) {
    let mut group = c.benchmark_group("overwrite_cells");

    for (w, h, label) in terminal_sizes() {
        // Overwrite ASCII with ASCII (no release needed — fast path).
        group.bench_with_input(
            BenchmarkId::new("ascii_to_ascii", label),
            &(w, h),
            |b, &(w, h)| {
                let mut buf = Buffer::new(w, h);
                // Pre-fill.
                for y in 0..h {
                    for x in 0..w {
                        buf.put_char(x, y, 'A', DEFAULT_STYLE);
                    }
                }
                b.iter(|| {
                    for y in 0..h {
                        for x in 0..w {
                            buf.put_char(x, y, 'B', HIGHLIGHT_STYLE);
                        }
                    }
                    black_box(&buf);
                });
            },
        );

        // Overwrite extended with extended (release + stash every cell).
        group.bench_with_input(
            BenchmarkId::new("emoji_to_emoji", label),
            &(w, h),
            |b, &(w, h)| {
                let mut buf = Buffer::with_capacity(w, h, 256 * 1024);
                // Pre-fill with emoji.
                for y in 0..h {
                    for x in 0..w {
                        let e = EMOJI_SEQUENCES[(x + y) % EMOJI_SEQUENCES.len()];
                        buf.put_str(x, y, e, DEFAULT_STYLE);
                    }
                }
                b.iter(|| {
                    for y in 0..h {
                        for x in 0..w {
                            let e = EMOJI_SEQUENCES[(x + y + 1) % EMOJI_SEQUENCES.len()];
                            buf.put_str(x, y, e, HIGHLIGHT_STYLE);
                        }
                    }
                    black_box(&buf.pool.used());
                });
            },
        );
    }

    group.finish();
}

fn bench_scroll_up(c: &mut Criterion) {
    let mut group = c.benchmark_group("scroll_up");

    for (w, h, label) in terminal_sizes() {
        group.bench_with_input(
            BenchmarkId::new("1_row_ascii", label),
            &(w, h),
            |b, &(w, h)| {
                let mut buf = Buffer::new(w, h);
                for y in 0..h {
                    buf.put_line(0, y, &ASCII_LINE[..w.min(ASCII_LINE.len())], DEFAULT_STYLE);
                }
                b.iter(|| {
                    buf.scroll_up(1);
                    // Re-fill the bottom row so we have consistent state.
                    buf.put_line(
                        0,
                        h - 1,
                        &ASCII_LINE[..w.min(ASCII_LINE.len())],
                        DEFAULT_STYLE,
                    );
                    black_box(&buf);
                });
            },
        );

        // Scroll with emoji in every 20th cell (mixed pool pressure).
        group.bench_with_input(
            BenchmarkId::new("1_row_mixed", label),
            &(w, h),
            |b, &(w, h)| {
                let mut buf = Buffer::with_capacity(w, h, 4096);
                for y in 0..h {
                    for x in 0..w {
                        if x % 20 == 0 {
                            let e = EMOJI_SEQUENCES[y % EMOJI_SEQUENCES.len()];
                            buf.put_str(x, y, e, DEFAULT_STYLE);
                        } else {
                            buf.put_char(x, y, ' ', DEFAULT_STYLE);
                        }
                    }
                }
                b.iter(|| {
                    buf.scroll_up(1);
                    // Refill bottom row.
                    let y = h - 1;
                    for x in 0..w {
                        if x % 20 == 0 {
                            let e = EMOJI_SEQUENCES[y % EMOJI_SEQUENCES.len()];
                            buf.put_str(x, y, e, DEFAULT_STYLE);
                        } else {
                            buf.put_char(x, y, ' ', DEFAULT_STYLE);
                        }
                    }
                    black_box(&buf.pool.used());
                });
            },
        );
    }

    group.finish();
}

fn bench_diff(c: &mut Criterion) {
    let mut group = c.benchmark_group("diff");

    for (w, h, label) in terminal_sizes() {
        // Identical buffers — best case for diff (no changes).
        group.bench_with_input(
            BenchmarkId::new("no_changes", label),
            &(w, h),
            |b, &(w, h)| {
                let mut buf_a = Buffer::new(w, h);
                let mut buf_b = Buffer::new(w, h);
                for y in 0..h {
                    let line = &ASCII_LINE[..w.min(ASCII_LINE.len())];
                    buf_a.put_line(0, y, line, DEFAULT_STYLE);
                    buf_b.put_line(0, y, line, DEFAULT_STYLE);
                }
                b.iter(|| {
                    let diff = buf_a.diff_indices(&buf_b);
                    black_box(diff.len());
                });
            },
        );

        // ~10% of cells differ (typical partial redraw).
        group.bench_with_input(
            BenchmarkId::new("10pct_changed", label),
            &(w, h),
            |b, &(w, h)| {
                let mut buf_a = Buffer::new(w, h);
                let mut buf_b = Buffer::new(w, h);
                for y in 0..h {
                    for x in 0..w {
                        buf_a.put_char(x, y, 'A', DEFAULT_STYLE);
                        if (x * 7 + y * 13) % 10 == 0 {
                            buf_b.put_char(x, y, 'B', HIGHLIGHT_STYLE);
                        } else {
                            buf_b.put_char(x, y, 'A', DEFAULT_STYLE);
                        }
                    }
                }
                b.iter(|| {
                    let diff = buf_a.diff_indices(&buf_b);
                    black_box(diff.len());
                });
            },
        );

        // 100% of cells differ (full redraw).
        group.bench_with_input(
            BenchmarkId::new("full_redraw", label),
            &(w, h),
            |b, &(w, h)| {
                let mut buf_a = Buffer::new(w, h);
                let mut buf_b = Buffer::new(w, h);
                for y in 0..h {
                    for x in 0..w {
                        buf_a.put_char(x, y, 'A', DEFAULT_STYLE);
                        buf_b.put_char(x, y, 'B', HIGHLIGHT_STYLE);
                    }
                }
                b.iter(|| {
                    let diff = buf_a.diff_indices(&buf_b);
                    black_box(diff.len());
                });
            },
        );
    }

    group.finish();
}

fn bench_read_cells(c: &mut Criterion) {
    let mut group = c.benchmark_group("read_cells");

    for (w, h, label) in terminal_sizes() {
        // Read every cell via as_str (the zero-copy fast path).
        group.bench_with_input(
            BenchmarkId::new("as_str_ascii", label),
            &(w, h),
            |b, &(w, h)| {
                let mut buf = Buffer::new(w, h);
                for y in 0..h {
                    for x in 0..w {
                        buf.put_char(x, y, 'A', DEFAULT_STYLE);
                    }
                }
                b.iter(|| {
                    let mut total_bytes = 0usize;
                    for cell in &buf.inner {
                        total_bytes += cell.grapheme.as_str(&buf.pool).len();
                    }
                    black_box(total_bytes);
                });
            },
        );

        // Read via with_str (closure path).
        group.bench_with_input(
            BenchmarkId::new("with_str_ascii", label),
            &(w, h),
            |b, &(w, h)| {
                let mut buf = Buffer::new(w, h);
                for y in 0..h {
                    for x in 0..w {
                        buf.put_char(x, y, 'A', DEFAULT_STYLE);
                    }
                }
                b.iter(|| {
                    let mut total_bytes = 0usize;
                    for cell in &buf.inner {
                        cell.grapheme
                            .with_str(&buf.pool, |s| total_bytes += s.len());
                    }
                    black_box(total_bytes);
                });
            },
        );

        // Read via resolve (enum path).
        group.bench_with_input(
            BenchmarkId::new("resolve_ascii", label),
            &(w, h),
            |b, &(w, h)| {
                let mut buf = Buffer::new(w, h);
                for y in 0..h {
                    for x in 0..w {
                        buf.put_char(x, y, 'A', DEFAULT_STYLE);
                    }
                }
                b.iter(|| {
                    let mut total_bytes = 0usize;
                    for cell in &buf.inner {
                        total_bytes += cell.grapheme.resolve(&buf.pool).len();
                    }
                    black_box(total_bytes);
                });
            },
        );
    }

    group.finish();
}

fn bench_realistic_tui_frame(c: &mut Criterion) {
    let mut group = c.benchmark_group("realistic_frame");

    // Simulate a typical TUI frame render:
    //   - Row 0: status bar (ASCII, highlighted)
    //   - Rows 1..h-2: text content (mostly ASCII, occasional CJK/emoji)
    //   - Row h-1: command bar (ASCII)
    for (w, h, label) in terminal_sizes() {
        group.bench_with_input(
            BenchmarkId::new("render", label),
            &(w, h),
            |b, &(w, h)| {
                let mut buf = Buffer::with_capacity(w, h, 4096);
                let cjk_chars: Vec<char> = CJK_LINE.chars().collect();
                let status = &ASCII_LINE[..w.min(ASCII_LINE.len())];

                b.iter(|| {
                    // Status bar.
                    buf.put_line(0, 0, status, HIGHLIGHT_STYLE);

                    // Content area.
                    for y in 1..h.saturating_sub(1) {
                        for x in 0..w {
                            match (x + y) % 50 {
                                0 if x + 1 < w => {
                                    // Occasional emoji.
                                    let e = EMOJI_SEQUENCES[y % EMOJI_SEQUENCES.len()];
                                    buf.put_str(x, y, e, DEFAULT_STYLE);
                                }
                                1..=3 => {
                                    // Some CJK.
                                    buf.put_char(
                                        x,
                                        y,
                                        cjk_chars[x % cjk_chars.len()],
                                        DEFAULT_STYLE,
                                    );
                                }
                                _ => {
                                    // Mostly ASCII.
                                    buf.put_char(x, y, 'a', DEFAULT_STYLE);
                                }
                            }
                        }
                    }

                    // Command bar.
                    if h > 1 {
                        buf.put_line(0, h - 1, ":wq", DEFAULT_STYLE);
                    }

                    black_box(&buf.pool.used());
                });
            },
        );
    }

    group.finish();
}

fn bench_pool_churn(c: &mut Criterion) {
    let mut group = c.benchmark_group("pool_churn");

    // Simulates what happens when emoji cells are continuously overwritten
    // (e.g., animated spinners, updating clocks with flag emoji, etc.)
    group.bench_function("overwrite_1000_extended", |b| {
        let mut pool = GraphemePool::with_capacity(64 * 1024);
        let mut graphemes: Vec<Grapheme> = Vec::with_capacity(1000);

        // Initial fill.
        for i in 0..1000 {
            let e = EMOJI_SEQUENCES[i % EMOJI_SEQUENCES.len()];
            graphemes.push(Grapheme::new(e, &mut pool));
        }

        b.iter(|| {
            for i in 0..1000 {
                graphemes[i].release(&mut pool);
                let e = EMOJI_SEQUENCES[(i + 1) % EMOJI_SEQUENCES.len()];
                graphemes[i] = Grapheme::new(e, &mut pool);
            }
            black_box(pool.used());
        });
    });

    // Pure inline (no pool interaction) — baseline.
    group.bench_function("overwrite_1000_inline", |b| {
        let mut pool = GraphemePool::new();
        let mut graphemes: Vec<Grapheme> = vec![Grapheme::from_char('A'); 1000];

        b.iter(|| {
            for i in 0..1000 {
                graphemes[i].release(&mut pool); // no-op for inline
                graphemes[i] = Grapheme::from_char(if i % 2 == 0 { 'B' } else { 'A' });
            }
            black_box(&graphemes);
        });
    });

    group.finish();
}

fn bench_grapheme_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("grapheme_create");

    group.bench_function("from_char_ascii", |b| {
        b.iter(|| black_box(Grapheme::from_char('A')));
    });

    group.bench_function("from_char_cjk", |b| {
        b.iter(|| black_box(Grapheme::from_char('中')));
    });

    group.bench_function("from_char_4byte_emoji", |b| {
        b.iter(|| black_box(Grapheme::from_char('🎉')));
    });

    group.bench_function("new_extended_family_emoji", |b| {
        let mut pool = GraphemePool::with_capacity(4096);
        let family = "👨\u{200D}👩\u{200D}👧\u{200D}👦";
        b.iter(|| {
            let g = Grapheme::new(family, &mut pool);
            let used = black_box(pool.used());
            g.release(&mut pool);
            used
        });
    });

    group.bench_function("try_inline_hit", |b| {
        b.iter(|| black_box(Grapheme::try_inline("AB")));
    });

    group.bench_function("try_inline_miss", |b| {
        let family = "👨\u{200D}👩\u{200D}👧\u{200D}👦";
        b.iter(|| black_box(Grapheme::try_inline(family)));
    });

    group.finish();
}

// ── Criterion config ───────────────────────────────────────────────────

criterion_group!(
    benches,
    bench_grapheme_creation,
    bench_buffer_creation,
    bench_buffer_clear,
    bench_put_char_ascii,
    bench_put_char_cjk,
    bench_put_str_extended,
    bench_put_line,
    bench_overwrite_cells,
    bench_scroll_up,
    bench_diff,
    bench_read_cells,
    bench_realistic_tui_frame,
    bench_pool_churn,
);

criterion_main!(benches);