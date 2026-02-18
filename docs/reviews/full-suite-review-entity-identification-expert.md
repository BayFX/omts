# Expert Review: OMTSF Full Spec Suite

**Reviewer:** Entity Identification Expert, Entity Identification & Corporate Hierarchy Specialist
**Specs Reviewed:** OMTSF-SPEC-001 through OMTSF-SPEC-006 (all Draft, 2026-02-18)
**Date:** 2026-02-18
**Review Type:** Full-suite cross-spec review from entity identification perspective

---

## Assessment

After seventeen years of building and maintaining entity resolution systems at D&B -- systems that resolved identity across half a billion business records daily -- I can state that this spec suite gets the foundational entity identification architecture right. The decomposition into six specs has sharpened focus: SPEC-002 (Entity Identification) now owns the identifier model cleanly, SPEC-003 (Merge Semantics) owns the identity predicate and merge algebra, and SPEC-001 (Graph Data Model) owns the structural graph without conflating it with resolution logic. This separation matters because entity identification is the load-bearing joint between the file format and the real world. When that joint is muddled into graph structure or serialization concerns, implementations invariably cut corners on the identity problem. Here, it has its own spec with its own validation tiers, and that is correct.

The six core schemes in SPEC-002 are well-chosen and the lifecycle handling is operationally accurate. The LEI five-status model with differentiated merge behavior -- particularly the correct treatment of LAPSED as merge-valid and ANNULLED as merge-prohibited -- exceeds what most commercial entity resolution platforms implement. The DUNS branch/HQ disambiguation with the full D&B Family Tree mapping is the most precise treatment I have seen outside of D&B's own documentation. The canonical string format with percent-encoding and test vectors in SPEC-002 Section 4 unblocks deterministic comparison in SPEC-003 and boundary reference hashing in SPEC-004, creating a clean dependency chain. The merge algebra in SPEC-003 -- commutativity, associativity, idempotency with union-find -- is formally correct and essential for decentralized merge scenarios where parties independently combine overlapping files.

However, reviewing the suite as an integrated whole reveals cross-spec gaps that individual spec reviews did not surface. The identity predicate in SPEC-003 Section 2 depends entirely on identifiers defined in SPEC-002, yet the two specs have no coordinated answer to the temporal identity problem: identifiers are reassigned (DUNS, GLN), and the predicate performs no temporal overlap check. The `same_as` edge in SPEC-003 Section 7 introduces a confidence enum (`definite`, `probable`, `possible`) that does not exist on identifier records in SPEC-002 -- creating an asymmetry where uncertain identity can be expressed at the edge level but not at the identifier level. SPEC-004's boundary reference hashing depends on public identifiers from SPEC-002, but there is no guidance on what happens when an entity's only identifiers transition from public to restricted (e.g., a `nat-reg` reclassified for a sole proprietorship under GDPR) -- the hash changes, breaking correlation across re-exports. And SPEC-005's enrichment lifecycle describes adding external identifiers to internal-only nodes, but does not address what happens to the merge graph when enrichment reveals that two previously distinct nodes share an identifier: the enrichment operation can retroactively invalidate a prior merge.

---

## Strengths

- **Composite identifier model as architectural foundation.** The array-of-identifiers design in SPEC-002 with no single mandatory scheme is the only correct approach. In my experience resolving identity across 500M+ entities, no single scheme ever covers more than 60% of a real supplier base.
- **Clean spec decomposition preserves identity model integrity.** SPEC-002 owns identifiers, SPEC-003 owns merge predicates, SPEC-001 owns graph structure. No conflation of concerns.
- **LEI lifecycle handling is production-grade.** The five-status model with merge behavior differentiation per status -- especially LAPSED as merge-valid and ANNULLED as merge-prohibited -- is more nuanced than most commercial implementations.
- **DUNS Family Tree mapping is operationally precise.** Five structural levels (Global Ultimate through Branch) mapped to correct OMTSF node types, with pragmatic handling of ambiguous HQ/branch status.
- **Canonical string format with test vectors.** The `scheme:authority:value` format with percent-encoding rules and concrete test vectors in SPEC-002 Section 4 enables byte-exact comparison, which is a prerequisite for deterministic merge and boundary reference hashing.
- **Merge algebra is formally correct.** Commutativity, associativity, idempotency with union-find in SPEC-003 ensures decentralized merge produces deterministic results regardless of file ordering or grouping.
- **Internal identifiers excluded from cross-file merge.** SPEC-003 Section 2 correctly excludes `internal` scheme from the identity predicate while SPEC-002 makes them first-class for within-file use. This is the right tradeoff.
- **GLEIF RA list versioning with snapshot cadence.** Decoupling validation from GLEIF's publication timing via quarterly snapshots in SPEC-002 Section 5.4 prevents external dependency from blocking file validation.
- **Extension scheme mechanism with reverse-domain notation.** SPEC-002 Section 5.2 enables ecosystem growth without fragmenting the core vocabulary.

---

## Concerns

- **[Critical] Identity predicate in SPEC-003 has no temporal overlap check, creating false merge risk across specs.** The identity predicate (SPEC-003 Section 2) checks scheme, value, and authority equality but ignores `valid_from`/`valid_to` defined in SPEC-002 Section 3. A DUNS number valid 2010-2015 for Entity A and the same DUNS reassigned 2020-2025 to Entity B will merge these unrelated entities. This is not hypothetical -- D&B reassigns DUNS numbers upon entity dissolution, and GLN reassignment is common in GS1. The temporal fields exist on identifier records but are invisible to the merge predicate. This is elevated to Critical because it is a correctness defect in the cross-spec contract: SPEC-002 provides temporal data that SPEC-003 silently ignores.

- **[Major] No confidence/verification metadata on identifier records creates an asymmetry with `same_as` edges.** SPEC-003 Section 7 defines confidence levels (`definite`, `probable`, `possible`) on `same_as` edges, but SPEC-002 identifier records carry no equivalent signal. A DUNS number verified against D&B's API and one self-reported on a supplier questionnaire are treated identically by the merge predicate. Risk-weighted merge, regulatory evidence reporting, and enrichment quality assessment all require this signal. The `same_as` confidence enum proves the spec authors understand the need for graduated certainty -- it should extend to identifiers.

- **[Major] Enrichment in SPEC-005 can retroactively invalidate prior merges with no reconciliation guidance.** SPEC-005 Section 5.2 describes an additive enrichment workflow: add external identifiers to internal-only nodes. But if enrichment reveals that two previously separate nodes share a DUNS, the graph must be re-merged. Conversely, if enrichment reveals that two nodes previously merged via DUNS actually have different LEIs, the merge may have been incorrect. SPEC-005 is informative, but the normative consequence -- that enrichment can change the merge graph -- is not acknowledged in SPEC-003.

- **[Major] Joint ventures and split-identity entities remain unmodeled across the suite.** The `ownership` edge in SPEC-001 Section 5.1 handles equity splits, but a 50/50 joint venture appears identically to a subsidiary with two co-investors. CSDDD Article 22 treats joint venture partners differently from parent-subsidiary relationships. No spec in the suite provides a `governance_structure` property or `joint_control` edge type. In the D&B Family Tree, joint ventures are a known modeling challenge -- they appear under multiple Global Ultimate trees with no shared-governance indicator.

- **[Major] Boundary reference stability across re-exports is fragile when identifier sensitivity changes.** SPEC-004 Section 4 hashes only `public` identifiers for boundary references. If a `nat-reg` identifier is reclassified from `public` to `restricted` (as the Security & Privacy Expert recommended for sole proprietorships), the hash input changes and the boundary reference becomes un-correlatable with prior exports. SPEC-004 does not address this interaction with SPEC-002 sensitivity defaults.

- **[Minor] No name normalization or fuzzy matching guidance anywhere in the suite.** SPEC-003 Section 8 recommends intra-file deduplication using "fuzzy name matching with address comparison" but provides no guidance on normalization rules (legal form removal, transliteration, abbreviation expansion). Name matching is the fallback when no shared external identifier exists, and without guidance, every implementation will produce different results.

- **[Minor] No cross-reference validation between identifier schemes on the same node.** SPEC-002 defines L3 rules for individual scheme validation but does not address cross-scheme consistency. If a node carries both `lei:X` and `nat-reg:Y`, GLEIF Level 1 data can verify whether LEI X is registered to the entity with registry number Y. L3-EID-03 mentions this but the rule is underspecified -- no guidance on what to do when cross-references conflict.

- **[Minor] SPEC-006 standards mapping does not reference ISO 5009 (Official Organizational Identifier).** ISO 5009, building on LEI infrastructure, is emerging as a broader organizational identifier standard. Its omission from the standards mapping creates a gap for forward-looking interoperability.

---

## Recommendations

1. **(P0) Extend the merge identity predicate in SPEC-003 to require temporal compatibility.** When both identifier records carry `valid_from`/`valid_to`, the predicate should require that the validity ranges overlap or that at least one range is open-ended (no `valid_to`). When temporal fields are absent, the current behavior (match on scheme+value+authority alone) should be preserved with an L2 warning recommending temporal annotation. This prevents false merges from reassigned identifiers while maintaining backward compatibility with files that omit temporal fields.

2. **(P1) Add confidence/verification fields to identifier records in SPEC-002.** Add optional `verification_status` enum (`verified`, `reported`, `inferred`, `unverified`) and optional `verification_date` (ISO 8601) to the identifier record structure. This aligns with the `same_as` confidence model in SPEC-003 and enables risk-weighted merge. The unknown-fields forward-compatibility clause in SPEC-002 Section 3 means older parsers will preserve these fields without breaking.

3. **(P1) Add enrichment-merge interaction guidance to SPEC-003 or SPEC-005.** Document that enrichment is not purely additive: adding an external identifier to a node may create new merge candidates or reveal that a prior merge was based on a reassigned identifier. Recommend that enrichment tooling re-evaluate merge groups after adding identifiers, and that the `merge_metadata` section (SPEC-003 Section 6) record whether the merge was performed pre- or post-enrichment.

4. **(P1) Add joint venture representation to SPEC-001.** Define a `governance_structure` property on `organization` nodes with enum values `sole_subsidiary`, `joint_venture`, `consortium`, `cooperative`. When `governance_structure` is `joint_venture`, multiple inbound `ownership` edges with combined percentage near 100% indicate shared control. This is lighter than a new edge type and captures the essential distinction for CSDDD Article 22.

5. **(P1) Document boundary reference stability constraints in SPEC-004.** State that once a boundary reference hash has been computed and shared, the sensitivity classification of the identifiers used in the hash computation should not change for the lifetime of that file. If sensitivity reclassification is necessary, the file should be re-exported with a fresh salt, and the old boundary references become un-correlatable by design.

6. **(P2) Add name normalization guidance to SPEC-003 Section 8.** Provide recommended (not required) normalization steps: trim whitespace, normalize Unicode (NFC), remove common legal form suffixes (GmbH, Ltd, Inc, S.A.), transliterate non-Latin scripts to Latin. Reference established approaches (e.g., the GLEIF entity name matching algorithm). This reduces implementation divergence in fuzzy deduplication.

7. **(P2) Add ISO 5009 to SPEC-006 standards mapping.** Reference ISO 5009 (Official Organizational Identifier) as a related standard building on LEI infrastructure, with a note that OMTSF's `lei` scheme is forward-compatible with ISO 5009 organizational identifiers.

---

## Cross-Expert Notes

- **For Graph Modeling Expert:** The temporal overlap gap in the identity predicate (my P0 recommendation) directly affects your merge algebra. Transitive closure across schemes remains correct, but without temporal guards, it can transitively link entities that were never contemporaneously identified by the same identifier. I propose we co-define a "temporally compatible identity predicate" that preserves the algebraic properties (commutativity, associativity, idempotency) while adding temporal safety. The key constraint: temporal compatibility must be transitive for the union-find to remain valid.

- **For Enterprise Integration Expert:** The enrichment-merge interaction I flag above has a concrete ERP consequence. When a SAP export produces internal-only nodes and a subsequent enrichment pass adds DUNS numbers, the enrichment may reveal that two vendor records (`LIFNR` V-100 and V-200) share a DUNS. The enrichment tool must either merge these nodes (changing the graph topology) or emit a `same_as` edge. SPEC-005 should provide explicit guidance for this scenario, ideally with a worked example showing the before and after states.

- **For Security & Privacy Expert:** The boundary reference stability concern I raise is the flip side of your recommendation to reclassify `nat-reg` sensitivity for sole proprietorships. Both recommendations are correct independently, but they interact: reclassifying an identifier from `public` to `restricted` changes the hash input for boundary references. The resolution is to treat sensitivity reclassification as a re-export event (fresh salt, new boundary references), which your anti-enumeration design already supports. This should be documented as an explicit interaction between SPEC-002 and SPEC-004.

- **For Regulatory Compliance Expert:** The joint venture modeling gap matters for CSDDD Article 22 analysis. Without a `governance_structure` indicator, a compliance tool cannot distinguish a 50/50 joint venture (where both parents share due diligence obligations) from a subsidiary with a minority co-investor (where only the majority parent has primary obligations). The `ownership` edge alone is insufficient because the legal governance relationship is not derivable from equity percentage alone.

- **For Supply Chain Expert:** Your recommendation for a confidence field on identifiers aligns with mine. I would add that the confidence hierarchy should be scheme-aware: an LEI match is inherently higher confidence than a DUNS match (LEI is verified annually by an accredited LOU; DUNS verification depends on D&B's internal processes and has no public audit trail). A scheme-level confidence ranking would help risk analysts calibrate their analysis without requiring per-identifier verification metadata on every record.
