//! Group 2: Graph construction benchmarks.
#![allow(clippy::expect_used)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use omts_bench::{SizeTier, generate_supply_chain};
use omts_core::build_graph;

fn bench_build_graph(c: &mut Criterion) {
    let mut group = c.benchmark_group("build_graph");

    for (name, tier) in [
        ("S", SizeTier::Small),
        ("M", SizeTier::Medium),
        ("L", SizeTier::Large),
        ("XL", SizeTier::XLarge),
    ] {
        let file = generate_supply_chain(&tier.config(42));
        let elements = (file.nodes.len() + file.edges.len()) as u64;

        group.throughput(Throughput::Elements(elements));
        group.bench_with_input(BenchmarkId::new("elements", name), &file, |b, file| {
            b.iter(|| {
                let _ = build_graph(file).expect("builds");
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_build_graph);
criterion_main!(benches);
