/// L1-GDM-01 through L1-GDM-06: Graph Data Model structural validation rules.
///
/// These rules enforce the MUST constraints from SPEC-001 Section 9.1 and 9.5.
/// Each rule is a stateless struct implementing [`crate::validation::ValidationRule`].
/// All rules collect every violation without early exit.
///
/// Rules are registered in [`crate::validation::build_registry`] when
/// [`crate::validation::ValidationConfig::run_l1`] is `true`.
use std::collections::{HashMap, HashSet};

use crate::enums::{EdgeType, EdgeTypeTag, NodeType, NodeTypeTag};
use crate::file::OmtsFile;
use crate::structures::Node;

use super::{Diagnostic, Level, Location, RuleId, Severity, ValidationRule};

#[cfg(test)]
mod tests;

/// Build a map from node id string to the node reference.
///
/// Used by multiple rules to avoid redundant iteration.
fn node_id_map(file: &OmtsFile) -> HashMap<&str, &Node> {
    file.nodes
        .iter()
        .map(|n| (n.id.as_ref() as &str, n))
        .collect()
}

/// Returns `true` if the string matches the reverse-domain extension convention
/// (contains at least one dot character, per SPEC-001 Section 8.2).
fn is_extension_type(s: &str) -> bool {
    s.contains('.')
}

/// L1-GDM-01 — Every node has a non-empty `id`, unique within the file.
///
/// The non-empty constraint is enforced by the [`crate::newtypes::NodeId`]
/// newtype at deserialization time, so this rule only checks for duplicate
/// IDs across nodes. Each duplicate (beyond the first occurrence) produces
/// one diagnostic.
pub struct GdmRule01;

impl ValidationRule for GdmRule01 {
    fn id(&self) -> RuleId {
        RuleId::L1Gdm01
    }

    fn level(&self) -> Level {
        Level::L1
    }

    fn check(
        &self,
        file: &OmtsFile,
        diags: &mut Vec<Diagnostic>,
        _external_data: Option<&dyn super::external::ExternalDataSource>,
    ) {
        let mut seen: HashSet<&str> = HashSet::new();
        for node in &file.nodes {
            let id: &str = &node.id;
            if !seen.insert(id) {
                diags.push(Diagnostic::new(
                    RuleId::L1Gdm01,
                    Severity::Error,
                    Location::Node {
                        node_id: id.to_owned(),
                        field: None,
                    },
                    format!("duplicate node id \"{id}\""),
                ));
            }
        }
    }
}

/// L1-GDM-02 — Every edge has a non-empty `id`, unique within the file.
///
/// The non-empty constraint is enforced by the [`crate::newtypes::NodeId`]
/// (aliased as `EdgeId`) newtype at deserialization time. This rule only
/// checks for duplicate IDs across edges. Each duplicate (beyond the first
/// occurrence) produces one diagnostic.
pub struct GdmRule02;

impl ValidationRule for GdmRule02 {
    fn id(&self) -> RuleId {
        RuleId::L1Gdm02
    }

    fn level(&self) -> Level {
        Level::L1
    }

    fn check(
        &self,
        file: &OmtsFile,
        diags: &mut Vec<Diagnostic>,
        _external_data: Option<&dyn super::external::ExternalDataSource>,
    ) {
        let mut seen: HashSet<&str> = HashSet::new();
        for edge in &file.edges {
            let id: &str = &edge.id;
            if !seen.insert(id) {
                diags.push(Diagnostic::new(
                    RuleId::L1Gdm02,
                    Severity::Error,
                    Location::Edge {
                        edge_id: id.to_owned(),
                        field: None,
                    },
                    format!("duplicate edge id \"{id}\""),
                ));
            }
        }
    }
}

/// L1-GDM-03 — Every edge `source` and `target` references an existing node `id`.
///
/// Both `source` and `target` are checked independently. Each dangling
/// reference produces a separate diagnostic with `field` set to `"source"` or
/// `"target"` respectively.
pub struct GdmRule03;

impl ValidationRule for GdmRule03 {
    fn id(&self) -> RuleId {
        RuleId::L1Gdm03
    }

    fn level(&self) -> Level {
        Level::L1
    }

    fn check(
        &self,
        file: &OmtsFile,
        diags: &mut Vec<Diagnostic>,
        _external_data: Option<&dyn super::external::ExternalDataSource>,
    ) {
        let node_ids: HashSet<&str> = file.nodes.iter().map(|n| n.id.as_ref() as &str).collect();

        for edge in &file.edges {
            let edge_id: &str = &edge.id;
            let source: &str = &edge.source;
            let target: &str = &edge.target;

            if !node_ids.contains(source) {
                diags.push(Diagnostic::new(
                    RuleId::L1Gdm03,
                    Severity::Error,
                    Location::Edge {
                        edge_id: edge_id.to_owned(),
                        field: Some("source".to_owned()),
                    },
                    format!(
                        "edge \"{edge_id}\" source \"{source}\" does not reference an existing node"
                    ),
                ));
            }

            if !node_ids.contains(target) {
                diags.push(Diagnostic::new(
                    RuleId::L1Gdm03,
                    Severity::Error,
                    Location::Edge {
                        edge_id: edge_id.to_owned(),
                        field: Some("target".to_owned()),
                    },
                    format!(
                        "edge \"{edge_id}\" target \"{target}\" does not reference an existing node"
                    ),
                ));
            }
        }
    }
}

/// L1-GDM-04 — Edge `type` is a recognised core type, `same_as`, or
/// reverse-domain extension.
///
/// All [`crate::enums::EdgeType`] variants (including `SameAs`) are accepted.
/// Extension strings that contain a dot are accepted per SPEC-001 Section 8.2.
/// Unrecognised strings without a dot are rejected.
pub struct GdmRule04;

impl ValidationRule for GdmRule04 {
    fn id(&self) -> RuleId {
        RuleId::L1Gdm04
    }

    fn level(&self) -> Level {
        Level::L1
    }

    fn check(
        &self,
        file: &OmtsFile,
        diags: &mut Vec<Diagnostic>,
        _external_data: Option<&dyn super::external::ExternalDataSource>,
    ) {
        for edge in &file.edges {
            match &edge.edge_type {
                EdgeTypeTag::Known(_) => {}
                EdgeTypeTag::Extension(s) => {
                    if !is_extension_type(s) {
                        diags.push(Diagnostic::new(
                            RuleId::L1Gdm04,
                            Severity::Error,
                            Location::Edge {
                                edge_id: edge.id.to_string(),
                                field: Some("type".to_owned()),
                            },
                            format!(
                                "edge \"{}\" has unrecognised type \"{s}\"; \
                                 must be a core type, \"same_as\", or a \
                                 reverse-domain extension (e.g. \"com.example.custom\")",
                                edge.id
                            ),
                        ));
                    }
                }
            }
        }
    }
}

/// L1-GDM-05 — `reporting_entity` if present references an existing
/// `organization` node.
///
/// The referenced node must both exist and have `type: "organization"`.
/// A missing node and a node of the wrong type each produce a distinct message.
pub struct GdmRule05;

impl ValidationRule for GdmRule05 {
    fn id(&self) -> RuleId {
        RuleId::L1Gdm05
    }

    fn level(&self) -> Level {
        Level::L1
    }

    fn check(
        &self,
        file: &OmtsFile,
        diags: &mut Vec<Diagnostic>,
        _external_data: Option<&dyn super::external::ExternalDataSource>,
    ) {
        let Some(ref reporting_entity) = file.reporting_entity else {
            return;
        };
        let ref_id: &str = reporting_entity;

        let node_map = node_id_map(file);

        match node_map.get(ref_id) {
            None => {
                diags.push(Diagnostic::new(
                    RuleId::L1Gdm05,
                    Severity::Error,
                    Location::Header {
                        field: "reporting_entity",
                    },
                    format!("reporting_entity \"{ref_id}\" does not reference an existing node"),
                ));
            }
            Some(node) => {
                if node.node_type != NodeTypeTag::Known(NodeType::Organization) {
                    diags.push(Diagnostic::new(
                        RuleId::L1Gdm05,
                        Severity::Error,
                        Location::Header {
                            field: "reporting_entity",
                        },
                        format!(
                            "reporting_entity \"{ref_id}\" references a node that is not an \
                             organization (found type: {})",
                            node_type_display(&node.node_type)
                        ),
                    ));
                }
            }
        }
    }
}

/// Returns a human-readable string for a [`NodeTypeTag`].
fn node_type_display(tag: &NodeTypeTag) -> String {
    match tag {
        NodeTypeTag::Known(NodeType::Organization) => "organization".to_owned(),
        NodeTypeTag::Known(NodeType::Facility) => "facility".to_owned(),
        NodeTypeTag::Known(NodeType::Good) => "good".to_owned(),
        NodeTypeTag::Known(NodeType::Person) => "person".to_owned(),
        NodeTypeTag::Known(NodeType::Attestation) => "attestation".to_owned(),
        NodeTypeTag::Known(NodeType::Consignment) => "consignment".to_owned(),
        NodeTypeTag::Known(NodeType::BoundaryRef) => "boundary_ref".to_owned(),
        NodeTypeTag::Extension(s) => s.clone(),
    }
}

/// L1-GDM-06 — Edge source/target node types match the permitted types table
/// (SPEC-001 Section 9.5). Extension edges are exempt.
///
/// For each core edge type the permitted source and target [`NodeType`] sets
/// are encoded in `permitted_types`. Extension edges (those with
/// [`EdgeTypeTag::Extension`]) are skipped entirely.
///
/// A diagnostic is emitted per invalid endpoint — both `source` and `target`
/// are checked independently.
pub struct GdmRule06;

/// Permitted source and target node types for each core edge type.
///
/// Returns `None` when the edge type imposes no type constraint (e.g. `same_as`).
fn permitted_types(edge_type: &EdgeType) -> Option<(TypeSet, TypeSet)> {
    use NodeType::{Attestation, Consignment, Facility, Good, Organization, Person};

    let org = &[Organization][..];
    let org_fac = &[Organization, Facility][..];
    let fac = &[Facility][..];
    let good_cons = &[Good, Consignment][..];
    let org_fac_good_cons = &[Organization, Facility, Good, Consignment][..];
    let att = &[Attestation][..];
    let person = &[Person][..];

    let (src, tgt): (&[NodeType], &[NodeType]) = match edge_type {
        EdgeType::Ownership => (org, org),
        EdgeType::OperationalControl => (org, org_fac),
        EdgeType::LegalParentage => (org, org),
        EdgeType::FormerIdentity => (org, org),
        EdgeType::BeneficialOwnership => (person, org),
        EdgeType::Supplies => (org, org),
        EdgeType::Subcontracts => (org, org),
        EdgeType::Tolls => (org_fac, org),
        EdgeType::Distributes => (org, org),
        EdgeType::Brokers => (org, org),
        EdgeType::Operates => (org, fac),
        EdgeType::Produces => (fac, good_cons),
        EdgeType::ComposedOf => (good_cons, good_cons),
        EdgeType::SellsTo => (org, org),
        EdgeType::AttestedBy => (org_fac_good_cons, att),
        EdgeType::SameAs => return None, // any → any
    };

    Some((src, tgt))
}

/// A slice of permitted node types.
type TypeSet = &'static [NodeType];

impl ValidationRule for GdmRule06 {
    fn id(&self) -> RuleId {
        RuleId::L1Gdm06
    }

    fn level(&self) -> Level {
        Level::L1
    }

    fn check(
        &self,
        file: &OmtsFile,
        diags: &mut Vec<Diagnostic>,
        _external_data: Option<&dyn super::external::ExternalDataSource>,
    ) {
        let node_map = node_id_map(file);

        for edge in &file.edges {
            let edge_id: &str = &edge.id;

            let edge_type = match &edge.edge_type {
                EdgeTypeTag::Extension(_) => continue,
                EdgeTypeTag::Known(et) => et,
            };

            let Some((permitted_src, permitted_tgt)) = permitted_types(edge_type) else {
                continue;
            };

            let source_id: &str = &edge.source;
            let target_id: &str = &edge.target;

            // Only check nodes that actually exist; dangling refs are L1-GDM-03's
            // responsibility.
            if let Some(src_node) = node_map.get(source_id) {
                if let NodeTypeTag::Known(ref src_type) = src_node.node_type {
                    // boundary_ref nodes may appear at any edge endpoint (SPEC-004
                    // Section 5.1): they preserve graph connectivity when a node is
                    // replaced during redaction, so the type-compatibility constraint
                    // does not apply to them.
                    let is_boundary_ref = *src_type == NodeType::BoundaryRef;
                    if !is_boundary_ref && !permitted_src.contains(src_type) {
                        diags.push(Diagnostic::new(
                            RuleId::L1Gdm06,
                            Severity::Error,
                            Location::Edge {
                                edge_id: edge_id.to_owned(),
                                field: Some("source".to_owned()),
                            },
                            format!(
                                "edge \"{edge_id}\" (type \"{}\") source \"{source_id}\" \
                                 has type \"{}\", which is not permitted; \
                                 expected one of: {}",
                                edge_type_display(edge_type),
                                node_type_display(&src_node.node_type),
                                format_type_set(permitted_src),
                            ),
                        ));
                    }
                }
            }

            if let Some(tgt_node) = node_map.get(target_id) {
                if let NodeTypeTag::Known(ref tgt_type) = tgt_node.node_type {
                    // boundary_ref nodes may appear at any edge endpoint (see above).
                    let is_boundary_ref = *tgt_type == NodeType::BoundaryRef;
                    if !is_boundary_ref && !permitted_tgt.contains(tgt_type) {
                        diags.push(Diagnostic::new(
                            RuleId::L1Gdm06,
                            Severity::Error,
                            Location::Edge {
                                edge_id: edge_id.to_owned(),
                                field: Some("target".to_owned()),
                            },
                            format!(
                                "edge \"{edge_id}\" (type \"{}\") target \"{target_id}\" \
                                 has type \"{}\", which is not permitted; \
                                 expected one of: {}",
                                edge_type_display(edge_type),
                                node_type_display(&tgt_node.node_type),
                                format_type_set(permitted_tgt),
                            ),
                        ));
                    }
                }
            }
        }
    }
}

/// Returns the `snake_case` string for an [`EdgeType`].
fn edge_type_display(et: &EdgeType) -> &'static str {
    match et {
        EdgeType::Ownership => "ownership",
        EdgeType::OperationalControl => "operational_control",
        EdgeType::LegalParentage => "legal_parentage",
        EdgeType::FormerIdentity => "former_identity",
        EdgeType::BeneficialOwnership => "beneficial_ownership",
        EdgeType::Supplies => "supplies",
        EdgeType::Subcontracts => "subcontracts",
        EdgeType::Tolls => "tolls",
        EdgeType::Distributes => "distributes",
        EdgeType::Brokers => "brokers",
        EdgeType::Operates => "operates",
        EdgeType::Produces => "produces",
        EdgeType::ComposedOf => "composed_of",
        EdgeType::SellsTo => "sells_to",
        EdgeType::AttestedBy => "attested_by",
        EdgeType::SameAs => "same_as",
    }
}

/// Formats a [`TypeSet`] as a comma-separated human-readable list.
fn format_type_set(types: &[NodeType]) -> String {
    types
        .iter()
        .map(|t| node_type_display(&NodeTypeTag::Known(t.clone())))
        .collect::<Vec<_>>()
        .join(", ")
}
