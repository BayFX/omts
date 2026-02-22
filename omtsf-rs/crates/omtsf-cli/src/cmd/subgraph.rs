//! Implementation of `omtsf subgraph <file> [node-id...] [selector flags]`.
//!
//! Parses an `.omts` file, builds the directed graph, and extracts the induced
//! subgraph for seed nodes selected by explicit IDs, property-based selector
//! flags, or both.  Optionally expands the seed set by `--expand` hops before
//! computing the induced subgraph.
//!
//! Flags:
//! - `--node-type`, `--edge-type`, `--label`, `--identifier`, `--jurisdiction`,
//!   `--name` (repeatable selector flags)
//! - `--expand <n>` (default 0): include neighbours up to `n` hops from the
//!   seed nodes before computing the induced subgraph.
//! - `--to <encoding>` (default json): output encoding (`json` or `cbor`).
//! - `--compress`: wrap serialized output in a zstd frame.
//!
//! Output: a valid `.omts` file written to stdout in the requested encoding.
//! The `--format` flag does not affect this command (the spec requires `.omts`
//! output regardless of `--format`).
//!
//! Exit codes: 0 = success, 1 = one or more node IDs not found or no selector
//! matches, 2 = parse/build failure or missing arguments.
use std::collections::HashSet;
use std::io::Write as _;

use omtsf_core::OmtsFile;
use omtsf_core::graph::queries::Direction as CoreDirection;
use omtsf_core::graph::{QueryError, build_graph, ego_graph, induced_subgraph, selector_subgraph};
use omtsf_core::newtypes::CalendarDate;

use crate::TargetEncoding;
use crate::cmd::init::today_string;
use crate::cmd::selectors::build_selector_set;
use crate::error::CliError;

/// Runs the `subgraph` command.
///
/// Seeds are collected from two sources:
/// 1. Explicit `node_ids` passed as positional arguments.
/// 2. Nodes (and edge endpoints) matched by selector flags.
///
/// At least one of these sources must be non-empty; otherwise an
/// `InvalidArgument` error is returned (exit 2).
///
/// When `expand > 0`, the neighbourhood of each seed node (within `expand`
/// hops in both directions) is added to the node set before the induced
/// subgraph is computed.
///
/// The resulting `.omts` file is serialized to stdout using `to` and,
/// optionally, compressed with zstd when `compress` is `true`.
///
/// # Errors
///
/// - [`CliError`] exit code 2 if the graph cannot be built, serialization
///   fails, or neither node IDs nor selectors were provided.
/// - [`CliError`] exit code 1 if any explicit node ID is not found in the
///   graph, or if selectors matched no elements.
#[allow(clippy::too_many_arguments)]
pub fn run(
    file: &OmtsFile,
    node_ids: &[String],
    node_types: &[String],
    edge_types: &[String],
    labels: &[String],
    identifiers: &[String],
    jurisdictions: &[String],
    names: &[String],
    expand: u32,
    to: &TargetEncoding,
    compress: bool,
) -> Result<(), CliError> {
    let has_selectors = !node_types.is_empty()
        || !edge_types.is_empty()
        || !labels.is_empty()
        || !identifiers.is_empty()
        || !jurisdictions.is_empty()
        || !names.is_empty();

    if node_ids.is_empty() && !has_selectors {
        return Err(CliError::InvalidArgument {
            detail: "at least one node ID or selector flag is required \
                     (--node-type, --edge-type, --label, --identifier, --jurisdiction, --name)"
                .to_owned(),
        });
    }

    let graph = build_graph(file).map_err(|e| CliError::GraphBuildError {
        detail: e.to_string(),
    })?;

    // Collect seed node IDs from selectors (expand=0 to get just the seeds).
    let mut seed_ids: HashSet<String> = HashSet::new();

    if has_selectors {
        let selector_set = build_selector_set(
            node_types,
            edge_types,
            labels,
            identifiers,
            jurisdictions,
            names,
        )?;
        let selector_result =
            selector_subgraph(&graph, file, &selector_set, 0).map_err(|e| match e {
                QueryError::EmptyResult => CliError::NoResults {
                    detail: "no nodes or edges matched the given selectors".to_owned(),
                },
                QueryError::NodeNotFound(id) => CliError::NodeNotFound { node_id: id },
            })?;
        for node in &selector_result.nodes {
            seed_ids.insert(node.id.to_string());
        }
    }

    // Add explicit node IDs (validate they exist).
    for id in node_ids {
        if graph.node_index(id).is_none() {
            return Err(CliError::NodeNotFound {
                node_id: id.clone(),
            });
        }
        seed_ids.insert(id.clone());
    }

    // Expand and extract induced subgraph.
    let mut subgraph_file = if expand == 0 {
        let id_refs: Vec<&str> = seed_ids.iter().map(String::as_str).collect();
        induced_subgraph(&graph, file, &id_refs).map_err(query_error_to_cli)?
    } else {
        compute_expanded_subgraph(&graph, file, &seed_ids, expand)?
    };

    let today = today_string().map_err(|e| CliError::IoError {
        source: "system clock".to_owned(),
        detail: e,
    })?;
    subgraph_file.snapshot_date =
        CalendarDate::try_from(today.as_str()).map_err(|e| CliError::IoError {
            source: "system clock".to_owned(),
            detail: format!("generated date is invalid: {e}"),
        })?;

    let bytes = serialize(&subgraph_file, to, compress)?;

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    out.write_all(&bytes).map_err(|e| CliError::IoError {
        source: "stdout".to_owned(),
        detail: e.to_string(),
    })?;

    // Append a trailing newline for uncompressed JSON so the shell prompt
    // appears on a new line.  Binary outputs (CBOR, any compressed payload)
    // must not have an appended newline because that would corrupt the stream.
    let is_text_output = matches!(to, TargetEncoding::Json) && !compress;
    if is_text_output {
        out.write_all(b"\n").map_err(|e| CliError::IoError {
            source: "stdout".to_owned(),
            detail: e.to_string(),
        })?;
    }

    Ok(())
}

/// Serializes `file` to bytes using the requested encoding and optional
/// compression.
///
/// - `--to json` (default): pretty-printed JSON.
/// - `--to cbor`: CBOR with self-describing tag 55799.
/// - `--compress`: wraps the serialized bytes in a zstd frame.
fn serialize(file: &OmtsFile, to: &TargetEncoding, compress: bool) -> Result<Vec<u8>, CliError> {
    match to {
        TargetEncoding::Cbor => omtsf_core::convert(file, omtsf_core::Encoding::Cbor, compress)
            .map_err(|e| CliError::InternalError {
                detail: e.to_string(),
            }),
        TargetEncoding::Json => {
            let json_bytes =
                serde_json::to_vec_pretty(file).map_err(|e| CliError::InternalError {
                    detail: format!("JSON pretty-print failed: {e}"),
                })?;
            if compress {
                omtsf_core::compress_zstd(&json_bytes).map_err(|e| CliError::InternalError {
                    detail: format!("zstd compression failed: {e}"),
                })
            } else {
                Ok(json_bytes)
            }
        }
    }
}

/// Computes the induced subgraph after expanding each node in `seed_ids` by
/// `expand` hops in both directions.
///
/// Algorithm:
/// 1. For each node in `seed_ids`, compute the ego-graph with radius `expand`
///    and direction `Both`.
/// 2. Union all ego-graph node sets.
/// 3. Extract the induced subgraph of the union.
fn compute_expanded_subgraph(
    graph: &omtsf_core::graph::OmtsGraph,
    file: &OmtsFile,
    seed_ids: &HashSet<String>,
    expand: u32,
) -> Result<OmtsFile, CliError> {
    let mut expanded_ids: HashSet<String> = HashSet::new();

    for id in seed_ids {
        let ego = ego_graph(graph, file, id, expand as usize, CoreDirection::Both)
            .map_err(query_error_to_cli)?;
        for node in &ego.nodes {
            expanded_ids.insert(node.id.to_string());
        }
    }

    let id_refs: Vec<&str> = expanded_ids.iter().map(String::as_str).collect();
    induced_subgraph(graph, file, &id_refs).map_err(query_error_to_cli)
}

/// Converts a [`QueryError`] to the appropriate [`CliError`].
fn query_error_to_cli(e: QueryError) -> CliError {
    match e {
        QueryError::NodeNotFound(id) => CliError::NodeNotFound { node_id: id },
        QueryError::EmptyResult => CliError::NoResults {
            detail: "no elements matched the given selectors".to_owned(),
        },
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]
    #![allow(clippy::panic)]

    use super::*;
    const SAMPLE_FILE: &str = r#"{
        "omtsf_version": "1.0.0",
        "snapshot_date": "2026-02-19",
        "file_salt": "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef",
        "nodes": [
            {"id": "org-1", "type": "organization", "name": "Acme Corp"},
            {"id": "fac-1", "type": "facility"},
            {"id": "org-2", "type": "organization", "name": "Beta Ltd"}
        ],
        "edges": [
            {"id": "e-1", "type": "supplies", "source": "org-1", "target": "fac-1"},
            {"id": "e-2", "type": "supplies", "source": "fac-1", "target": "org-2"}
        ]
    }"#;

    fn empty() -> Vec<String> {
        vec![]
    }

    fn strs(v: &[&str]) -> Vec<String> {
        v.iter().map(std::string::ToString::to_string).collect()
    }

    fn parse(s: &str) -> OmtsFile {
        serde_json::from_str(s).expect("valid OMTS JSON")
    }

    /// Selecting organizations with expand=0 produces a valid .omts file.
    #[test]
    fn test_selector_org_expand_0() {
        let file = parse(SAMPLE_FILE);
        let result = run(
            &file,
            &empty(),
            &strs(&["organization"]),
            &empty(),
            &empty(),
            &empty(),
            &empty(),
            &empty(),
            0,
            &TargetEncoding::Json,
            false,
        );
        assert!(result.is_ok(), "should succeed: {result:?}");
    }

    /// Selecting facilities with expand=1 includes adjacent org nodes.
    #[test]
    fn test_selector_facility_expand_1() {
        let file = parse(SAMPLE_FILE);
        let result = run(
            &file,
            &empty(),
            &strs(&["facility"]),
            &empty(),
            &empty(),
            &empty(),
            &empty(),
            &empty(),
            1,
            &TargetEncoding::Json,
            false,
        );
        assert!(result.is_ok(), "should succeed with expand=1: {result:?}");
    }

    /// Selecting by edge type succeeds when matching edges exist.
    #[test]
    fn test_selector_edge_type_supplies() {
        let file = parse(SAMPLE_FILE);
        let result = run(
            &file,
            &empty(),
            &empty(),
            &strs(&["supplies"]),
            &empty(),
            &empty(),
            &empty(),
            &empty(),
            0,
            &TargetEncoding::Json,
            false,
        );
        assert!(
            result.is_ok(),
            "should succeed for supplies edges: {result:?}"
        );
    }

    /// Selecting a type with no matches returns `NoResults` (exit code 1).
    #[test]
    fn test_selector_no_match_returns_exit_1() {
        let file = parse(SAMPLE_FILE);
        let result = run(
            &file,
            &empty(),
            &strs(&["good"]),
            &empty(),
            &empty(),
            &empty(),
            &empty(),
            &empty(),
            0,
            &TargetEncoding::Json,
            false,
        );
        let err = result.expect_err("no good nodes -> NoResults");
        assert_eq!(err.exit_code(), 1);
    }

    /// No node IDs and no selector flags returns `InvalidArgument` (exit code 2).
    #[test]
    fn test_no_ids_no_selectors_returns_exit_2() {
        let file = parse(SAMPLE_FILE);
        let result = run(
            &file,
            &empty(),
            &empty(),
            &empty(),
            &empty(),
            &empty(),
            &empty(),
            &empty(),
            0,
            &TargetEncoding::Json,
            false,
        );
        let err = result.expect_err("no selectors -> error");
        assert_eq!(err.exit_code(), 2);
    }

    /// The subgraph produced from selector extraction round-trips through serde.
    #[test]
    fn test_selector_produces_valid_omts() {
        let file: OmtsFile = serde_json::from_str(SAMPLE_FILE).expect("parse");
        let graph = omtsf_core::build_graph(&file).expect("build");

        let selector_set = crate::cmd::selectors::build_selector_set(
            &strs(&["organization"]),
            &[],
            &[],
            &[],
            &[],
            &[],
        )
        .expect("build selector set");

        let subgraph = omtsf_core::graph::selector_subgraph(&graph, &file, &selector_set, 0)
            .expect("extract subgraph");

        let json = serde_json::to_string_pretty(&subgraph).expect("serialize");
        let back: OmtsFile = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.nodes.len(), subgraph.nodes.len());
        assert_eq!(back.edges.len(), subgraph.edges.len());
    }

    /// expand=0 with org selector produces only org nodes (no facility),
    /// since facility is not in the seed and there is no expansion.
    #[test]
    fn test_selector_expand_0_excludes_non_seeds() {
        let file: OmtsFile = serde_json::from_str(SAMPLE_FILE).expect("parse");
        let graph = omtsf_core::build_graph(&file).expect("build");

        let selector_set = crate::cmd::selectors::build_selector_set(
            &strs(&["organization"]),
            &[],
            &[],
            &[],
            &[],
            &[],
        )
        .expect("build selector set");

        let subgraph = omtsf_core::graph::selector_subgraph(&graph, &file, &selector_set, 0)
            .expect("extract subgraph");

        let ids: Vec<String> = subgraph.nodes.iter().map(|n| n.id.to_string()).collect();
        assert!(ids.contains(&"org-1".to_owned()), "org-1 must be present");
        assert!(ids.contains(&"org-2".to_owned()), "org-2 must be present");
        assert!(
            !ids.contains(&"fac-1".to_owned()),
            "fac-1 must not be in seed-only result"
        );
    }

    /// expand=1 with org selector includes facility (1 hop from orgs).
    #[test]
    fn test_selector_expand_1_includes_adjacent() {
        let file: OmtsFile = serde_json::from_str(SAMPLE_FILE).expect("parse");
        let graph = omtsf_core::build_graph(&file).expect("build");

        let selector_set = crate::cmd::selectors::build_selector_set(
            &strs(&["organization"]),
            &[],
            &[],
            &[],
            &[],
            &[],
        )
        .expect("build selector set");

        let subgraph = omtsf_core::graph::selector_subgraph(&graph, &file, &selector_set, 1)
            .expect("extract subgraph");

        let ids: Vec<String> = subgraph.nodes.iter().map(|n| n.id.to_string()).collect();
        assert!(
            ids.contains(&"fac-1".to_owned()),
            "fac-1 must be included with expand=1"
        );
        assert_eq!(subgraph.nodes.len(), 3, "all 3 nodes with expand=1");
    }

    /// Combining explicit node IDs with selectors unions both seed sets.
    #[test]
    fn test_combined_node_ids_and_selectors() {
        let file = parse(SAMPLE_FILE);
        let result = run(
            &file,
            &strs(&["fac-1"]),
            &strs(&["organization"]),
            &empty(),
            &empty(),
            &empty(),
            &empty(),
            &empty(),
            0,
            &TargetEncoding::Json,
            false,
        );
        assert!(result.is_ok(), "should succeed: {result:?}");
    }

    /// Explicit node IDs alone (no selectors) still works.
    #[test]
    fn test_explicit_node_ids_only() {
        let file = parse(SAMPLE_FILE);
        let result = run(
            &file,
            &strs(&["org-1"]),
            &empty(),
            &empty(),
            &empty(),
            &empty(),
            &empty(),
            &empty(),
            0,
            &TargetEncoding::Json,
            false,
        );
        assert!(result.is_ok(), "should succeed: {result:?}");
    }
}
