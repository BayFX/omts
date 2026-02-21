/// L2-GDM-01 through L2-GDM-04 and L2-EID-01 through L2-EID-04:
/// Semantic warning rules enforcing SHOULD constraints from SPEC-001 and SPEC-002.
///
/// These rules are stateless structs implementing [`crate::validation::ValidationRule`].
/// All rules produce [`crate::validation::Severity::Warning`] diagnostics and collect
/// every violation without early exit.
///
/// Rules are registered in [`crate::validation::build_registry`] when
/// [`crate::validation::ValidationConfig::run_l2`] is `true`.
use std::collections::HashSet;

use crate::enums::{EdgeType, EdgeTypeTag, NodeType, NodeTypeTag};
use crate::file::OmtsFile;

use super::{Diagnostic, Level, Location, RuleId, Severity, ValidationRule};

#[cfg(test)]
mod tests;

/// Returns `true` if `code` is a valid ISO 3166-1 alpha-2 country code.
///
/// The list is a static snapshot embedded at compile time. Codes are the
/// 249 officially assigned alpha-2 codes per ISO 3166 Maintenance Agency.
pub fn is_valid_iso3166_alpha2(code: &str) -> bool {
    const CODES: &[&str] = &[
        "AD", "AE", "AF", "AG", "AI", "AL", "AM", "AO", "AQ", "AR", "AS", "AT", "AU", "AW", "AX",
        "AZ", "BA", "BB", "BD", "BE", "BF", "BG", "BH", "BI", "BJ", "BL", "BM", "BN", "BO", "BQ",
        "BR", "BS", "BT", "BV", "BW", "BY", "BZ", "CA", "CC", "CD", "CF", "CG", "CH", "CI", "CK",
        "CL", "CM", "CN", "CO", "CR", "CU", "CV", "CW", "CX", "CY", "CZ", "DE", "DJ", "DK", "DM",
        "DO", "DZ", "EC", "EE", "EG", "EH", "ER", "ES", "ET", "FI", "FJ", "FK", "FM", "FO", "FR",
        "GA", "GB", "GD", "GE", "GF", "GG", "GH", "GI", "GL", "GM", "GN", "GP", "GQ", "GR", "GS",
        "GT", "GU", "GW", "GY", "HK", "HM", "HN", "HR", "HT", "HU", "ID", "IE", "IL", "IM", "IN",
        "IO", "IQ", "IR", "IS", "IT", "JE", "JM", "JO", "JP", "KE", "KG", "KH", "KI", "KM", "KN",
        "KP", "KR", "KW", "KY", "KZ", "LA", "LB", "LC", "LI", "LK", "LR", "LS", "LT", "LU", "LV",
        "LY", "MA", "MC", "MD", "ME", "MF", "MG", "MH", "MK", "ML", "MM", "MN", "MO", "MP", "MQ",
        "MR", "MS", "MT", "MU", "MV", "MW", "MX", "MY", "MZ", "NA", "NC", "NE", "NF", "NG", "NI",
        "NL", "NO", "NP", "NR", "NU", "NZ", "OM", "PA", "PE", "PF", "PG", "PH", "PK", "PL", "PM",
        "PN", "PR", "PS", "PT", "PW", "PY", "QA", "RE", "RO", "RS", "RU", "RW", "SA", "SB", "SC",
        "SD", "SE", "SG", "SH", "SI", "SJ", "SK", "SL", "SM", "SN", "SO", "SR", "SS", "ST", "SV",
        "SX", "SY", "SZ", "TC", "TD", "TF", "TG", "TH", "TJ", "TK", "TL", "TM", "TN", "TO", "TR",
        "TT", "TV", "TW", "TZ", "UA", "UG", "UM", "US", "UY", "UZ", "VA", "VC", "VE", "VG", "VI",
        "VN", "VU", "WF", "WS", "YE", "YT", "ZA", "ZM", "ZW",
    ];
    CODES.binary_search(&code).is_ok()
}

/// Returns the set of facility node IDs that have at least one edge connecting
/// them to an organisation node, either via an `operates` or `operational_control`
/// edge (where the facility is the target) or via the `Node::operator` field.
///
/// Used by [`L2Gdm01`] to detect isolated facilities.
fn facility_ids_with_org_connection<'a>(file: &'a OmtsFile) -> HashSet<&'a str> {
    let org_ids: HashSet<&str> = file
        .nodes
        .iter()
        .filter(|n| n.node_type == NodeTypeTag::Known(NodeType::Organization))
        .map(|n| n.id.as_ref() as &str)
        .collect();

    let facility_ids: HashSet<&str> = file
        .nodes
        .iter()
        .filter(|n| n.node_type == NodeTypeTag::Known(NodeType::Facility))
        .map(|n| n.id.as_ref() as &str)
        .collect();

    let mut connected: HashSet<&'a str> = HashSet::new();

    for node in &file.nodes {
        if node.node_type != NodeTypeTag::Known(NodeType::Facility) {
            continue;
        }
        if let Some(ref op) = node.operator {
            let op_str: &str = op.as_ref() as &str;
            if org_ids.contains(op_str) {
                connected.insert(node.id.as_ref() as &str);
            }
        }
    }

    for edge in &file.edges {
        let edge_type = match &edge.edge_type {
            EdgeTypeTag::Known(et) => et,
            EdgeTypeTag::Extension(_) => continue,
        };

        let src: &str = edge.source.as_ref() as &str;
        let tgt: &str = edge.target.as_ref() as &str;

        let (facility_side, org_side): (&str, &str) = match edge_type {
            EdgeType::Operates => (tgt, src),
            EdgeType::OperationalControl => (tgt, src),
            EdgeType::Tolls => (src, tgt),
            EdgeType::Ownership
            | EdgeType::LegalParentage
            | EdgeType::FormerIdentity
            | EdgeType::BeneficialOwnership
            | EdgeType::Supplies
            | EdgeType::Subcontracts
            | EdgeType::Distributes
            | EdgeType::Brokers
            | EdgeType::Produces
            | EdgeType::ComposedOf
            | EdgeType::SellsTo
            | EdgeType::AttestedBy
            | EdgeType::SameAs => continue,
        };

        if facility_ids.contains(facility_side) && org_ids.contains(org_side) {
            connected.insert(facility_side);
        }
    }

    connected
}

/// L2-GDM-01 — Every `facility` node SHOULD be connected to an `organization`
/// node via an edge or the `operator` property (SPEC-001 Section 9.2).
///
/// An isolated facility (one with no `operates`, `operational_control`, or `tolls`
/// edge to an organisation, and no `operator` field referencing an organisation)
/// is likely an incomplete graph. Each such facility produces one warning.
pub struct L2Gdm01;

impl ValidationRule for L2Gdm01 {
    fn id(&self) -> RuleId {
        RuleId::L2Gdm01
    }

    fn level(&self) -> Level {
        Level::L2
    }

    fn check(
        &self,
        file: &OmtsFile,
        diags: &mut Vec<Diagnostic>,
        _external_data: Option<&dyn super::external::ExternalDataSource>,
    ) {
        let connected = facility_ids_with_org_connection(file);

        for node in &file.nodes {
            if node.node_type != NodeTypeTag::Known(NodeType::Facility) {
                continue;
            }
            let id: &str = &node.id;
            if !connected.contains(id) {
                diags.push(Diagnostic::new(
                    RuleId::L2Gdm01,
                    Severity::Warning,
                    Location::Node {
                        node_id: id.to_owned(),
                        field: None,
                    },
                    format!(
                        "facility \"{id}\" has no edge or `operator` field connecting it to \
                         an organisation; consider adding an `operates` or `operational_control` \
                         edge"
                    ),
                ));
            }
        }
    }
}

/// L2-GDM-02 — `ownership` edges SHOULD have `valid_from` set
/// (SPEC-001 Section 9.2).
///
/// An ownership edge without `valid_from` is ambiguous in temporal merges.
/// Each such edge produces one warning.
pub struct L2Gdm02;

impl ValidationRule for L2Gdm02 {
    fn id(&self) -> RuleId {
        RuleId::L2Gdm02
    }

    fn level(&self) -> Level {
        Level::L2
    }

    fn check(
        &self,
        file: &OmtsFile,
        diags: &mut Vec<Diagnostic>,
        _external_data: Option<&dyn super::external::ExternalDataSource>,
    ) {
        for edge in &file.edges {
            if edge.edge_type != EdgeTypeTag::Known(EdgeType::Ownership) {
                continue;
            }
            if edge.properties.valid_from.is_none() {
                let id: &str = &edge.id;
                diags.push(Diagnostic::new(
                    RuleId::L2Gdm02,
                    Severity::Warning,
                    Location::Edge {
                        edge_id: id.to_owned(),
                        field: Some("properties.valid_from".to_owned()),
                    },
                    format!(
                        "ownership edge \"{id}\" is missing `valid_from`; temporal merge \
                         correctness requires a start date on ownership relationships"
                    ),
                ));
            }
        }
    }
}

/// L2-GDM-03 — Every `organization` and `facility` node, and every `supplies`,
/// `subcontracts`, and `tolls` edge, SHOULD carry a `data_quality` object
/// (SPEC-001 Section 9.2).
///
/// Provenance metadata is essential for merge conflict resolution and regulatory
/// audit trails. Nodes and edges without `data_quality` each produce one warning.
pub struct L2Gdm03;

impl ValidationRule for L2Gdm03 {
    fn id(&self) -> RuleId {
        RuleId::L2Gdm03
    }

    fn level(&self) -> Level {
        Level::L2
    }

    fn check(
        &self,
        file: &OmtsFile,
        diags: &mut Vec<Diagnostic>,
        _external_data: Option<&dyn super::external::ExternalDataSource>,
    ) {
        for node in &file.nodes {
            let should_check = matches!(
                &node.node_type,
                NodeTypeTag::Known(NodeType::Organization) | NodeTypeTag::Known(NodeType::Facility)
            );
            if !should_check {
                continue;
            }
            if node.data_quality.is_none() {
                let id: &str = &node.id;
                let type_str = match &node.node_type {
                    NodeTypeTag::Known(NodeType::Organization) => "organization",
                    NodeTypeTag::Known(NodeType::Facility) => "facility",
                    NodeTypeTag::Known(NodeType::Good)
                    | NodeTypeTag::Known(NodeType::Person)
                    | NodeTypeTag::Known(NodeType::Attestation)
                    | NodeTypeTag::Known(NodeType::Consignment)
                    | NodeTypeTag::Known(NodeType::BoundaryRef)
                    | NodeTypeTag::Extension(_) => continue,
                };
                diags.push(Diagnostic::new(
                    RuleId::L2Gdm03,
                    Severity::Warning,
                    Location::Node {
                        node_id: id.to_owned(),
                        field: Some("data_quality".to_owned()),
                    },
                    format!(
                        "{type_str} node \"{id}\" is missing a `data_quality` object; \
                         provenance metadata is essential for merge conflict resolution"
                    ),
                ));
            }
        }

        for edge in &file.edges {
            let should_check = matches!(
                &edge.edge_type,
                EdgeTypeTag::Known(EdgeType::Supplies)
                    | EdgeTypeTag::Known(EdgeType::Subcontracts)
                    | EdgeTypeTag::Known(EdgeType::Tolls)
            );
            if !should_check {
                continue;
            }
            if edge.properties.data_quality.is_none() {
                let id: &str = &edge.id;
                let type_str = match &edge.edge_type {
                    EdgeTypeTag::Known(EdgeType::Supplies) => "supplies",
                    EdgeTypeTag::Known(EdgeType::Subcontracts) => "subcontracts",
                    EdgeTypeTag::Known(EdgeType::Tolls) => "tolls",
                    EdgeTypeTag::Known(EdgeType::Ownership)
                    | EdgeTypeTag::Known(EdgeType::OperationalControl)
                    | EdgeTypeTag::Known(EdgeType::LegalParentage)
                    | EdgeTypeTag::Known(EdgeType::FormerIdentity)
                    | EdgeTypeTag::Known(EdgeType::BeneficialOwnership)
                    | EdgeTypeTag::Known(EdgeType::Distributes)
                    | EdgeTypeTag::Known(EdgeType::Brokers)
                    | EdgeTypeTag::Known(EdgeType::Operates)
                    | EdgeTypeTag::Known(EdgeType::Produces)
                    | EdgeTypeTag::Known(EdgeType::ComposedOf)
                    | EdgeTypeTag::Known(EdgeType::SellsTo)
                    | EdgeTypeTag::Known(EdgeType::AttestedBy)
                    | EdgeTypeTag::Known(EdgeType::SameAs)
                    | EdgeTypeTag::Extension(_) => continue,
                };
                diags.push(Diagnostic::new(
                    RuleId::L2Gdm03,
                    Severity::Warning,
                    Location::Edge {
                        edge_id: id.to_owned(),
                        field: Some("properties.data_quality".to_owned()),
                    },
                    format!(
                        "{type_str} edge \"{id}\" is missing a `data_quality` object; \
                         provenance metadata is essential for merge conflict resolution"
                    ),
                ));
            }
        }
    }
}

/// L2-GDM-04 — If any `supplies` edge carries a `tier` property, the file
/// SHOULD declare `reporting_entity` in the file header (SPEC-001 Section 9.2).
///
/// Without a reporting entity, `tier` values are ambiguous. One warning is
/// emitted per offending `supplies` edge.
pub struct L2Gdm04;

impl ValidationRule for L2Gdm04 {
    fn id(&self) -> RuleId {
        RuleId::L2Gdm04
    }

    fn level(&self) -> Level {
        Level::L2
    }

    fn check(
        &self,
        file: &OmtsFile,
        diags: &mut Vec<Diagnostic>,
        _external_data: Option<&dyn super::external::ExternalDataSource>,
    ) {
        if file.reporting_entity.is_some() {
            return;
        }

        for edge in &file.edges {
            if edge.edge_type != EdgeTypeTag::Known(EdgeType::Supplies) {
                continue;
            }
            if edge.properties.tier.is_some() {
                let id: &str = &edge.id;
                diags.push(Diagnostic::new(
                    RuleId::L2Gdm04,
                    Severity::Warning,
                    Location::Edge {
                        edge_id: id.to_owned(),
                        field: Some("properties.tier".to_owned()),
                    },
                    format!(
                        "supplies edge \"{id}\" carries a `tier` property but the file has no \
                         `reporting_entity`; `tier` values are ambiguous without an anchor"
                    ),
                ));
            }
        }
    }
}

/// L2-EID-01 — Every `organization` node SHOULD have at least one external
/// identifier (scheme other than `internal`) (SPEC-002 Section 6.2).
///
/// An organisation with only internal identifiers cannot participate in
/// cross-file merge. Each such node produces one warning.
pub struct L2Eid01;

impl ValidationRule for L2Eid01 {
    fn id(&self) -> RuleId {
        RuleId::L2Eid01
    }

    fn level(&self) -> Level {
        Level::L2
    }

    fn check(
        &self,
        file: &OmtsFile,
        diags: &mut Vec<Diagnostic>,
        _external_data: Option<&dyn super::external::ExternalDataSource>,
    ) {
        for node in &file.nodes {
            if node.node_type != NodeTypeTag::Known(NodeType::Organization) {
                continue;
            }
            let id: &str = &node.id;

            let has_external = match &node.identifiers {
                None => false,
                Some(ids) => ids.iter().any(|ident| ident.scheme != "internal"),
            };

            if !has_external {
                diags.push(Diagnostic::new(
                    RuleId::L2Eid01,
                    Severity::Warning,
                    Location::Node {
                        node_id: id.to_owned(),
                        field: Some("identifiers".to_owned()),
                    },
                    format!(
                        "organisation \"{id}\" has no external identifiers (non-`internal` \
                         scheme); cross-file merge requires at least one external identifier \
                         such as `lei`, `duns`, `nat-reg`, or `vat`"
                    ),
                ));
            }
        }
    }
}

/// L2-EID-04 — `vat` authority values SHOULD be valid ISO 3166-1 alpha-2
/// country codes (SPEC-002 Section 6.2).
///
/// The `vat` scheme requires the `authority` field to contain a 2-letter
/// country code. An invalid or unrecognised country code produces one warning
/// per identifier entry. Missing `authority` on `vat` identifiers is already
/// an L1 error (L1-EID-03) and is not re-reported here.
pub struct L2Eid04;

impl ValidationRule for L2Eid04 {
    fn id(&self) -> RuleId {
        RuleId::L2Eid04
    }

    fn level(&self) -> Level {
        Level::L2
    }

    fn check(
        &self,
        file: &OmtsFile,
        diags: &mut Vec<Diagnostic>,
        _external_data: Option<&dyn super::external::ExternalDataSource>,
    ) {
        for node in &file.nodes {
            let node_id: &str = &node.id;
            let Some(ref identifiers) = node.identifiers else {
                continue;
            };
            for (index, ident) in identifiers.iter().enumerate() {
                if ident.scheme != "vat" {
                    continue;
                }
                // Missing authority is an L1-EID-03 concern; skip silently.
                let Some(ref authority) = ident.authority else {
                    continue;
                };
                if !is_valid_iso3166_alpha2(authority.as_str()) {
                    diags.push(Diagnostic::new(
                        RuleId::L2Eid04,
                        Severity::Warning,
                        Location::Identifier {
                            node_id: node_id.to_owned(),
                            index,
                            field: Some("authority".to_owned()),
                        },
                        format!(
                            "node \"{node_id}\" identifiers[{index}]: `vat` authority \
                             \"{authority}\" is not a valid ISO 3166-1 alpha-2 country code"
                        ),
                    ));
                }
            }
        }
    }
}
