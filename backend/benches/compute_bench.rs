use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use tsi_backend::compute::analyze_values;

fn bench_analyze_small(c: &mut Criterion) {
    let values: Vec<f64> = (0..100).map(|i| i as f64).collect();
    
    c.bench_function("analyze_values 100", |b| {
        b.iter(|| analyze_values(black_box(&values)))
    });
}

fn bench_analyze_medium(c: &mut Criterion) {
    let values: Vec<f64> = (0..10_000).map(|i| i as f64).collect();
    
    c.bench_function("analyze_values 10k", |b| {
        b.iter(|| analyze_values(black_box(&values)))
    });
}

fn bench_analyze_large(c: &mut Criterion) {
    let values: Vec<f64> = (0..1_000_000).map(|i| i as f64).collect();
    
    c.bench_function("analyze_values 1M", |b| {
        b.iter(|| analyze_values(black_box(&values)))
    });
}

fn bench_analyze_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("analyze_scaling");
    
    for size in [100, 1_000, 10_000, 100_000].iter() {
        let values: Vec<f64> = (0..*size).map(|i| i as f64).collect();
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| analyze_values(black_box(&values)));
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_analyze_small,
    bench_analyze_medium,
    bench_analyze_large,
    bench_analyze_scaling
);
criterion_main!(benches);
