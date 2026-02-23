use omts_core::{
    DiffResult, DiffSummary, EdgeDiff, EdgeRef, EdgesDiff, IdentifierSetDiff, LabelSetDiff,
    NodeDiff, NodeRef, NodesDiff, PropertyChange,
};

/// Writes the diff result in unified-diff-inspired human-readable format.
pub(super) fn write_human<W: std::io::Write>(
    w: &mut W,
    result: &DiffResult,
    ids_only: bool,
    summary_only: bool,
) -> std::io::Result<()> {
    let summary = result.summary();

    if summary_only {
        return write_summary_line(w, &summary);
    }

    write_nodes_human(w, &result.nodes, ids_only)?;
    write_edges_human(w, &result.edges, ids_only)?;

    for warning in &result.warnings {
        writeln!(w, "! {warning}")?;
    }

    write_summary_line(w, &summary)
}

fn write_nodes_human<W: std::io::Write>(
    w: &mut W,
    nodes: &NodesDiff,
    ids_only: bool,
) -> std::io::Result<()> {
    if nodes.added.is_empty() && nodes.removed.is_empty() && nodes.modified.is_empty() {
        return Ok(());
    }

    writeln!(w, "Nodes:")?;

    for node in &nodes.added {
        write_node_ref_human(w, node, "+")?;
    }
    for node in &nodes.removed {
        write_node_ref_human(w, node, "-")?;
    }
    for node_diff in &nodes.modified {
        write_node_diff_human(w, node_diff, ids_only)?;
    }

    writeln!(w)
}

fn write_node_ref_human<W: std::io::Write>(
    w: &mut W,
    node: &NodeRef,
    prefix: &str,
) -> std::io::Result<()> {
    if let Some(name) = &node.name {
        writeln!(
            w,
            "  {prefix} {} ({}) \"{}\"",
            node.id, node.node_type, name
        )
    } else {
        writeln!(w, "  {prefix} {} ({})", node.id, node.node_type)
    }
}

fn write_node_diff_human<W: std::io::Write>(
    w: &mut W,
    node_diff: &NodeDiff,
    ids_only: bool,
) -> std::io::Result<()> {
    if node_diff.id_a == node_diff.id_b {
        writeln!(w, "  ~ {} ({})", node_diff.id_a, node_diff.node_type)?;
    } else {
        writeln!(
            w,
            "  ~ {}/{} ({})",
            node_diff.id_a, node_diff.id_b, node_diff.node_type
        )?;
    }

    if ids_only {
        return Ok(());
    }

    if !node_diff.matched_by.is_empty() {
        writeln!(w, "    matched by: {}", node_diff.matched_by.join(", "))?;
    }

    write_property_changes_human(w, &node_diff.property_changes)?;
    write_identifier_set_diff_human(w, &node_diff.identifier_changes)?;
    write_label_set_diff_human(w, &node_diff.label_changes)?;

    Ok(())
}

fn write_edges_human<W: std::io::Write>(
    w: &mut W,
    edges: &EdgesDiff,
    ids_only: bool,
) -> std::io::Result<()> {
    if edges.added.is_empty() && edges.removed.is_empty() && edges.modified.is_empty() {
        return Ok(());
    }

    writeln!(w, "Edges:")?;

    for edge in &edges.added {
        write_edge_ref_human(w, edge, "+")?;
    }
    for edge in &edges.removed {
        write_edge_ref_human(w, edge, "-")?;
    }
    for edge_diff in &edges.modified {
        write_edge_diff_human(w, edge_diff, ids_only)?;
    }

    writeln!(w)
}

fn write_edge_ref_human<W: std::io::Write>(
    w: &mut W,
    edge: &EdgeRef,
    prefix: &str,
) -> std::io::Result<()> {
    writeln!(
        w,
        "  {prefix} {} ({}) {} -> {}",
        edge.id, edge.edge_type, edge.source, edge.target
    )
}

fn write_edge_diff_human<W: std::io::Write>(
    w: &mut W,
    edge_diff: &EdgeDiff,
    ids_only: bool,
) -> std::io::Result<()> {
    if edge_diff.id_a == edge_diff.id_b {
        writeln!(w, "  ~ {} ({})", edge_diff.id_a, edge_diff.edge_type)?;
    } else {
        writeln!(
            w,
            "  ~ {}/{} ({})",
            edge_diff.id_a, edge_diff.id_b, edge_diff.edge_type
        )?;
    }

    if ids_only {
        return Ok(());
    }

    write_property_changes_human(w, &edge_diff.property_changes)?;
    write_identifier_set_diff_human(w, &edge_diff.identifier_changes)?;
    write_label_set_diff_human(w, &edge_diff.label_changes)?;

    Ok(())
}

fn write_property_changes_human<W: std::io::Write>(
    w: &mut W,
    changes: &[PropertyChange],
) -> std::io::Result<()> {
    for change in changes {
        match (&change.old_value, &change.new_value) {
            (None, Some(new)) => writeln!(w, "    + {}: {new}", change.field)?,
            (Some(old), None) => writeln!(w, "    - {}: {old}", change.field)?,
            (Some(old), Some(new)) => {
                writeln!(w, "    ~ {}: {old} -> {new}", change.field)?;
            }
            (None, None) => {}
        }
    }
    Ok(())
}

fn write_identifier_set_diff_human<W: std::io::Write>(
    w: &mut W,
    diff: &IdentifierSetDiff,
) -> std::io::Result<()> {
    for id in &diff.added {
        writeln!(w, "    + identifier: {}:{}", id.scheme, id.value)?;
    }
    for id in &diff.removed {
        writeln!(w, "    - identifier: {}:{}", id.scheme, id.value)?;
    }
    for id_diff in &diff.modified {
        writeln!(w, "    ~ identifier: {}", id_diff.canonical_key)?;
        write_property_changes_human(w, &id_diff.field_changes)?;
    }
    Ok(())
}

fn write_label_set_diff_human<W: std::io::Write>(
    w: &mut W,
    diff: &LabelSetDiff,
) -> std::io::Result<()> {
    for label in &diff.added {
        if let Some(val) = &label.value {
            writeln!(w, "    + label: {{{}:{val}}}", label.key)?;
        } else {
            writeln!(w, "    + label: {{{}}}", label.key)?;
        }
    }
    for label in &diff.removed {
        if let Some(val) = &label.value {
            writeln!(w, "    - label: {{{}:{val}}}", label.key)?;
        } else {
            writeln!(w, "    - label: {{{}}}", label.key)?;
        }
    }
    Ok(())
}

fn write_summary_line<W: std::io::Write>(w: &mut W, summary: &DiffSummary) -> std::io::Result<()> {
    writeln!(
        w,
        "Summary: {} added, {} removed, {} modified, {} unchanged nodes; \
         {} added, {} removed, {} modified, {} unchanged edges",
        summary.nodes_added,
        summary.nodes_removed,
        summary.nodes_modified,
        summary.nodes_unchanged,
        summary.edges_added,
        summary.edges_removed,
        summary.edges_modified,
        summary.edges_unchanged,
    )
}
