use grid::{Bounds, Position};
use criterion::{ criterion_group, criterion_main, Criterion};
use std::hint::black_box;

fn step(c:&mut Criterion) {
    let mut g = c
        .benchmark_group("step");

    g.bench_function("map", |b| {
        let iter =  Bounds::corners(0, 0, 1024, 1024).into_iter();

        b.iter(|| iter.map(|p| black_box(p)))
    });

    g.bench_function("for_each", |b| {
        let iter =  Bounds::corners(0, 0, 1024, 1024).into_iter();

        b.iter(|| {
            iter.for_each(|p| {
                black_box(p);
            })
        })
    });



    g.bench_function("fold", |b| {
        let iter =  Bounds::corners(0, 0, 1024, 1024).into_iter();

        b.iter(|| {
            iter.fold(Position::ZERO, |acc, init| {
                acc + init
            });
        })
    });

    g.finish();
}
fn hand(c:&mut Criterion) {
    let mut g = c
        .benchmark_group("hand");

    g.bench_function("map", |b| {
        let bounds =  Bounds::corners(0, 0, 1024, 1024);

        b.iter(|| {
            (bounds.min.col..bounds.max.col).flat_map(|x| (bounds.min.row..bounds.max.row).map(move |y| Position::new(x, y)))
        })
    });

    g.bench_function("for_each", |b| {
        let bounds =  Bounds::corners(0, 0, 1024, 1024);

        b.iter(|| {
            (bounds.min.col..bounds.max.col).flat_map(|x| (bounds.min.row..bounds.max.row).map(move |y| Position::new(x, y))).for_each(|p| {
                black_box(p);
            })
        })
    });


    g.bench_function("fold", |b| {
        let bounds =  Bounds::corners(0, 0, 1024, 1024);

        b.iter(|| {
            (bounds.min.col..bounds.max.col).flat_map(|x| (bounds.min.row..bounds.max.row).map(move |y| Position::new(x, y))).fold(Position::ZERO, |acc, init| {
                acc + init
            });
        })
    });

    g.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().with_output_color(true).with_plots();
    targets = step, hand
);

criterion_main!(benches);