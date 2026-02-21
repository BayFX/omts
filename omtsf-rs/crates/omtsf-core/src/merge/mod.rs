/// Property merge, conflict recording, and `same_as` handling for the merge engine.
///
/// This module implements the per-property merge strategy described in merge.md
/// Sections 4.1–4.3 and the `same_as` edge processing described in Sections
/// 7.1–7.3.
///
/// # Responsibilities
///
/// - [`merge_scalars`] — compare N optional scalar values; produce the winner or
///   a conflict record.
/// - [`merge_identifiers`] — set-union of [`Identifier`] arrays, deduplicated by
///   canonical string and sorted.
/// - [`merge_labels`] — set-union of [`Label`] arrays, sorted by `(key, value)`.
/// - [`Conflict`] / [`ConflictEntry`] — deterministic conflict representation.
/// - [`MergeMetadata`] — provenance record written into the merged file header.
/// - [`SameAsThreshold`] — configurable confidence gate for `same_as` edges.
/// - [`apply_same_as_edges`] — feeds qualifying `same_as` edges into a
///   [`UnionFind`] structure after the identifier-based pass.
///
/// [`Identifier`]: crate::types::Identifier
/// [`Label`]: crate::types::Label
/// [`UnionFind`]: crate::union_find::UnionFind
mod ops;
mod types;

pub use ops::{
    apply_same_as_edges, build_conflicts_value, merge_identifiers, merge_labels, merge_scalars,
};
pub use types::{Conflict, ConflictEntry, MergeMetadata, SameAsThreshold, ScalarMergeResult};

#[cfg(test)]
mod tests;
