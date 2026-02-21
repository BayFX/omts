use omtsf_core::{
    DiffResult, EdgeDiff, EdgeRef, IdentifierFieldDiff, IdentifierSetDiff, LabelSetDiff, NodeDiff,
    NodeRef, PropertyChange,
};

/// Writes the diff result as a single JSON object to stdout.
///
/// The structure mirrors the diff spec Section 5.2.
pub(super) fn write_json<W: std::io::Write>(w: &mut W, result: &DiffResult) -> std::io::Result<()> {
    let summary = result.summary();

    let summary_obj = serde_json::json!({
        "nodes_added":     summary.nodes_added,
        "nodes_removed":   summary.nodes_removed,
        "nodes_modified":  summary.nodes_modified,
        "nodes_unchanged": summary.nodes_unchanged,
        "edges_added":     summary.edges_added,
        "edges_removed":   summary.edges_removed,
        "edges_modified":  summary.edges_modified,
        "edges_unchanged": summary.edges_unchanged,
    });

    let nodes_obj = serde_json::json!({
        "added":    node_refs_to_json(&result.nodes.added),
        "removed":  node_refs_to_json(&result.nodes.removed),
        "modified": node_diffs_to_json(&result.nodes.modified),
    });

    let edges_obj = serde_json::json!({
        "added":    edge_refs_to_json(&result.edges.added),
        "removed":  edge_refs_to_json(&result.edges.removed),
        "modified": edge_diffs_to_json(&result.edges.modified),
    });

    let output = serde_json::json!({
        "summary": summary_obj,
        "nodes": nodes_obj,
        "edges": edges_obj,
        "warnings": result.warnings,
    });

    let json = serde_json::to_string_pretty(&output)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    writeln!(w, "{json}")
}

fn node_refs_to_json(refs: &[NodeRef]) -> serde_json::Value {
    serde_json::Value::Array(
        refs.iter()
            .map(|n| {
                let mut obj = serde_json::Map::new();
                obj.insert("id".to_owned(), serde_json::Value::String(n.id.to_string()));
                obj.insert(
                    "node_type".to_owned(),
                    serde_json::Value::String(n.node_type.clone()),
                );
                if let Some(name) = &n.name {
                    obj.insert("name".to_owned(), serde_json::Value::String(name.clone()));
                }
                serde_json::Value::Object(obj)
            })
            .collect(),
    )
}

fn edge_refs_to_json(refs: &[EdgeRef]) -> serde_json::Value {
    serde_json::Value::Array(
        refs.iter()
            .map(|e| {
                serde_json::json!({
                    "id":        e.id.to_string(),
                    "edge_type": e.edge_type,
                    "source":    e.source.to_string(),
                    "target":    e.target.to_string(),
                })
            })
            .collect(),
    )
}

fn node_diffs_to_json(diffs: &[NodeDiff]) -> serde_json::Value {
    serde_json::Value::Array(
        diffs
            .iter()
            .map(|d| {
                serde_json::json!({
                    "id_a":               d.id_a,
                    "id_b":               d.id_b,
                    "node_type":          d.node_type,
                    "matched_by":         d.matched_by,
                    "property_changes":   property_changes_to_json(&d.property_changes),
                    "identifier_changes": identifier_set_diff_to_json(&d.identifier_changes),
                    "label_changes":      label_set_diff_to_json(&d.label_changes),
                })
            })
            .collect(),
    )
}

fn edge_diffs_to_json(diffs: &[EdgeDiff]) -> serde_json::Value {
    serde_json::Value::Array(
        diffs
            .iter()
            .map(|d| {
                serde_json::json!({
                    "id_a":               d.id_a,
                    "id_b":               d.id_b,
                    "edge_type":          d.edge_type,
                    "property_changes":   property_changes_to_json(&d.property_changes),
                    "identifier_changes": identifier_set_diff_to_json(&d.identifier_changes),
                    "label_changes":      label_set_diff_to_json(&d.label_changes),
                })
            })
            .collect(),
    )
}

fn property_changes_to_json(changes: &[PropertyChange]) -> serde_json::Value {
    serde_json::Value::Array(
        changes
            .iter()
            .map(|c| {
                serde_json::json!({
                    "field":     c.field,
                    "old_value": c.old_value,
                    "new_value": c.new_value,
                })
            })
            .collect(),
    )
}

fn identifier_set_diff_to_json(diff: &IdentifierSetDiff) -> serde_json::Value {
    let modified: Vec<serde_json::Value> = diff
        .modified
        .iter()
        .map(identifier_field_diff_to_json)
        .collect();

    let added: Vec<serde_json::Value> = diff
        .added
        .iter()
        .map(|id| {
            serde_json::json!({
                "scheme": id.scheme,
                "value":  id.value,
            })
        })
        .collect();

    let removed: Vec<serde_json::Value> = diff
        .removed
        .iter()
        .map(|id| {
            serde_json::json!({
                "scheme": id.scheme,
                "value":  id.value,
            })
        })
        .collect();

    serde_json::json!({
        "added":    added,
        "removed":  removed,
        "modified": modified,
    })
}

fn identifier_field_diff_to_json(diff: &IdentifierFieldDiff) -> serde_json::Value {
    serde_json::json!({
        "canonical_key":  diff.canonical_key.to_string(),
        "field_changes":  property_changes_to_json(&diff.field_changes),
    })
}

fn label_set_diff_to_json(diff: &LabelSetDiff) -> serde_json::Value {
    let added: Vec<serde_json::Value> = diff
        .added
        .iter()
        .map(|l| serde_json::json!({"key": l.key, "value": l.value}))
        .collect();
    let removed: Vec<serde_json::Value> = diff
        .removed
        .iter()
        .map(|l| serde_json::json!({"key": l.key, "value": l.value}))
        .collect();
    serde_json::json!({
        "added":   added,
        "removed": removed,
    })
}
