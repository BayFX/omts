# Expert Panel Report: Use Case Validity Review

**Date:** 2026-02-25
**Topic:** Evaluate whether the six documented OMTS use cases are valid, realistic, and well-structured
**Documents Reviewed:** `usecases/README.md`, SPEC-001 through SPEC-007

---

## Panel Chair Summary

Seven domain experts independently reviewed the six OMTS use cases for practical feasibility, completeness, regulatory accuracy, and alignment with the specification. The consensus is that the use cases are **strategically well-chosen** — they cover the core regulatory drivers (EUDR, LkSG/CSDDD, CBAM, AMLD) and the two most common operational pain points (fragmented ERP data, sensitive data sharing). The graph data model, composite identifier system, and selective disclosure mechanism are all praised as technically sound and well-aligned with real-world requirements.

However, the panel converged on a significant gap: **the use cases describe end states, not journeys**. They assume the data already exists in clean, enrichable form. In reality, the hardest problems are upstream: collecting geolocation data from informal cooperatives (EUDR), obtaining installation-level emissions from third-country operators (CBAM), enriching ERP vendor masters that have <40% external identifier coverage (Multi-ERP), and accessing beneficial ownership registries that are suspended or legally restricted in multiple EU member states. Five of seven experts flagged some variant of this "data sourcing gap" as a critical or major concern.

A second area of consensus involves **spec-to-use-case inconsistencies**: the `produces` edge documentation references only `good` nodes while the type constraint table also permits `consignment` targets (affecting UC5/CBAM), and UC4 references `share_percent` instead of the spec's `percentage` property name. The LkSG/CSDDD use case conflates two regulations with materially different scope and timeline — a distinction that multiple experts (Supply Chain, Regulatory, Procurement) independently flagged.

The panel identified two issues not represented in any use case: **disruption analysis** (leveraging `share_of_buyer_demand`, `sole_source`, `lead_time_days` for "what if node X fails?" queries) and **temporal comparison** (using `previous_snapshot_ref` and `snapshot_sequence` to track supply chain changes over time). Both are high-value applications that would strengthen the case for graph-based supply chain modeling.

---

## Panel Composition

| Panelist | Role | Key Focus Area |
|----------|------|----------------|
| Supply Chain Expert | Supply Chain Visibility & Risk Analyst | Multi-tier mapping realism, disruption analysis, regulatory data requirements |
| Procurement Expert | Chief Procurement Officer | Operational usability, supplier burden, ERP integration, adoption cost |
| Regulatory Compliance Expert | Supply Chain Regulatory Compliance Advisor | Regulatory accuracy, data requirements, cross-jurisdictional compliance |
| Enterprise Integration Expert | Enterprise Systems Architect | ERP export feasibility, master data quality, delta updates, EDI coexistence |
| Security & Privacy Expert | Data Security & Privacy Architect | Selective disclosure, boundary reference attacks, GDPR, file integrity |
| Entity Identification Expert | Entity Identification & Corporate Hierarchy Specialist | Identifier coverage, entity resolution, corporate hierarchy, informal entities |
| Graph Modeling Expert | Graph Data Modeling & Algorithm Specialist | Graph structure soundness, merge algebra, cycle constraints, type safety |

---

## Consensus Findings

The following issues were independently identified by 3+ experts, giving them the highest confidence:

1. **Use cases describe end states, not data collection workflows** (Supply Chain, Procurement, ERP Integration, Entity Identification). UC1/UC2/UC5 assume the buyer already has geo coordinates, sub-supplier identifiers, and installation-level emissions data. The hard part — collecting this from suppliers — is unaddressed.

2. **LkSG and CSDDD should be separated or distinguished** (Supply Chain, Regulatory, Procurement). LkSG requires tier 2 investigation only upon "substantiated knowledge" of violations; CSDDD requires systematic multi-tier mapping. Grouping them without distinction is misleading.

3. **Identifier coverage in production ERPs is insufficient for immediate merge** (Procurement, ERP Integration, Entity Identification). Fewer than 40% of SAP vendor master records carry any external identifier; Oracle and D365 are worse. UC3 needs a data enrichment prerequisite.

4. **`produces` edge documentation inconsistency** (Supply Chain, Graph Modeling). SPEC-001 Section 6.7 describes `produces` as facility→good, but the type constraint table (Section 9.5) also permits facility→consignment. The CBAM use case (UC5) depends on the latter.

5. **Graph-structural re-identification risk for boundary_ref nodes** (Security & Privacy, ERP Integration, Graph Modeling). Degree sequences, edge types, and commodity codes on edges connected to boundary_ref nodes create quasi-identifier fingerprints that can enable de-anonymization, especially with auxiliary knowledge.

6. **No disruption analysis use case** (Supply Chain). The spec's `share_of_buyer_demand`, `sole_source`, and `lead_time_days` properties were designed for "what if this node fails?" analysis, yet no use case demonstrates this — arguably the strongest differentiator for a graph-based approach.

---

## Critical Issues

| # | Issue | Flagged By | Summary |
|---|-------|-----------|---------|
| C1 | **Supplier data collection workflow missing** | Procurement, Supply Chain | UC1/UC2/UC5 lack any description of how data originating outside the buyer's ERP (geo coordinates, sub-supplier IDs, emissions data) gets into the `.omts` file. Without this, the use cases are aspirational, not actionable. |
| C2 | **Informal entity identification gap (EUDR)** | Entity Identification | Cocoa cooperatives, smallholder plantations, and artisanal operations often have no formal registration, no LEI, no DUNS. `internal` identifiers cannot trigger cross-file merge. No guidance exists for handling informally registered entities that dominate EUDR-relevant upstream supply chains. |
| C3 | **Beneficial ownership registry unreliability** | Entity Identification | As of early 2026, multiple EU member states have suspended or restricted public access to UBO registers. Italy, Slovakia, Czechia, and 11 others face infringement proceedings. UC4 presents a clean graph but understates the unreliability of the underlying data. |
| C4 | **Graph-structural re-identification of boundary_ref nodes** | Security & Privacy | SPEC-004 protects against identifier enumeration but does not address structural de-anonymization attacks. Edge types, commodity codes, and degree sequences connected to boundary_ref nodes create fingerprints exploitable with auxiliary knowledge. No threat model section exists. |
| C5 | **No file-level digital signature mechanism** | Security & Privacy | Content hashes provide tamper detection but not authentication. An adversary can modify a file and recompute the hash. For UC4 (UBO evidence) and UC1 (DDS), file-level signatures are essential for evidentiary integrity. |
| C6 | **ERP identifier coverage gaps make UC3 infeasible without enrichment** | ERP Integration, Procurement | SAP BUT0ID has <40% external identifier population; Oracle DUNSNumber ~20%; D365 even sparser. If most nodes carry only `internal` identifiers, the merge engine has nothing to merge on. |
| C7 | **Delta/patch mechanism absent for operational UC3** | ERP Integration | A conglomerate with 40,000+ vendors across three ERPs cannot regenerate complete `.omts` files on every change. Without delta support, UC3 is a one-time migration, not a sustainable process. |

---

## Major Issues

| # | Issue | Flagged By | Summary |
|---|-------|-----------|---------|
| M1 | LkSG/CSDDD conflated without distinguishing scope, timeline, or tier requirements | Supply Chain, Regulatory, Procurement | LkSG is in force now (with reporting removed); CSDDD transposition pushed to July 2028. Different scope thresholds, enforcement mechanisms. |
| M2 | `produces` edge text says `good` only; type constraint table also permits `consignment` | Supply Chain, Graph Modeling | Spec inconsistency affecting UC5 (CBAM). |
| M3 | UC4 references `share_percent`; spec defines the property as `percentage` | Procurement | Documentation error. |
| M4 | EUDR DDS not a submission format — OMTS relationship to TRACES unclear | Regulatory | UC1 should clarify OMTS is pre-submission data organization, not a regulatory filing format. |
| M5 | Missing CN-code granularity for EUDR | Regulatory | EUDR requires 8-digit CN codes; UC1 references 4-digit HS heading 1801. |
| M6 | No supplier onboarding use case | Procurement | A supplier submitting an `.omts` file during qualification would demonstrate value at data entry, not just consolidation. |
| M7 | Name normalization and fuzzy matching not addressed in UC3 | ERP Integration | The same entity appears as "Acme GmbH" / "ACME Manufacturing GmbH" / "Acme Mfg GmbH" across ERPs. Identifier-based merge alone is insufficient. |
| M8 | `tier` property lacks formal graph-distance validation | Graph Modeling | No L2 rule checks whether `tier` values are consistent with actual graph distance from `reporting_entity`. |
| M9 | Merge transitive closure has no type-homogeneity guard | Graph Modeling | A `facility` and `organization` sharing a DUNS (branch DUNS scenario) could be merged into a type-inconsistent node. |
| M10 | Person node deletion from public files leaves observable structural gaps | Security & Privacy | Organization nodes that had `beneficial_ownership` edges appear in public files without them, enabling inference of UBO data existence. |
| M11 | No salt lifecycle or retention guidance | Security & Privacy | A salt leak combined with knowledge of public identifiers enables retrospective de-hashing of all boundary_ref nodes. |
| M12 | CBAM product-level specificity missing | Regulatory | CBAM covers specific CN-code product categories; UC5 is generic. |
| M13 | UC2 depends on UC3 (vendor master consolidation) but does not acknowledge this | ERP Integration | Multi-tier mapping requires consolidated vendor masters first. |
| M14 | No cross-scheme identifier conflict resolution specified | Entity Identification | When a node's LEI and nat-reg identifiers conflict with GLEIF Level 1 data, behavior is undefined beyond "recommend manual review." |
| M15 | CBAM installation-level data sourcing challenge unaddressed | ERP Integration | Emissions data comes from installation operators, not ERPs. The data collection bottleneck is not acknowledged. |

---

## Minor Issues

| # | Issue | Flagged By |
|---|-------|-----------|
| m1 | UC1 does not model multi-hop commodity chains (intermediary traders) | Supply Chain |
| m2 | No temporal comparison use case (previous_snapshot_ref, snapshot_sequence) | Supply Chain |
| m3 | CBAM default value fallback path not demonstrated | Supply Chain, Regulatory |
| m4 | No round-trip import workflow for UC3 (export only) | Procurement |
| m5 | UC1 geo coordinate precision not specified (4-hectare polygon threshold) | ERP Integration |
| m6 | UC4 lacks control-test modeling (non-ownership UBO indicators) | Regulatory |
| m7 | `same_as` edge is semantically undirected but stored in directed structure | Graph Modeling |
| m8 | Ownership percentage sum rule is L3 only; should be L2 | Graph Modeling |
| m9 | No minimum entropy requirement on file_salt | Security & Privacy |
| m10 | Boundary reference hash lacks domain separation prefix | Security & Privacy |
| m11 | `_property_sensitivity` metadata is itself informative to attackers | Security & Privacy |
| m12 | VAT normalization rule (`vat:DE:DE123456789`) is counterintuitive | Entity Identification |
| m13 | CBAM installation ID is extension scheme, not core | Entity Identification |
| m14 | Regulatory timeline context missing (EUDR postponed, CSDDD delayed) | Regulatory |

---

## Consolidated Recommendations

### P0 — Immediate

| # | Recommendation | Origin |
|---|---------------|--------|
| P0-1 | **Add a "Supplier Data Collection" use case** showing the buyer→supplier→buyer round-trip for populating `.omts` files with data that originates outside the buyer's ERP (geo coordinates, sub-supplier IDs, emissions) | Procurement, Supply Chain |
| P0-2 | **Define an identity bridge pattern for informal entities** (cooperatives, smallholders) lacking formal registration. Options: informal registration schemes, geo-based identity supplements, or explicit `same_as` guidance | Entity Identification |
| P0-3 | **Fix `produces` edge documentation** to explicitly mention `consignment` as a permitted target type, matching the type constraint table | Supply Chain, Graph Modeling |
| P0-4 | **Separate LkSG and CSDDD** in UC2 — distinguish event-triggered tier 2 (LkSG) from systematic multi-tier mapping (CSDDD) | Supply Chain, Regulatory |
| P0-5 | **Add a structural re-identification risk section to SPEC-004** documenting graph de-anonymization threats and recommending mitigations (edge generalization, degree-sequence padding) | Security & Privacy |
| P0-6 | **Define a file-level digital signature mechanism** in SPEC-007 (e.g., JWS detached payload or COSE Sign1 for CBOR) covering content hash, file_salt, and disclosure_scope | Security & Privacy |
| P0-7 | **Add identifier coverage and data quality prerequisites to UC3** with realistic coverage rates (DUNS ~20-30%, VAT ~40-60%, LEI <5%) and the enrichment workflow as a mandatory prerequisite | ERP Integration |
| P0-8 | **Clarify OMTS is not a regulatory submission format** — add a note to UC1/UC5 that OMTS organizes pre-submission data; actual filing requires TRACES (EUDR) or CBAM registry | Regulatory |
| P0-9 | **Add verification_status guidance for beneficial ownership data** acknowledging EU UBO registry unreliability and recommending `verification_status` on `beneficial_ownership` edges | Entity Identification |
| P0-10 | **Add type-homogeneity guard to merge** — merge candidates MUST have the same node type; cross-type identifier matches should produce an error or prominent warning | Graph Modeling |

### P1 — Before v1

| # | Recommendation | Origin |
|---|---------------|--------|
| P1-1 | **Add a disruption analysis use case** demonstrating "node failure impact" queries using `share_of_buyer_demand`, `sole_source`, `lead_time_days` | Supply Chain |
| P1-2 | **Add a temporal comparison use case** demonstrating `previous_snapshot_ref` and `snapshot_sequence` for detecting supply chain changes | Supply Chain |
| P1-3 | **Add a supplier onboarding use case** — new supplier submits `.omts` as qualification package | Procurement |
| P1-4 | **Fix UC4 `share_percent` → `percentage`** property name | Procurement |
| P1-5 | **Update UC1 for EUDR amendments** — December 2026 postponement, simplified downstream trader obligations, CN-code granularity (8-digit, not 4-digit HS) | Regulatory, Procurement |
| P1-6 | **Specify CBAM product categories** in UC5 — reference Annex I CN codes, demonstrate annual declaration workflow | Regulatory |
| P1-7 | **Add consignment-level traceability to UC1** — show `consignment` nodes with `production_date` linked via `produces`/`composed_of` for EUDR chain of custody | Regulatory |
| P1-8 | **Prioritize delta/patch specification** for sustainable UC3 operations — reference SAP CDHDR/CDPOS, Oracle LAST_UPDATE_DATE, D365 change tracking | ERP Integration |
| P1-9 | **Expand UC3 for name normalization** — position `same_as` edges as bridge between MDM fuzzy matching tools and OMTS merge engine | ERP Integration |
| P1-10 | **Add L2 rule for `tier` consistency** checking values against graph distance from `reporting_entity` | Graph Modeling |
| P1-11 | **Clarify `produces` edge for `consignment` targets** in Section 6.7 text | Graph Modeling |
| P1-12 | **Add salt lifecycle guidance** — treat `file_salt` as confidential, define retention posture | Security & Privacy |
| P1-13 | **Add domain separation to boundary reference hash** — prefix with `"omts:boundary-ref:v1\0"` before v1 freeze | Security & Privacy |
| P1-14 | **Document identity degradation at depth** — acknowledge identifier coverage decreases with tier depth in UC2 | Entity Identification |
| P1-15 | **Introduce match-confidence annotation on merge results** — record number/types of matching identifiers | Entity Identification |
| P1-16 | **Add explicit dependency between UC2 and UC3** — multi-tier mapping requires consolidated vendor master | ERP Integration |
| P1-17 | **Promote L3-MRG-01 (ownership sum ≤ 100%) to L2** — pure structural check, no external data needed | Graph Modeling |
| P1-18 | **Add structural gap mitigation guidance for person node omission** in public files | Security & Privacy |

### P2 — Future

| # | Recommendation | Origin |
|---|---------------|--------|
| P2-1 | Expand EUDR use case with multi-hop commodity chains using `brokers`/`distributes` | Supply Chain |
| P2-2 | Demonstrate mixed-quality emissions data in CBAM use case | Supply Chain, Regulatory |
| P2-3 | Add round-trip ERP import example to UC3 | Procurement |
| P2-4 | Add geo coordinate precision guidance referencing EUDR 4-hectare threshold | ERP Integration |
| P2-5 | Add re-identification risk acknowledgment to UC6 | ERP Integration |
| P2-6 | Add control-test example (non-ownership UBO indicators) to UC4 | Regulatory |
| P2-7 | Add regulatory timeline context notes | Regulatory |
| P2-8 | Add `same_as` normalization convention (source < target lexicographically) | Graph Modeling |
| P2-9 | Consider promoting `com.omts.arrangement` to core node type | Graph Modeling |
| P2-10 | Promote `org.eu.cbam-installation` to core identifier scheme | Entity Identification |
| P2-11 | Simplify VAT normalization (derive prefix from authority) | Entity Identification |
| P2-12 | Add minimum entropy test for file_salt | Security & Privacy |
| P2-13 | Recommend omitting `_property_sensitivity` from partner-scope files | Security & Privacy |

---

## Cross-Domain Interactions

These interdependencies — where one expert's recommendation directly affects another's domain — are among the most valuable insights from the panel:

1. **UC3 is a prerequisite for UC2.** Multi-tier supplier mapping (LkSG/CSDDD) requires a consolidated vendor master. ERP Integration and Supply Chain experts independently identified this dependency. The use cases should explicitly sequence them.

2. **Identifier coverage limits merge viability.** Entity Identification, ERP Integration, and Procurement experts converge: without an enrichment step, UC3's merge engine has insufficient identifiers to operate on. The enrichment lifecycle (SPEC-005, Section 6) is theoretically sound but the use case does not acknowledge it as a mandatory prerequisite.

3. **Structural de-anonymization undermines selective disclosure.** The Security & Privacy Expert's re-identification concern directly intersects with the Graph Modeling Expert's structural analysis. Supply chain graphs with distinctive commodity-edge patterns may be more vulnerable than the social network literature suggests, because supply chains are sparser and more structurally distinctive.

4. **`tier` consistency affects regulatory compliance.** The Graph Modeling Expert's recommendation for an L2 tier-distance validation rule has direct implications for the Regulatory Compliance Expert's assessment of LkSG/CSDDD compliance — incorrect tier assignments can change due diligence obligations.

5. **Merge type-homogeneity intersects with branch DUNS risk.** The Graph Modeling Expert's type-homogeneity guard recommendation directly addresses the Entity Identification Expert's branch-DUNS-on-organization-node concern. A facility and organization sharing a branch DUNS could be incorrectly merged without this guard.

6. **Person node privacy creates a tension with regulatory transparency.** The Security & Privacy Expert notes that omitting person nodes from public files leaves observable structural gaps, while the Regulatory Compliance Expert notes that AMLD 6 transparency requirements may conflict with GDPR restrictions. This tension needs explicit resolution in SPEC-004.

7. **Sensitivity classification affects merge.** When multiple ERP exports with different `_property_sensitivity` values are merged (UC3→UC6 pipeline), the Security & Privacy Expert recommends a "most restrictive wins" rule, which the ERP Integration Expert confirms is the standard approach in enterprise MDM.

---

## Individual Expert Reports

### Supply Chain Expert

#### Assessment

From 18 years mapping multi-tier supply networks, these six use cases cover the regulatory and operational scenarios that dominate supply chain transparency work today. The selection is strategically sound. The inclusion of `visibility_depth` on `supplies` edges directly addresses the critical problem of distinguishing "we mapped this far and found nothing" versus "we have not mapped beyond this point."

However, several use cases contain structural inaccuracies relative to the spec, and some important real-world scenarios are underrepresented. The use cases lean heavily toward static network snapshots and do not adequately address the temporal and dynamic aspects of supply chain disruption analysis.

#### Strengths
- Multi-tier visibility is a first-class concern (`tier`, `reporting_entity`, `visibility_depth`)
- EUDR geolocation modeling well-aligned with regulatory requirements (point + polygon)
- ERP consolidation scenario (UC3) reflects real enterprise pain
- Selective disclosure (UC6) addresses the primary adoption barrier
- Attestation coverage periods distinguish document validity from reporting periods

#### Concerns
- **[Major]** UC5 (CBAM) `produces` edge target type inconsistency between prose and type constraint table
- **[Major]** UC2 (LkSG) overstates tier 2 requirement — LkSG is event-triggered, CSDDD is systematic
- **[Major]** No disruption analysis use case despite properties designed for it
- **[Minor]** UC1 does not model multi-hop commodity chains
- **[Minor]** No temporal evolution use case
- **[Minor]** CBAM default value fallback not demonstrated

#### Recommendations
1. (P0) Fix `produces` edge definition inconsistency
2. (P0) Separate LkSG/CSDDD narratives
3. (P1) Add disruption analysis use case
4. (P1) Add temporal comparison use case
5. (P2) Expand EUDR with multi-hop chains
6. (P2) Demonstrate mixed-quality CBAM emissions data

---

### Procurement Expert

#### Assessment

UC3 (Multi-ERP Consolidation) addresses the single most expensive ongoing problem in enterprise procurement: fragmented vendor master data. The composite identifier approach maps well to real deduplication workflows. The file-based, vendor-neutral design avoids platform lock-in. However, the use cases assume clean data and describe end states without addressing the journey — particularly the supplier data collection burden.

#### Strengths
- UC3 directly addresses highest-pain procurement problem with practical ERP table-level mappings
- Composite identifier model mirrors real vendor identification reality
- `same_as` edges match existing MDM team workflows
- Incremental enrichment lifecycle is realistic
- `visibility_depth` distinguishes incomplete from genuinely flat supply chains

#### Concerns
- **[Critical]** UC1/UC2/UC5 lack supplier data collection workflows — the hard part is collecting data from suppliers
- **[Critical]** UC3 does not address data quality pre-processing — merge on dirty identifiers produces garbage
- **[Major]** No supplier onboarding use case
- **[Major]** UC4 references `share_percent` instead of spec's `percentage`
- **[Major]** UC1 does not reflect December 2025 EUDR simplifications
- **[Minor]** No round-trip import workflow
- **[Minor]** CBAM transitional period default values not mentioned

#### Recommendations
1. (P0) Add supplier data collection use case or extend UC1/UC2
2. (P0) Add data quality pre-processing guidance to UC3
3. (P1) Add supplier onboarding use case
4. (P1) Fix `share_percent` → `percentage` in UC4
5. (P1) Update UC1 for EUDR amendments
6. (P2) Add round-trip import example
7. (P2) Show `emission_factor_source` enum in action

---

### Regulatory Compliance Expert

#### Assessment

The use cases represent a thoughtful and largely accurate mapping of OMTS capabilities to real regulatory obligations. The data structures correspond closely to what regulators demand. However, the regulatory landscape is not static: EUDR postponed to December 2026, CSDDD pushed to July 2028, LkSG reporting obligation removed. The CBAM use case is particularly well-timed with the definitive period beginning January 2026.

#### Strengths
- Accurate EUDR geolocation modeling (point + polygon per Article 9)
- CBAM emissions structure is regulatory-grade (`emission_factor_source` enum mirrors Annex III)
- Attestation node design covers full compliance lifecycle
- Beneficial ownership threshold aligned with AMLD 6 25% UBO rule
- Labels for regulatory scoping enable single graph for multiple compliance regimes

#### Concerns
- **[Major]** EUDR DDS submission gap — OMTS relationship to TRACES system unclear
- **[Major]** Missing CN-code granularity — EUDR requires 8-digit CN, UC1 uses 4-digit HS
- **[Major]** CSDDD/LkSG conflated — different scope, enforcement, timeline
- **[Major]** CBAM product-level specificity missing — should reference Annex I categories
- **[Minor]** No temporal chain-of-custody in EUDR use case
- **[Minor]** UC4 lacks control-test modeling for non-ownership UBO indicators
- **[Minor]** No cross-jurisdictional GDPR/AMLD conflict handling

#### Recommendations
1. (P0) Add CN-code granularity to EUDR use case
2. (P0) Clarify OMTS is not a regulatory submission format
3. (P1) Separate LkSG and CSDDD
4. (P1) Add consignment-level traceability to UC1
5. (P1) Specify CBAM product categories
6. (P2) Add control-test example to UC4
7. (P2) Add regulatory timeline context

---

### Enterprise Integration Expert

#### Assessment

The SPEC-005 ERP mappings (SAP LFA1/BUT000, Oracle PrcPozSuppliersVO, D365 VendorsV2) provide a credible starting point for ERP-to-graph extraction. The composite identifier model, `internal` scheme exclusion from merge, transitive closure, and `same_as` edges form a sound architecture. However, UC3 is a paragraph-long sketch for what is in reality a 6-12 month project with well-documented failure modes: identifier coverage gaps (<15% DUNS in many systems), 8-20% duplicate rates, conflicting name spellings, and organizational boundary ambiguity.

#### Strengths
- Composite identifiers (LEI + DUNS + VAT) match real MDM cross-reference approach
- `same_as` with confidence levels maps to SAP MDG fuzzy duplicate detection workflows
- Internal identifiers preserved but excluded from merge — correct design
- Label mapping preserves ERP-specific classification semantics via reverse-domain convention
- Merge-group safety limits protect against false-positive cascades

#### Concerns
- **[Critical]** UC3 does not address identifier coverage gaps — most organization nodes will carry only `internal` identifiers
- **[Critical]** Delta/patch mechanism absent — full re-export/re-merge is operationally infeasible at scale
- **[Major]** Name normalization challenges unaddressed
- **[Major]** Organizational boundary differences across ERPs not covered (multi-client SAP, multi-BU Oracle, multi-company D365)
- **[Major]** UC2 depends on UC3 but does not acknowledge this
- **[Major]** UC5 CBAM installation-level data sourcing challenge unaddressed
- **[Minor]** UC1 geo precision not specified
- **[Minor]** UC4 data sourcing gap (UBO data not in ERPs)

#### Recommendations
1. (P0) Add identifier coverage prerequisites to UC3 with realistic rates
2. (P0) Prioritize delta/patch envelope specification
3. (P1) Expand UC3 for name normalization and fuzzy matching pre-merge step
4. (P1) Add dependency between UC2 and UC3
5. (P1) Expand UC5 for installation-level data sourcing
6. (P2) Add geo precision guidance to UC1
7. (P2) Add re-identification risk acknowledgment to UC6

---

### Security & Privacy Expert

#### Assessment

OMTS demonstrates a mature, privacy-aware design. The three-tier sensitivity model, boundary reference hashing with per-file CSPRNG salt, and categorical prohibition on person nodes in public files reflect a design team that understands the tension between visibility and confidentiality. The SHA-256 with 32-byte salt is defensible, and the fallback to random tokens when no public identifiers exist avoids the empty-hash collision.

However, graph-structural re-identification is the primary unaddressed threat class. Edge types, commodity codes, and degree sequences on boundary_ref neighborhoods create quasi-identifier fingerprints exploitable with auxiliary knowledge.

#### Strengths
- Per-file CSPRNG salt prevents rainbow-table enumeration (256 bits entropy)
- Categorical exclusion of person nodes from public files aligns with GDPR minimization
- Default `confidential` on person node identifiers — correct default
- Edge property sensitivity with per-property overrides provides fine-grained control
- L2-SDI-01 flags person identifiers set to `public` — guardrail against accidental GDPR violations

#### Concerns
- **[Critical]** Graph-structural re-identification of boundary_ref nodes — no threat model for structural de-anonymization
- **[Critical]** No file-level digital signature mechanism — content hash provides no authentication
- **[Major]** Person node deletion from public files leaves observable structural gaps
- **[Major]** No salt lifecycle or retention guidance
- **[Major]** `_property_sensitivity` metadata is itself informative to attackers
- **[Minor]** No minimum entropy requirement on file_salt
- **[Minor]** Boundary reference hash lacks domain separation prefix

#### Recommendations
1. (P0) Add structural re-identification risk section to SPEC-004
2. (P0) Define file-level digital signature mechanism in SPEC-007
3. (P1) Add salt lifecycle and retention guidance
4. (P1) Specify domain separation for boundary reference hash
5. (P1) Add guidance on structural gap mitigation for person node omission
6. (P2) Define minimum entropy test for file_salt
7. (P2) Recommend omitting `_property_sensitivity` from partner-scope files

---

### Entity Identification Expert

#### Assessment

SPEC-002's composite identifier model is the most thoughtful treatment of the "is this the same company?" problem I have seen in a supply chain format. No mandatory scheme, internal identifiers as first-class citizens, temporal validity, canonical string format with percent-encoding, branch DUNS merge risk callout — these details could only come from people who have been burned by these issues in production.

However, the EUDR use case involves entities (cooperatives, smallholders) that have no formal identifiers at all, making cross-file merge impossible for exactly the entities that matter most. The beneficial ownership use case depends on UBO registries that are fragmented and unreliable across EU member states.

#### Strengths
- No mandatory scheme is correct — LEI covers ~2.9M entities out of hundreds of millions
- Branch DUNS merge risk explicitly addressed (L2-EID-09)
- Temporal validity prevents false merges on reassigned identifiers
- `same_as` with confidence levels correctly models probabilistic entity resolution
- Merge-group safety limits protect against transitive closure explosion
- `former_identity` edges correctly model M&A identity transformation
- Extension scheme mechanism (UEI, EORI) demonstrates forward-thinking extensibility

#### Concerns
- **[Critical]** No practical identity solution for informal entities (EUDR cooperatives, smallholders)
- **[Critical]** Beneficial ownership registry unreliability understated — multiple EU states suspended/restricted access
- **[Major]** No identifier confidence or match scoring beyond binary match/no-match
- **[Major]** Cross-scheme identifier conflict resolution not specified (LEI vs. nat-reg mismatch)
- **[Major]** Identity degradation at tier depth not addressed in UC2
- **[Minor]** CBAM installation ID is extension scheme, not core
- **[Minor]** VAT normalization `vat:DE:DE123456789` is counterintuitive

#### Recommendations
1. (P0) Define identity bridge pattern for informal entities
2. (P0) Add verification_status guidance for beneficial ownership data
3. (P1) Introduce match-confidence annotation on merge results
4. (P1) Document identity degradation at depth for UC2
5. (P1) Specify conflict resolution for cross-scheme identifier mismatches
6. (P2) Promote CBAM installation to core scheme
7. (P2) Simplify VAT normalization

---

### Graph Modeling Expert

#### Assessment

The OMTS graph model is a well-grounded directed labeled property multigraph aligned with ISO/IEC 39075 (GQL). The multigraph semantics, per-subgraph cycle policies, formally specified merge algebra (commutativity, associativity, idempotency), and graph type constraints table are all technically sound. The six use cases exercise the model across its structural dimensions.

#### Strengths
- Correct multigraph semantics with independent edge IDs
- Per-subgraph cycle policy (forest for legal_parentage, DAG-advisory for composed_of, cycles for supplies/ownership)
- Formally specified merge algebra with necessary algebraic properties
- N-ary reification pattern via intermediate nodes
- Graph type constraints table enables type-safe traversal
- Merge-group safety limits with tiered thresholds

#### Concerns
- **[Major]** `produces` edge documentation inconsistency (prose vs. type constraint table for consignment targets)
- **[Major]** `tier` property lacks formal graph-distance validation
- **[Major]** Merge transitive closure has no type-homogeneity guard
- **[Minor]** `same_as` edge is semantically undirected but stored in directed structure
- **[Minor]** No hyperedge mechanism for multi-commodity supply relationships
- **[Minor]** Ownership percentage sum rule (L3-MRG-01) should be L2

#### Recommendations
1. (P0) Add type-homogeneity guard to merge
2. (P1) Add L2 rule for `tier` consistency with graph distance
3. (P1) Clarify `produces` edge documentation for `consignment` targets
4. (P1) Promote ownership sum ≤ 100% to L2
5. (P2) Add `same_as` normalization convention
6. (P2) Consider promoting arrangement node to core type
