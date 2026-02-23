//! Huge-tier benchmarks (~737K nodes, ~1.5M edges, ~500 MB JSON, 20-tier supply chain).
//!
//! This benchmark binary is intentionally separate from the smaller-tier benchmarks
//! so that `cargo bench` remains fast for development. Run via `just bench-huge`.
//!
//! The JSON fixture is pre-generated to disk by `just gen-huge` and loaded here.
//! CBOR benchmarks live in `huge_cbor.rs` (separate binary to avoid OOM).
//!
//! Setup is split into two phases to avoid OOM:
//! - `get_base_setup()`: parses the fixture into `OmtsFile` (no graph).
//!   Used by serde benchmarks that need the JSON string alongside the file.
//! - `get_graph_setup()`: additionally builds the `OmtsGraph`.
//!   Used by graph query, selector, and validation benchmarks.
#![allow(clippy::expect_used)]

use std::collections::HashSet;
use std::sync::OnceLock;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use omts_bench::huge_fixture_path;
use omts_core::enums::{EdgeType, EdgeTypeTag, NodeType, NodeTypeTag};
use omts_core::graph::queries::{self, Direction};
use omts_core::graph::{OmtsGraph, Selector, SelectorSet, extraction};
use omts_core::validation::{ValidationConfig, validate};
use omts_core::{OmtsFile, build_graph};

struct BaseSetup {
    file: OmtsFile,
    node_count: usize,
    edge_count: usize,
    byte_size: u64,
    root_id: String,
    leaf_id: String,
    mid_id: String,
}

struct GraphSetup {
    base: &'static BaseSetup,
    graph: OmtsGraph,
}

static BASE: OnceLock<BaseSetup> = OnceLock::new();
static GRAPH: OnceLock<GraphSetup> = OnceLock::new();

fn get_base_setup() -> &'static BaseSetup {
    BASE.get_or_init(|| {
        let path = huge_fixture_path();
        eprintln!("Loading huge fixture from {}...", path.display());
        let json = std::fs::read_to_string(&path)
            .expect("Failed to read huge fixture. Run `just gen-huge` first to generate it.");
        let byte_size = json.len() as u64;
        let file: OmtsFile = serde_json::from_str(&json).expect("deserialize huge fixture");
        let node_count = file.nodes.len();
        let edge_count = file.edges.len();
        eprintln!(
            "Huge tier ready: {} nodes, {} edges, {:.1} MB JSON",
            node_count,
            edge_count,
            byte_size as f64 / (1024.0 * 1024.0)
        );

        let root_id = file.nodes[0].id.to_string();
        let leaf_id = file.nodes[file.nodes.len() - 1].id.to_string();
        let mid_id = file.nodes[file.nodes.len() / 2].id.to_string();

        BaseSetup {
            file,
            node_count,
            edge_count,
            byte_size,
            root_id,
            leaf_id,
            mid_id,
        }
    })
}

fn get_graph_setup() -> &'static GraphSetup {
    GRAPH.get_or_init(|| {
        let base = get_base_setup();
        eprintln!("Building graph...");
        let graph = build_graph(&base.file).expect("builds");
        eprintln!("Graph ready.");
        GraphSetup { base, graph }
    })
}

fn bench_huge_deserialize(c: &mut Criterion) {
    let path = huge_fixture_path();
    let json = std::fs::read_to_string(&path).expect("read huge fixture for deserialize bench");
    let byte_size = json.len() as u64;

    let mut group = c.benchmark_group("huge/deserialize");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(30));
    group.throughput(Throughput::Bytes(byte_size));

    group.bench_function(BenchmarkId::from_parameter("Huge"), |b| {
        b.iter(|| {
            let _: OmtsFile = serde_json::from_str(&json).expect("deserialize");
        });
    });
    group.finish();
}

fn bench_huge_serialize(c: &mut Criterion) {
    let s = get_base_setup();
    let mut group = c.benchmark_group("huge/serialize_compact");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(20));
    group.throughput(Throughput::Bytes(s.byte_size));

    group.bench_function(BenchmarkId::from_parameter("Huge"), |b| {
        b.iter(|| {
            let _ = serde_json::to_string(&s.file).expect("serialize");
        });
    });
    group.finish();
}

fn bench_huge_build_graph(c: &mut Criterion) {
    let s = get_base_setup();
    let elements = (s.node_count + s.edge_count) as u64;

    let mut group = c.benchmark_group("huge/build_graph");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(15));
    group.throughput(Throughput::Elements(elements));

    group.bench_function(BenchmarkId::from_parameter("Huge"), |b| {
        b.iter(|| {
            let _ = build_graph(&s.file).expect("builds");
        });
    });
    group.finish();
}

fn bench_huge_reachability(c: &mut Criterion) {
    let gs = get_graph_setup();
    let mut group = c.benchmark_group("huge/reachable_from");
    group.sample_size(20);
    group.measurement_time(std::time::Duration::from_secs(30));

    group.bench_function(BenchmarkId::new("forward_root", "Huge"), |b| {
        b.iter(|| {
            let _ = queries::reachable_from(&gs.graph, &gs.base.root_id, Direction::Forward, None)
                .expect("works");
        });
    });

    let filter: HashSet<EdgeTypeTag> = [EdgeTypeTag::Known(EdgeType::Supplies)]
        .into_iter()
        .collect();
    group.bench_function(BenchmarkId::new("filtered_supplies", "Huge"), |b| {
        b.iter(|| {
            let _ = queries::reachable_from(
                &gs.graph,
                &gs.base.root_id,
                Direction::Forward,
                Some(&filter),
            )
            .expect("works");
        });
    });

    group.bench_function(BenchmarkId::new("both_mid", "Huge"), |b| {
        b.iter(|| {
            let _ = queries::reachable_from(&gs.graph, &gs.base.mid_id, Direction::Both, None)
                .expect("works");
        });
    });

    group.finish();
}

fn bench_huge_shortest_path(c: &mut Criterion) {
    let gs = get_graph_setup();
    let mut group = c.benchmark_group("huge/shortest_path");
    group.sample_size(20);
    group.measurement_time(std::time::Duration::from_secs(30));

    group.bench_function(BenchmarkId::new("root_to_leaf", "Huge"), |b| {
        b.iter(|| {
            let _ = queries::shortest_path(
                &gs.graph,
                &gs.base.root_id,
                &gs.base.leaf_id,
                Direction::Forward,
                None,
            )
            .expect("works");
        });
    });

    group.bench_function(BenchmarkId::new("root_to_mid", "Huge"), |b| {
        b.iter(|| {
            let _ = queries::shortest_path(
                &gs.graph,
                &gs.base.root_id,
                &gs.base.mid_id,
                Direction::Forward,
                None,
            )
            .expect("works");
        });
    });

    group.bench_function(BenchmarkId::new("no_path", "Huge"), |b| {
        b.iter(|| {
            let _ = queries::shortest_path(
                &gs.graph,
                &gs.base.leaf_id,
                &gs.base.root_id,
                Direction::Forward,
                None,
            )
            .expect("works");
        });
    });

    group.finish();
}

fn bench_huge_selector_match(c: &mut Criterion) {
    let gs = get_graph_setup();
    let element_count = (gs.base.node_count + gs.base.edge_count) as u64;

    let mut group = c.benchmark_group("huge/selector_match");
    group.sample_size(20);
    group.throughput(Throughput::Elements(element_count));

    let label_ss = SelectorSet::from_selectors(vec![Selector::LabelKey("certified".to_owned())]);
    group.bench_function(BenchmarkId::new("label", "Huge"), |b| {
        b.iter(|| {
            let _ = extraction::selector_match(&gs.base.file, &label_ss);
        });
    });

    let type_ss = SelectorSet::from_selectors(vec![Selector::NodeType(NodeTypeTag::Known(
        NodeType::Organization,
    ))]);
    group.bench_function(BenchmarkId::new("node_type", "Huge"), |b| {
        b.iter(|| {
            let _ = extraction::selector_match(&gs.base.file, &type_ss);
        });
    });

    let multi_ss = SelectorSet::from_selectors(vec![
        Selector::NodeType(NodeTypeTag::Known(NodeType::Organization)),
        Selector::LabelKey("certified".to_owned()),
    ]);
    group.bench_function(BenchmarkId::new("multi", "Huge"), |b| {
        b.iter(|| {
            let _ = extraction::selector_match(&gs.base.file, &multi_ss);
        });
    });

    group.finish();
}

fn bench_huge_selector_subgraph(c: &mut Criterion) {
    let gs = get_graph_setup();

    let mut group = c.benchmark_group("huge/selector_subgraph");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(60));

    let narrow_ss = SelectorSet::from_selectors(vec![Selector::NodeType(NodeTypeTag::Known(
        NodeType::Attestation,
    ))]);

    let output = extraction::selector_subgraph(&gs.graph, &gs.base.file, &narrow_ss, 0)
        .expect("attestations exist");
    let out_nodes = output.nodes.len() as u64;
    group.throughput(Throughput::Elements(out_nodes.max(1)));

    group.bench_function(BenchmarkId::new("narrow_exp0", "Huge"), |b| {
        b.iter(|| {
            let _ = extraction::selector_subgraph(&gs.graph, &gs.base.file, &narrow_ss, 0)
                .expect("works");
        });
    });

    group.bench_function(BenchmarkId::new("narrow_exp1", "Huge"), |b| {
        b.iter(|| {
            let _ = extraction::selector_subgraph(&gs.graph, &gs.base.file, &narrow_ss, 1)
                .expect("works");
        });
    });

    group.bench_function(BenchmarkId::new("narrow_exp3", "Huge"), |b| {
        b.iter(|| {
            let _ = extraction::selector_subgraph(&gs.graph, &gs.base.file, &narrow_ss, 3)
                .expect("works");
        });
    });

    let broad_ss = SelectorSet::from_selectors(vec![Selector::NodeType(NodeTypeTag::Known(
        NodeType::Organization,
    ))]);

    group.bench_function(BenchmarkId::new("broad_exp0", "Huge"), |b| {
        b.iter(|| {
            let _ = extraction::selector_subgraph(&gs.graph, &gs.base.file, &broad_ss, 0)
                .expect("works");
        });
    });

    group.bench_function(BenchmarkId::new("broad_exp1", "Huge"), |b| {
        b.iter(|| {
            let _ = extraction::selector_subgraph(&gs.graph, &gs.base.file, &broad_ss, 1)
                .expect("works");
        });
    });

    group.finish();
}

fn bench_huge_validation(c: &mut Criterion) {
    let gs = get_graph_setup();
    let elements = (gs.base.node_count + gs.base.edge_count) as u64;

    let mut group = c.benchmark_group("huge/validation");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(60));
    group.throughput(Throughput::Elements(elements));

    group.bench_function(BenchmarkId::new("L1", "Huge"), |b| {
        let config = ValidationConfig {
            run_l1: true,
            run_l2: false,
            run_l3: false,
        };
        b.iter(|| {
            let _ = validate(&gs.base.file, &config, None);
        });
    });

    group.bench_function(BenchmarkId::new("L1_L2_L3", "Huge"), |b| {
        let config = ValidationConfig {
            run_l1: true,
            run_l2: true,
            run_l3: true,
        };
        b.iter(|| {
            let _ = validate(&gs.base.file, &config, None);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_huge_deserialize,
    bench_huge_serialize,
    bench_huge_build_graph,
    bench_huge_reachability,
    bench_huge_shortest_path,
    bench_huge_selector_match,
    bench_huge_selector_subgraph,
    bench_huge_validation,
);
criterion_main!(benches);
