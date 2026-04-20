//! Benchmarks for Cell operations (bd-19x)
//!
//! Performance budgets:
//! - Cell comparison: < 1ns
//! - Cell bits_eq SIMD: < 0.5ns
//!
//! Run with: cargo bench --bench cell

use ansi::{Attribute, Color, Style};
use criterion::{Criterion, criterion_group, criterion_main};
use gloss::{Arena, Cell, Grapheme};
use std::hint::black_box;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

// =============================================================================
// Cell creation
// =============================================================================

fn bench_cell_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("cell/create");

    group.bench_function("default", |b| b.iter(|| black_box(Cell::default())));

    group.bench_function("inline", |b| b.iter(|| black_box(Cell::inline('A'))));

    group.bench_function("inline_cjk", |b| {
        b.iter(|| black_box(Cell::inline('\u{4E2D}')))
    });

    let mut arena = Arena::new();

    group.bench_function("extended", |b| {
        b.iter(|| black_box(Cell::extended("A", &mut arena)))
    });

    group.bench_function("extended_cjk", |b| {
        b.iter(|| black_box(Cell::extended("中", &mut arena)))
    });

    group.bench_function("with_foreground_background", |b| {
        b.iter(|| {
            black_box(
                Cell::inline('X')
                    .with_foreground(Color::Rgb(255, 128, 0))
                    .with_background(Color::Rgb(0, 0, 128)),
            )
        })
    });

    group.finish();
}

// =============================================================================
// Cell comparison (the hot path for diffing)
// =============================================================================

fn bench_cell_compare(c: &mut Criterion) {
    let mut group = c.benchmark_group("cell/compare");

    let cell_a = Cell::inline('A').with_foreground(Color::Rgb(255, 0, 0));
    let cell_b = Cell::inline('A').with_foreground(Color::Rgb(255, 0, 0));
    let cell_c = Cell::inline('B').with_foreground(Color::Rgb(0, 255, 0));

    // bits_eq: branchless SIMD-friendly comparison
    group.bench_function("eq_bitwise/same", |b| {
        b.iter(|| black_box(cell_a.eq_bitwise(black_box(&cell_b))))
    });

    group.bench_function("eq_bitwise/different", |b| {
        b.iter(|| black_box(cell_a.eq_bitwise(black_box(&cell_c))))
    });

    // PartialEq: standard Rust comparison
    group.bench_function("eq/same", |b| {
        b.iter(|| black_box(black_box(&cell_a) == black_box(&cell_b)))
    });

    group.bench_function("eq/different", |b| {
        b.iter(|| black_box(black_box(&cell_a) == black_box(&cell_c)))
    });

    // Compare default (empty) cells — common case
    let empty_a = Cell::default();
    let empty_b = Cell::default();
    group.bench_function("eq_bitwise/default", |b| {
        b.iter(|| black_box(empty_a.eq_bitwise(black_box(&empty_b))))
    });

    group.finish();
}

// =============================================================================
// Grapheme operations
// =============================================================================

fn bench_grapheme(c: &mut Criterion) {
    let mut group = c.benchmark_group("cell/content");

    let mut arena = Arena::new();
    let ascii = Grapheme::inline('A');
    let cjk = Grapheme::inline('\u{4E2D}');
    let str = Grapheme::extended("中", &mut arena);
    let mut arena = Arena::new();

    group.bench_function("extended", |b| {
        b.iter(|| black_box(Cell::extended("A", &mut arena)))
    });

    group.bench_function("extended_cjk", |b| {
        b.iter(|| black_box(Cell::extended("中", &mut arena)))
    });

    group.bench_function("width/str", |b| {
        b.iter(|| black_box(black_box(str).as_str(&arena).width()))
    });

    group.bench_function("is_empty", |b| {
        b.iter(|| black_box(black_box(Grapheme::EMPTY).is_empty()))
    });

    group.bench_function("is_continuation", |b| {
        b.iter(|| black_box(black_box(Grapheme::CONTINUATION).is_continuation()))
    });

    group.bench_function("as_str", |b| {
        b.iter(|| {
            black_box(black_box(str).as_str(&arena));
        })
    });
    group.finish();
}

// =============================================================================
// Color operations
// =============================================================================

fn bench_packed_color(c: &mut Criterion) {
    let mut group = c.benchmark_group("cell/packed_rgba");

    let src = Color::Rgb(200, 100, 50);
    let dst = Color::Rgb(50, 100, 200);

    group.bench_function("rgb_create", |b| {
        b.iter(|| black_box(Color::Rgb(255, 128, 0)))
    });

    group.finish();
}

// =============================================================================
// Row-level comparison (simulating diff inner loop)
// =============================================================================

fn bench_row_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("cell/row_compare");

    // Simulate comparing two rows of 80 cells
    let row_a: Vec<Cell> = (0..80)
        .map(|i| Cell::inline(char::from(b'A' + (i % 26) as u8)))
        .collect();
    let row_b = row_a.clone();
    let mut row_c = row_a.clone();
    row_c[40] = Cell::inline('!').with_foreground(Color::Rgb(255, 0, 0));

    group.bench_function("80_cells_identical", |b| {
        b.iter(|| {
            let mut all_eq = true;
            for (a, bb) in black_box(&row_a).iter().zip(black_box(&row_b).iter()) {
                all_eq &= a.eq_bitwise(bb);
            }
            black_box(all_eq)
        })
    });

    group.bench_function("80_cells_one_diff", |b| {
        b.iter(|| {
            let mut all_eq = true;
            for (a, cc) in black_box(&row_a).iter().zip(black_box(&row_c).iter()) {
                all_eq &= a.eq_bitwise(cc);
            }
            black_box(all_eq)
        })
    });

    // 200-column row
    let wide_row: Vec<Cell> = (0..200)
        .map(|i| Cell::inline(char::from(b'A' + (i % 26) as u8)))
        .collect();
    let wide_row_b = wide_row.clone();

    group.bench_function("200_cells_identical", |b| {
        b.iter(|| {
            let mut all_eq = true;
            for (a, bb) in black_box(&wide_row)
                .iter()
                .zip(black_box(&wide_row_b).iter())
            {
                all_eq &= a.eq_bitwise(bb);
            }
            black_box(all_eq)
        })
    });

    group.finish();
}

// =============================================================================
// Attribute and Style
// =============================================================================

fn bench_attrs(c: &mut Criterion) {
    let mut group = c.benchmark_group("cell/attrs");

    let flags = Attribute::Bold | Attribute::Italic;
    let attrs = Style::default().with(flags);

    group.bench_function("attributes", |b| {
        b.iter(|| black_box(black_box(attrs).attributes))
    });

    // group.bench_function("link_id_extract", |b| {
    //     b.iter(|| black_box(black_box(attrs).link_id()))
    // });

    group.bench_function("with", |b| {
        b.iter(|| black_box(black_box(attrs).with(black_box(flags))))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_cell_creation,
    bench_cell_compare,
    bench_grapheme,
    bench_row_comparison,
    bench_attrs,
);
criterion_main!(benches);
