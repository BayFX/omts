//! Group 5: Cycle detection benchmarks.
#![allow(clippy::expect_used)]

use std::collections::HashSet;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use omts_bench::{SizeTier, generate_supply_chain};
use omts_core::build_graph;
use omts_core::enums::{EdgeType, EdgeTypeTag};
use omts_core::graph::cycles::detect_cycles;

fn all_edge_types() -> HashSet<EdgeTypeTag> {
    [
        EdgeType::Supplies,
        EdgeType::Subcontracts,
        EdgeType::Brokers,
        EdgeType::SellsTo,
        EdgeType::Ownership,
        EdgeType::BeneficialOwnership,
        EdgeType::OperationalControl,
        EdgeType::LegalParentage,
        EdgeType::FormerIdentity,
        EdgeType::Distributes,
        EdgeType::ComposedOf,
        EdgeType::AttestedBy,
        EdgeType::SameAs,
        EdgeType::Operates,
        EdgeType::Produces,
        EdgeType::Tolls,
    ]
    .into_iter()
    .map(EdgeTypeTag::Known)
    .collect()
}

fn legal_parentage_only() -> HashSet<EdgeTypeTag> {
    [EdgeTypeTag::Known(EdgeType::LegalParentage)]
        .into_iter()
        .collect()
}

fn bench_acyclic(c: &mut Criterion) {
    let mut group = c.benchmark_group("cycle_detection_acyclic");

    for (name, tier) in [
        ("S", SizeTier::Small),
        ("M", SizeTier::Medium),
        ("L", SizeTier::Large),
        ("XL", SizeTier::XLarge),
    ] {
        let file = generate_supply_chain(&tier.config(42));
        let graph = build_graph(&file).expect("builds");

        group.bench_function(BenchmarkId::new("all_types", name), |b| {
            let types = all_edge_types();
            b.iter(|| {
                let _ = detect_cycles(&graph, &types);
            });
        });

        group.bench_function(BenchmarkId::new("legal_parentage", name), |b| {
            let types = legal_parentage_only();
            b.iter(|| {
                let _ = detect_cycles(&graph, &types);
            });
        });
    }
    group.finish();
}

fn bench_cyclic(c: &mut Criterion) {
    let mut group = c.benchmark_group("cycle_detection_cyclic");

    for (name, tier) in [
        ("S", SizeTier::Small),
        ("M", SizeTier::Medium),
        ("L", SizeTier::Large),
    ] {
        let mut config = tier.config(42);
        config.inject_cycles = true;
        let file = generate_supply_chain(&config);
        let graph = build_graph(&file).expect("builds");

        group.bench_function(BenchmarkId::new("all_types", name), |b| {
            let types = all_edge_types();
            b.iter(|| {
                let _ = detect_cycles(&graph, &types);
            });
        });

        group.bench_function(BenchmarkId::new("legal_parentage", name), |b| {
            let types = legal_parentage_only();
            b.iter(|| {
                let _ = detect_cycles(&graph, &types);
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_acyclic, bench_cyclic);
criterion_main!(benches);
