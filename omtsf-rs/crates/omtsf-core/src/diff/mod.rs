/// Structural diff engine for OMTSF graph files.
///
/// Implements the matching algorithm and classification described in diff.md
/// Sections 2–4. Two parsed [`OmtsFile`] values are compared; the result
/// describes which nodes and edges were added, removed, or modified.
///
/// # Scope
///
/// - Node matching via canonical identifier indices and union-find transitive closure.
/// - Ambiguity detection (warning when a match group contains multiple nodes from
///   the same file).
/// - Edge matching using resolved endpoints, type, and per-type identity properties.
/// - Property comparison: scalar fields, identifiers set, labels set.
/// - Classification of matched pairs as `modified` or `unchanged` based on
///   whether any property changed.
mod compare;
mod engine;
mod helpers;
mod matching;
mod props;
mod types;

#[cfg(test)]
mod tests;

pub use engine::{diff, diff_filtered};
pub use types::{
    DiffFilter, DiffResult, DiffSummary, EdgeDiff, EdgeRef, EdgesDiff, IdentifierFieldDiff,
    IdentifierSetDiff, LabelSetDiff, NodeDiff, NodeRef, NodesDiff, PropertyChange,
};
