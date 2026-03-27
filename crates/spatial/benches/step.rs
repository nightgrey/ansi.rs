use criterion::{Criterion, criterion_group, criterion_main};
use geometry::{Point, Rect};
use std::hint::black_box;

fn step(c: &mut Criterion) {
    let mut g = c.benchmark_group("step");

    g.bench_function("iter.map", |b| {
        let iter = Rect::bounds(0, 0, 1024, 1024).into_iter();

        b.iter(|| iter.map(|p| black_box(p)))
    });

    g.bench_function("iter.for_each", |b| {
        let iter = Rect::bounds(0, 0, 1024, 1024).into_iter();

        b.iter(|| {
            iter.for_each(|p| {
                black_box(p);
            })
        })
    });

    g.bench_function("iter.fold", |b| {
        let iter = Rect::bounds(0, 0, 1024, 1024).into_iter();

        b.iter(|| {
            iter.fold(Point::ZERO, |acc, init| acc + init);
        })
    });

    g.finish();
}
fn manual(c: &mut Criterion) {
    let mut g = c.benchmark_group("manual");

    g.bench_function("iter.map", |b| {
        let bounds = Rect::<Point>::bounds(0, 0, 1024, 1024);

        b.iter(|| {
            (bounds.min.x..bounds.max.x)
                .flat_map(|x| (bounds.min.y..bounds.max.y).map(move |y| Point::new(x, y)))
        })
    });

    g.bench_function("iter.for_each", |b| {
        let bounds = Rect::<Point>::bounds(0, 0, 1024, 1024);

        b.iter(|| {
            (bounds.min.x..bounds.max.x)
                .flat_map(|x| (bounds.min.y..bounds.max.y).map(move |y| Point::new(x, y)))
                .for_each(|p| {
                    black_box(p);
                })
        })
    });

    g.bench_function("iter.fold", |b| {
        let bounds = Rect::<Point>::bounds(0, 0, 1024, 1024);

        b.iter(|| {
            (bounds.min.x..bounds.max.x)
                .flat_map(|x| (bounds.min.y..bounds.max.y).map(move |y| Point::new(x, y)))
                .fold(Point::ZERO, |acc, init| acc + init);
        })
    });

    g.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().with_output_color(true).with_plots();
    targets = step, manual
);

criterion_main!(benches);
