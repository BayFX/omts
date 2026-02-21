use std::collections::HashMap;

use crate::enums::{EdgeType, EdgeTypeTag};
use crate::newtypes::CalendarDate;
use crate::structures::{Edge, EdgeProperties};
use crate::types::Identifier;

/// Returns `true` when two [`Identifier`] records should be considered the
/// same identifier for merge purposes.
///
/// The predicate is symmetric by construction; every comparison is symmetric
/// (string equality, case-insensitive equality, interval overlap), so
/// `identifiers_match(a, b) == identifiers_match(b, a)` always holds.
///
/// # Rules (applied in order)
///
/// 1. **Internal scheme excluded** — if either identifier uses the `"internal"`
///    scheme, return `false`. Internal identifiers are private to each
///    reporting entity and must never trigger a merge.
/// 2. **Schemes must match** — `a.scheme != b.scheme` → `false`.
/// 3. **Values must match (whitespace-trimmed)** — leading/trailing whitespace
///    in a stored value is normalised away before comparison.
/// 4. **Authority check** — if either record carries an `authority` field,
///    *both* must carry it and it must match case-insensitively. If one has
///    authority and the other does not, return `false`.
/// 5. **Temporal compatibility** — the validity intervals must overlap; see
///    [`temporal_compatible`] for the detailed rules.
pub fn identifiers_match(a: &Identifier, b: &Identifier) -> bool {
    if a.scheme == "internal" || b.scheme == "internal" {
        return false;
    }

    if a.scheme != b.scheme {
        return false;
    }

    if a.value.trim() != b.value.trim() {
        return false;
    }

    if a.authority.is_some() || b.authority.is_some() {
        match (&a.authority, &b.authority) {
            (Some(aa), Some(ba)) => {
                if !aa.eq_ignore_ascii_case(ba) {
                    return false;
                }
            }
            (Some(_), None) | (None, Some(_)) => return false,
            // Both None is handled by the outer `is_some()` guard above and
            // can never reach this arm, but the match must be exhaustive.
            (None, None) => {}
        }
    }

    temporal_compatible(a, b)
}

/// Returns `true` when two identifier records' validity intervals overlap.
///
/// The full three-state semantics of `valid_to` on [`Identifier`] are:
/// - `None` — field absent (temporal bounds not supplied at all).
/// - `Some(None)` — explicit JSON `null` (identifier has no expiry; open-ended
///   into the future).
/// - `Some(Some(date))` — expires on the given date.
///
/// # Rules
///
/// - If *either* record omits **both** `valid_from` and `valid_to` entirely
///   (i.e. both fields are `None`), temporal compatibility is assumed.
/// - Two intervals overlap when it is *not* the case that one ends strictly
///   before the other begins. Specifically, incompatibility is declared only
///   when both records have a concrete `valid_to` date, one `valid_to` is
///   strictly less than the other's `valid_from`, and that `valid_from` is
///   present. An explicit `valid_to: null` (no-expiry) never causes
///   incompatibility.
pub fn temporal_compatible(a: &Identifier, b: &Identifier) -> bool {
    let a_has_temporal = a.valid_from.is_some() || a.valid_to.is_some();
    let b_has_temporal = b.valid_from.is_some() || b.valid_to.is_some();
    if !a_has_temporal || !b_has_temporal {
        return true;
    }

    if intervals_disjoint(a.valid_to.as_ref(), b.valid_from.as_ref()) {
        return false;
    }

    if intervals_disjoint(b.valid_to.as_ref(), a.valid_from.as_ref()) {
        return false;
    }

    true
}

/// Returns `true` when an interval that ends at `end` is strictly before an
/// interval that starts at `start`.
///
/// - `end = None` — field absent; treated as open-ended (never disjoint on
///   this end).
/// - `end = Some(None)` — explicit no-expiry; open-ended (never disjoint).
/// - `end = Some(Some(date))` — concrete end date.
/// - `start = None` — field absent; treated as open-ended from the beginning.
///
/// Disjoint only when `end < start` with both values concrete.
fn intervals_disjoint(end: Option<&Option<CalendarDate>>, start: Option<&CalendarDate>) -> bool {
    let Some(start_date) = start else {
        return false;
    };

    match end {
        None => false,
        Some(None) => false,
        Some(Some(end_date)) => end_date < start_date,
    }
}

/// Returns `true` when an LEI identifier is known to be in ANNULLED status.
///
/// The `VerificationStatus` enum does not include an `Annulled` variant; GLEIF
/// ANNULLED status is typically carried as enrichment data outside the core
/// schema. This function inspects the identifier's `extra` extension fields for
/// a best-effort detection of annulled LEIs.
///
/// # Detection strategy
///
/// The function checks:
/// 1. `id.scheme == "lei"`.
/// 2. The `extra` map contains `"entity_status"` with the string value
///    `"ANNULLED"` (case-sensitive, as GLEIF uses all-caps status codes).
///
/// If L2 enrichment data is unavailable, this function returns `false` (no
/// false-positive exclusions). Callers that have richer LEI data should apply
/// their own filtering before index construction.
pub fn is_lei_annulled(id: &Identifier) -> bool {
    if id.scheme != "lei" {
        return false;
    }
    matches!(
        id.extra.get("entity_status").and_then(|v| v.as_str()),
        Some("ANNULLED")
    )
}

/// Composite key used to group edge merge candidates.
///
/// Two edges belong to the same candidate bucket when their source nodes
/// resolve to the same union-find representative, their target nodes resolve to
/// the same representative, and their edge types are equal.
///
/// This is an opaque key suitable for use as a `HashMap` key; it does not
/// expose the individual fields beyond equality and hashing.
///
/// Constructed via [`edge_composite_key`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EdgeCompositeKey {
    /// Union-find representative of the source node.
    pub source_rep: usize,
    /// Union-find representative of the target node.
    pub target_rep: usize,
    /// Edge type tag.
    pub edge_type: EdgeTypeTag,
}

/// Returns the composite key `(find(source_ordinal), find(target_ordinal), type)`
/// for an edge, using the provided union-find representatives.
///
/// The caller is responsible for resolving source and target node ordinals to
/// their union-find representatives before calling this function. This keeps
/// the function pure and the union-find mutation explicit in the caller.
///
/// # Parameters
///
/// - `source_rep`: `find(source_node_ordinal)` from the union-find structure.
/// - `target_rep`: `find(target_node_ordinal)` from the union-find structure.
/// - `edge`: the edge whose type field is used as the third component.
///
/// # Returns
///
/// Returns `None` when the edge type is `same_as` — such edges are excluded
/// from candidate grouping per merge.md Section 3.2.
pub fn edge_composite_key(
    source_rep: usize,
    target_rep: usize,
    edge: &Edge,
) -> Option<EdgeCompositeKey> {
    if let EdgeTypeTag::Known(EdgeType::SameAs) = &edge.edge_type {
        return None;
    }

    Some(EdgeCompositeKey {
        source_rep,
        target_rep,
        edge_type: edge.edge_type.clone(),
    })
}

/// Builds a composite-key index that groups edge ordinals by their resolved
/// `(find(source), find(target), type)` triple.
///
/// The caller supplies a `node_ordinal` function that maps a node's graph-local
/// `NodeId` string to its position in the flat node array, and a `find`
/// closure that returns the union-find representative for a given ordinal.
/// Both may return `None` when the node is not found (dangling references).
///
/// Edges whose source or target is dangling — i.e. the node ordinal cannot be
/// resolved — are silently skipped. `same_as` edges are also skipped per the
/// spec (they are never merge candidates).
///
/// Construction is O(total edges) after the union-find is settled.
///
/// # Parameters
///
/// - `edges`: flat slice of all edges from the concatenated input files.
/// - `node_ordinal`: returns the ordinal of the node with the given id string,
///   or `None` if not found.
/// - `find`: returns the union-find representative of the given ordinal.
///
/// # Returns
///
/// A `HashMap` from [`EdgeCompositeKey`] to a `Vec<usize>` of edge ordinals
/// that share the same composite key.
pub fn build_edge_candidate_index<F, G>(
    edges: &[Edge],
    node_ordinal: F,
    find: G,
) -> HashMap<EdgeCompositeKey, Vec<usize>>
where
    F: Fn(&str) -> Option<usize>,
    G: Fn(usize) -> usize,
{
    let mut index: HashMap<EdgeCompositeKey, Vec<usize>> = HashMap::new();

    for (edge_idx, edge) in edges.iter().enumerate() {
        let Some(src_ord) = node_ordinal(edge.source.as_ref()) else {
            continue;
        };
        let Some(tgt_ord) = node_ordinal(edge.target.as_ref()) else {
            continue;
        };

        let src_rep = find(src_ord);
        let tgt_rep = find(tgt_ord);

        let Some(key) = edge_composite_key(src_rep, tgt_rep, edge) else {
            continue;
        };

        index.entry(key).or_default().push(edge_idx);
    }

    index
}

/// Returns `true` when two edges' type-specific identity properties are equal
/// per the SPEC-003 Section 3.1 table.
///
/// This predicate is evaluated only for edges that **lack external identifiers**
/// (or whose external identifiers do not produce a match). When both edges have
/// no external identifiers (or only `internal`-scheme identifiers), this
/// property comparison is the sole basis for deciding whether the edges are
/// merge candidates.
///
/// # Per-type identity property table
///
/// | Edge type             | Identity properties beyond type + endpoints |
/// |-----------------------|---------------------------------------------|
/// | `ownership`           | `percentage`, `direct`                      |
/// | `operational_control` | `control_type`                              |
/// | `legal_parentage`     | `consolidation_basis`                       |
/// | `former_identity`     | `event_type`, `effective_date`              |
/// | `beneficial_ownership`| `control_type`, `percentage`                |
/// | `supplies`            | `commodity`, `contract_ref`                 |
/// | `subcontracts`        | `commodity`, `contract_ref`                 |
/// | `tolls`               | `commodity`                                 |
/// | `distributes`         | `service_type`                              |
/// | `brokers`             | `commodity`                                 |
/// | `operates`            | *(type + endpoints suffice)*                |
/// | `produces`            | *(type + endpoints suffice)*                |
/// | `composed_of`         | *(type + endpoints suffice)*                |
/// | `sells_to`            | `commodity`, `contract_ref`                 |
/// | `attested_by`         | `scope`                                     |
/// | `same_as`             | *(never matched — always unique)*           |
/// | Extension             | *(type + endpoints suffice)*                |
///
/// For edge types where "type + endpoints suffice," this function always
/// returns `true` (the composite key check already guarantees type and endpoint
/// identity).
pub fn edge_identity_properties_match(
    edge_type: &EdgeTypeTag,
    a: &EdgeProperties,
    b: &EdgeProperties,
) -> bool {
    match edge_type {
        EdgeTypeTag::Known(EdgeType::Ownership) => {
            options_eq(&a.percentage, &b.percentage) && a.direct == b.direct
        }
        EdgeTypeTag::Known(EdgeType::OperationalControl) => a.control_type == b.control_type,
        EdgeTypeTag::Known(EdgeType::LegalParentage) => {
            a.consolidation_basis == b.consolidation_basis
        }
        EdgeTypeTag::Known(EdgeType::FormerIdentity) => {
            a.event_type == b.event_type && a.effective_date == b.effective_date
        }
        EdgeTypeTag::Known(EdgeType::BeneficialOwnership) => {
            a.control_type == b.control_type && options_eq(&a.percentage, &b.percentage)
        }
        EdgeTypeTag::Known(EdgeType::Supplies) => {
            a.commodity == b.commodity && a.contract_ref == b.contract_ref
        }
        EdgeTypeTag::Known(EdgeType::Subcontracts) => {
            a.commodity == b.commodity && a.contract_ref == b.contract_ref
        }
        EdgeTypeTag::Known(EdgeType::Tolls) => a.commodity == b.commodity,
        EdgeTypeTag::Known(EdgeType::Distributes) => a.service_type == b.service_type,
        EdgeTypeTag::Known(EdgeType::Brokers) => a.commodity == b.commodity,
        EdgeTypeTag::Known(EdgeType::Operates) => true,
        EdgeTypeTag::Known(EdgeType::Produces) => true,
        EdgeTypeTag::Known(EdgeType::ComposedOf) => true,
        EdgeTypeTag::Known(EdgeType::SellsTo) => {
            a.commodity == b.commodity && a.contract_ref == b.contract_ref
        }
        EdgeTypeTag::Known(EdgeType::AttestedBy) => a.scope == b.scope,
        // same_as is never matched; this arm is unreachable in normal usage
        // because edge_composite_key returns None for same_as, but the
        // exhaustive match is required.
        EdgeTypeTag::Known(EdgeType::SameAs) => false,
        // Extension edge types: type + endpoints suffice.
        EdgeTypeTag::Extension(_) => true,
    }
}

/// Compares two `Option<f64>` values for identity-predicate equality.
///
/// `None == None` is `true`. `Some(a) == Some(b)` uses bitwise comparison
/// (same bit pattern), which is appropriate for identity purposes where the
/// values came from the same JSON representation. In particular, this treats
/// `NaN != NaN` (standard IEEE 754 semantics), which is correct: two edges
/// with NaN percentages should not be considered the same.
fn options_eq(a: &Option<f64>, b: &Option<f64>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(x), Some(y)) => x.to_bits() == y.to_bits(),
        _ => false,
    }
}

/// Returns `true` when two edges are merge candidates.
///
/// Two edges are merge candidates when **all** of the following hold:
///
/// 1. Their source nodes belong to the same union-find group (`source_rep_a ==
///    source_rep_b`).
/// 2. Their target nodes belong to the same union-find group (`target_rep_a ==
///    target_rep_b`).
/// 3. Their `type` fields are equal.
/// 4. Either they share an external identifier (same predicate as nodes, per
///    [`identifiers_match`]) — or they lack external identifiers and their
///    type-specific identity properties match per
///    [`edge_identity_properties_match`].
///
/// Condition 4 is evaluated as: if *any* pair of external identifiers matches
/// (via [`identifiers_match`]), the edges are candidates. Otherwise, if both
/// edges have **no** external identifiers (or only `internal`-scheme ones), the
/// type-specific property table is consulted.
///
/// `same_as` edges are excluded: this function returns `false` for any edge
/// whose type is `same_as`, before any further checks.
///
/// # Parameters
///
/// - `source_rep_a`, `target_rep_a`: resolved union-find representatives for
///   the source and target of edge `a`.
/// - `source_rep_b`, `target_rep_b`: resolved union-find representatives for
///   the source and target of edge `b`.
/// - `a`, `b`: the two edges to compare.
#[allow(clippy::too_many_arguments)]
pub fn edges_match(
    source_rep_a: usize,
    target_rep_a: usize,
    source_rep_b: usize,
    target_rep_b: usize,
    a: &Edge,
    b: &Edge,
) -> bool {
    if let EdgeTypeTag::Known(EdgeType::SameAs) = &a.edge_type {
        return false;
    }
    if let EdgeTypeTag::Known(EdgeType::SameAs) = &b.edge_type {
        return false;
    }

    if source_rep_a != source_rep_b || target_rep_a != target_rep_b {
        return false;
    }

    if a.edge_type != b.edge_type {
        return false;
    }

    let a_external: Vec<&Identifier> = a
        .identifiers
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .filter(|id| id.scheme != "internal")
        .collect();

    let b_external: Vec<&Identifier> = b
        .identifiers
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .filter(|id| id.scheme != "internal")
        .collect();

    if !a_external.is_empty() || !b_external.is_empty() {
        for id_a in &a_external {
            for id_b in &b_external {
                if identifiers_match(id_a, id_b) {
                    return true;
                }
            }
        }
        return false;
    }

    edge_identity_properties_match(&a.edge_type, &a.properties, &b.properties)
}
