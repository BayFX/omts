//! Group 8: Redaction benchmarks.
#![allow(clippy::expect_used)]

use std::collections::HashSet;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use omtsf_bench::{SizeTier, generate_supply_chain};
use omtsf_core::enums::DisclosureScope;
use omtsf_core::redact;

fn bench_redact_partner(c: &mut Criterion) {
    let mut group = c.benchmark_group("redact_partner");

    for (name, tier) in [
        ("S", SizeTier::Small),
        ("M", SizeTier::Medium),
        ("L", SizeTier::Large),
        ("XL", SizeTier::XLarge),
    ] {
        let file = generate_supply_chain(&tier.config(42));
        let all_ids: HashSet<_> = file.nodes.iter().map(|n| n.id.clone()).collect();

        group.bench_with_input(BenchmarkId::new("retain_all", name), &file, |b, file| {
            b.iter(|| {
                let _ = redact(file, DisclosureScope::Partner, &all_ids).expect("redact succeeds");
            });
        });
    }
    group.finish();
}

fn bench_redact_public(c: &mut Criterion) {
    let mut group = c.benchmark_group("redact_public");

    for (name, tier) in [
        ("S", SizeTier::Small),
        ("M", SizeTier::Medium),
        ("L", SizeTier::Large),
        ("XL", SizeTier::XLarge),
    ] {
        let file = generate_supply_chain(&tier.config(42));
        let all_ids: HashSet<_> = file.nodes.iter().map(|n| n.id.clone()).collect();

        group.bench_with_input(BenchmarkId::new("retain_all", name), &file, |b, file| {
            b.iter(|| {
                let _ = redact(file, DisclosureScope::Public, &all_ids).expect("redact succeeds");
            });
        });
    }
    group.finish();
}

fn bench_redact_varying_retain(c: &mut Criterion) {
    let mut group = c.benchmark_group("redact_varying_retain");

    let file = generate_supply_chain(&SizeTier::Medium.config(42));
    let all_ids: Vec<_> = file.nodes.iter().map(|n| n.id.clone()).collect();
    let n = all_ids.len();

    for (pct_label, fraction) in [("10pct", 0.1), ("50pct", 0.5), ("90pct", 0.9)] {
        let count = ((n as f64) * fraction) as usize;
        let retain: HashSet<_> = all_ids[..count].iter().cloned().collect();

        group.bench_function(BenchmarkId::new("partner", pct_label), |b| {
            b.iter(|| {
                let _ = redact(&file, DisclosureScope::Partner, &retain).expect("redact succeeds");
            });
        });

        group.bench_function(BenchmarkId::new("public", pct_label), |b| {
            b.iter(|| {
                let _ = redact(&file, DisclosureScope::Public, &retain).expect("redact succeeds");
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_redact_partner,
    bench_redact_public,
    bench_redact_varying_retain
);
criterion_main!(benches);
