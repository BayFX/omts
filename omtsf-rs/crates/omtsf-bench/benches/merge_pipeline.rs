//! Group 7: Merge pipeline benchmarks.
#![allow(clippy::expect_used)]

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use omtsf_bench::{SizeTier, generate_supply_chain};
use omtsf_core::merge;

fn bench_merge_disjoint(c: &mut Criterion) {
    let mut group = c.benchmark_group("merge_disjoint");

    for (name, tier) in [
        ("S", SizeTier::Small),
        ("M", SizeTier::Medium),
        ("L", SizeTier::Large),
    ] {
        let file_a = generate_supply_chain(&tier.config(42));
        let file_b = generate_supply_chain(&tier.config(99));

        group.bench_function(BenchmarkId::new("2_files", name), |b| {
            b.iter(|| {
                let _ = merge(&[file_a.clone(), file_b.clone()]).expect("merge succeeds");
            });
        });
    }
    group.finish();
}

fn bench_merge_self(c: &mut Criterion) {
    let mut group = c.benchmark_group("merge_self");

    for (name, tier) in [
        ("S", SizeTier::Small),
        ("M", SizeTier::Medium),
        ("L", SizeTier::Large),
    ] {
        let file = generate_supply_chain(&tier.config(42));

        group.bench_function(BenchmarkId::new("full_overlap", name), |b| {
            b.iter(|| {
                let _ = merge(&[file.clone(), file.clone()]).expect("merge succeeds");
            });
        });
    }
    group.finish();
}

fn bench_merge_three_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("merge_three_files");
    group.sample_size(20);

    for (name, tier) in [("S", SizeTier::Small), ("M", SizeTier::Medium)] {
        let file_a = generate_supply_chain(&tier.config(42));
        let file_b = generate_supply_chain(&tier.config(99));
        let file_c = generate_supply_chain(&tier.config(7));

        group.bench_function(BenchmarkId::new("3_files", name), |b| {
            b.iter(|| {
                let _ = merge(&[file_a.clone(), file_b.clone(), file_c.clone()])
                    .expect("merge succeeds");
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_merge_disjoint,
    bench_merge_self,
    bench_merge_three_files
);
criterion_main!(benches);
