//! Group 9: Diff benchmarks.
#![allow(clippy::expect_used)]

use std::collections::HashSet;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use omts_bench::{SizeTier, generate_supply_chain};
use omts_core::diff::{DiffFilter, diff, diff_filtered};

fn bench_diff_identical(c: &mut Criterion) {
    let mut group = c.benchmark_group("diff_identical");

    for (name, tier) in [
        ("S", SizeTier::Small),
        ("M", SizeTier::Medium),
        ("L", SizeTier::Large),
        ("XL", SizeTier::XLarge),
    ] {
        let file = generate_supply_chain(&tier.config(42));

        group.bench_function(BenchmarkId::new("self", name), |b| {
            b.iter(|| {
                let _ = diff(&file, &file);
            });
        });
    }
    group.finish();
}

fn bench_diff_disjoint(c: &mut Criterion) {
    let mut group = c.benchmark_group("diff_disjoint");

    for (name, tier) in [
        ("S", SizeTier::Small),
        ("M", SizeTier::Medium),
        ("L", SizeTier::Large),
    ] {
        let file_a = generate_supply_chain(&tier.config(42));
        let file_b = generate_supply_chain(&tier.config(99));

        group.bench_function(BenchmarkId::new("full_diff", name), |b| {
            b.iter(|| {
                let _ = diff(&file_a, &file_b);
            });
        });
    }
    group.finish();
}

fn bench_diff_filtered(c: &mut Criterion) {
    let mut group = c.benchmark_group("diff_filtered");

    for (name, tier) in [
        ("S", SizeTier::Small),
        ("M", SizeTier::Medium),
        ("L", SizeTier::Large),
    ] {
        let file_a = generate_supply_chain(&tier.config(42));
        let file_b = generate_supply_chain(&tier.config(99));

        let node_filter: HashSet<String> = ["organization".to_owned()].into_iter().collect();
        let edge_filter: HashSet<String> = ["supplies".to_owned()].into_iter().collect();
        let filter = DiffFilter {
            node_types: Some(node_filter),
            edge_types: Some(edge_filter),
            ignore_fields: HashSet::new(),
        };

        group.bench_function(BenchmarkId::new("org_supplies_only", name), |b| {
            b.iter(|| {
                let _ = diff_filtered(&file_a, &file_b, Some(&filter));
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_diff_identical,
    bench_diff_disjoint,
    bench_diff_filtered
);
criterion_main!(benches);
