use std::collections::HashSet;

use serde::Serialize;

use crate::canonical::CanonicalId;
use crate::enums::{EdgeType, EdgeTypeTag};
use crate::structures::Edge;
use crate::types::{Identifier, Label};
use crate::union_find::UnionFind;

use super::types::{Conflict, ConflictEntry, SameAsThreshold, ScalarMergeResult};

/// Merges multiple optional scalar values into a single result.
///
/// The input is a slice of `(value, source_file)` pairs. The function:
///
/// 1. Collects all `Some` values, serialising each to a `serde_json::Value`
///    for comparison.
/// 2. If all `Some` values are JSON-equal (or there are no `Some` values at
///    all), returns [`ScalarMergeResult::Agreed`] with the common value.
/// 3. If any two `Some` values differ, returns [`ScalarMergeResult::Conflict`]
///    with one entry per distinct `(source_file, value)` pair, sorted by
///    `(source_file, json_value_as_string)`.
///
/// # Type parameters
///
/// - `T`: must implement [`Serialize`] (for JSON comparison) and [`Clone`].
pub fn merge_scalars<T>(inputs: &[(Option<T>, &str)]) -> ScalarMergeResult<T>
where
    T: Serialize + Clone,
{
    let mut present: Vec<(serde_json::Value, &str, &T)> = Vec::new();

    for (opt, source) in inputs {
        if let Some(val) = opt {
            let json_val = serde_json::to_value(val).unwrap_or(serde_json::Value::Null);
            present.push((json_val, source, val));
        }
    }

    if present.is_empty() {
        return ScalarMergeResult::Agreed(None);
    }

    let first_json = &present[0].0;
    let all_equal = present.iter().all(|(v, _, _)| v == first_json);

    if all_equal {
        return ScalarMergeResult::Agreed(Some(present[0].2.clone()));
    }

    let mut entries: Vec<ConflictEntry> = present
        .into_iter()
        .map(|(json_val, source, _)| ConflictEntry {
            value: json_val,
            source_file: source.to_owned(),
        })
        .collect();

    entries.sort_by(|a, b| {
        let af = &a.source_file;
        let bf = &b.source_file;
        let av = a.value.to_string();
        let bv = b.value.to_string();
        af.cmp(bf).then_with(|| av.cmp(&bv))
    });

    entries.dedup_by(|a, b| a.source_file == b.source_file && a.value == b.value);

    ScalarMergeResult::Conflict(entries)
}

/// Merges multiple `Identifier` arrays into a deduplicated, sorted union.
///
/// Deduplication uses the [`CanonicalId`] string as the key: two identifiers
/// that produce the same canonical string are considered identical and only the
/// first occurrence (in input order) is retained.
///
/// The merged array is sorted by canonical string in lexicographic UTF-8 byte
/// order (merge.md Section 4.2).
///
/// # Parameters
///
/// - `inputs`: each element is an `Option<&[Identifier]>` (the `identifiers`
///   field from a source node/edge, which is `Option<Vec<Identifier>>`).
///
/// # Returns
///
/// A `Vec<Identifier>` that is the sorted set-union, or an empty `Vec` when
/// all inputs are `None` or empty.
pub fn merge_identifiers(inputs: &[Option<&[Identifier]>]) -> Vec<Identifier> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut result: Vec<(String, Identifier)> = Vec::new();

    for input in inputs {
        let Some(ids) = input else { continue };
        for id in *ids {
            let cid = CanonicalId::from_identifier(id);
            let key = cid.into_string();
            if seen.insert(key.clone()) {
                result.push((key, id.clone()));
            }
        }
    }

    result.sort_by(|(a, _), (b, _)| a.cmp(b));

    result.into_iter().map(|(_, id)| id).collect()
}

/// Merges multiple `Label` arrays into a deduplicated, sorted union.
///
/// Deduplication uses `(key, value)` as the composite key. Sorting follows
/// merge.md Section 4.2:
/// - Primary key: `key` ascending.
/// - Secondary key: `value` ascending, with `None` (absent value) sorting
///   before `Some(_)` (present value).
///
/// # Parameters
///
/// - `inputs`: each element is an `Option<&[Label]>`.
///
/// # Returns
///
/// A `Vec<Label>` that is the sorted set-union.
pub fn merge_labels(inputs: &[Option<&[Label]>]) -> Vec<Label> {
    let mut seen: HashSet<(String, Option<String>)> = HashSet::new();
    let mut result: Vec<Label> = Vec::new();

    for input in inputs {
        let Some(labels) = input else { continue };
        for label in *labels {
            let key = (label.key.clone(), label.value.clone());
            if seen.insert(key) {
                result.push(label.clone());
            }
        }
    }

    result.sort_by(|a, b| {
        a.key.cmp(&b.key).then_with(|| match (&a.value, &b.value) {
            (None, None) => std::cmp::Ordering::Equal,
            (None, Some(_)) => std::cmp::Ordering::Less,
            (Some(_), None) => std::cmp::Ordering::Greater,
            (Some(av), Some(bv)) => av.cmp(bv),
        })
    });

    result
}

/// Processes `same_as` edges and applies qualifying ones to a [`UnionFind`].
///
/// This function implements merge.md Section 7.1. It scans `edges` for edges of
/// type [`EdgeType::SameAs`] and, for each edge whose confidence level meets
/// `threshold`, calls `uf.union(src_ord, tgt_ord)`.
///
/// Node ordinals are resolved via `node_id_to_ordinal`, which maps a node's
/// graph-local `id` string to its index in the concatenated node slice.
///
/// `same_as` edges that are honoured are collected and returned so the caller
/// can record which merge groups were extended by `same_as` (merge.md
/// Section 7.3).  Edges that fail the threshold or whose source/target cannot
/// be resolved are silently skipped.
///
/// # Parameters
///
/// - `edges`: the full list of edges from all source files.
/// - `node_id_to_ordinal`: a function mapping a node ID string to its ordinal
///   index in the union-find structure.  Returns `None` for unknown IDs.
/// - `uf`: mutable reference to the [`UnionFind`] structure (already populated
///   by the identifier-based pass).
/// - `threshold`: the [`SameAsThreshold`] gate.
///
/// # Returns
///
/// A `Vec<&Edge>` containing the `same_as` edges that were honoured.
pub fn apply_same_as_edges<'a, F>(
    edges: &'a [Edge],
    node_id_to_ordinal: F,
    uf: &mut UnionFind,
    threshold: SameAsThreshold,
) -> Vec<&'a Edge>
where
    F: Fn(&str) -> Option<usize>,
{
    let mut honoured: Vec<&Edge> = Vec::new();

    for edge in edges {
        let is_same_as = match &edge.edge_type {
            EdgeTypeTag::Known(EdgeType::SameAs) => true,
            EdgeTypeTag::Known(_) | EdgeTypeTag::Extension(_) => false,
        };
        if !is_same_as {
            continue;
        }

        // The spec puts `confidence` in the edge's `extra` map for same_as
        // edges, not in `data_quality`. We check `properties.extra` first,
        // then the edge-level `extra` as a fallback.
        let confidence_str: Option<&str> = edge
            .properties
            .extra
            .get("confidence")
            .and_then(|v| v.as_str());

        let confidence_str =
            confidence_str.or_else(|| edge.extra.get("confidence").and_then(|v| v.as_str()));

        if !threshold.honours(confidence_str) {
            continue;
        }

        let Some(src_ord) = node_id_to_ordinal(&edge.source) else {
            continue;
        };
        let Some(tgt_ord) = node_id_to_ordinal(&edge.target) else {
            continue;
        };

        uf.union(src_ord, tgt_ord);
        honoured.push(edge);
    }

    honoured
}

/// Builds a sorted `_conflicts` JSON array from a slice of [`Conflict`] records.
///
/// Conflicts are sorted by `field` name (merge.md Section 5, invariant 4).
/// This function is used when writing the merged node's `_conflicts` property.
///
/// Returns `None` when `conflicts` is empty (no `_conflicts` key should be
/// written in that case).
pub fn build_conflicts_value(mut conflicts: Vec<Conflict>) -> Option<serde_json::Value> {
    if conflicts.is_empty() {
        return None;
    }
    conflicts.sort_by(|a, b| a.field.cmp(&b.field));
    let val = serde_json::to_value(&conflicts).unwrap_or(serde_json::Value::Null);
    Some(val)
}
