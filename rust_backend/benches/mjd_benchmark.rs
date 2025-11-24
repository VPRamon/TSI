use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use tsi_rust::time::{mjd_to_datetime_rust, datetime_to_mjd_rust, parse_visibility_string};
use tsi_rust::parsing::visibility::VisibilityParser;
use chrono::Utc;

fn bench_mjd_to_datetime(c: &mut Criterion) {
    let mut group = c.benchmark_group("mjd_conversions");
    
    group.bench_function("mjd_to_datetime", |b| {
        b.iter(|| {
            for i in 0..1000 {
                let mjd = 59580.0 + (i as f64 * 0.01);
                black_box(mjd_to_datetime_rust(black_box(mjd)));
            }
        });
    });
    
    group.finish();
}

fn bench_datetime_to_mjd(c: &mut Criterion) {
    let mut group = c.benchmark_group("mjd_conversions");
    
    let dt = Utc::now();
    group.bench_function("datetime_to_mjd", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                black_box(datetime_to_mjd_rust(black_box(&dt)));
            }
        });
    });
    
    group.finish();
}

fn bench_visibility_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("visibility_parsing");
    
    let single = "[(59580.0, 59581.0)]";
    group.bench_with_input(BenchmarkId::new("single_period", "1"), &single, |b, input| {
        b.iter(|| parse_visibility_string(black_box(input)));
    });
    
    let multiple = "[(59580.0, 59580.5), (59581.0, 59581.25), (59582.0, 59582.75)]";
    group.bench_with_input(BenchmarkId::new("multiple_periods", "3"), &multiple, |b, input| {
        b.iter(|| parse_visibility_string(black_box(input)));
    });
    
    let many = "[(59580.0, 59580.1), (59580.2, 59580.3), (59580.4, 59580.5), (59580.6, 59580.7), (59580.8, 59580.9), (59581.0, 59581.1), (59581.2, 59581.3), (59581.4, 59581.5), (59581.6, 59581.7), (59581.8, 59581.9)]";
    group.bench_with_input(BenchmarkId::new("many_periods", "10"), &many, |b, input| {
        b.iter(|| parse_visibility_string(black_box(input)));
    });
    
    group.finish();
}

fn bench_batch_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_parsing");
    
    let inputs: Vec<&str> = (0..100)
        .map(|_| "[(59580.0, 59581.0), (59582.0, 59583.0)]")
        .collect();
    
    group.bench_function("parse_100_strings", |b| {
        b.iter(|| {
            VisibilityParser::parse_batch(black_box(&inputs));
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_mjd_to_datetime,
    bench_datetime_to_mjd,
    bench_visibility_parsing,
    bench_batch_parsing
);
criterion_main!(benches);
