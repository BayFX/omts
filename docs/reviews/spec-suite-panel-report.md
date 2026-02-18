# Expert Panel Report: Full OMTSF Specification Suite

**Date:** 2026-02-18
**Scope:** OMTSF-SPEC-001 through OMTSF-SPEC-006 (all Draft, 2026-02-18)
**Panel Size:** 11 experts
**Review Round:** Post-revision (prior panel findings addressed)

---

## Panel Chair Summary

The OMTSF specification suite was reviewed by an 11-expert panel spanning supply chain operations, procurement, standards development, systems engineering (Rust), graph theory, enterprise integration, regulatory compliance, data serialization, open source strategy, security/privacy, and entity identification. The panel finds that the spec suite has matured substantially from prior review rounds -- 15+ critical and major findings from earlier panels have been resolved -- and now represents a technically sound, domain-informed foundation for supply chain data exchange. The directed labeled property multigraph model, composite identifier strategy with no mandatory scheme, formal merge algebra with algebraic guarantees (commutativity, associativity, idempotency), and layered privacy architecture are each well-designed and mutually reinforcing.

Three areas of strong consensus emerged as remaining gaps. First, **the absence of a machine-readable JSON Schema** was flagged as Critical or Major by 4 experts (Standards, Rust Engineer, Data Format, Open Source) and is the single highest-priority artifact for the ecosystem -- it simultaneously enables automated validation, code generation, IDE integration, and conformance testing. Second, **canonical JSON serialization for the `file_integrity` content hash** was independently identified by 3 experts (Rust Engineer, Data Format, Security/Privacy) as required to make the integrity mechanism portable across implementations. Third, **governance and ecosystem infrastructure** -- no CONTRIBUTING.md, no DCO/CLA, no conformance test suite, single-company copyright under MIT -- was flagged as Critical by the Open Source Strategist and Major by the Standards Expert; the spec quality bottleneck has shifted from "is the spec good enough?" to "can anyone outside BayFX participate?"

Areas of productive disagreement include the priority of the delta/patch mechanism (Critical per Enterprise Integration, deferred by the broader panel), the severity of the missing CBAM emissions model (Critical per Regulatory Compliance, addressable via extension per others), and whether the `legal_parentage` forest constraint should be L2 or L3 (Graph Theorist argues L2; current spec places it at L3). These reflect genuine priority tradeoffs rather than technical errors and should be resolved through TSC governance.

The panel's overall assessment: the specification text is now strong enough for **implementation prototyping** to begin. The P0 issues below should be resolved before any v1.0 declaration or conformance claims.

---

## Panel Composition

| Panelist | Role | Key Focus Area |
|----------|------|----------------|
| Supply Chain Expert | Supply Chain Visibility & Risk Analyst | Multi-tier representability, disruption modeling, regulatory alignment |
| Procurement Expert | Chief Procurement Officer | Operational usability, supplier burden, ERP integration, adoption cost |
| Standards Expert | Standards Development & Interoperability Specialist | Standards alignment, spec rigor, governance, versioning |
| Systems Engineering Expert | Senior Rust Engineer | Rust/WASM implementation, parsing safety, performance |
| Graph Modeling Expert | Graph Data Modeling & Algorithm Specialist | Graph model correctness, merge semantics, algorithm efficiency |
| Enterprise Integration Expert | Enterprise Systems Architect | ERP mapping, EDI coexistence, master data quality, delta updates |
| Regulatory Compliance Expert | Supply Chain Regulatory Compliance Advisor | CSDDD, EUDR, LkSG, CBAM, AMLD alignment |
| Data Format Expert | Data Format Architect | Serialization, schema evolution, compression, integrity |
| Open Source Strategy Expert | Open Source Strategy & Governance Lead | Governance, licensing, adoption flywheel, ecosystem |
| Security & Privacy Expert | Data Security & Privacy Architect | Sensitivity model, selective disclosure, integrity, GDPR |
| Entity Identification Expert | Entity Identification & Corporate Hierarchy Specialist | Identifier coverage, entity resolution, corporate hierarchy, M&A |

---

## Consensus Findings

Issues independently raised by 3+ experts carry the highest weight.

### 1. No Machine-Readable JSON Schema (4 experts)

**Flagged by:** Standards, Rust Engineer, Data Format, Open Source

The entire structural contract exists as Markdown prose tables. No JSON Schema (draft 2020-12), no ABNF grammar, no formal conformance clause structure. Every implementation must hand-translate prose tables into validation code, guaranteeing cross-implementation divergence. The JSON Schema is the single highest-leverage artifact: it enables automated validation, Rust code generation via `schemars`, VS Code intellisense, Python/TypeScript validation via `jsonschema`/`ajv`, and conformance testing -- all without depending on the reference implementation.

### 2. Canonical JSON Serialization Required for Content Hashing (3 experts)

**Flagged by:** Rust Engineer, Data Format, Security/Privacy

SPEC-004 Section 6 defines a SHA-256 content hash, but JSON serialization is not deterministic -- key ordering, whitespace, numeric formatting, and Unicode escaping all vary across implementations. Two semantically identical files will produce different hashes. Without adopting RFC 8785 (JSON Canonicalization Scheme) or equivalent, the `file_integrity` mechanism is implementation-specific and non-portable. This is a hard dependency for digital signature verification.

### 3. Governance and Ecosystem Infrastructure Gaps (3 experts)

**Flagged by:** Open Source, Standards, Procurement

No CONTRIBUTING.md, no DCO/CLA, no code of conduct, no conformance test suite, no reference implementation, single-company (BayFX) copyright under MIT. Enterprise legal departments will not approve contributions without a DCO or CLA. MIT on specifications permits proprietary forking without attribution; CC-BY-4.0 is the industry standard for open specifications (used by OpenAPI, AsyncAPI, CloudEvents). The adoption flywheel has no moving parts.

### 4. Delta/Incremental Update Model Absent (3 experts)

**Flagged by:** Enterprise Integration, Procurement, Supply Chain

Full-file re-export at enterprise scale (40,000+ vendors) is operationally infeasible. ERP change document tables (SAP CDHDR/CDPOS, Oracle audit columns, D365 change tracking) produce incremental deltas natively. Without a delta envelope, every OMTSF integration requires a custom reconciliation layer, negating standardization benefits.

---

## Critical Issues

| # | Issue | Raised By | Summary |
|---|-------|-----------|---------|
| C1 | No JSON Schema | Standards, Rust Engineer, Data Format, Open Source | Prose-only specification guarantees cross-implementation divergence. Machine-readable schema is the single highest-priority artifact. |
| C2 | No CONTRIBUTING.md or DCO/CLA | Open Source | Blocks all external participation. Enterprise legal departments require formal contribution agreements. |
| C3 | Single-company copyright on specifications | Open Source | BayFX copyright under MIT permits proprietary forking. Specs should use CC-BY-4.0 with "OMTSF Contributors" copyright. |
| C4 | No conformance test suite | Open Source, Standards | 30+ validation rules with no machine-executable test fixtures. Conformance claims are unverifiable. |
| C5 | No CBAM embedded emissions data | Regulatory Compliance | CBAM definitive phase began 2026-01-01; first surrender deadline 2027-09-30. No fields for direct/indirect emissions on consignment nodes. |
| C6 | No supplier-facing data collection guidance | Procurement | Format defines file structure but provides zero guidance on collecting data from suppliers, especially Tier-2+ SMEs. |
| C7 | Delta/patch mechanism absent | Enterprise Integration | Full re-export at enterprise scale is infeasible. Deferred to P2 by prior panel but remains a deployment blocker. |

---

## Major Issues

| # | Issue | Raised By | Summary |
|---|-------|-----------|---------|
| M1 | Content hash requires canonical JSON | Rust Engineer, Data Format, Security/Privacy | `file_integrity` hash non-portable without RFC 8785 or equivalent. |
| M2 | No streaming format for large files | Rust Engineer, Data Format | At advisory limits (~1.5 GB JSON), single-document structure prevents streaming. |
| M3 | First-key requirement conflicts with RFC 8259 | Data Format | JSON objects are formally unordered; first-key detection depends on serializer behavior. |
| M4 | Merge false-positive amplification | Graph Theorist | Single erroneous identifier match cascades through transitive closure. No size limits or confidence scoring. |
| M5 | No formal graph schema definition | Graph Theorist | Node/edge type constraints are prose-only. No machine-readable graph type definition. |
| M6 | No n-ary relationship support | Graph Theorist | Multi-party supply arrangements require binary decomposition without documented reification pattern. |
| M7 | Oracle/D365 ERP mappings shallow | Enterprise Integration, Procurement | SAP mapping is production-grade; Oracle/D365 lack API-level detail. |
| M8 | No temporal snapshot linkage | Regulatory Compliance, Supply Chain | CSDDD/LkSG require continuous monitoring; no mechanism to link successive snapshots. |
| M9 | EUDR geolocation precision unspecified | Regulatory Compliance | No minimum coordinate precision for EUDR plot verification (requires 6+ decimal digits). |
| M10 | No risk severity on attestation outcomes | Regulatory Compliance | `outcome` field is binary, not severity-weighted for risk-prioritized due diligence. |
| M11 | Merge provenance lacks authentication | Security/Privacy | `merge_metadata` has no cryptographic attribution. Data poisoning vector. |
| M12 | No edge property sensitivity model | Security/Privacy | `contract_ref`, `annual_value` leak competitive intelligence with no classification mechanism. |
| M13 | ISO 5009 mischaracterization | Standards | SPEC-006 incorrectly describes ISO 5009 as organizational identification; it defines Official Organizational Roles. |
| M14 | TSC Charter referenced but incomplete | Standards | SPEC-002 Section 5.3 references charter; governance process depends on it. |
| M15 | No IP policy | Standards | No CLA, patent grant, or IP commitment. Risks patent assertion by contributors. |
| M16 | No version compatibility rules | Standards | No defined consumer behavior for version mismatch. |
| M17 | LEI lapsed rates by jurisdiction | Entity Identification | China 96.9%, Russia 71.8% lapsed. L2-EID-05 warnings will overwhelm implementations. |
| M18 | GLEIF Level 2 coverage gaps | Entity Identification | `legal_parentage` terminates at entities without LEIs (state-owned enterprises, family offices). |
| M19 | US UEI absent from scheme vocabulary | Entity Identification | 350,000+ US government contractors use UEI with no DUNS cross-reference. |
| M20 | No cost/ROI framework for enrichment | Procurement | LEI $50-200/entity/year, DUNS hierarchy $50K-250K/year. No cost-ordered path. |
| M21 | No procurement extension namespace | Procurement | Approval status, payment terms, spend category undefined; every adopter reinvents. |
| M22 | Supply capacity/allocation unrepresentable | Supply Chain | No `share_of_buyer_demand` property for disruption modeling. |
| M23 | `data_quality` too optional to drive adoption | Supply Chain | No L2 rule encouraging adoption; most files will ship without provenance metadata. |
| M24 | `authority` naming convention informal | Enterprise Integration, Procurement | Recommended in informative doc but not enforced at any validation level. |
| M25 | STCD1/STCD2 disambiguation incomplete | Enterprise Integration | Missing India GSTIN, China USCC, Mexico RFC, Italy, STCD3/STCD4. |
| M26 | No conformance clause definitions | Open Source | No formal "conformant producer/consumer/validator" roles with testable obligations. |
| M27 | No reference implementation or roadmap | Open Source | Six specifications and zero code. |

---

## Minor Issues

| # | Issue | Raised By |
|---|-------|-----------|
| m1 | No MIME type registration | Rust Engineer, Data Format |
| m2 | `geo` field is unstructured (untagged union) | Rust Engineer |
| m3 | `null` vs. absent semantics unspecified | Rust Engineer |
| m4 | `snapshot_date` is date-only | Data Format |
| m5 | Signature key distribution unspecified | Data Format, Security/Privacy |
| m6 | `tolls` edge direction unintuitive | Supply Chain |
| m7 | No commodity classification enforcement | Supply Chain |
| m8 | No explicit suspected-relationship representation | Supply Chain |
| m9 | DUNS redistribution risk unacknowledged | Standards |
| m10 | Regulatory alignment table stale (Omnibus I) | Standards, Regulatory Compliance |
| m11 | Boundary reference stability gap (no opt-in stable mode) | Security/Privacy |
| m12 | `beneficial_ownership` sensitivity inheritance implicit | Security/Privacy |
| m13 | UFLPA entity list linkage absent | Regulatory Compliance |
| m14 | Conflict minerals regulation coverage incomplete | Regulatory Compliance |
| m15 | `legal_parentage` forest constraint at L3, should be L2 | Graph Theorist |
| m16 | No graph-level summary metadata | Graph Theorist |
| m17 | `boundary_ref` creates phantom connectivity | Graph Theorist |
| m18 | No quick-start guide / minimum viable file | Procurement |
| m19 | No M&A operational guidance for procurement | Procurement |
| m20 | Cycle detection requirements underspecified | Graph Theorist |
| m21 | No identifier scheme trust hierarchy | Entity Identification |
| m22 | L3-EID-03 underspecified for multi-jurisdiction entities | Entity Identification |
| m23 | Consignment node lacks identifier scheme guidance | Entity Identification |
| m24 | Name normalization guidance absent | Entity Identification |
| m25 | No code of conduct | Open Source |
| m26 | No RFC/spec change process for normative changes | Open Source |
| m27 | No ERP extraction scheduling guidance | Enterprise Integration |
| m28 | `sells_to` edge has no ERP mapping | Enterprise Integration |
| m29 | No purchasing org/company code mapping guidance | Enterprise Integration |

---

## Consolidated Recommendations

### P0 -- Immediate

| # | Recommendation | Source Expert(s) |
|---|---------------|-----------------|
| P0-1 | **Publish normative JSON Schema (draft 2020-12)** for the `.omts` file format. Include all node types, edge types with `"properties"` wrapper, identifier records with conditional `authority`, and `file_integrity`. Store at `schema/omts-v0.1.0.schema.json`. | Standards, Data Format, Rust Engineer, Open Source |
| P0-2 | **Create CONTRIBUTING.md with DCO** defining contribution workflow, sign-off requirements, and interaction with TSC governance. | Open Source |
| P0-3 | **Separate spec and code licensing.** CC-BY-4.0 for specifications in `spec/`, Apache 2.0 for code. Copyright "OMTSF Contributors." | Open Source |
| P0-4 | **Publish conformance test fixtures** for all L1 validation rules. One valid and one invalid `.omts` file per rule, with expected results. | Open Source, Standards |
| P0-5 | **Add CBAM embedded emissions properties** to `consignment` nodes or define extension. Fields: `direct_emissions_co2e`, `indirect_emissions_co2e`, `emission_factor_source`, `installation_id`. First surrender deadline: 2027-09-30. | Regulatory Compliance |
| P0-6 | **Create supplier data collection guide** with minimum viable data request template for SME suppliers. | Procurement |
| P0-7 | **Define delta/patch envelope specification** with `update_type: "delta"`, operations array (add/modify/remove), `disclosure_scope` inheritance. | Enterprise Integration |
| P0-8 | **Add merge-group size limits and confidence scoring** to prevent false-positive transitive closure cascading. Advisory max group size with warning. | Graph Theorist |

### P1 -- Before v1.0

| # | Recommendation | Source Expert(s) |
|---|---------------|-----------------|
| P1-1 | Adopt RFC 8785 (JCS) for canonical JSON when computing `file_integrity` content hash | Rust Engineer, Data Format, Security/Privacy |
| P1-2 | Define compression envelope (`.omts.zst` for zstandard, `.omts.gz` as alternative) | Data Format |
| P1-3 | Document RFC 8259 key ordering tension for first-key requirement | Data Format |
| P1-4 | Correct ISO 5009 entry in SPEC-006 (defines Organizational Roles, not identifiers) | Standards |
| P1-5 | Create TSC Charter (verify existence) and establish IP policy with CLA/patent commitment | Standards, Open Source |
| P1-6 | Define version compatibility rules (minor = backward-compatible, major = may break) | Standards |
| P1-7 | Publish ABNF grammar for canonical identifier string format | Standards |
| P1-8 | Publish formal graph schema (permitted source/target node types per edge type) | Graph Theorist |
| P1-9 | Document reification pattern for n-ary relationships in informative appendix | Graph Theorist |
| P1-10 | Promote `legal_parentage` forest constraint to L2 | Graph Theorist |
| P1-11 | Expand Oracle SCM Cloud and D365 mappings to API-level detail | Enterprise Integration, Procurement |
| P1-12 | Expand STCD1/STCD2 disambiguation to cover India, China, Mexico, Italy, STCD3/STCD4 | Enterprise Integration |
| P1-13 | Clarify purchasing organization mapping for multi-org SAP deployments | Enterprise Integration |
| P1-14 | Define snapshot versioning (`previous_snapshot_ref`, `snapshot_sequence`) for audit trails | Regulatory Compliance, Supply Chain |
| P1-15 | Specify EUDR geolocation precision (6+ decimal digits, polygon for >4ha) as L2 rule | Regulatory Compliance |
| P1-16 | Add risk classification to attestation nodes (`risk_severity`, `risk_likelihood`) | Regulatory Compliance |
| P1-17 | Add authenticated provenance to merge metadata (source file hashes, contributor IDs) | Security/Privacy |
| P1-18 | Extend sensitivity model to edge properties (at minimum `contract_ref`, `annual_value`) | Security/Privacy |
| P1-19 | Add `share_of_buyer_demand` property (0-100%) to `supplies`/`subcontracts` edges | Supply Chain |
| P1-20 | Add L2 rule encouraging `data_quality` on all nodes and supply edges | Supply Chain |
| P1-21 | Add sub-national `region` field (ISO 3166-2) to facility nodes | Supply Chain |
| P1-22 | Publish cost-ordered identifier enrichment path (free vs. paid sources) | Procurement |
| P1-23 | Define recommended procurement extension namespace (`com.omtsf.procurement`) | Procurement |
| P1-24 | Create one-page quick-start guide with minimum viable `.omts` file | Procurement |
| P1-25 | Publish worked multi-ERP deduplication example | Procurement |
| P1-26 | Define formal conformance clauses (producer/consumer/validator) | Open Source, Standards |
| P1-27 | Name adoption wedge and publish roadmap (recommended: German LkSG + automotive) | Open Source |
| P1-28 | Publish `omtsf-rs` repository (even empty with README and crate structure) | Open Source |
| P1-29 | Add jurisdiction-aware guidance for LEI lapsed status warnings | Entity Identification |
| P1-30 | Document GLEIF Level 2 coverage limitations in SPEC-001 Section 5.3 | Entity Identification |
| P1-31 | Add UEI as extension scheme (`org.sam.uei`); consider fast-tracking to core | Entity Identification |
| P1-32 | Clarify `null` vs. absent semantics for optional fields | Rust Engineer |
| P1-33 | Promote `authority` naming convention from SPEC-005 to SHOULD-level in SPEC-002 | Enterprise Integration, Procurement |
| P1-34 | Add restricted-party-list flag or extension for UFLPA entity list | Regulatory Compliance |

### P2 -- Future

| # | Recommendation | Source Expert(s) |
|---|---------------|-----------------|
| P2-1 | Register MIME type `application/vnd.omtsf+json` | Rust Engineer, Data Format |
| P2-2 | Define NDJSON streaming variant for large graphs | Rust Engineer |
| P2-3 | Structure `geo` as discriminated union | Rust Engineer |
| P2-4 | Add key distribution guidance for digital signatures | Data Format, Security/Privacy |
| P2-5 | Consider upgrading `snapshot_date` to datetime | Data Format |
| P2-6 | Add DUNS redistribution notice to SPEC-002 | Standards |
| P2-7 | Update SPEC-006 regulatory table for Omnibus I | Standards, Regulatory Compliance |
| P2-8 | Evaluate foundation hosting (LF, OASIS, Eclipse) | Open Source |
| P2-9 | Plan language bindings (Python via PyO3, TypeScript via WASM) | Open Source |
| P2-10 | Establish community extension scheme registry | Open Source |
| P2-11 | Add graph database export guidance (Neo4j, Neptune, TigerGraph) | Graph Theorist |
| P2-12 | Add graph-level summary metadata to file header | Graph Theorist |
| P2-13 | Consider stable boundary reference mode (opt-in persistent salt) | Security/Privacy |
| P2-14 | Formalize edge sensitivity inheritance model | Security/Privacy |
| P2-15 | Track EUDR simplification review (due 2026-04-30) | Regulatory Compliance |
| P2-16 | Define commodity classification precision field | Supply Chain |
| P2-17 | Add tolling worked example to SPEC-001 | Supply Chain |
| P2-18 | Define `relationship_basis` property on supply edges | Supply Chain |
| P2-19 | Add `sells_to` ERP mapping to SPEC-005 | Enterprise Integration |
| P2-20 | Add ERP extraction scheduling guidance | Enterprise Integration |
| P2-21 | Add non-normative scheme trust hierarchy for conflict resolution | Entity Identification |
| P2-22 | Clarify L3-EID-03 for multi-jurisdiction entities | Entity Identification |
| P2-23 | Add consignment identifier scheme guidance | Entity Identification |
| P2-24 | Reference GLEIF entity matching algorithm in SPEC-003 | Entity Identification |
| P2-25 | Add M&A operational guidance for procurement | Procurement |
| P2-26 | Define procurement contract metadata extensions | Procurement |

---

## Cross-Domain Interactions

These interdependencies are the most valuable insights from a multi-expert review.

### 1. JSON Schema Enables Everything

The Standards Expert's P0-1 (JSON Schema) is a prerequisite for the Rust Engineer's code generation via `schemars`, the Data Format Expert's schema evolution strategy, the Open Source Strategist's conformance test suite, and the Graph Theorist's graph type definitions. This single artifact unblocks 4+ expert domains simultaneously.

### 2. Canonical JSON Is a Hard Dependency for Integrity

The Data Format Expert's RFC 8785 recommendation (P1-1) is required by the Security/Privacy Expert's content hash verification and the Rust Engineer's cross-implementation signature validation. Without it, the entire `file_integrity` mechanism is implementation-specific. The Rust Engineer notes that `serde_json` does not guarantee key order for `HashMap`-backed structures -- `BTreeMap` or `IndexMap` would be required.

### 3. GLEIF Level 2 Gaps Affect Corporate Hierarchy Across Domains

The Entity Identification Expert's finding that GLEIF Level 2 data terminates at entities without LEIs (state-owned enterprises, sovereign wealth funds, family offices) affects the Graph Theorist's forest constraint (shallower hierarchies than expected), the Regulatory Compliance Expert's CSDDD value chain mapping (Article 6 requires mapping corporate structures), and the Supply Chain Expert's corporate risk concentration analysis.

### 4. Delta/Patch Is an Enterprise Integration Prerequisite

The Enterprise Integration Expert's P0-7 directly enables the Procurement Expert's operational adoption and the Supply Chain Expert's temporal graph versioning. The Security/Privacy Expert notes that delta files are categorically more intelligence-dense than snapshots and should inherit `disclosure_scope` constraints with `restricted` as the floor.

### 5. Edge Property Sensitivity Closes a Confidentiality Gap

The Security/Privacy Expert's P1-18 (edge sensitivity) addresses the Procurement Expert's concern about competitive intelligence leakage (`annual_value`, `contract_ref`) and the Enterprise Integration Expert's SAP pricing field mapping. Currently node identifiers are carefully protected but edge metadata passes through unprotected.

### 6. Adoption Wedge Connects Governance to Implementation

The Open Source Strategist's recommended adoption wedge (German LkSG + automotive) directly leverages the Enterprise Integration Expert's SAP S/4HANA mapping, the Regulatory Compliance Expert's LkSG coverage, and the Procurement Expert's request for a reference extractor. The Procurement Expert offers to co-sponsor a reference SAP extractor as a community project.

### 7. Merge Confidence Scoring Prevents Graph Corruption

The Graph Theorist's P0-8 (merge-group size limits) directly addresses the Entity Identification Expert's concern about false merges from lapsed or reassigned identifiers (China LEI lapse rate 96.9%) and the Supply Chain Expert's data quality concerns. The Entity Identification Expert recommends a non-normative trust ordering (LEI > nat-reg > GLN > DUNS > VAT > internal) to help implementations weight merge evidence.

### 8. CBAM Emissions Require ERP Mapping

The Regulatory Compliance Expert's P0-5 (CBAM emissions) creates a dependency on the Enterprise Integration Expert to map SAP S/4HANA CBAM module and Oracle sustainability reporting data. The Supply Chain Expert notes that `facility` -> `produces` -> `good` path exists but lacks emissions intensity data.

### 9. Temporal Audit Trails Have Integrity Requirements

The Regulatory Compliance Expert's snapshot versioning recommendation has a Security/Privacy implication: a chain of snapshot references needs tamper-evident linking (each snapshot referencing the hash of its predecessor) to be credible as a regulatory audit trail. Without integrity, a company could retroactively alter earlier snapshots.

### 10. LEI Lapsed Rates Hit Asian Supply Chains Hardest

The Entity Identification Expert's LEI lapse data (China 96.9%, Russia 71.8%) directly impacts the Supply Chain Expert's risk models for Asian electronics manufacturing and Russian raw materials. The recommended treatment: LEI status as a data quality signal, not an entity validity signal. The `verification_date` field on identifiers enables a "freshness" check.

---

## Individual Expert Reports

### Supply Chain Expert
*Full report:* [`full-suite-review-supply-chain-expert.md`](full-suite-review-supply-chain-expert.md)

The spec suite avoids the two common failure modes -- oversimplifying real-world complexity and demanding unattainable data completeness. The supply edge taxonomy with regulatory annotations (CSDDD Article 3(e), LkSG Section 2(7)) is the strongest domain feature. Volume/value properties on supply edges, consignment traceability, and `composed_of` BOM edges resolve prior critical gaps. Remaining concerns: no supply capacity/allocation for disruption modeling [Major], `data_quality` too optional [Major], no geographic risk modeling beyond coordinates [Major], snapshot-only temporal model [Major]. 7 recommendations (P1-P2).

### Procurement Expert
*Full report:* [`spec-suite-review-procurement-expert.md`](spec-suite-review-procurement-expert.md)

Materially stronger as a procurement artifact after revisions. SAP BP mapping, attestation lifecycle states, and STCD disambiguation are directly actionable. However, the suite remains a data model, not an operational procurement tool. Critical gaps: no supplier data collection guidance, no enrichment cost analysis. Major gaps: shallow Oracle/D365 mappings, no delta updates, no procurement extension namespace. 10 recommendations (P0-P2).

### Standards Expert
*Review returned inline.*

Well-conceived format with correct identifier strategy. Strongest praise for GLEIF RA list reuse, LEI lifecycle handling, ISO 6523 mapping, and EPCIS complementarity. Critical: entire suite is prose-only with no JSON Schema or ABNF grammar. Major: ISO 5009 mischaracterized, missing IP policy, TSC Charter incomplete, no conformance test suite, no version compatibility rules. 8 recommendations.

### Systems Engineering Expert (Rust)
*Full report:* [`spec-suite-review-systems-engineer.md`](spec-suite-review-systems-engineer.md)

Prior implementation-blocking issues resolved (edge property wrapper, size limits). Four SPEC-004 test vectors enable conformance testing. `"properties"` wrapper maps cleanly to `serde` with `#[serde(tag = "type")]`. Remaining: no JSON Schema for code generation [Major], `file_integrity` needs canonical serialization [Major], no streaming format for large files [Major]. 6 recommendations (P1-P2).

### Graph Modeling Expert
*Full report:* [`spec-suite-review-graph-modeling.md`](spec-suite-review-graph-modeling.md)

Correct graph model aligned with ISO/IEC 39075 (GQL). Union-find merge is optimal. Nuanced cycle policy reflects domain reality. Key concerns: transitive closure false-positive amplification [Major] -- single erroneous match cascades into massive incorrect merges with no safety net; no formal graph schema [Major]; no n-ary relationship support [Major]. 6 recommendations (P0-P2).

### Enterprise Integration Expert
*Full report:* [`spec-suite-review-enterprise-integration.md`](spec-suite-review-enterprise-integration.md)

SAP BP mapping resolved the prior Critical finding. `composed_of` edge, EDI coexistence, STCD disambiguation all correct. Critical: delta/patch still absent -- full re-export infeasible for 40K+ vendor masters. Major: Oracle/D365 at whiteboard level, STCD table incomplete (missing India/China/Mexico/Italy), no purchasing org mapping guidance. 7 recommendations (P0-P2).

### Regulatory Compliance Expert
*Full report:* [`spec-suite-review-regulatory-compliance.md`](spec-suite-review-regulatory-compliance.md)

Remarkably well-informed regulatory mapping. EUDR coverage strong with geolocation and DDS support. Inline regulatory annotations on edge types are valuable. Critical: no CBAM embedded emissions (first surrender deadline 2027-09-30). Major: no temporal audit trail for continuous monitoring, EUDR geo precision unspecified, no risk severity on attestations. 7 recommendations (P0-P2). Grounded in December 2025 CSDDD Omnibus I agreement and revised EUDR.

### Data Format Expert
*Full report:* [`spec-suite-review-data-format-r2.md`](spec-suite-review-data-format-r2.md)

Five of six prior Critical findings resolved. Format has coherent integrity story end-to-end. Critical: still no JSON Schema -- the strongest R1 consensus finding, unresolved. Major: content hash underspecified without canonical JSON, no compression envelope, first-key/RFC 8259 tension. 7 recommendations (P0-P2).

### Open Source Strategy Expert
*Full report:* [`spec-suite-review-open-source-strategist.md`](spec-suite-review-open-source-strategist.md)

Spec quality is now strong enough that the bottleneck has shifted to participation pathways. TSC Charter is production-grade governance scaffolding. Three Critical gaps: no CONTRIBUTING.md/DCO (blocks all external participation), single-company copyright (anti-pattern for open standards), no conformance test suite (conformance claims unverifiable). Compares to OpenTelemetry, CloudEvents, Open Supply Hub governance models. 9 recommendations (P0-P2).

### Security & Privacy Expert
*Full report:* [`spec-suite-review-security-privacy.md`](spec-suite-review-security-privacy.md)

One of the most privacy-conscious supply chain exchange formats reviewed. Previously critical gaps resolved (file integrity, `nat-reg` sensitivity). Remaining: merge provenance lacks authentication [Major] -- data poisoning vector; no edge property sensitivity [Major]; content hash needs canonical JSON [Major]. Person node privacy posture (confidential default, mandatory omission from public files) exceeds most formats. 6 recommendations (P1-P2).

### Entity Identification Expert
*Full report:* [`full-suite-review-entity-identification-expert.md`](full-suite-review-entity-identification-expert.md)

Prior P0 finding (temporal merge compatibility) fully resolved. Spec passes the bar for production entity resolution configuration. New concerns grounded in GLEIF operational data: LEI lapsed rates by jurisdiction [Major] (China 96.9%), GLEIF Level 2 coverage gaps [Major] (hierarchy terminates at un-LEI'd parents), US UEI absent [Major] (350K+ entities). 7 recommendations (P1-P2).
