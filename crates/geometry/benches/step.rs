use geometry::{Bounds};
use criterion::{ criterion_group, criterion_main, Criterion};
use geometry::{Position};
use std::hint::black_box;
use std::time::Duration;

fn bench_iter(c:&mut Criterion) {
    let mut c = c
        .benchmark_group("iter");
    let bounds = Bounds::corners(0, 0, 1024, 1024);

    c.bench_function("cursor_iter", |b| {
        let cursor = bounds.cursor(Position::new(0, 0));

        b.iter(|| cursor.for_each(|p| { black_box(p); }))
    });

    c.bench_function("step_iter", |b| {
        let iter =  bounds.into_iter();

        b.iter(|| iter.for_each(|p| { black_box(p); }))
    });

    c.bench_function("optimal_loop", |b| {

        b.iter(|| {
            let mut row = bounds.min.row;
            while row <= bounds.max.row {
                let mut col = bounds.min.col;
                while col <= bounds.max.col {
                    black_box(row * bounds.width() + col);
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