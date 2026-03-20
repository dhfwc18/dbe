use criterion::{Criterion, black_box, criterion_group, criterion_main};
use dbe_cashflow as cashflow;

fn bench_pv_batch(c: &mut Criterion) {
    c.bench_function("cal_pv batch of 1000", |b| {
        b.iter(|| {
            for i in 0..1000 {
                black_box(cashflow::cal_pv(1000.0, 0.05, i as f64));
            }
        })
    });
}

criterion_group!(benches, bench_pv_batch);
criterion_main!(benches);
