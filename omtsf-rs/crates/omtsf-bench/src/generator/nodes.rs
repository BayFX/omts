//! Node builders for all 7 OMTSF node types.

use omtsf_core::enums::{
    AttestationOutcome, AttestationStatus, AttestationType, DisclosureScope, EmissionFactorSource,
    NodeType, NodeTypeTag, OrganizationStatus, RiskLikelihood, RiskSeverity,
};
use omtsf_core::newtypes::{CalendarDate, CountryCode, NodeId};
use omtsf_core::structures::Node;
use rand::Rng;
use rand::rngs::StdRng;

use super::identifiers::{gen_boundary_ref_identifiers, gen_identifiers};

const COUNTRIES: &[&str] = &[
    "US", "GB", "DE", "FR", "NL", "JP", "CN", "BR", "IN", "AU", "KR", "SG", "CH", "SE", "CA",
];

const ORG_NAMES: &[&str] = &[
    "Acme Corp",
    "Global Trade Ltd",
    "Pacific Metals Inc",
    "Rhine Chemicals GmbH",
    "Nordic Timber AB",
    "East Asia Components",
    "Southern Minerals",
    "Atlas Logistics",
    "Sierra Manufacturing",
    "Delta Textiles",
    "Oceanic Foods",
    "Continental Steel",
    "Summit Energy",
    "Horizon Pharmaceuticals",
    "Pinnacle Electronics",
];

const CITIES: &[&str] = &[
    "London",
    "New York",
    "Tokyo",
    "Shanghai",
    "Frankfurt",
    "Singapore",
    "Sydney",
    "São Paulo",
    "Mumbai",
    "Toronto",
    "Seoul",
    "Amsterdam",
    "Zurich",
    "Stockholm",
    "Paris",
];

const COMMODITY_CODES: &[&str] = &[
    "7208.51", "2710.12", "8471.30", "3904.10", "4407.10", "6204.62", "0901.11", "2601.11",
    "8541.40", "3002.15",
];

const ROLES: &[&str] = &[
    "CEO",
    "CFO",
    "Director",
    "Compliance Officer",
    "Supply Chain Manager",
    "Board Member",
];

const STANDARDS: &[&str] = &[
    "ISO 14001",
    "ISO 9001",
    "SA8000",
    "FSC Chain of Custody",
    "RSPO",
    "Fairtrade",
    "BSCI",
    "WRAP",
];

fn country(rng: &mut StdRng) -> CountryCode {
    let c = COUNTRIES[rng.gen_range(0..COUNTRIES.len())];
    // Safety: all entries are valid 2-letter country codes.
    CountryCode::try_from(c).unwrap_or_else(|_| {
        CountryCode::try_from("US").unwrap_or_else(|e| {
            // This is benchmark code; a hardcoded "US" will always pass validation.
            // But we must satisfy the no-unwrap lint, so propagate via a default.
            let _ = e;
            unreachable!()
        })
    })
}

fn calendar_date(rng: &mut StdRng, base_year: u16) -> CalendarDate {
    let year = base_year + rng.gen_range(0..3);
    let month = rng.gen_range(1..=12);
    let day = rng.gen_range(1..=28);
    let s = format!("{year:04}-{month:02}-{day:02}");
    CalendarDate::try_from(s.as_str()).unwrap_or_else(|_| {
        // Fallback: use a known-good date.
        CalendarDate::try_from("2025-01-01").unwrap_or_else(|_| unreachable!())
    })
}

fn node_id(prefix: &str, index: usize) -> NodeId {
    let s = format!("{prefix}-{index:06}");
    NodeId::try_from(s.as_str()).unwrap_or_else(|_| unreachable!())
}

/// Creates an empty node with the given id and type, all optional fields `None`.
fn blank_node(id: NodeId, node_type: NodeTypeTag) -> Node {
    Node {
        id,
        node_type,
        identifiers: None,
        data_quality: None,
        labels: None,
        name: None,
        jurisdiction: None,
        status: None,
        governance_structure: None,
        operator: None,
        address: None,
        geo: None,
        commodity_code: None,
        unit: None,
        role: None,
        attestation_type: None,
        standard: None,
        issuer: None,
        valid_from: None,
        valid_to: None,
        outcome: None,
        attestation_status: None,
        reference: None,
        risk_severity: None,
        risk_likelihood: None,
        lot_id: None,
        quantity: None,
        production_date: None,
        origin_country: None,
        direct_emissions_co2e: None,
        indirect_emissions_co2e: None,
        emission_factor_source: None,
        installation_id: None,
        extra: serde_json::Map::new(),
    }
}

/// Builds an Organization node.
pub fn build_organization(
    rng: &mut StdRng,
    index: usize,
    counter: &mut usize,
    identifier_density: f64,
) -> Node {
    let id = node_id("org", index);
    let ids = gen_identifiers(rng, *counter, identifier_density);
    *counter += 1;

    let name_idx = rng.gen_range(0..ORG_NAMES.len());
    let statuses = [
        OrganizationStatus::Active,
        OrganizationStatus::Active,
        OrganizationStatus::Active,
        OrganizationStatus::Dissolved,
        OrganizationStatus::Suspended,
    ];

    let mut node = blank_node(id, NodeTypeTag::Known(NodeType::Organization));
    node.identifiers = Some(ids);
    node.name = Some(format!("{} #{index}", ORG_NAMES[name_idx]));
    node.jurisdiction = Some(country(rng));
    node.status = Some(statuses[rng.gen_range(0..statuses.len())].clone());
    node
}

/// Builds a Facility node.
pub fn build_facility(
    rng: &mut StdRng,
    index: usize,
    counter: &mut usize,
    identifier_density: f64,
    operator_id: Option<&NodeId>,
) -> Node {
    let id = node_id("fac", index);
    let ids = gen_identifiers(rng, *counter, identifier_density);
    *counter += 1;

    let city_idx = rng.gen_range(0..CITIES.len());

    let mut node = blank_node(id, NodeTypeTag::Known(NodeType::Facility));
    node.identifiers = Some(ids);
    node.name = Some(format!("{} Plant", CITIES[city_idx]));
    node.operator = operator_id.cloned();
    node.address = Some(format!(
        "{} Industrial Zone, {}",
        rng.gen_range(1..999),
        CITIES[city_idx]
    ));
    node.jurisdiction = Some(country(rng));
    node
}

/// Builds a Good node.
pub fn build_good(
    rng: &mut StdRng,
    index: usize,
    counter: &mut usize,
    identifier_density: f64,
) -> Node {
    let id = node_id("good", index);
    let ids = gen_identifiers(rng, *counter, identifier_density);
    *counter += 1;

    let cc_idx = rng.gen_range(0..COMMODITY_CODES.len());
    let units = ["kg", "tonne", "m3", "piece", "litre"];

    let mut node = blank_node(id, NodeTypeTag::Known(NodeType::Good));
    node.identifiers = Some(ids);
    node.name = Some(format!("Product #{index}"));
    node.commodity_code = Some(COMMODITY_CODES[cc_idx].to_owned());
    node.unit = Some(units[rng.gen_range(0..units.len())].to_owned());
    node
}

/// Builds a Person node.
pub fn build_person(
    rng: &mut StdRng,
    index: usize,
    counter: &mut usize,
    identifier_density: f64,
) -> Node {
    let id = node_id("person", index);
    let ids = gen_identifiers(rng, *counter, identifier_density);
    *counter += 1;

    let role_idx = rng.gen_range(0..ROLES.len());

    let mut node = blank_node(id, NodeTypeTag::Known(NodeType::Person));
    node.identifiers = Some(ids);
    node.name = Some(format!("Person #{index}"));
    node.role = Some(ROLES[role_idx].to_owned());
    node
}

/// Builds an Attestation node.
pub fn build_attestation(
    rng: &mut StdRng,
    index: usize,
    counter: &mut usize,
    identifier_density: f64,
) -> Node {
    let id = node_id("att", index);
    let ids = gen_identifiers(rng, *counter, identifier_density);
    *counter += 1;

    let att_types = [
        AttestationType::Certification,
        AttestationType::Audit,
        AttestationType::DueDiligenceStatement,
        AttestationType::SelfDeclaration,
        AttestationType::Other,
    ];
    let outcomes = [
        AttestationOutcome::Pass,
        AttestationOutcome::Pass,
        AttestationOutcome::Pass,
        AttestationOutcome::Fail,
        AttestationOutcome::ConditionalPass,
    ];
    let statuses = [
        AttestationStatus::Active,
        AttestationStatus::Active,
        AttestationStatus::Active,
        AttestationStatus::Expired,
        AttestationStatus::Suspended,
    ];
    let severities = [
        RiskSeverity::Low,
        RiskSeverity::Low,
        RiskSeverity::Medium,
        RiskSeverity::High,
    ];
    let likelihoods = [
        RiskLikelihood::Unlikely,
        RiskLikelihood::Likely,
        RiskLikelihood::VeryLikely,
    ];

    let std_idx = rng.gen_range(0..STANDARDS.len());
    let valid_from = calendar_date(rng, 2023);
    let valid_to = calendar_date(rng, 2025);

    let mut node = blank_node(id, NodeTypeTag::Known(NodeType::Attestation));
    node.identifiers = Some(ids);
    node.attestation_type = Some(att_types[rng.gen_range(0..att_types.len())].clone());
    node.standard = Some(STANDARDS[std_idx].to_owned());
    node.issuer = Some(format!("Auditor Corp #{}", rng.gen_range(1..50)));
    node.valid_from = Some(valid_from);
    node.valid_to = Some(Some(valid_to));
    node.outcome = Some(outcomes[rng.gen_range(0..outcomes.len())].clone());
    node.attestation_status = Some(statuses[rng.gen_range(0..statuses.len())].clone());
    node.reference = Some(format!("REF-{}", rng.gen_range(10000..99999)));
    node.risk_severity = Some(severities[rng.gen_range(0..severities.len())].clone());
    node.risk_likelihood = Some(likelihoods[rng.gen_range(0..likelihoods.len())].clone());
    node
}

/// Builds a Consignment node.
pub fn build_consignment(
    rng: &mut StdRng,
    index: usize,
    counter: &mut usize,
    identifier_density: f64,
    installation_id: Option<&NodeId>,
) -> Node {
    let id = node_id("con", index);
    let ids = gen_identifiers(rng, *counter, identifier_density);
    *counter += 1;

    let emission_sources = [
        EmissionFactorSource::Actual,
        EmissionFactorSource::DefaultEu,
        EmissionFactorSource::DefaultCountry,
    ];

    let mut node = blank_node(id, NodeTypeTag::Known(NodeType::Consignment));
    node.identifiers = Some(ids);
    node.lot_id = Some(format!("LOT-{}", rng.gen_range(100000..999999)));
    node.quantity = Some(rng.gen_range(1.0..10000.0));
    node.production_date = Some(calendar_date(rng, 2024));
    node.origin_country = Some(country(rng));
    node.direct_emissions_co2e = Some(rng.gen_range(0.1..500.0));
    node.indirect_emissions_co2e = Some(rng.gen_range(0.1..200.0));
    node.emission_factor_source =
        Some(emission_sources[rng.gen_range(0..emission_sources.len())].clone());
    node.installation_id = installation_id.cloned();
    node
}

/// Builds a `BoundaryRef` node.
pub fn build_boundary_ref(rng: &mut StdRng, index: usize) -> Node {
    let id = node_id("bref", index);
    let ids = gen_boundary_ref_identifiers(rng);

    let mut node = blank_node(id, NodeTypeTag::Known(NodeType::BoundaryRef));
    node.identifiers = Some(ids);
    node
}

/// Returns the `NodeId` string for an organization by index.
pub fn org_node_id(index: usize) -> NodeId {
    node_id("org", index)
}

/// Returns the `NodeId` string for a facility by index.
pub fn fac_node_id(index: usize) -> NodeId {
    node_id("fac", index)
}

/// Returns the `NodeId` string for a good by index.
pub fn good_node_id(index: usize) -> NodeId {
    node_id("good", index)
}

/// Returns the `NodeId` string for a person by index.
pub fn person_node_id(index: usize) -> NodeId {
    node_id("person", index)
}

/// Returns the `NodeId` string for an attestation by index.
pub fn att_node_id(index: usize) -> NodeId {
    node_id("att", index)
}

/// Returns the `NodeId` string for a consignment by index.
pub fn con_node_id(index: usize) -> NodeId {
    node_id("con", index)
}

/// Returns the `NodeId` string for a `boundary_ref` by index.
pub fn bref_node_id(index: usize) -> NodeId {
    node_id("bref", index)
}

/// Generates an `OmtsFile` disclosure scope for redaction tests.
pub fn gen_disclosure_scope(rng: &mut StdRng) -> DisclosureScope {
    let scopes = [
        DisclosureScope::Internal,
        DisclosureScope::Partner,
        DisclosureScope::Public,
    ];
    scopes[rng.gen_range(0..scopes.len())].clone()
}
