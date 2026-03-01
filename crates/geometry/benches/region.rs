use geometry::{Bounds};
use criterion::{ criterion_group, criterion_main, Criterion};
use geometry::{Position};
use std::hint::black_box;
use std::ops::Add;
use std::time::Duration;

fn bench_iter(c:&mut Criterion) {
    let mut c = c
        .benchmark_group("iter");

    let end = Position::new(1024, 1024);
    let region = Bounds {
        min: Position::new(0, 0),
        max: end,
    };
    let region = region.into_iter();
    let cursor = region.cursor(Position::new(0, 0));
    c.bench_function("cursor", |b| {
        b.iter(|| cursor.for_each(|p| { black_box(p); }))
    });
    c.bench_function("region", |b| {
        b.iter(|| region.clone().for_each(|p| { black_box(p); }))
    });

    c.bench_function("optimal_loop", |b| {
        b.iter(|| {
            let region = region.clone();
            let mut row = region.min.row;
            while row <= region.max.row {
                let mut col = region.min.col;
                while col <= region.max.col {
                    black_box(row * region.width() + col);
                    col += 1;
                }
                row += 1;
            }
        })
    });


    c.finish();
}


criterion_group!(
    name = benches;
    config = Criterion::default().warm_up_time(Duration::from_millis(500)).measurement_time(Duration::from_secs(1)).with_output_color(true).with_plots();
    targets = bench_iter
);

criterion_main!(benches);