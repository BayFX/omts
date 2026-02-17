# Expert Panel Report: Entity Identification Specification (OMTSF-SPEC-001)

**Date:** 2026-02-17
**Document Reviewed:** `docs/specs/entity-identification.md`
**Panel Size:** 11 experts

---

## Panel Chair Summary

The OMTSF Entity Identification Specification represents a substantial and technically mature response to the single most critical gap identified in the vision document review: the absence of an entity identification strategy. All eleven panelists acknowledge that the spec directly addresses the P0-1 recommendation that nine of them flagged during the vision review. The composite identifier model -- treating LEI, DUNS, GLN, national registry numbers, VAT numbers, and internal ERP codes as co-equal peers in a scheme-qualified array -- received universal endorsement as the correct architectural choice. The separation of graph-local identity from external identity, the tiered validation levels, the sensitivity classification with scheme defaults, and the corporate hierarchy edge types were cited as strengths by a majority of panelists.

However, the panel identified two classes of remaining gaps. First, **the spec's own constructs require tighter specification**: the boundary reference hash construction has a collision vulnerability for entities with only restricted identifiers (Tanaka, Engstrom, Kowalski, Varga), the canonical string format for identifiers remains an open question that blocks hashing interoperability (Petrova, Kowalski, Nakamura, Engstrom), and the merge operation lacks formal algebraic properties -- commutativity, associativity, and transitivity decisions (Varga). Second, **the spec's scope needs expansion in specific directions**: no supply relationship edge type taxonomy exists alongside the well-specified corporate hierarchy edges (Osei), no attestation or certification model supports regulatory compliance workflows (Moreau), no beneficial ownership representation handles natural persons required by CSDDD and AMLD (Moreau), no intra-file deduplication guidance helps ERP producers (Krishnamurthy), and no BOM decomposition supports material-to-material traceability (Krishnamurthy, Osei).

The panel surfaced a productive consensus on Open Question #2 (minimum identifier requirement): all experts who addressed it recommended keeping external identifiers at Level 2, not Level 1, to preserve the adoption ramp for ERP-only exports. On Open Question #1 (canonical string format), the panel unanimously recommends resolving it immediately as `scheme:authority:value` since the boundary reference hash already depends on it. The governance gaps flagged during the vision review (licensing, DCO, scheme registry process) remain unresolved and were re-raised by Okafor and Nakamura as adoption blockers for the specification itself.

---

## Panel Composition

| Name | Role | Key Focus Area |
|------|------|---------------|
| Dr. Amara Osei | Supply Chain Visibility & Risk Analyst | Supply relationship taxonomy, data quality signals, disruption modeling |
| Marcus Lindgren | Chief Procurement Officer | Identifier enrichment workflows, ERP adoption, supplier burden |
| Dr. Kenji Nakamura | Standards & Interoperability Specialist | ISO 6523 alignment, conformance clauses, GLEIF RA code usage |
| Sofia Petrova | Senior Systems Engineer (Rust) | Canonical encoding, memory efficiency, WASM constraints, type mapping |
| Prof. Elena Varga | Graph Data Modeling & Algorithm Specialist | Merge algebraic properties, identity predicate transitivity, edge merge |
| Rajesh Krishnamurthy | Enterprise Systems Architect | SAP/Oracle/D365 mapping depth, intra-file dedup, BOM structures, delta updates |
| Dr. Isabelle Moreau | Regulatory Compliance Advisor | Beneficial ownership, attestation model, sanctions screening, EUDR fields |
| Dr. Tomasz Kowalski | Data Format Architect | Canonical encoding for hashing, schema evolution, binary encoding, file_salt |
| Danielle Okafor | Open Source Strategy & Governance Lead | Scheme registry governance, GLEIF dependency, spec licensing, adoption strategy |
| Dr. Yuki Tanaka | Data Security & Privacy Architect | Boundary reference collision, sensitivity enforcement, CSPRNG, GDPR edge cases |
| Patricia Engstrom | Entity Identification & Corporate Hierarchy Specialist | DUNS branch/HQ ambiguity, LEI lifecycle, name normalization, confidence scoring |

---

## Consensus Findings

These findings were independently raised by multiple experts, lending them the highest weight:

1. **The composite identifier model is the correct architecture** (all 11 panelists). Universal endorsement of the multi-scheme, no-mandatory-identifier design. The refusal to privilege any single identifier system was cited as the single most important design decision for adoption.

2. **The canonical string format (Open Question #1) must be resolved immediately** (Petrova, Kowalski, Nakamura, Engstrom, Tanaka, Varga -- 6 of 11). The boundary reference hash in Section 8.2 already depends on a canonical form. Multiple experts identified that without byte-exact canonicalization, two conformant implementations will produce different hashes for the same entity. Unanimous recommendation: adopt `scheme:authority:value` with absent authority omitted.

3. **Boundary reference hash has a collision vulnerability** (Tanaka, Engstrom, Varga, Kowalski -- 4 of 11). Entities with only `restricted` or `confidential` identifiers produce a degenerate hash `SHA-256(file_salt)`, collapsing all such entities into a single boundary reference. This breaks graph connectivity in redacted subgraphs.

4. **Open Question #2 should be resolved in favor of Level 2** (Osei, Lindgren, Okafor, Engstrom -- 4 of 11 explicitly, no dissent). Requiring external identifiers at Level 1 would block ERP-only exports and contradict the "internal identifiers are first-class" design principle.

5. **Merge algebraic properties must be formalized** (Varga, Kowalski, Petrova -- 3 of 11). Merge must be commutative, associative, and idempotent for the decentralized merge model to work. The identity predicate's non-transitivity creates ordering-dependent results that the spec does not acknowledge or resolve.

6. **Governance and licensing gaps remain unresolved** (Okafor, Nakamura -- 2 of 11, both carrying forward Critical findings from the vision review). No scheme registry governance process, no spec license (should be CC-BY-4.0), no DCO, no contribution process.

---

## Critical Issues

Issues rated **[Critical]** by at least one expert.

| # | Issue | Flagged By | Summary |
|---|-------|-----------|---------|
| C1 | **Boundary reference hash collision for restricted-only entities** | Tanaka, Engstrom | Entities with no `public` identifiers produce identical hashes (`SHA-256(file_salt)`), collapsing distinct nodes in redacted subgraphs. |
| C2 | **No enforcement mechanism for sensitivity levels** | Tanaka | Sensitivity is purely declarative; `confidential` identifiers can appear in shared files with no validation-level check. |
| C3 | **Merge commutativity/associativity not formally required** | Varga | Without formal algebraic properties, different tools merging the same files in different orders produce different graphs. |
| C4 | **Identity predicate is non-transitive** | Varga | Nodes sharing identifiers via intermediate nodes may or may not merge depending on file ordering. Spec must resolve whether transitive closure is applied. |
| C5 | **No supply relationship edge type taxonomy** | Osei | Corporate hierarchy edges are well-specified but commercial supply edges (`supplies`, `subcontracts`, `tolls`) are informal. Breaks merge and regulatory reporting. |
| C6 | **No beneficial ownership (natural person) representation** | Moreau | CSDDD and AMLD require tracing ownership to natural persons. No `person` node type or `beneficial_ownership` edge type exists. |
| C7 | **No attestation or certification model** | Moreau | EUDR due diligence statements, LkSG risk analysis, and ISO certifications cannot be attached to entities or facilities. |
| C8 | **No intra-file deduplication guidance for ERP exports** | Krishnamurthy | Same entity as multiple vendor records in ERP produces duplicate nodes. No guidance on whether to deduplicate or how to mark equivalence. |
| C9 | **SAP mapping omits purchasing info records and edge derivation** | Krishnamurthy | Section 11 covers vendor master headers but not the tables (`EINA`/`EINE`, `EKKO`/`EKPO`) that populate supply edges. |
| C10 | **Canonical encoding for boundary reference hashing is underspecified** | Kowalski | Sort order, delimiter, salt encoding, and normalization rules are ambiguous. Two implementations will produce divergent hashes. |
| C11 | **No governance process for identifier scheme registry** | Okafor | No process for adding, promoting, or deprecating schemes. Vocabulary either ossifies or fragments. |
| C12 | **Hard dependency on GLEIF RA list with no fallback** | Okafor | If GLEIF changes the list, all validators are affected. Needs versioned OMTSF-maintained snapshot. |
| C13 | **No DUNS branch/HQ disambiguation guidance** | Engstrom | D&B assigns separate DUNS to HQ vs. branches of the same entity. Two files may contain the same entity under different DUNS numbers. |
| C14 | **No LEI lifecycle status handling** | Engstrom | LAPSED, MERGED, and ANNULLED LEIs need explicit guidance for merge and validation behavior. |
| C15 | **No identifier enrichment workflow guidance** | Lindgren | No conceptual model for how files progress from internal-only to fully enriched identifiers. |

---

## Major Issues

| # | Issue | Flagged By |
|---|-------|-----------|
| M1 | No ISO 6523 ICD mapping table | Nakamura |
| M2 | No conformance clauses (producer, consumer, validator) | Nakamura |
| M3 | Extension scheme namespace disconnected from ISO 6523 | Nakamura |
| M4 | No canonical string representation specified (blocks implementation) | Petrova |
| M5 | `value` is opaque string; no max length per scheme (memory) | Petrova |
| M6 | No maximum cardinality on `identifiers` array (OOM in WASM) | Petrova |
| M7 | Edge merge fallback on property equality underspecified | Varga |
| M8 | No canonical ordering for merged identifier arrays | Varga |
| M9 | No BOM / `composed_of` edge type for material structures | Krishnamurthy |
| M10 | Oracle and D365 mappings too shallow | Krishnamurthy |
| M11 | No delta/patch update pattern | Krishnamurthy |
| M12 | No sanctions screening compatibility (aliases, addresses) | Moreau |
| M13 | EUDR due diligence statement fields not mappable (address, quantity, production date) | Moreau |
| M14 | No regulatory role annotation mechanism | Moreau |
| M15 | No schema evolution rules for identifier records | Kowalski |
| M16 | No canonical encoding for file format (RFC 8785 JCS) | Kowalski |
| M17 | Binary encoding not addressed | Kowalski |
| M18 | Specification license undefined (should be CC-BY-4.0) | Okafor |
| M19 | No contributor process (DCO, CLA, CONTRIBUTING file) | Okafor |
| M20 | Identifier complexity may overwhelm small-supplier adoption | Okafor |
| M21 | No CSPRNG requirement for `file_salt` generation | Tanaka |
| M22 | `nat-reg` default sensitivity `public` incorrect for sole proprietorships (GDPR) | Tanaka |
| M23 | No boundary reference stability across re-exports | Tanaka |
| M24 | No trust domain separation in merge provenance | Tanaka |
| M25 | No data quality/confidence signal on identifier records | Osei |
| M26 | No capacity/volume attributes on supply edges | Osei |
| M27 | Merge conflict resolution deferred without structured conflict record | Osei |
| M28 | `internal` merge exclusion needs carve-out for same-system intra-org merge | Lindgren |
| M29 | GLEIF RA codes burdensome; allow ISO 3166-1 at Level 1 | Lindgren |
| M30 | Joint ventures and split-identity entities unmodeled | Engstrom |
| M31 | Identity predicate has no confidence scoring across schemes | Engstrom |
| M32 | No name normalization or fuzzy matching guidance | Engstrom |

---

## Minor Issues

| Issue | Flagged By |
|-------|-----------|
| `file_salt` encoding ambiguous (hex? base64?) | Kowalski |
| No content addressing for identifier records | Kowalski |
| Extension scheme regex/ABNF grammar not provided | Petrova |
| Date fields should be restricted to `YYYY-MM-DD` profile | Petrova |
| `sensitivity` enum not extensible (no fail-closed rule) | Petrova |
| Hyperedge / n-ary relationship gap unacknowledged | Varga |
| `boundary_ref` not listed in node type taxonomy (Section 5) | Varga |
| `former_identity` edge direction counterintuitive for traversal | Varga |
| Serialization example has confusing 0% ownership edge | Lindgren |
| Coupa/Jaggaer procurement platform mappings missing | Lindgren |
| `good` node type lacks batch/lot-level granularity for EUDR | Osei |
| No sub-tier visibility depth / mapping completeness metadata | Osei |
| `former_identity` does not capture successor entity liability | Moreau |
| No identifier provenance/verification_status field | Moreau |
| No mention of SAP Business Partner model (`BUT000`) | Krishnamurthy |
| No EDI coexistence positioning | Krishnamurthy |
| `authority` convention for `internal` scheme lacks structure | Krishnamurthy |
| OpenCorporates reconciliation not referenced | Engstrom |
| No guidance for US entity identification (no federal registry) | Engstrom |
| LEI check digit should move to L1 (not L2) | Nakamura |
| No identifier lifecycle event guidance (GLN reassignment, LEI renewal) | Nakamura |
| No conformance test suite plan for identifier validation | Okafor |

---

## Consolidated Recommendations

### P0 -- Immediate (must resolve before spec finalization)

| # | Recommendation | Origin |
|---|---------------|--------|
| P0-1 | **Fix boundary reference hash for restricted-only entities.** Use random opaque token when no public identifiers exist, or include restricted identifiers with separate keying. | Tanaka, Engstrom |
| P0-2 | **Resolve canonical string format (Open Question #1).** Adopt `scheme:authority:value` with byte-exact encoding rules, sort order, delimiter, and test vectors. | Kowalski, Petrova, Nakamura, Engstrom, Tanaka |
| P0-3 | **Formalize merge algebraic properties.** Require commutativity, associativity, and idempotency. Resolve identity predicate transitivity (transitive closure vs. pairwise). | Varga |
| P0-4 | **Define supply relationship edge type taxonomy.** At minimum: `supplies`, `subcontracts`, `tolls`, `distributes`, `brokers` with defined properties. | Osei |
| P0-5 | **Add beneficial ownership model.** `person` node type or `beneficial_ownership` edge type with percentage, control type, and temporal validity. | Moreau |
| P0-6 | **Design attestation/certification model.** Node type or edge type for audits, certifications, and due diligence statements attached to entities/facilities. | Moreau |
| P0-7 | **Add intra-file deduplication guidance.** Define whether producers should merge duplicate vendor records or declare equivalence via `same_as` edges. | Krishnamurthy |
| P0-8 | **Expand SAP mapping to include edge-deriving tables.** Add `EINA`/`EINE`, `EKKO`/`EKPO`, `MARA`/`MARC` mappings. | Krishnamurthy |
| P0-9 | **Define identifier enrichment conceptual model.** Document how files progress from internal-only to enriched identifiers over time. | Lindgren |
| P0-10 | **Publish versioned GLEIF RA list snapshot.** Decouple validation from GLEIF publication timing. | Okafor |
| P0-11 | **Define governance process for scheme vocabulary.** RFC process for additions, promotions, and deprecations. | Okafor |
| P0-12 | **Separate spec and code licensing.** CC-BY-4.0 for spec, Apache 2.0 for code, adopt DCO. | Okafor |
| P0-13 | **Add DUNS branch/HQ disambiguation guidance.** Document D&B family tree model and mapping to OMTSF node types. | Engstrom |
| P0-14 | **Define LEI lifecycle status handling.** LAPSED = still valid for merge; MERGED = generate `former_identity` edge; ANNULLED = L2 warning. | Engstrom |

### P1 -- Before v1

| # | Recommendation | Origin |
|---|---------------|--------|
| P1-1 | Define canonical form for merged output (ordering for identifiers, conflicts). | Varga, Kowalski |
| P1-2 | Add normative ISO 6523 ICD mapping table. | Nakamura |
| P1-3 | Define conformance clauses (producer, consumer, validator). | Nakamura |
| P1-4 | Promote check digit verification to Level 1. | Nakamura |
| P1-5 | Specify maximum `value` length per core scheme. | Petrova |
| P1-6 | Add recommended max cardinality for `identifiers` array (64 per node). | Petrova |
| P1-7 | Restrict ISO 8601 date profile to `YYYY-MM-DD`. | Petrova |
| P1-8 | Provide regex/ABNF for extension scheme codes. | Petrova |
| P1-9 | Add `composed_of` edge type for BOM/material structure. | Krishnamurthy |
| P1-10 | Expand Oracle and D365 mappings to cover supplier sites and BOM. | Krishnamurthy |
| P1-11 | Define delta/patch envelope for incremental updates. | Krishnamurthy |
| P1-12 | Extend `organization` nodes with aliases, address, contact fields. | Moreau |
| P1-13 | Add regulatory role annotation mechanism. | Moreau |
| P1-14 | Add identifier provenance metadata (verification method, date). | Moreau, Osei |
| P1-15 | Define unknown-field handling for identifier records (must-preserve). | Kowalski |
| P1-16 | Reserve numeric field keys for binary encoding. | Kowalski |
| P1-17 | Mandate RFC 8785 (JCS) for JSON canonical encoding. | Kowalski |
| P1-18 | Specify `file_salt` encoding explicitly (hex in JSON, raw in binary). | Kowalski |
| P1-19 | Publish conformance test suite for identifier validation. | Okafor |
| P1-20 | Define "minimum viable file" profile for small-supplier adoption. | Okafor |
| P1-21 | Resolve Open Question #2 in favor of Level 2. | Okafor, Lindgren, Osei, Engstrom |
| P1-22 | Add L1 validation: if `disclosure_scope=external`, no `confidential` identifiers. | Tanaka |
| P1-23 | Mandate CSPRNG for `file_salt` generation. | Tanaka |
| P1-24 | Change `nat-reg` default sensitivity to `restricted` for sole-proprietorship jurisdictions. | Tanaka |
| P1-25 | Add `confidence` field to identifier records (verified/reported/inferred/unverified). | Osei |
| P1-26 | Define structured conflict record for merge property disagreements. | Osei |
| P1-27 | Allow ISO 3166-1 as alternative `nat-reg` authority at Level 1. | Lindgren |
| P1-28 | Expand Section 11 to include procurement platform mappings (Ariba, Coupa). | Lindgren |
| P1-29 | Add controlled exception for intra-system `internal` identifier merge. | Lindgren |
| P1-30 | Add name normalization appendix (NFKC, case folding, legal form removal). | Engstrom |
| P1-31 | Define confidence hierarchy for identifier scheme matches. | Engstrom |
| P1-32 | Define extension scheme registration process with ISO 6523 alignment. | Nakamura |

### P2 -- Future

| # | Recommendation | Origin |
|---|---------------|--------|
| P2-1 | Define content-addressable identifier records. | Kowalski |
| P2-2 | Evaluate block-level compression with zstd. | Kowalski |
| P2-3 | Specify fail-closed behavior for unknown `sensitivity` values. | Petrova |
| P2-4 | Add `#[non_exhaustive]` guidance for all enums. | Petrova |
| P2-5 | Acknowledge hyperedge gap; document intermediate-node pattern. | Varga |
| P2-6 | Specify traversal semantics for `former_identity` edges. | Varga |
| P2-7 | EUDR due diligence statement XML mapping. | Moreau |
| P2-8 | Add `address` structure to `facility` nodes beyond geo coordinates. | Moreau |
| P2-9 | Introduce `trust_domain` field in merge provenance. | Tanaka |
| P2-10 | Address boundary reference stability across re-exports (optional stable salt). | Tanaka |
| P2-11 | Add lot/batch-level support to `good` node type. | Osei |
| P2-12 | Introduce `mapping_completeness` metadata field. | Osei |
| P2-13 | Update SAP mapping to reference Business Partner model (`BUT000`). | Krishnamurthy |
| P2-14 | Publish EDI coexistence guidance. | Krishnamurthy |
| P2-15 | Establish GLEIF and GS1 formal liaison. | Okafor, Nakamura |
| P2-16 | Plan identifier scheme extension registry (community YAML). | Okafor |
| P2-17 | Non-normative guidance for US entity identification. | Engstrom |
| P2-18 | Reference OpenCorporates reconciliation API. | Engstrom |
| P2-19 | Publish identifier lifecycle guidance (GLN reassignment, LEI renewal). | Nakamura |

---

## Cross-Domain Interactions

These interdependencies between expert domains surfaced during review:

1. **Canonical encoding x Boundary reference hashing x Merge determinism** (Kowalski + Tanaka + Petrova + Varga). The canonical string format, boundary reference hash construction, merge identity predicate, and file-level canonical encoding all depend on the same foundational decision: a byte-exact canonical representation of identifiers. These must be co-designed as a single coherent system, not resolved independently.

2. **Beneficial ownership x GDPR x Sensitivity model** (Moreau + Tanaka). Adding a `person` node type for beneficial ownership triggers GDPR Article 9 considerations. Person nodes must be `confidential` by default with purpose-limitation constraints beyond the current three-tier model.

3. **Supply edge taxonomy x ERP edge derivation x BOM decomposition** (Osei + Krishnamurthy). Formalizing supply relationship edges requires knowing which ERP tables populate them. BOM decomposition (`composed_of` edges) connects to material master tables that Krishnamurthy's SAP mapping expansion would cover.

4. **GLEIF RA dependency x Governance x Validator resilience** (Okafor + Nakamura). The normative dependency on the GLEIF RA list requires both a versioned snapshot strategy (governance) and validator fallback behavior (when encountering unknown RA codes).

5. **Merge transitivity x Rust implementation x Performance** (Varga + Petrova). If transitive closure is required, the Rust implementation needs a union-find data structure (O(n*alpha(n))). If pairwise only, a simple hash-join suffices (O(n+m)). This algorithmic choice must be settled at the spec level.

6. **Internal identifier merge exclusion x Intra-org dedup x Enrichment lifecycle** (Lindgren + Krishnamurthy + Engstrom). The interaction between same-system `internal` identifiers, ERP export deduplication, and the enrichment workflow from internal-only to external identifiers is a three-way dependency that needs a coherent "producer guidance" section.

7. **Identifier complexity x Small-supplier adoption x Minimum viable file** (Okafor + Lindgren). The identifier model is technically excellent but conceptually dense. The "minimum viable file" profile and guided authoring tool are essential for adoption below the enterprise tier.

---

## Individual Expert Reports

### Dr. Amara Osei -- Supply Chain Visibility & Risk Analyst

**Overall Assessment:** The spec is a substantial response to the #1 critical gap. The composite identifier model, sensitivity classification, temporal validity, and boundary references are all well-designed. However, the spec is heavily weighted toward entity identification and corporate structure while leaving operational supply relationships underspecified.

**Critical:** No formal supply relationship edge type taxonomy (direct supply, subcontracting, tolling, brokerage have distinct regulatory implications under CSDDD/LkSG).

**Major:** No data quality/confidence signal on identifiers; no capacity/volume attributes on supply edges; merge conflict resolution deferred without structured conflict records.

**Top Recommendations:** (P0) Define supply relationship edge taxonomy. (P1) Add confidence field to identifiers. (P1) Define structured conflict records. (P1) Guidance for quantitative supply edge properties.

---

### Marcus Lindgren -- Chief Procurement Officer

**Overall Assessment:** The spec directly addresses the most impactful gap from the vision review. Internal identifiers as first-class citizens, tiered validation, and ERP mappings are exactly right for enterprise adoption. However, the enrichment workflow from internal-only to externally-identified is undefined.

**Critical:** No identifier enrichment workflow guidance for how files progress from Level 1 to Level 3 quality.

**Major:** `internal` merge exclusion needs a carve-out for same-system intra-org merge; supplier data collection burden unaddressed; GLEIF RA codes too burdensome for Level 1 (allow ISO 3166-1).

**Top Recommendations:** (P0) Define enrichment lifecycle model. (P1) Allow ISO 3166-1 at L1 for nat-reg. (P1) Expand ERP mappings to include procurement platforms. (P1) Resolve Open Question #2 at Level 2.

---

### Dr. Kenji Nakamura -- Standards & Interoperability Specialist

**Overall Assessment:** The spec reflects a mature understanding of identifier interoperability, drawing correctly from PEPPOL, ISO 6523, and GLEIF patterns. The GLEIF RA code list for nat-reg qualification is particularly well-chosen. However, the spec stops short of formal ISO 6523 ICD alignment, which will create friction with EU procurement infrastructure.

**Major:** No ISO 6523 ICD mapping table; extension scheme namespace disconnected from ICD framework; no conformance clauses.

**Top Recommendations:** (P0) Add normative ISO 6523 ICD mapping. (P1) Define conformance clauses. (P1) Promote check digit verification to L1. (P1) Define extension scheme registration process.

---

### Sofia Petrova -- Senior Systems Engineer (Rust)

**Overall Assessment:** The identifier model maps well to Rust's type system (enums for schemes, trait-based tiered validation). The spec avoids fatal design errors. However, the all-string identifier model creates memory pressure in WASM, and the absence of canonical encoding blocks implementation of boundary reference hashing.

**Major:** No canonical string representation; no max value length per scheme; no max cardinality on identifiers array.

**Top Recommendations:** (P0) Define canonical identifier string format. (P0) Specify max value length per core scheme. (P1) Add max cardinality for identifier arrays. (P1) Restrict ISO 8601 to YYYY-MM-DD.

---

### Prof. Elena Varga -- Graph Data Modeling & Algorithm Specialist

**Overall Assessment:** The two-layer identity model (graph-local vs. external) is sound. The multigraph support and temporal property graph design are correct. However, the merge operation lacks formal algebraic properties, and the identity predicate is non-transitive, creating ordering-dependent merge results.

**Critical:** Merge commutativity/associativity not formally required; identity predicate non-transitivity creates inconsistent multi-file merge.

**Major:** Edge merge fallback on property equality underspecified; no canonical ordering for merged identifier arrays.

**Top Recommendations:** (P0) Formalize merge algebraic properties. (P0) Resolve transitivity question. (P1) Define canonical form for merged output. (P1) Tighten edge property comparison semantics.

---

### Rajesh Krishnamurthy -- Enterprise Systems Architect

**Overall Assessment:** The composite identifier model with `internal` as first-class and the tiered validation are exactly what enterprise integration requires. The ERP mappings demonstrate the spec authors took integration seriously. However, the mappings cover entity data but not the tables that populate edges, and BOM decomposition is absent.

**Critical:** No intra-file deduplication guidance for ERP exports; SAP mapping omits purchasing info records and edge-deriving tables.

**Major:** No BOM/`composed_of` edge type; Oracle/D365 mappings too shallow; no delta/patch pattern.

**Top Recommendations:** (P0) Add intra-file dedup guidance. (P0) Expand SAP mapping to edge tables. (P1) Add `composed_of` edge type. (P1) Define delta/patch envelope.

---

### Dr. Isabelle Moreau -- Regulatory Compliance Advisor

**Overall Assessment:** The spec's composite identifier model, temporal validity, corporate hierarchy edges, and regulatory alignment table directly address compliance requirements. The three-tier entity taxonomy with GeoJSON facility support covers EUDR needs. However, critical regulatory constructs are missing.

**Critical:** No beneficial ownership (natural person) representation for CSDDD/AMLD; no attestation/certification model for EUDR/LkSG.

**Major:** No sanctions screening compatibility layer; EUDR due diligence statement fields not fully mappable; no regulatory role annotation.

**Top Recommendations:** (P0) Add person node type / beneficial ownership edges. (P0) Design attestation/certification model. (P1) Extend organization nodes with aliases and address. (P1) Add regulatory role annotations.

---

### Dr. Tomasz Kowalski -- Data Format Architect

**Overall Assessment:** The identifier model is well-designed from a serialization perspective -- all-string values, conditional authority, scheme-qualified records. However, the canonical encoding for boundary reference hashing is underspecified, and schema evolution rules for identifier records are missing.

**Critical:** Canonical encoding for boundary reference hashing is ambiguous (sort order, delimiter, salt encoding).

**Major:** No schema evolution rules for identifier records; no canonical encoding for the file format; binary encoding not addressed.

**Top Recommendations:** (P0) Specify byte-exact canonical encoding with test vectors. (P0) Define unknown-field handling (must-preserve). (P1) Reserve numeric field keys for binary. (P1) Mandate RFC 8785 JCS.

---

### Danielle Okafor -- Open Source Strategy & Governance Lead

**Overall Assessment:** The spec is adoption-friendly: no mandatory proprietary identifiers, tiered validation as an adoption ramp, ERP mappings as ecosystem enablement. However, the scheme vocabulary requires governance that does not exist, and the GLEIF RA dependency is an unmanaged external risk.

**Critical:** No governance process for identifier scheme registry; hard dependency on GLEIF RA list with no fallback.

**Major:** Specification license undefined; no contributor process; identifier complexity may overwhelm small suppliers.

**Top Recommendations:** (P0) Define scheme vocabulary governance. (P0) Publish versioned GLEIF RA snapshot. (P0) Separate spec/code licensing. (P1) Conformance test suite. (P1) "Minimum viable file" profile.

---

### Dr. Yuki Tanaka -- Data Security & Privacy Architect

**Overall Assessment:** The per-identifier sensitivity classification, boundary reference design, and internal-identifier merge exclusion are well-designed privacy primitives. However, the boundary reference hash has a critical collision vulnerability, sensitivity levels lack enforcement, and GDPR edge cases for sole proprietorships are unaddressed.

**Critical:** Boundary reference hash collision for entities with only restricted/confidential identifiers; no enforcement mechanism for sensitivity levels.

**Major:** No CSPRNG requirement for file_salt; nat-reg default sensitivity incorrect for sole proprietorships; no boundary reference stability; no trust domain separation in merge.

**Top Recommendations:** (P0) Fix boundary reference hash for restricted-only entities. (P0) Resolve canonical format for hash interoperability. (P1) Add disclosure_scope-based L1 check. (P1) Mandate CSPRNG. (P1) Jurisdiction-aware nat-reg sensitivity defaults.

---

### Patricia Engstrom -- Entity Identification & Corporate Hierarchy Specialist

**Overall Assessment:** The spec addresses the critical gap with unusual thoroughness. The composite identifier model, GLEIF RA code usage, corporate hierarchy edges with temporal validity, and sensitivity classification are all architecturally sound. However, the spec handles the "happy path" well while underspecifying the messy reality of entity resolution.

**Critical:** No DUNS branch/HQ disambiguation guidance (most common entity resolution failure mode); no LEI lifecycle status handling (LAPSED, MERGED, ANNULLED).

**Major:** No name normalization or fuzzy matching guidance; joint ventures unmodeled; identity predicate has no confidence scoring.

**Top Recommendations:** (P0) Add DUNS branch/HQ disambiguation. (P0) Define LEI lifecycle status handling. (P1) Name normalization appendix. (P1) Confidence hierarchy for identifier schemes.
