//! E2E test: Import `omts-import-example.xlsx` → exercise every CLI command.
//!
//! Imports the full multi-sheet Excel workbook and runs every CLI command on
//! the result, verifying that commands produce correct, consistent output.
#![allow(clippy::expect_used)]

use std::io::Write as _;
use std::path::PathBuf;
use std::process::Command;

/// Path to the compiled `omts` binary.
fn omts_bin() -> PathBuf {
    let mut path = std::env::current_exe().expect("current exe");
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.push("omts");
    path
}

/// Path to an Excel fixture file relative to the repo root.
fn excel_fixture(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../../templates/excel");
    path.push(name);
    path
}

/// Import → validate → inspect → query → reach → path → subgraph → convert →
/// CBOR round-trip → diff (with JSON verification) → redact → validate(redacted) →
/// export → export round-trip → export(supplier-list).
///
/// Uses `omts-import-example.xlsx` (5 nodes, 5 edges):
///   - 2 organizations (org-acme, org-bolt)
///   - 1 facility (fac-bolt-sheffield)
///   - 1 good (good-steel-bolts)
///   - 1 attestation (att-sa8000)
///   - 5 edges: supplies, operates, produces, ownership, `attested_by`
#[test]
fn pipeline_import_through_all_commands() {
    // -- Step 1: import --
    let import_out = Command::new(omts_bin())
        .args([
            "import",
            excel_fixture("omts-import-example.xlsx")
                .to_str()
                .expect("path"),
        ])
        .output()
        .expect("run omts import");
    assert_eq!(
        import_out.status.code(),
        Some(0),
        "import must succeed; stderr: {}",
        String::from_utf8_lossy(&import_out.stderr)
    );

    let import_stdout = String::from_utf8(import_out.stdout).expect("UTF-8 stdout");
    let graph: serde_json::Value =
        serde_json::from_str(&import_stdout).expect("import output must be valid JSON");
    let nodes = graph["nodes"].as_array().expect("nodes array");
    let edges = graph["edges"].as_array().expect("edges array");
    assert_eq!(nodes.len(), 5, "expected 5 nodes, got {}", nodes.len());
    assert_eq!(edges.len(), 5, "expected 5 edges, got {}", edges.len());

    // Write imported graph to a temp file for subsequent commands.
    let mut imported_tmp = tempfile::NamedTempFile::new().expect("temp file");
    imported_tmp
        .write_all(import_stdout.as_bytes())
        .expect("write imported graph");
    let imported_path = imported_tmp.path().to_str().expect("path").to_owned();

    // -- Step 2: validate --
    let validate_out = Command::new(omts_bin())
        .args(["validate", "--level", "1", &imported_path])
        .output()
        .expect("run omts validate");
    assert_eq!(
        validate_out.status.code(),
        Some(0),
        "validate L1 must succeed; stderr: {}",
        String::from_utf8_lossy(&validate_out.stderr)
    );

    // -- Step 3: inspect --
    let inspect_out = Command::new(omts_bin())
        .args(["inspect", "-f", "json", &imported_path])
        .output()
        .expect("run omts inspect");
    assert_eq!(
        inspect_out.status.code(),
        Some(0),
        "inspect must succeed; stderr: {}",
        String::from_utf8_lossy(&inspect_out.stderr)
    );
    let inspect_stdout = String::from_utf8_lossy(&inspect_out.stdout);
    let inspect_json: serde_json::Value =
        serde_json::from_str(inspect_stdout.trim()).expect("inspect JSON");
    assert_eq!(inspect_json["node_count"], 5, "inspect node_count");
    assert_eq!(inspect_json["edge_count"], 5, "inspect edge_count");
    assert!(
        inspect_json.get("version").is_some(),
        "inspect must include version"
    );

    // -- Step 4: query --node-type organization -f json --
    let query_out = Command::new(omts_bin())
        .args([
            "query",
            "--node-type",
            "organization",
            "-f",
            "json",
            &imported_path,
        ])
        .output()
        .expect("run omts query");
    assert_eq!(
        query_out.status.code(),
        Some(0),
        "query must succeed; stderr: {}",
        String::from_utf8_lossy(&query_out.stderr)
    );
    let query_stdout = String::from_utf8_lossy(&query_out.stdout);
    let query_json: serde_json::Value =
        serde_json::from_str(query_stdout.trim()).expect("query JSON");
    let query_nodes = query_json["nodes"].as_array().expect("query nodes array");
    assert_eq!(
        query_nodes.len(),
        2,
        "import example has exactly 2 organization nodes"
    );
    for node in query_nodes {
        assert_eq!(
            node["type"].as_str(),
            Some("organization"),
            "every queried node must be type organization"
        );
    }

    // -- Step 5: query --count --
    let count_out = Command::new(omts_bin())
        .args([
            "query",
            "--node-type",
            "organization",
            "--count",
            &imported_path,
        ])
        .output()
        .expect("run omts query --count");
    assert_eq!(
        count_out.status.code(),
        Some(0),
        "query --count must succeed; stderr: {}",
        String::from_utf8_lossy(&count_out.stderr)
    );
    let count_stdout = String::from_utf8_lossy(&count_out.stdout);
    assert!(
        count_stdout.contains("nodes:"),
        "count output should contain 'nodes:'; stdout: {count_stdout}"
    );

    // -- Step 6: reach --
    let first_node_id = nodes[0]["id"].as_str().expect("first node id");
    let reach_out = Command::new(omts_bin())
        .args(["reach", "-f", "json", &imported_path, first_node_id])
        .output()
        .expect("run omts reach");
    assert_eq!(
        reach_out.status.code(),
        Some(0),
        "reach must succeed; stderr: {}",
        String::from_utf8_lossy(&reach_out.stderr)
    );
    let reach_stdout = String::from_utf8_lossy(&reach_out.stdout);
    let reach_json: serde_json::Value =
        serde_json::from_str(reach_stdout.trim()).expect("reach JSON");
    let reach_ids = reach_json["node_ids"].as_array().expect("node_ids array");
    let reach_count = reach_json["count"].as_u64().expect("count");
    assert!(
        reach_count >= 1,
        "reach should find at least 1 reachable node"
    );
    assert_eq!(
        reach_ids.len() as u64,
        reach_count,
        "reach count must match node_ids array length"
    );

    // -- Step 6b: reach --direction both --
    let reach_both = Command::new(omts_bin())
        .args([
            "reach",
            "-f",
            "json",
            "--direction",
            "both",
            &imported_path,
            first_node_id,
        ])
        .output()
        .expect("run omts reach --direction both");
    assert_eq!(
        reach_both.status.code(),
        Some(0),
        "reach --direction both must succeed"
    );
    let reach_both_json: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&reach_both.stdout).trim())
            .expect("reach both JSON");
    let reach_both_count = reach_both_json["count"].as_u64().expect("count");
    assert!(
        reach_both_count >= reach_count,
        "bidirectional reach ({reach_both_count}) must find >= outgoing reach ({reach_count})"
    );

    // -- Step 7: path --
    let edge_source = edges[0]["source"].as_str().expect("edge source");
    let edge_target = edges[0]["target"].as_str().expect("edge target");
    let path_out = Command::new(omts_bin())
        .args([
            "path",
            "-f",
            "json",
            &imported_path,
            edge_source,
            edge_target,
        ])
        .output()
        .expect("run omts path");
    assert_eq!(
        path_out.status.code(),
        Some(0),
        "path must succeed; stderr: {}",
        String::from_utf8_lossy(&path_out.stderr)
    );
    let path_stdout = String::from_utf8_lossy(&path_out.stdout);
    let path_json: serde_json::Value = serde_json::from_str(path_stdout.trim()).expect("path JSON");
    let paths = path_json["paths"].as_array().expect("paths array");
    assert!(!paths.is_empty(), "paths array should be non-empty");
    let path_count = path_json["count"].as_u64().expect("count");
    assert_eq!(
        paths.len() as u64,
        path_count,
        "path count must match paths array length"
    );

    // -- Step 8: subgraph --
    let subgraph_out = Command::new(omts_bin())
        .args(["subgraph", &imported_path, "--node-type", "organization"])
        .output()
        .expect("run omts subgraph");
    assert_eq!(
        subgraph_out.status.code(),
        Some(0),
        "subgraph must succeed; stderr: {}",
        String::from_utf8_lossy(&subgraph_out.stderr)
    );
    let subgraph_stdout = String::from_utf8_lossy(&subgraph_out.stdout);
    let subgraph_json: serde_json::Value =
        serde_json::from_str(subgraph_stdout.trim()).expect("subgraph JSON");
    let sub_nodes = subgraph_json["nodes"].as_array().expect("subgraph nodes");
    let sub_edges = subgraph_json["edges"].as_array().expect("subgraph edges");
    assert_eq!(sub_nodes.len(), 2, "org subgraph has exactly 2 org nodes");
    assert_eq!(
        sub_edges.len(),
        2,
        "org subgraph has 2 edges between orgs (supplies + ownership)"
    );

    // Write subgraph output for later diff.
    let mut subgraph_tmp = tempfile::NamedTempFile::new().expect("temp file");
    subgraph_tmp
        .write_all(subgraph_out.stdout.as_slice())
        .expect("write subgraph");
    let subgraph_path = subgraph_tmp.path().to_str().expect("path").to_owned();

    // -- Step 9: convert --compact --
    let compact_out = Command::new(omts_bin())
        .args(["convert", "--compact", &imported_path])
        .output()
        .expect("run omts convert --compact");
    assert_eq!(
        compact_out.status.code(),
        Some(0),
        "convert --compact must succeed; stderr: {}",
        String::from_utf8_lossy(&compact_out.stderr)
    );

    // -- Step 10: convert --to cbor --
    let cbor_out = Command::new(omts_bin())
        .args(["convert", "--to", "cbor", &imported_path])
        .output()
        .expect("run omts convert --to cbor");
    assert_eq!(
        cbor_out.status.code(),
        Some(0),
        "convert --to cbor must succeed; stderr: {}",
        String::from_utf8_lossy(&cbor_out.stderr)
    );
    assert!(
        cbor_out.stdout.len() >= 3,
        "CBOR output should have at least 3 bytes"
    );
    assert_eq!(
        &cbor_out.stdout[..3],
        &[0xD9, 0xD9, 0xF7],
        "CBOR output must start with self-describing tag 55799"
    );

    // -- Step 10b: CBOR round-trip (inspect + validate the CBOR output) --
    let mut cbor_tmp = tempfile::NamedTempFile::new().expect("temp file");
    cbor_tmp.write_all(&cbor_out.stdout).expect("write CBOR");
    let cbor_path = cbor_tmp.path().to_str().expect("path").to_owned();

    let cbor_inspect = Command::new(omts_bin())
        .args(["inspect", "-f", "json", &cbor_path])
        .output()
        .expect("inspect CBOR");
    assert_eq!(
        cbor_inspect.status.code(),
        Some(0),
        "inspect on CBOR input must succeed; stderr: {}",
        String::from_utf8_lossy(&cbor_inspect.stderr)
    );
    let cbor_stats: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&cbor_inspect.stdout).trim())
            .expect("inspect CBOR JSON");
    assert_eq!(cbor_stats["node_count"], 5, "CBOR round-trip node_count");
    assert_eq!(cbor_stats["edge_count"], 5, "CBOR round-trip edge_count");

    let cbor_validate = Command::new(omts_bin())
        .args(["validate", "--level", "1", &cbor_path])
        .output()
        .expect("validate CBOR");
    assert_eq!(
        cbor_validate.status.code(),
        Some(0),
        "CBOR file must pass L1 validation; stderr: {}",
        String::from_utf8_lossy(&cbor_validate.stderr)
    );

    // -- Step 11: diff (identical) --
    let diff_identical = Command::new(omts_bin())
        .args(["diff", &imported_path, &imported_path])
        .output()
        .expect("run omts diff (identical)");
    assert_eq!(
        diff_identical.status.code(),
        Some(0),
        "diff of a file against itself must report no differences (exit 0); stderr: {}",
        String::from_utf8_lossy(&diff_identical.stderr)
    );

    // -- Step 12: diff (different) --
    let diff_out = Command::new(omts_bin())
        .args(["diff", &imported_path, &subgraph_path])
        .output()
        .expect("run omts diff (different)");
    assert_eq!(
        diff_out.status.code(),
        Some(1),
        "diff of full graph vs org-only subgraph must show differences (exit 1)"
    );

    // -- Step 12b: diff --summary-only --
    let diff_summary_out = Command::new(omts_bin())
        .args(["diff", "--summary-only", &imported_path, &subgraph_path])
        .output()
        .expect("run omts diff --summary-only");
    assert_eq!(
        diff_summary_out.status.code(),
        Some(1),
        "diff --summary-only must also exit 1 for different files"
    );
    let diff_summary_stdout = String::from_utf8_lossy(&diff_summary_out.stdout);
    assert!(
        diff_summary_stdout.contains("Summary:"),
        "diff --summary-only should contain Summary line; stdout: {diff_summary_stdout}"
    );

    // -- Step 12c: diff -f json (verify summary counts) --
    let diff_json_out = Command::new(omts_bin())
        .args(["diff", "-f", "json", &imported_path, &subgraph_path])
        .output()
        .expect("run omts diff -f json");
    assert_eq!(
        diff_json_out.status.code(),
        Some(1),
        "diff -f json must exit 1 for different files"
    );
    let diff_json: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&diff_json_out.stdout).trim())
            .expect("diff JSON output");
    let ds = &diff_json["summary"];
    assert_eq!(
        ds["nodes_removed"], 3,
        "3 non-org nodes removed from full graph"
    );
    assert_eq!(ds["nodes_added"], 0, "no nodes added in subgraph");
    assert_eq!(ds["edges_removed"], 3, "3 non-org edges removed");
    assert_eq!(ds["edges_added"], 0, "no edges added in subgraph");

    // -- Step 13: redact --scope public --
    // The imported file has disclosure_scope=partner and a reporting_entity.
    // Redacting to public converts nodes to boundary_refs, which conflicts with
    // the reporting_entity validation rule. To exercise redact successfully, we
    // create a copy without reporting_entity.
    let mut graph_no_re = graph.clone();
    graph_no_re
        .as_object_mut()
        .expect("object")
        .remove("reporting_entity");
    let no_re_json = serde_json::to_string(&graph_no_re).expect("serialize");
    let mut no_re_tmp = tempfile::NamedTempFile::new().expect("temp file");
    no_re_tmp
        .write_all(no_re_json.as_bytes())
        .expect("write no-reporting-entity graph");
    let no_re_path = no_re_tmp.path().to_str().expect("path").to_owned();

    let redact_out = Command::new(omts_bin())
        .args(["redact", "--scope", "public", &no_re_path])
        .output()
        .expect("run omts redact --scope public");
    assert_eq!(
        redact_out.status.code(),
        Some(0),
        "redact must succeed; stderr: {}",
        String::from_utf8_lossy(&redact_out.stderr)
    );
    let redact_stdout = String::from_utf8_lossy(&redact_out.stdout);
    let redact_json: serde_json::Value =
        serde_json::from_str(redact_stdout.trim()).expect("redact JSON");
    assert_eq!(
        redact_json["disclosure_scope"].as_str(),
        Some("public"),
        "redacted output must have disclosure_scope = public"
    );
    let redacted_nodes = redact_json["nodes"].as_array().expect("redacted nodes");
    let boundary_ref_count = redacted_nodes
        .iter()
        .filter(|n| n["type"].as_str() == Some("boundary_ref"))
        .count();
    assert!(
        boundary_ref_count > 0,
        "public redaction of partner-scope graph must produce boundary_ref nodes"
    );
    // Sensitive identifiers (restricted/confidential) must not survive public redaction.
    for node in redacted_nodes {
        if let Some(identifiers) = node["identifiers"].as_array() {
            for ident in identifiers {
                let scheme = ident["scheme"].as_str().unwrap_or("");
                assert!(
                    !matches!(scheme, "nat-reg" | "vat" | "internal"),
                    "sensitive identifier '{scheme}' must not survive public redaction"
                );
            }
        }
    }

    // -- Step 14: validate redacted --
    let mut redacted_tmp = tempfile::NamedTempFile::new().expect("temp file");
    redacted_tmp
        .write_all(redact_out.stdout.as_slice())
        .expect("write redacted");
    let redacted_path = redacted_tmp.path().to_str().expect("path").to_owned();

    let validate_redacted = Command::new(omts_bin())
        .args(["validate", "--level", "1", &redacted_path])
        .output()
        .expect("run omts validate on redacted output");
    assert_eq!(
        validate_redacted.status.code(),
        Some(0),
        "redacted output must pass L1 validate; stderr: {}",
        String::from_utf8_lossy(&validate_redacted.stderr)
    );

    // -- Step 15: export --
    let export_tmp = tempfile::NamedTempFile::new().expect("temp file");
    let export_path = export_tmp.path().with_extension("xlsx");
    let export_out = Command::new(omts_bin())
        .args([
            "export",
            &imported_path,
            "-o",
            export_path.to_str().expect("export path"),
        ])
        .output()
        .expect("run omts export");
    assert_eq!(
        export_out.status.code(),
        Some(0),
        "export must succeed; stderr: {}",
        String::from_utf8_lossy(&export_out.stderr)
    );
    let export_bytes = std::fs::read(&export_path).expect("read exported xlsx");
    assert!(
        export_bytes.starts_with(&[0x50, 0x4B, 0x03, 0x04]),
        "exported file must start with ZIP magic bytes"
    );

    // -- Step 15b: export round-trip (reimport exported xlsx) --
    let reimport_out = Command::new(omts_bin())
        .args(["import", export_path.to_str().expect("path")])
        .output()
        .expect("reimport exported xlsx");
    assert_eq!(
        reimport_out.status.code(),
        Some(0),
        "reimport of exported xlsx must succeed; stderr: {}",
        String::from_utf8_lossy(&reimport_out.stderr)
    );
    let reimport_json: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&reimport_out.stdout).trim())
            .expect("reimport JSON");
    let reimport_nodes = reimport_json["nodes"].as_array().expect("reimport nodes");
    let reimport_edges = reimport_json["edges"].as_array().expect("reimport edges");
    assert_eq!(
        reimport_nodes.len(),
        nodes.len(),
        "export→reimport must preserve node count"
    );
    assert_eq!(
        reimport_edges.len(),
        edges.len(),
        "export→reimport must preserve edge count"
    );

    // -- Step 16: export --output-format excel-supplier-list --
    let sl_export_tmp = tempfile::NamedTempFile::new().expect("temp file");
    let sl_export_path = sl_export_tmp.path().with_extension("xlsx");
    let sl_export_out = Command::new(omts_bin())
        .args([
            "export",
            "--output-format",
            "excel-supplier-list",
            &imported_path,
            "-o",
            sl_export_path.to_str().expect("sl export path"),
        ])
        .output()
        .expect("run omts export --output-format excel-supplier-list");
    assert_eq!(
        sl_export_out.status.code(),
        Some(0),
        "supplier-list export must succeed; stderr: {}",
        String::from_utf8_lossy(&sl_export_out.stderr)
    );
    let sl_bytes = std::fs::read(&sl_export_path).expect("read supplier-list xlsx");
    assert!(
        sl_bytes.starts_with(&[0x50, 0x4B, 0x03, 0x04]),
        "supplier-list export must start with ZIP magic bytes"
    );
}
