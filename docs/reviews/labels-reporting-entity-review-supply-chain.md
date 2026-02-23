# Expert Review: Labels Mechanism and Reporting Entity Field

**Reviewer:** Supply Chain Expert (Supply Chain Visibility & Risk Analyst)
**Date:** 2026-02-18
**Topic:** Review of the labels mechanism (Section 8.4) and `reporting_entity` field additions to SPEC-001 and SPEC-003, implementing P0 recommendations R1 and R2 from the supply chain segmentation panel review.

---

## Assessment

As the panelist who originally flagged both the classification gap and the reporting entity gap, I consider these additions a solid implementation of the two P0 recommendations. The `reporting_entity` field directly resolves the ambiguity I identified with `tier` values: when Acme Manufacturing exports a file with `tier: 1` on its Bolt Supplies relationship, any downstream consumer or merge engine now knows that "tier 1" is relative to `org-acme`. The L2-GDM-04 validation rule that warns when `tier` is present without `reporting_entity` is exactly the right enforcement posture -- advisory rather than blocking, since many initial adopters will produce files incrementally.

The labels mechanism hits the right design point for real-world supply chain segmentation. The `{key, value}` pair model with optional `value` for boolean flags is flexible enough to carry Kraljic quadrant classifications (`com.acme.kraljic-quadrant: "strategic"`), regulatory scope tags (`de.bafa.lksg-priority`), diversity certifications (`org.nmsdc.certified: "2026"`), and business unit assignments (`com.acme.buying-org: "EKORG-1000"`) without overcomplicating the core data model. The reverse-domain namespacing convention is critical -- without it, I have seen firsthand how "risk-tier" defined by one organization's procurement team collides with "risk-tier" from another's compliance function during data exchange. The decision to keep labels out of identity predicates is correct: two files classifying the same supplier differently (one as "strategic", another as "routine") should merge cleanly rather than producing two separate supplier nodes.

My one substantive concern is about the *temporal dimension*. Supply chain classifications change -- a supplier's Kraljic quadrant shifts as market conditions evolve, a regulatory risk priority is reassigned after remediation. The current labels model is purely point-in-time (tied to `snapshot_date`), which is workable but means classification history requires retaining and comparing successive snapshots. This is consistent with the spec's broader snapshot-based temporal model, but it is worth noting explicitly in informative guidance because auditors under LkSG and [CSDDD](https://commission.europa.eu/business-economy-euro/doing-business-eu/sustainability-due-diligence-responsible-business/corporate-sustainability-due-diligence_en) will ask "when did you first classify this supplier as high-risk?"

---

## Strengths

- **`reporting_entity` resolves perspective ambiguity.** The `tier` property on `supplies` edges is now anchored. This was the most urgent semantic gap: under [LkSG](https://netzerocompare.com/policies/german-supply-chain-due-diligence-act-lksg), the distinction between direct (tier 1) and indirect suppliers determines different levels of due diligence obligation. An unanchored tier value is useless for regulatory compliance.
- **Set-union merge semantics for labels are clean and conflict-free.** The purely additive merge behavior (SPEC-003, Section 4, step 4) avoids the conflict resolution headaches that would arise if labels participated in property merge. When Acme's file tags Bolt Supplies as `com.acme.strategic-supplier` and a second-party file tags the same entity as `org.ecovadis.gold-rated`, the merged graph carries both classifications without any conflict record. This matches real-world behavior where different parties legitimately classify the same entity differently.
- **Namespaced keys prevent the collision problem I predicted.** The `SHOULD use reverse-domain notation` convention for custom keys, combined with reserving dot-free keys for future OMTS vocabularies, creates a clean namespace separation. This directly addresses the interoperability concern raised by all five panelists.
- **Boolean flags elegantly handle presence-based classifications.** Tags like `com.acme.critical-path` (no value) versus `com.acme.risk-tier: "high"` (with value) map naturally to how procurement teams actually think: some classifications are binary (is this a sole-source supplier? yes/no) and others are enumerated.
- **L1-GDM-05 validation prevents dangling `reporting_entity` references.** Requiring the referenced ID to exist and be an `organization` node avoids orphan references that would confuse consumers.
- **Advisory size limit of 100 labels is reasonable.** In my experience, even heavily classified suppliers rarely exceed 30-40 distinct classification dimensions. The 100 limit provides headroom while protecting parsers.

---

## Concerns

- **[Minor] No temporal validity on individual labels.** As noted in my assessment, real-world classifications have effective dates. A supplier classified as `com.acme.kraljic-quadrant: "strategic"` in Q1 may be reclassified to `"leverage"` in Q3. The current model relies on `snapshot_date` for temporal context, which is adequate but means classification change history requires snapshot-level diffing. Regulatory auditors under [LkSG](https://www.sedex.com/blog/top-5-key-points-about-the-german-supply-chain-act/) and CSDDD expect timeline-aware risk classification records.
- **[Minor] `reporting_entity` merge semantics could lose important perspective data.** SPEC-003 Section 6 states that when source files declare different reporting entities, the merged file SHOULD omit `reporting_entity`. This is semantically correct -- a merged graph is no longer single-perspective -- but downstream consumers need clear guidance on how to interpret `tier` values in a multi-perspective merged graph. If file A (from Acme's perspective) says Bolt is tier 1, and file B (from Bolt's perspective) says Steel Co is tier 1, the merged graph has two unrelated tier hierarchies. The provenance recording helps, but an informative note on tier interpretation in merged graphs would strengthen the spec.
- **[Minor] No guidance on label cardinality for the same key.** The spec does not state whether multiple labels with the same `key` but different `value` values are permitted on the same node. For example, can a supplier carry both `{key: "com.acme.commodity-category", value: "fasteners"}` and `{key: "com.acme.commodity-category", value: "raw-steel"}`? The set-union merge model implies yes (since `{key, value}` pairs are the identity unit), but explicit confirmation would prevent implementer confusion.
- **[Minor] `tier` property still limited to `supplies` edges only.** The panel review (M6) identified that [LkSG Section 2(7)](https://www.fieldfisher.com/en/locations/germany/insights/client-insight-update-lksg-und-csddd-januar-2026) explicitly includes subcontracting as a supply chain relationship subject to due diligence, and CSDDD applies to the full "chain of activities." While the labels mechanism could encode tier as `com.acme.tier: "2"` on a `subcontracts` edge, having `tier` as a first-class property only on `supplies` edges sends the wrong signal about regulatory scope. This was noted as P1 recommendation R5 in the panel report and remains unaddressed here.

---

## Recommendations

1. **(P1) Add an informative note in Section 8.4 clarifying that multiple labels with the same key but different values are permitted.** State explicitly that `{key, value}` is the atomic unit of identity for labels, so a node may carry `{key: "X", value: "A"}` and `{key: "X", value: "B"}` simultaneously. This is implied by the merge semantics but should be explicit.

2. **(P1) Add an informative note in SPEC-003 Section 6 on tier interpretation in multi-perspective merged graphs.** When a merged file omits `reporting_entity` and contains `tier` values from multiple perspectives, state that `tier` values SHOULD be interpreted in conjunction with the `merge_metadata` provenance to determine which reporting entity each tier is relative to. Consider recommending that merge engines annotate tier-carrying edges with a label like `omts.tier-perspective: "<reporting_entity_id>"`.

3. **(P1) Extend the `tier` property to `subcontracts`, `tolls`, `distributes`, and `brokers` edge types** per panel recommendation R5. This is a regulatory alignment issue: CSDDD Article 2 defines the "chain of activities" broadly, and [LkSG](https://www.roedl.com/en/insights/whats-new-german-supply-chain-act-legal-update-on-lksg-and-csddd/) does not distinguish supply relationship types when determining tier-based due diligence obligations.

4. **(P1) Define a recommended labels vocabulary in an informative appendix or SPEC-006** covering common supply chain segmentation dimensions: Kraljic quadrant, regulatory scope (LkSG, CSDDD, EUDR, UFLPA, CBAM), compliance status, supplier diversity, and approval status. This was panel recommendation R3 and is essential for interoperability -- without recommended keys, every adopter will invent their own, defeating the purpose of a standard classification mechanism.

5. **(P2) Consider adding optional `valid_from`/`valid_to` fields to individual label entries** for use cases requiring label-level temporal tracking (e.g., tracking when a supplier was first classified as high-risk for CSDDD compliance). This could be deferred to a future minor version but should be designed so it does not break the current set-union merge model.

---

## Cross-Expert Notes

- **For Graph Modeling Expert:** The single `labels` array (combining the Graph Modeling Expert's proposed separate `labels` and `annotations` arrays) is a pragmatic simplification. I would ask whether this impacts GQL/ISO 39075 alignment for graph database loading -- specifically, whether Neo4j and similar systems can efficiently index `{key, value}` pair arrays for the pattern-matching queries that supply chain analytics require (e.g., "find all organizations where labels contains `{key: 'com.acme.risk-tier', value: 'high'}`").

- **For Enterprise Integration Expert:** The labels mechanism now provides a concrete mapping target for the ERP segmentation fields identified in M4 (SAP `EKORG`, Oracle `ProcurementBUId`, D365 `VendorGroupId`). SPEC-005 should be updated with examples showing how these fields map to labels on `supplies` edges (e.g., `{key: "com.sap.ekorg", value: "1000"}`). This is panel recommendation R4.

- **For Regulatory Compliance Expert:** The `reporting_entity` field resolves the tier ambiguity concern, but the broader question of multi-entity corporate group compliance remains. Under CSDDD, a parent company's due diligence obligations extend to subsidiaries. A file produced from the parent's perspective with `reporting_entity` pointing to the parent may need to express tier relationships relative to multiple subsidiaries. This may require guidance in a future revision.

- **For Procurement Expert:** The labels mechanism can carry the `classifications` entries the Procurement Expert proposed (e.g., `{key: "org.unspsc", value: "31162800"}` for fasteners). However, the lack of a `label` (display name) field on label entries means that human-readable taxonomy names must be resolved by tooling, not carried in the file. This is a tradeoff worth acknowledging.

---

Sources:
- [EU CSDDD - European Commission](https://commission.europa.eu/business-economy-euro/doing-business-eu/sustainability-due-diligence-responsible-business/corporate-sustainability-due-diligence_en)
- [German LkSG Overview](https://netzerocompare.com/policies/german-supply-chain-due-diligence-act-lksg)
- [LkSG and CSDDD Update - Roedl](https://www.roedl.com/en/insights/whats-new-german-supply-chain-act-legal-update-on-lksg-and-csddd/)
- [LkSG Key Points - Sedex](https://www.sedex.com/blog/top-5-key-points-about-the-german-supply-chain-act/)
- [LkSG and CSDDD January 2026 Update - Fieldfisher](https://www.fieldfisher.com/en/locations/germany/insights/client-insight-update-lksg-und-csddd-januar-2026)
- [EUDR Due Diligence Statement Guide](https://www.coolset.com/academy/what-is-an-eudr-due-diligence-statement)
- [Supply Chain Knowledge Graphs - Arxiv](https://arxiv.org/html/2408.07705v1)
- [Neo4j Supply Chain Use Cases](https://neo4j.com/use-cases/supply-chain-management/)
