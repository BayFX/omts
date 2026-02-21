/// Identity predicates for the merge engine.
///
/// Implements the node identity predicate and temporal compatibility check
/// described in merge.md Sections 2.2 and 3.1, and the edge identity predicate
/// described in merge.md Section 3.2 and diff.md Section 2.2.
///
/// All functions in this module are pure (no side-effects, no I/O).
mod predicates;

pub use predicates::{
    EdgeCompositeKey, build_edge_candidate_index, edge_composite_key,
    edge_identity_properties_match, edges_match, identifiers_match, is_lei_annulled,
    temporal_compatible,
};

#[cfg(test)]
mod tests;
