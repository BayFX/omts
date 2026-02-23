/// L1-SDI-01 and L1-SDI-02: Selective Disclosure structural validation rules.
///
/// These rules enforce the MUST constraints from SPEC-004 as listed in the
/// validation specification (docs/validation.md Section 4.1, L1-SDI table).
/// Each rule is a stateless struct implementing [`crate::validation::ValidationRule`].
/// All rules collect every violation without early exit.
///
/// Rules are registered in [`crate::validation::build_registry`] when
/// [`crate::validation::ValidationConfig::run_l1`] is `true`.
use crate::enums::{DisclosureScope, NodeType, NodeTypeTag, Sensitivity};
use crate::file::OmtsFile;
use crate::sensitivity::effective_sensitivity;

use super::{Diagnostic, Level, Location, RuleId, Severity, ValidationRule};

#[cfg(test)]
mod tests;

/// L1-SDI-01 — `boundary_ref` nodes have exactly one identifier with scheme `opaque`.
///
/// Per SPEC-004 and the redaction specification (docs/redaction.md Section 5.1),
/// a `boundary_ref` node MUST carry exactly one identifier and that identifier's
/// `scheme` MUST be `"opaque"`. Zero identifiers, more than one identifier, or
/// any identifier whose scheme is not `"opaque"` are all violations.
pub struct L1Sdi01;

impl ValidationRule for L1Sdi01 {
    fn id(&self) -> RuleId {
        RuleId::L1Sdi01
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
        for node in &file.nodes {
            if node.node_type != NodeTypeTag::Known(NodeType::BoundaryRef) {
                continue;
            }

            let node_id: &str = &node.id;

            let Some(identifiers) = &node.identifiers else {
                diags.push(Diagnostic::new(
                    RuleId::L1Sdi01,
                    Severity::Error,
                    Location::Node {
                        node_id: node_id.to_owned(),
                        field: Some("identifiers".to_owned()),
                    },
                    format!(
                        "boundary_ref node \"{node_id}\" has no identifiers; \
                         must have exactly one identifier with scheme \"opaque\""
                    ),
                ));
                continue;
            };

            let opaque_count = identifiers
                .iter()
                .filter(|id| id.scheme == "opaque")
                .count();

            let total_count = identifiers.len();

            if total_count == 0 {
                diags.push(Diagnostic::new(
                    RuleId::L1Sdi01,
                    Severity::Error,
                    Location::Node {
                        node_id: node_id.to_owned(),
                        field: Some("identifiers".to_owned()),
                    },
                    format!(
                        "boundary_ref node \"{node_id}\" has an empty identifiers array; \
                         must have exactly one identifier with scheme \"opaque\""
                    ),
                ));
                continue;
            }

            if opaque_count == 0 {
                diags.push(Diagnostic::new(
                    RuleId::L1Sdi01,
                    Severity::Error,
                    Location::Node {
                        node_id: node_id.to_owned(),
                        field: Some("identifiers".to_owned()),
                    },
                    format!(
                        "boundary_ref node \"{node_id}\" has no identifier with scheme \
                         \"opaque\"; must have exactly one"
                    ),
                ));
            } else if opaque_count > 1 {
                diags.push(Diagnostic::new(
                    RuleId::L1Sdi01,
                    Severity::Error,
                    Location::Node {
                        node_id: node_id.to_owned(),
                        field: Some("identifiers".to_owned()),
                    },
                    format!(
                        "boundary_ref node \"{node_id}\" has {opaque_count} identifiers with \
                         scheme \"opaque\"; must have exactly one"
                    ),
                ));
            }

            if total_count > 1 {
                diags.push(Diagnostic::new(
                    RuleId::L1Sdi01,
                    Severity::Error,
                    Location::Node {
                        node_id: node_id.to_owned(),
                        field: Some("identifiers".to_owned()),
                    },
                    format!(
                        "boundary_ref node \"{node_id}\" has {total_count} identifiers; \
                         must have exactly one identifier with scheme \"opaque\""
                    ),
                ));
            }
        }
    }
}

/// L1-SDI-02 — If `disclosure_scope` is declared, sensitivity constraints are satisfied.
///
/// The constraints are derived from the redaction specification
/// (docs/redaction.md Sections 3.1 and 3.2):
///
/// - `partner` scope: identifiers with effective sensitivity `confidential` must
///   not appear.
/// - `public` scope: identifiers with effective sensitivity `confidential` OR
///   `restricted` must not appear.
/// - `internal` scope: no sensitivity constraints.
///
/// If `disclosure_scope` is absent from the file header, this rule emits no
/// diagnostics (the constraint only applies when a scope is explicitly declared).
pub struct L1Sdi02;

impl ValidationRule for L1Sdi02 {
    fn id(&self) -> RuleId {
        RuleId::L1Sdi02
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
        let Some(scope) = &file.disclosure_scope else {
            return;
        };

        let max_allowed = match scope {
            DisclosureScope::Internal => return,
            DisclosureScope::Partner => Sensitivity::Restricted,
            DisclosureScope::Public => Sensitivity::Public,
        };

        for node in &file.nodes {
            let node_id: &str = &node.id;
            let Some(identifiers) = &node.identifiers else {
                continue;
            };

            for (index, identifier) in identifiers.iter().enumerate() {
                let eff = effective_sensitivity(identifier, &node.node_type);

                let violates = match max_allowed {
                    Sensitivity::Restricted => eff == Sensitivity::Confidential,
                    Sensitivity::Public => {
                        eff == Sensitivity::Confidential || eff == Sensitivity::Restricted
                    }
                    // Internal scope returns early above; this arm is unreachable
                    // but the exhaustive match is required by workspace rules.
                    Sensitivity::Confidential => false,
                };

                if violates {
                    let scope_label = match scope {
                        DisclosureScope::Internal => "internal",
                        DisclosureScope::Partner => "partner",
                        DisclosureScope::Public => "public",
                    };
                    let sensitivity_label = match eff {
                        Sensitivity::Public => "public",
                        Sensitivity::Restricted => "restricted",
                        Sensitivity::Confidential => "confidential",
                    };
                    diags.push(Diagnostic::new(
                        RuleId::L1Sdi02,
                        Severity::Error,
                        Location::Identifier {
                            node_id: node_id.to_owned(),
                            index,
                            field: Some("sensitivity".to_owned()),
                        },
                        format!(
                            "node \"{node_id}\" identifiers[{index}] has effective sensitivity \
                             \"{sensitivity_label}\" which violates disclosure_scope \
                             \"{scope_label}\""
                        ),
                    ));
                }
            }
        }
    }
}
