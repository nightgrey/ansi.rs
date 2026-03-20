use criterion::{Criterion, criterion_group, criterion_main};
use ansi::Attribute;


fn lib(c: &mut Criterion) {
    c.bench_function("sgr", |b| b.iter(|| Attribute::MAX.sgr().collect::<Vec<_>>()));
}

criterion_group!(benches, lib);
criterion_main!(benches);
