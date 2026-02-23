use crate::file::OmtsFile;
use crate::merge::{MergeMetadata, SameAsThreshold};

/// Errors that can occur during the merge pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeError {
    /// The input slice was empty; at least one file is required.
    NoInputFiles,
    /// Post-merge L1 validation found structural errors in the merged output.
    ///
    /// The inner string describes the first error found. This should not occur
    /// under normal operation; if it does it indicates a bug in the pipeline.
    PostMergeValidationFailed(String),
    /// The random file salt could not be generated (platform CSPRNG failure).
    SaltGenerationFailed(String),
    /// A required OMTS version or date string could not be constructed.
    InternalDataError(String),
}

impl std::fmt::Display for MergeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoInputFiles => f.write_str("merge requires at least one input file"),
            Self::PostMergeValidationFailed(msg) => {
                write!(f, "post-merge L1 validation failed: {msg}")
            }
            Self::SaltGenerationFailed(msg) => {
                write!(f, "could not generate file salt: {msg}")
            }
            Self::InternalDataError(msg) => {
                write!(f, "internal data error during merge: {msg}")
            }
        }
    }
}

impl std::error::Error for MergeError {}

/// Non-fatal warning produced during the merge pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeWarning {
    /// A merge group exceeded the configured size limit.
    ///
    /// This may indicate a false-positive cascade where a single erroneous
    /// identifier match pulls unrelated entities into the same group.
    OversizedMergeGroup {
        /// The representative node ordinal for the group.
        representative_ordinal: usize,
        /// The number of nodes in the group.
        group_size: usize,
        /// The configured limit that was exceeded.
        limit: usize,
    },
}

impl std::fmt::Display for MergeWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OversizedMergeGroup {
                representative_ordinal,
                group_size,
                limit,
            } => write!(
                f,
                "merge group (representative ordinal {representative_ordinal}) has {group_size} \
                 nodes, exceeding the limit of {limit}"
            ),
        }
    }
}

/// Configuration for the merge pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeConfig {
    /// Maximum number of nodes allowed in a single merge group before a
    /// [`MergeWarning::OversizedMergeGroup`] is emitted.
    ///
    /// Default: 50.
    pub group_size_limit: usize,

    /// Confidence threshold for honouring `same_as` edges.
    ///
    /// Default: [`SameAsThreshold::Definite`].
    pub same_as_threshold: SameAsThreshold,

    /// Source-file label used in conflict entries when a file has no path.
    ///
    /// Default: `"<unknown>"`.
    pub default_source_label: String,
}

impl Default for MergeConfig {
    fn default() -> Self {
        Self {
            group_size_limit: 50,
            same_as_threshold: SameAsThreshold::default(),
            default_source_label: "<unknown>".to_owned(),
        }
    }
}

/// The result of a successful merge operation.
#[derive(Debug, Clone)]
pub struct MergeOutput {
    /// The merged OMTS file.
    pub file: OmtsFile,
    /// Provenance metadata written into [`MergeOutput::file`]'s `extra` map.
    pub metadata: MergeMetadata,
    /// Non-fatal warnings produced during the merge.
    pub warnings: Vec<MergeWarning>,
    /// Total number of conflict records across all merged nodes and edges.
    pub conflict_count: usize,
}
