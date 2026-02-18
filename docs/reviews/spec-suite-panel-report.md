# Expert Panel Report: OMTSF Specification Suite

**Specs Reviewed:** OMTSF-SPEC-001 through OMTSF-SPEC-006 (all Draft, Revision 1, 2026-02-18)
**Panel Date:** 2026-02-18
**Panel Chair:** Automated synthesis of 11 independent expert reviews

---

## Panel Chair Summary

The OMTSF specification suite -- six documents decomposed from the original monolithic entity identification spec -- represents a well-architected foundation for supply chain data exchange. All eleven panelists confirmed that the prior R1 and R2 panel P0 findings have been resolved. The decomposition into four normative specs (Graph Data Model, Entity Identification, Merge Semantics, Selective Disclosure) and two informative guides (ERP Integration, Standards Mapping) is structurally sound and follows established standards practice. The core architectural decisions -- composite identifier model with no mandatory scheme, flat adjacency list serialization, tiered validation (L1/L2/L3), and formal merge algebra with algebraic guarantees -- were praised across all perspectives.

However, the panel identifies a systematic gap between the specification's strength as a **data model** and its readiness as an **implementation specification**. The strongest consensus finding -- flagged independently by three or more experts from different perspectives -- is the absence of a machine-readable schema (JSON Schema), which affects every implementation. The second strongest consensus is the need for a confidence/verification field on identifier records, flagged by three experts (Supply Chain, Entity Identification, Regulatory Compliance). A third cross-cutting consensus is the need for a `composed_of`/BOM edge type for material traceability, flagged from three different angles (Enterprise Integration for ERP BOM data, Regulatory Compliance for EUDR consignment traceability, Supply Chain for disruption propagation). The edge property serialization ambiguity (flat fields vs. `"properties"` wrapper) was flagged by both the Systems Engineering Expert and Data Format Expert as implementation-blocking.

Areas of productive disagreement center on scope and timing. The Supply Chain Expert and Procurement Expert want quantitative properties on supply edges (volume, spend, criticality) in the core spec; the vision document explicitly defers domain-specific fields. The Security & Privacy Expert wants file-level integrity with digital signatures; the Data Format Expert wants canonical JSON (RFC 8785) first. The Open Source Strategy Expert argues for CC-BY-4.0 licensing for specs; the current MIT license applies uniformly. These disagreements are healthy and should be resolved through TSC deliberation rather than unilateral action.

The panel's overall assessment: the spec suite is **ready for implementation prototyping** but **not yet ready for v1.0 declaration**. The P0 issues identified below should be resolved before any conformance claims are made.

## Panel Composition

| Panelist | Role | Key Focus Area |
|----------|------|----------------|
| Supply Chain Expert | Supply Chain Visibility & Risk Analyst | Multi-tier visibility, disruption modeling, data quality |
| Procurement Expert | Chief Procurement Officer | Operational usability, ERP integration, adoption cost |
| Standards Expert | Standards Development & Interoperability Specialist | ISO/GS1 alignment, conformance clauses, governance |
| Systems Engineering Expert | Senior Systems Engineer (Rust) | Parsing safety, WASM, crate architecture, performance |
| Graph Modeling Expert | Graph Data Modeling & Algorithm Specialist | Merge algebra, graph formalism, edge identity |
| Enterprise Integration Expert | Enterprise Systems Architect | ERP export/import, SAP/Oracle/D365, delta updates |
| Regulatory Compliance Expert | Supply Chain Regulatory Compliance Advisor | CSDDD, EUDR, LkSG, attestation, beneficial ownership |
| Data Format Expert | Data Format Architect | Serialization, schema evolution, integrity, compression |
| Open Source Strategy Expert | Open Source Strategy & Governance Lead | TSC charter, licensing, adoption flywheel, community |
| Security & Privacy Expert | Data Security & Privacy Architect | Sensitivity, boundary references, GDPR, cryptographic integrity |
| Entity Identification Expert | Entity Identification & Corporate Hierarchy Specialist | Entity resolution, DUNS/LEI lifecycle, merge identity |

---

## Consensus Findings

Issues independently identified by three or more panelists carry the highest confidence weight.

### 1. No machine-readable schema definition (3 panelists)

**Flagged by:** Data Format Expert, Systems Engineering Expert, Standards Expert

The specification is defined entirely in prose Markdown tables. There is no JSON Schema, no Avro IDL, no formal grammar. Every implementation must hand-translate prose into validation logic, guaranteeing cross-implementation inconsistencies. A JSON Schema (draft 2020-12) should be the normative structural definition, with the Markdown as the human-readable companion.

### 2. Confidence/verification field needed on identifier records (3 panelists)

**Flagged by:** Supply Chain Expert, Entity Identification Expert, Regulatory Compliance Expert

No mechanism exists to distinguish a DUNS number verified against D&B's API from one self-reported on a supplier questionnaire. The `same_as` edge in SPEC-003 has a `confidence` enum, but identifier records in SPEC-002 carry no equivalent. This creates an asymmetry: uncertain identity can be expressed at the edge level but not at the identifier level. Risk-weighted merge, regulatory evidence reporting, and enrichment quality assessment all require this signal.

### 3. No BOM/`composed_of` edge type for material traceability (3 panelists)

**Flagged by:** Enterprise Integration Expert (ERP BOM tables), Regulatory Compliance Expert (EUDR consignment traceability), Supply Chain Expert (disruption propagation)

The `good` node type has no mechanism to express that Good A is composed of Goods B, C, and D. This blocks EUDR derived product traceability, CBAM embedded emissions calculation, UFLPA input origin determination, and manufacturing disruption analysis. ERP systems store this data (SAP STPO/STKO, Oracle BOM_STRUCTURES_B) -- the format must support it.

### 4. Incomplete test vectors in SPEC-004 (3 panelists)

**Flagged by:** Data Format Expert, Systems Engineering Expert, Security & Privacy Expert

The boundary reference hash test vector in SPEC-004 Section 4 describes input construction but omits the expected SHA-256 output hash value. Without the expected output, the test vector is a worked example missing its answer. Independent implementations cannot verify their hashing pipeline. Additional test vectors needed for: nodes with no public identifiers, identifiers requiring percent-encoding, and single-identifier nodes.

### 5. Edge property serialization ambiguity (2 panelists, both implementation-blocking)

**Flagged by:** Systems Engineering Expert, Data Format Expert

SPEC-001 Sections 5-7 define edge properties as flat fields on the edge object, but the serialization example in Section 10 wraps them in a `"properties"` wrapper. This is a breaking ambiguity: `edge.percentage` vs `edge.properties.percentage` are different JSON paths. Every implementation must choose one interpretation; two implementations that choose differently produce incompatible files.

### 6. No delta/patch mechanism for incremental updates (2 panelists)

**Flagged by:** Enterprise Integration Expert, Procurement Expert

Full-file re-export is infeasible for enterprise-scale vendor masters (40K+ vendors). ERP change document tables (SAP CDHDR/CDPOS, Oracle audit columns) produce incremental deltas. The spec has no way to express "add 3 nodes, modify 2 edges, remove 1 node." This is a deployment blocker for production enterprise adoption.

### 7. No file-level integrity mechanism (2 panelists)

**Flagged by:** Security & Privacy Expert, Data Format Expert

No checksums, digital signatures, or MACs. A file can be tampered with in transit -- including modification of `disclosure_scope`, identifiers, or properties -- with no detection by the recipient. For a format designed for inter-organizational exchange of commercially sensitive data, this is a foundational gap.

---

## Critical Issues

All **[Critical]** concerns from any panelist, deduplicated and cross-referenced.

| # | Issue | Flagged By | Impact |
|---|-------|-----------|--------|
| C1 | No machine-readable schema (JSON Schema) | Data Format, Systems Engineering, Standards | Every implementation hand-codes validation; divergence guaranteed |
| C2 | Edge property serialization ambiguity | Systems Engineering, Data Format | Implementation-blocking: flat vs. `"properties"` wrapper |
| C3 | No maximum cardinalities or string length limits | Systems Engineering | DoS vector for any parser processing untrusted input |
| C4 | No file-level integrity mechanism | Security & Privacy, Data Format | Tamper detection impossible across trust boundaries |
| C5 | `nat-reg` default sensitivity `public` exposes sole proprietorships | Security & Privacy | GDPR Article 5(1)(c) data minimization violation by design |
| C6 | Incomplete SPEC-004 test vectors | Data Format, Systems Engineering, Security & Privacy | Boundary reference hashing unverifiable across implementations |
| C7 | No CONTRIBUTING.md or DCO/CLA | Open Source Strategy | Cannot accept external contributions with legal clarity |
| C8 | No conformance test suite | Open Source Strategy, Standards | Conformance claims unverifiable; implementations will diverge |
| C9 | Single-company copyright (BayFX) with MIT for specs | Open Source Strategy | Enterprise adoption signal: "what stops proprietary fork?" |
| C10 | No formal conformance clauses | Standards, Open Source Strategy | "Conformant producer" and "conformant validator" are undefined |
| C11 | No consignment/lot-level traceability | Regulatory Compliance | EUDR Article 9 requires consignment-to-plot linkage |
| C12 | No attestation revocation/lifecycle model | Regulatory Compliance | Cannot distinguish revoked from active mid-validity certifications |
| C13 | No volume/capacity on supply edges | Supply Chain | Risk quantification impossible; $50M sole-source = $500 spot buy |
| C14 | No data confidence/provenance on nodes and edges | Supply Chain | Cannot distinguish "verified by audit" from "self-declared" |
| C15 | No supplier-facing data collection guidance | Procurement | Adoption stalls at buyer-side export, never reaches multi-tier |
| C16 | No cost analysis for identifier enrichment | Procurement | Cannot build business case for enrichment (LEI: $50-200/entity/yr) |
| C17 | Identity predicate has no temporal overlap check | Entity Identification | False merges from reassigned DUNS/GLN identifiers |
| C18 | No delta/patch mechanism | Enterprise Integration | Full re-export infeasible for 40K+ vendor masters |
| C19 | SAP Business Partner model not mapped | Enterprise Integration | SPEC-005 covers only legacy SAP, not greenfield S/4HANA |

---

## Major Issues

| # | Issue | Flagged By |
|---|-------|-----------|
| M1 | No JSON Schema or formal grammar | Data Format, Systems Engineering, Standards |
| M2 | No canonical JSON serialization (RFC 8785) | Data Format |
| M3 | No compression envelope for large files | Data Format |
| M4 | No streaming/chunked format for large graphs | Systems Engineering |
| M5 | Merge conflict record structure undefined | Systems Engineering, Supply Chain |
| M6 | `file_salt` validation rules incomplete | Systems Engineering |
| M7 | Edge merge "property equality" fallback underspecified | Graph Modeling |
| M8 | Post-merge structural validation not specified | Graph Modeling |
| M9 | Boundary references create traversal discontinuities | Graph Modeling |
| M10 | Oracle SCM Cloud and D365 mappings too shallow | Enterprise Integration, Procurement |
| M11 | No BOM/`composed_of` edge type | Enterprise Integration, Regulatory, Supply Chain |
| M12 | No EDI coexistence positioning | Enterprise Integration |
| M13 | `authority` naming convention for `internal` is informal | Enterprise Integration, Procurement |
| M14 | CSDDD downstream "chain of activities" not modeled | Regulatory Compliance |
| M15 | No risk assessment linkage on supply edges | Regulatory Compliance |
| M16 | `disclosure_scope` has no cryptographic binding | Security & Privacy |
| M17 | Merge provenance has no trust domain attribution | Security & Privacy |
| M18 | No transport security guidance | Security & Privacy |
| M19 | Boundary reference stability across re-exports unaddressed | Security & Privacy, Entity Identification |
| M20 | No reference implementation or public roadmap | Open Source Strategy |
| M21 | No adoption wedge identified | Open Source Strategy |
| M22 | No normative ISO 6523 ICD mapping table | Standards |
| M23 | GS1 EPCIS 2.0 relationship unaddressed | Standards |
| M24 | Attestation model does not reference W3C Verifiable Credentials | Standards |
| M25 | ISO 6523 relationship language understated ("informed by" vs "aligns with") | Standards |
| M26 | No n-tier depth or tier labeling | Supply Chain |
| M27 | No risk/criticality scoring on nodes or edges | Supply Chain |
| M28 | Temporal modeling is date-only, no graph versioning | Supply Chain |
| M29 | No procurement-specific relationship metadata | Procurement |
| M30 | No multi-ERP deduplication worked example | Procurement |
| M31 | No confidence/verification on identifier records | Supply Chain, Entity ID, Regulatory |
| M32 | Enrichment can retroactively invalidate prior merges | Entity Identification |
| M33 | Joint ventures and split-identity entities unmodeled | Entity Identification |

---

## Minor Issues

| # | Issue | Flagged By |
|---|-------|-----------|
| m1 | ISO 8601 date format underspecified (YYYY-MM-DD not mandated) | Systems Engineering |
| m2 | `geo` field on facility is unstructured (point vs GeoJSON) | Systems Engineering |
| m3 | No Content-Type / MIME type registration | Systems Engineering, Data Format |
| m4 | Percent-encoding incomplete (newlines not covered) | Systems Engineering |
| m5 | No magic bytes or file signature | Data Format |
| m6 | `snapshot_date` is date-only, not datetime | Data Format |
| m7 | No canonical ordering for merged identifier arrays | Graph Modeling |
| m8 | `boundary_ref` not in SPEC-001 Section 4 taxonomy | Graph Modeling |
| m9 | Cycle legality unclear by subgraph layer | Graph Modeling |
| m10 | N-ary relationship decomposition undocumented | Graph Modeling |
| m11 | STCD1/STCD2 to `vat` scheme mapping oversimplifies | Enterprise Integration |
| m12 | No ERP extraction scheduling guidance | Enterprise Integration |
| m13 | Attestation `scope` is free-text, no controlled vocabulary | Regulatory Compliance |
| m14 | No data recency/freshness indicator | Regulatory Compliance |
| m15 | `tolls` edge direction may confuse bidirectional flows | Supply Chain |
| m16 | `good` node lacks batch/lot granularity | Supply Chain |
| m17 | Commodity volume units not standardized | Supply Chain |
| m18 | No quick-start guide for small suppliers | Procurement, Open Source Strategy |
| m19 | No M&A operational guidance for procurement | Procurement |
| m20 | Scheme vocabulary lacks versioning semantics | Standards |
| m21 | UNTDID 3055 reference imprecise | Standards |
| m22 | No code of conduct | Open Source Strategy |
| m23 | Open questions in a normative spec (SPEC-003) | Open Source Strategy |
| m24 | No name normalization or fuzzy matching guidance | Entity Identification |
| m25 | Cross-scheme validation on same node underspecified | Entity Identification |
| m26 | ISO 5009 missing from standards mapping | Entity Identification |

---

## Consolidated Recommendations

### P0 -- Resolve before any conformance claims

| # | Recommendation | Originated By |
|---|---------------|--------------|
| P0-1 | **Resolve edge property serialization ambiguity.** Choose flat or `"properties"` wrapper; update all tables and examples to be consistent. | Systems Engineering, Data Format |
| P0-2 | **Publish normative JSON Schema (draft 2020-12).** Machine-readable structural definition for `.omts` files. | Data Format, Systems Engineering, Standards |
| P0-3 | **Define maximum cardinalities and string lengths.** Max nodes/edges per file, max identifiers per node, max string length. Contractual upper bounds for parser safety. | Systems Engineering, Data Format |
| P0-4 | **Complete SPEC-004 test vectors.** Add expected SHA-256 output. Add vectors for: no public identifiers, percent-encoded identifiers, single identifier. | Data Format, Systems Engineering, Security & Privacy |
| P0-5 | **Define formal conformance clauses.** What must a conformant producer, consumer, and validator do? Testable obligations per role. | Standards, Open Source Strategy |
| P0-6 | **Create CONTRIBUTING.md with DCO.** Contribution workflow, IP commitment, process for external contributors. | Open Source Strategy |
| P0-7 | **Publish conformance test suite plan with initial test vectors.** One valid + one invalid `.omts` file per L1 rule. Ship as fixtures in the repo. | Open Source Strategy, Standards |
| P0-8 | **Add volume/value/share properties to supply edges.** At minimum on `supplies` and `subcontracts`: `volume` (numeric + unit), `annual_value` (numeric + currency), `share_of_buyer_demand` (0-100%). | Supply Chain |
| P0-9 | **Add confidence/data_quality property to all nodes and edges.** Enum: `verified`, `reported`, `inferred`, `estimated` with optional `source`. Generalize the `same_as` confidence pattern. | Supply Chain, Entity Identification, Regulatory |
| P0-10 | **Change `nat-reg` default sensitivity to `restricted`.** Protects sole proprietorships under GDPR by default. | Security & Privacy |
| P0-11 | **Add consignment-level traceability.** New `consignment` or `lot` node type linking batches to origin facilities and attestations. Required for EUDR DDS compliance. | Regulatory Compliance |
| P0-12 | **Add attestation lifecycle states.** `status` enum: `active`, `suspended`, `revoked`, `expired`, `withdrawn`. Distinct from `valid_to` and `outcome`. | Regulatory Compliance |
| P0-13 | **Extend merge identity predicate with temporal compatibility.** When identifier records carry `valid_from`/`valid_to`, require temporal overlap or open-ended range. Prevent false merges from reassigned DUNS/GLN. | Entity Identification, Graph Modeling |

### P1 -- Resolve before v1.0

| # | Recommendation | Originated By |
|---|---------------|--------------|
| P1-1 | Mandate `YYYY-MM-DD` as sole ISO 8601 date profile | Systems Engineering |
| P1-2 | Define merge conflict record schema | Systems Engineering, Supply Chain |
| P1-3 | Specify `file_salt` validation precisely (`^[0-9a-f]{64}$`) | Systems Engineering |
| P1-4 | Publish JSON Schema for SPEC-001 (also enables VS Code validation) | Data Format, Open Source Strategy |
| P1-5 | Adopt RFC 8785 (JCS) for canonical JSON serialization | Data Format |
| P1-6 | Define file integrity mechanism (SHA-256 digest + optional signatures) | Security & Privacy, Data Format |
| P1-7 | Add magic bytes (first key MUST be `"omtsf_version"`) | Data Format |
| P1-8 | Define compression envelope (`.omts.zst`) | Data Format |
| P1-9 | Require post-merge structural validation with failure semantics | Graph Modeling |
| P1-10 | Define reachability semantics for `boundary_ref` nodes | Graph Modeling |
| P1-11 | Mandate canonical ordering for identifier arrays after merge | Graph Modeling |
| P1-12 | Explicitly state cycle legality by subgraph layer | Graph Modeling |
| P1-13 | Enumerate per-edge-type merge-identity properties | Graph Modeling |
| P1-14 | Add `composed_of` edge type for BOM decomposition | Enterprise Integration, Regulatory, Supply Chain |
| P1-15 | Expand Oracle SCM Cloud and D365 mappings to API level | Enterprise Integration, Procurement |
| P1-16 | Add SAP Business Partner model mapping (BUT000/BUT0ID) | Enterprise Integration, Procurement |
| P1-17 | Publish EDI coexistence statement | Enterprise Integration |
| P1-18 | Formalize `authority` naming convention for `internal` | Enterprise Integration, Procurement |
| P1-19 | Add downstream supply chain edge type (`sells_to`) | Regulatory Compliance |
| P1-20 | Introduce controlled vocabulary for attestation scope | Regulatory Compliance |
| P1-21 | Add transport security guidance | Security & Privacy |
| P1-22 | Add provenance authentication to merge metadata | Security & Privacy |
| P1-23 | Add normative ISO 6523 ICD mapping table | Standards |
| P1-24 | Add EPCIS 2.0 relationship statement to SPEC-006 | Standards |
| P1-25 | Add W3C Verifiable Credentials alignment for attestations | Standards |
| P1-26 | Upgrade ISO 6523 relationship language to "aligns with" | Standards |
| P1-27 | Define conformance clauses for producer/consumer/validator | Standards, Open Source Strategy |
| P1-28 | Name the adoption wedge (recommended: LkSG + German automotive) | Open Source Strategy |
| P1-29 | Publish reference implementation roadmap | Open Source Strategy |
| P1-30 | Resolve SPEC-003 open questions before draft-final | Open Source Strategy |
| P1-31 | Add confidence/verification fields to identifier records | Entity Identification, Supply Chain, Regulatory |
| P1-32 | Add enrichment-merge interaction guidance | Entity Identification |
| P1-33 | Add joint venture representation (`governance_structure`) | Entity Identification |
| P1-34 | Document boundary reference stability constraints | Entity Identification, Security & Privacy |
| P1-35 | Define a `consumes`/`requires_input` edge for good-to-good relationships | Supply Chain |
| P1-36 | Add tier/relationship_basis annotation on supply edges | Supply Chain |
| P1-37 | Publish cost-ordered identifier enrichment path | Procurement |
| P1-38 | Create supplier data collection guide | Procurement |
| P1-39 | Publish multi-ERP deduplication worked example | Procurement |
| P1-40 | Create one-page minimum viable file quick-start | Procurement, Open Source Strategy |

### P2 -- Future improvements

| # | Recommendation | Originated By |
|---|---------------|--------------|
| P2-1 | Define streaming variant (NDJSON) for large files | Systems Engineering |
| P2-2 | Register MIME type `application/vnd.omtsf+json` | Systems Engineering, Data Format |
| P2-3 | Extend percent-encoding to cover newlines | Systems Engineering |
| P2-4 | Advisory size limits (1M nodes, 50 identifiers/node) | Data Format |
| P2-5 | Upgrade `snapshot_date` to datetime | Data Format |
| P2-6 | Document n-ary relationship decomposition pattern | Graph Modeling |
| P2-7 | Add `traversal_boundary` boolean on `boundary_ref` nodes | Graph Modeling |
| P2-8 | STCD1/STCD2 disambiguation guidance in SPEC-005 | Enterprise Integration |
| P2-9 | ERP extraction scheduling guidance | Enterprise Integration |
| P2-10 | Data verification timestamp (`last_verified`) on nodes | Regulatory Compliance |
| P2-11 | Risk assessment linkage on supply edges | Regulatory Compliance |
| P2-12 | Stable boundary reference mode (opt-in persistent salt) | Security & Privacy |
| P2-13 | Sensitivity model for edge properties | Security & Privacy |
| P2-14 | `scheme_vocabulary_version` field or versioning policy | Standards |
| P2-15 | Correct UNTDID 3055 reference | Standards |
| P2-16 | GS1 EPC URI conversion detail (Company Prefix limitation) | Standards |
| P2-17 | Version the TSC charter | Standards |
| P2-18 | Evaluate foundation hosting (LF, OASIS, Eclipse) | Open Source Strategy |
| P2-19 | Plan language bindings (Python via PyO3, TypeScript via WASM) | Open Source Strategy |
| P2-20 | Community extension scheme registry | Open Source Strategy |
| P2-21 | Separate spec licensing (CC-BY-4.0) from code (Apache 2.0) | Open Source Strategy |
| P2-22 | Define snapshot sequencing for temporal graph analysis | Supply Chain |
| P2-23 | Risk score extension pattern (`com.omtsf.risk.score`) | Supply Chain |
| P2-24 | Procurement extension namespace (`com.omtsf.procurement`) | Procurement |
| P2-25 | Formalize `authority` naming in L2 validation | Procurement |
| P2-26 | M&A operational guidance for procurement | Procurement |
| P2-27 | Delta/patch update envelope specification | Enterprise Integration, Procurement |
| P2-28 | Name normalization guidance for fuzzy dedup | Entity Identification |
| P2-29 | Add ISO 5009 to standards mapping | Entity Identification |
| P2-30 | Reference OpenCorporates as enrichment source | Entity Identification |

---

## Cross-Domain Interactions

These are points where one expert's recommendations directly affect another's domain -- often the most valuable insights from a multi-expert review.

### 1. Temporal Identity Predicate x Merge Algebra

**Entity Identification Expert + Graph Modeling Expert** -- Transitive closure across identifier schemes can link entities that should not be linked when identifier reassignment has occurred. Both recommend extending the identity predicate with temporal overlap checking. The key constraint: temporal compatibility must itself be transitive for the union-find to remain valid. They offer to co-define a formal "temporally compatible identity predicate."

### 2. BOM Decomposition x Consignment Attestation x Disruption Analysis

**Enterprise Integration Expert + Regulatory Compliance Expert + Supply Chain Expert** -- Three experts converge on the same gap from different angles. Enterprise Integration needs `composed_of` edges for ERP BOM structures. Regulatory Compliance needs consignment-level attestation for EUDR. Supply Chain needs lot-level `good` nodes and input/output linkage for disruption propagation. Together these define the full material traceability chain: raw material (lot) -> components (BOM) -> finished product, with attestations at each level.

### 3. Schema Definition x Conformance Testing x Ecosystem Enablement

**Data Format Expert + Systems Engineering Expert + Open Source Strategy Expert** -- JSON Schema serves three purposes simultaneously: it is the normative structural definition (Data Format), it enables code generation via `schemars` for Rust (Systems Engineering), and it enables VS Code validation and third-party tooling without the reference implementation (Open Source Strategy). This is the single highest-leverage artifact for the ecosystem.

### 4. Edge Property Ambiguity x Graph Databases x Serialization

**Systems Engineering Expert + Data Format Expert + Graph Modeling Expert** -- The `"properties"` wrapper issue affects all three domains. For Rust, it determines `serde` struct design. For graph databases, it affects import/export compatibility. For the graph model, it determines whether structural fields (`id`, `type`, `source`, `target`) are cleanly separated from domain properties. All three recommend resolution before any implementation work.

### 5. Confidence Field x Merge Quality x Regulatory Reporting

**Supply Chain Expert + Entity Identification Expert + Regulatory Compliance Expert** -- The confidence/verification field serves different purposes for each: Supply Chain needs it for risk-weighted analysis, Entity Identification for merge quality assessment, Regulatory Compliance for regulatory evidence strength. A unified confidence metadata model applicable to identifiers, edges, and nodes would satisfy all three.

### 6. Governance Scope x Merge Stability

**Open Source Strategy Expert + Graph Modeling Expert** -- Governance authority should extend beyond the scheme registry to cover merge semantics as a stability-critical component. If a future spec version changes the edge identity predicate or transitive closure behavior, previously merged datasets become inconsistent. The TSC charter should treat SPEC-003 as requiring major version increments for breaking changes.

### 7. Privacy x Graph Topology Inference

**Security & Privacy Expert + Regulatory Compliance Expert** -- Ownership edge chains in public-scope files can reveal UBO-adjacent information even when `person` nodes are stripped. Chains terminating at high-percentage ownership nodes strongly imply redacted natural persons. This is an inherent limitation of graph-based selective disclosure that should be documented.

### 8. Boundary References x Mergeability Trade-off

**Security & Privacy Expert + Entity Identification Expert** -- Fresh salt per file makes boundary references un-correlatable across files. Redacted subgraphs are inherently un-mergeable for entities lacking public identifiers. Both agree this is by design but should be explicitly documented as a trade-off: privacy vs. merge completeness.

### 9. Delta Updates x Security Sensitivity

**Enterprise Integration Expert + Security & Privacy Expert** -- Delta files revealing "3 new Chinese suppliers added this week" are more intelligence-dense than a 40K-node snapshot. Any delta specification must inherit `disclosure_scope` constraints. Delta files should default to `restricted` sensitivity.

### 10. Downstream Supply Chains x ERP Sales Modules

**Regulatory Compliance Expert + Enterprise Integration Expert** -- CSDDD's downstream due diligence obligation means the spec will eventually need mappings from ERP sales/distribution modules (SAP SD, Oracle Order Management), not just procurement modules. Currently the spec is upstream-only.

### 11. SAP Tax Fields x Identifier Scheme Disambiguation

**Standards Expert + Enterprise Integration Expert** -- SAP's `STCD1`/`STCD2` fields store various tax identifiers depending on country (VAT, EIN, CNPJ), not exclusively VAT numbers. Scheme assignment (`vat` vs `nat-reg`) depends on the identifier type, not the field name. The `BUT0ID` table handles this more cleanly.

### 12. Adoption Strategy x Reference Implementation

**Procurement Expert + Open Source Strategy Expert** -- The single most impactful ecosystem deliverable would be a reference SAP S/4HANA extractor producing valid `.omts` files. Both recommend prioritizing this in the open source roadmap. The adoption wedge (LkSG + German automotive) leverages SAP's >70% market share in German enterprise procurement.

---

## Individual Expert Reports

Full individual reviews are available in:

| Expert | Review File |
|--------|------------|
| Supply Chain Expert | (inline in panel synthesis, extracted from agent output) |
| Procurement Expert | `docs/reviews/spec-suite-review-procurement-expert.md` |
| Standards Expert | (inline in panel synthesis, extracted from agent output) |
| Systems Engineering Expert | `docs/reviews/spec-suite-review-systems-engineer.md` |
| Graph Modeling Expert | (inline in panel synthesis, extracted from agent output) |
| Enterprise Integration Expert | `docs/reviews/spec-suite-review-enterprise-integration.md` |
| Regulatory Compliance Expert | (inline in panel synthesis, extracted from agent output) |
| Data Format Expert | `docs/reviews/spec-suite-review-data-format.md` |
| Open Source Strategy Expert | `docs/reviews/spec-suite-review-open-source-strategist.md` |
| Security & Privacy Expert | `docs/reviews/spec-suite-review-security-privacy.md` |
| Entity Identification Expert | `docs/reviews/full-suite-review-entity-identification-expert.md` |

### Supply Chain Expert -- Summary

**Overall Verdict:** Core architecture is sound. Supply edge taxonomy with regulatory annotations is strong. No new Critical design flaws vs. R2 panel.

**Key Concerns:**
- **[Critical]** No volume/capacity/quantity on supply edges -- blocks disruption modeling
- **[Critical]** No data confidence/provenance metadata on nodes/edges
- **[Major]** No n-tier depth labeling; no risk/criticality scoring; temporal modeling is date-only
- **[Minor]** `tolls` direction confusing; `good` lacks batch/lot; no commodity units

**Top Recommendations:** P0: add volume/value/share to supply edges, add confidence field. P1: `consumes`/`requires_input` edge for BOM, tier annotation on supply edges, data completeness metadata. P2: snapshot sequencing, risk score extension pattern.

---

### Procurement Expert -- Summary

**Overall Verdict:** Internal identifiers as first-class and tiered validation are the right design. ERP mappings are actionable. Excellent as a data model but not yet a practical implementation guide.

**Key Concerns:**
- **[Critical]** No supplier-facing data collection guidance; no cost analysis for enrichment
- **[Major]** No procurement-specific relationship metadata; Oracle/D365 too shallow; no delta model; SAP BP missing; no multi-ERP dedup example
- **[Minor]** Informal `authority` naming; no M&A guidance; no quick-start; six specs with no entry point

**Top Recommendations:** P0: cost-ordered enrichment path, supplier data collection guide. P1: procurement extension namespace, expanded ERP mappings, SAP BP mapping, multi-ERP dedup example, quick-start guide. P2: delta/patch, `authority` formalization.

---

### Standards Expert -- Summary

**Overall Verdict:** Standards alignment is credible. Governance process well-implemented. Ready for formal review period pending conformance clause gaps.

**Key Concerns:**
- **[Critical]** No formal conformance clauses for Producer/Consumer/Validator
- **[Major]** No normative ISO 6523 ICD mapping; EPCIS 2.0 unaddressed; W3C VC not referenced; ISO 6523 language understated
- **[Minor]** Scheme vocabulary lacks versioning; UNTDID 3055 imprecise; TSC charter not versioned

**Top Recommendations:** P0: conformance clauses. P1: ISO 6523 ICD mapping table, EPCIS statement, VC alignment, language upgrade. P2: UNTDID correction, GS1 EPC URI detail, charter versioning.

---

### Systems Engineering Expert -- Summary

**Overall Verdict:** Clean decomposition maps to Rust workspace. Flat serialization is streaming-friendly. Merge algebra correct. Boundary reference hashing cryptographically sound.

**Key Concerns:**
- **[Critical]** No max cardinalities/string lengths (DoS); edge property ambiguity (flat vs. wrapper)
- **[Major]** No JSON Schema; no streaming format; unstructured merge conflicts; incomplete `file_salt` validation
- **[Minor]** ISO 8601 underspecified; `geo` unstructured; no MIME type; incomplete percent-encoding

**Top Recommendations:** P0: resolve property nesting, define max limits. P1: JSON Schema, YYYY-MM-DD mandate, conflict schema, salt validation, concrete hash in test vector. P2: NDJSON streaming, MIME type, newline encoding.

---

### Graph Modeling Expert -- Summary

**Overall Verdict:** Merge algebra formally correct. Union-find optimal. Multigraph model with independent edge identity essential and well-designed.

**Key Concerns:**
- **[Major]** Edge merge "property equality" underspecified; post-merge validation unspecified; boundary refs create traversal discontinuities
- **[Minor]** No canonical identifier array ordering; `boundary_ref` taxonomy gap; cycle legality unclear; n-ary undocumented

**Top Recommendations:** P0: enumerate per-edge-type merge-identity properties. P1: post-merge validation, reachability semantics for boundary_ref, canonical identifier ordering, cycle legality statement. P2: n-ary decomposition pattern.

---

### Enterprise Integration Expert -- Summary

**Overall Verdict:** SAP field mappings domain-accurate. Spec implementable for pilot SAP integration. Production deployment blocked by delta gap and SAP BP omission.

**Key Concerns:**
- **[Critical]** No delta/patch mechanism; SAP Business Partner model not mapped
- **[Major]** Oracle/D365 too shallow; no BOM edge; no EDI positioning; `authority` informal
- **[Minor]** STCD1/STCD2 oversimplified; no extraction scheduling guidance

**Top Recommendations:** P0: delta/patch envelope, SAP BP mapping. P1: API-level Oracle/D365, `composed_of` edge, EDI statement, `authority` formalization. P2: STCD disambiguation, scheduling guidance.

---

### Regulatory Compliance Expert -- Summary

**Overall Verdict:** Beneficial ownership and attestation models well-designed. GDPR/AMLD privacy tension handled correctly. Regulatory alignment table credible.

**Key Concerns:**
- **[Critical]** No consignment/lot-level traceability (EUDR Article 9); no attestation revocation
- **[Major]** No BOM for material traceability; no downstream edges (CSDDD); no risk assessment linkage
- **[Minor]** Attestation scope is free-text; no data freshness indicator

**Top Recommendations:** P0: consignment traceability, attestation lifecycle states. P1: `composed_of` edge, downstream edge type, attestation scope vocabulary. P2: verification timestamp, risk linkage.

---

### Data Format Expert -- Summary

**Overall Verdict:** JSON is defensible for v0.1 adoption. Flat adjacency list is merge-friendly. Critical gaps in format engineering: no schema, no integrity, no canonical form.

**Key Concerns:**
- **[Critical]** No JSON Schema; no file integrity; incomplete test vectors
- **[Major]** No canonical JSON (RFC 8785); edge property inconsistency; no compression; no size limits
- **[Minor]** No magic bytes; no MIME type; `snapshot_date` date-only

**Top Recommendations:** P0: JSON Schema, complete test vectors, resolve property ambiguity. P1: RFC 8785, integrity mechanism, magic bytes, compression envelope. P2: size limits, MIME type, datetime upgrade.

---

### Open Source Strategy Expert -- Summary

**Overall Verdict:** Governance progress is substantial. TSC Charter addresses prior critical finding. Scaffolding exists but is not yet load-bearing.

**Key Concerns:**
- **[Critical]** No CONTRIBUTING.md/DCO; no conformance test suite; single-company copyright + MIT for specs
- **[Major]** No reference implementation; no conformance clauses; no adoption wedge; no code of conduct
- **[Minor]** No RFC process for spec changes; open questions in normative spec

**Top Recommendations:** P0: CONTRIBUTING.md + DCO, separate spec/code licensing, conformance test suite plan. P1: conformance clauses, adoption wedge, reference impl roadmap, resolve open questions. P2: foundation hosting, language bindings.

---

### Security & Privacy Expert -- Summary

**Overall Verdict:** Privacy-by-design foundations are strong. Boundary reference hashing cryptographically sound. Disclosure scope enforcement at L1 correct.

**Key Concerns:**
- **[Critical]** No file-level integrity; `nat-reg` default `public` exposes sole proprietorships
- **[Major]** `disclosure_scope` no cryptographic binding; merge provenance no trust domain; no transport guidance
- **[Minor]** Boundary references unstable across re-exports; incomplete test vectors

**Top Recommendations:** P0: file integrity spec, change `nat-reg` default to `restricted`. P1: transport guidance, complete test vectors, provenance authentication. P2: stable boundary reference mode, edge property sensitivity.

---

### Entity Identification Expert -- Summary

**Overall Verdict:** Foundational entity identification architecture is correct. LEI lifecycle handling is production-grade. DUNS Family Tree mapping operationally precise. Cross-spec integration reveals gaps not visible in individual reviews.

**Key Concerns:**
- **[Critical]** Identity predicate has no temporal overlap check (false merge risk from DUNS/GLN reassignment)
- **[Major]** No confidence on identifiers (asymmetry with `same_as`); enrichment can invalidate merges; joint ventures unmodeled; boundary reference stability fragile
- **[Minor]** No name normalization; cross-scheme validation underspecified; ISO 5009 missing

**Top Recommendations:** P0: temporal compatibility in merge predicate. P1: confidence/verification fields, enrichment-merge interaction guidance, joint venture representation, boundary reference stability documentation. P2: name normalization, ISO 5009 reference.
