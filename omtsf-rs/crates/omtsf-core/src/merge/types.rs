use serde::{Deserialize, Serialize};

/// Configures which `same_as` edges are honoured during union-find processing.
///
/// The spec defines three confidence levels for `same_as` edges (merge.md
/// Section 7.1). The threshold controls the minimum level that triggers a
/// `union` call on the underlying [`UnionFind`] structure.
///
/// ```text
/// Definite  → only "definite" edges are honoured  (most conservative)
/// Probable  → "definite" and "probable" edges are honoured
/// Possible  → all three levels are honoured        (most permissive)
/// ```
///
/// The default is [`SameAsThreshold::Definite`].
///
/// [`UnionFind`]: crate::union_find::UnionFind
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum SameAsThreshold {
    /// Honour only `same_as` edges with `confidence: "definite"` (default).
    #[default]
    Definite,
    /// Honour `same_as` edges with `confidence: "definite"` or `"probable"`.
    Probable,
    /// Honour all `same_as` edges regardless of confidence level.
    Possible,
}

impl SameAsThreshold {
    /// Returns `true` when a `same_as` edge carrying the given `confidence`
    /// string should be honoured under this threshold.
    ///
    /// Unrecognised confidence strings are treated as `"possible"` (the weakest
    /// level), meaning they are honoured only when the threshold is
    /// [`SameAsThreshold::Possible`].
    ///
    /// # Parameters
    ///
    /// - `confidence`: the value of the `confidence` property on the `same_as`
    ///   edge (e.g. `"definite"`, `"probable"`, `"possible"`).  `None` (field
    ///   absent) is treated as `"possible"`.
    pub fn honours(&self, confidence: Option<&str>) -> bool {
        let level = SameAsLevel::from_str(confidence.unwrap_or("possible"));
        match self {
            SameAsThreshold::Definite => matches!(level, SameAsLevel::Definite),
            SameAsThreshold::Probable => {
                matches!(level, SameAsLevel::Definite | SameAsLevel::Probable)
            }
            SameAsThreshold::Possible => true,
        }
    }
}

/// Internal helper for the three `same_as` confidence levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SameAsLevel {
    Definite,
    Probable,
    Possible,
}

impl SameAsLevel {
    fn from_str(s: &str) -> Self {
        match s {
            "definite" => Self::Definite,
            "probable" => Self::Probable,
            _ => Self::Possible,
        }
    }
}

/// A single conflicting value observed in a merge group, with its provenance.
///
/// Conflict entries are sorted by `(source_file, json_value)` to guarantee
/// deterministic output (merge.md Section 4.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictEntry {
    /// JSON-serialized form of the conflicting value.
    pub value: serde_json::Value,
    /// The source file that contributed this value.
    pub source_file: String,
}

/// A recorded conflict on a single property within a merge group.
///
/// When two or more source nodes/edges disagree on a scalar property, the
/// property is omitted from the merged output and a `Conflict` is appended to
/// the `_conflicts` array (merge.md Section 4.1).
///
/// Entries within a `Conflict` are sorted by `(source_file, json_value)`;
/// multiple `Conflict` records in a `_conflicts` array are sorted by `field`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conflict {
    /// Name of the property that conflicted (e.g. `"name"`, `"status"`).
    pub field: String,
    /// All distinct values seen for this property, with provenance.
    pub values: Vec<ConflictEntry>,
}

/// Provenance record written into the merged file header.
///
/// Corresponds to the `merge_metadata` object described in merge.md Section 4.3.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeMetadata {
    /// Sorted list of source file paths or identifiers that were merged.
    pub source_files: Vec<String>,
    /// Reporting entity values collected from all source files.
    ///
    /// When source files declare different `reporting_entity` values, the merged
    /// header omits `reporting_entity` and records all values here.
    pub reporting_entities: Vec<String>,
    /// ISO 8601 timestamp of when the merge was performed.
    pub timestamp: String,
    /// Number of merged output nodes.
    pub merged_node_count: usize,
    /// Number of merged output edges.
    pub merged_edge_count: usize,
    /// Total number of conflicts recorded across all nodes and edges.
    pub conflict_count: usize,
}

/// Result of merging N optional scalar values from a merge group.
///
/// Returned by [`super::merge_scalars`].
#[derive(Debug, Clone, PartialEq)]
pub enum ScalarMergeResult<T> {
    /// All sources agree on this value (or the value is absent in all sources).
    Agreed(Option<T>),
    /// Sources disagree; the caller should record a [`Conflict`].
    Conflict(Vec<ConflictEntry>),
}
