/// Full merge pipeline for combining multiple OMTS files.
///
/// This module implements the eight-step merge procedure described in
/// merge.md, orchestrating:
///
/// 1. Identifier-index construction and union-find for node identity resolution.
/// 2. `same_as` edge processing to extend merge groups.
/// 3. Merge-group safety-limit warnings.
/// 4. Per-group property merge (scalars, identifiers, labels, conflicts).
/// 5. Edge candidate grouping and property merge.
/// 6. Deterministic output ordering.
/// 7. Post-merge L1 validation.
///
/// The primary entry point is [`merge`].
mod pipeline;
mod types;

pub use pipeline::{merge, merge_with_config};
pub use types::{MergeConfig, MergeError, MergeOutput, MergeWarning};

#[cfg(test)]
mod tests;
