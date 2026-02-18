# Enterprise Integration Expert Review: OMTSF Spec Suite

**Reviewer:** Enterprise Integration Expert, Enterprise Systems Architect
**Date:** 2026-02-18
**Specs Reviewed:** OMTSF-SPEC-001 through OMTSF-SPEC-006 (all six specifications)
**Review Focus:** ERP export/import feasibility, master data alignment, EDI coexistence, data quality realities, batch vs. incremental update support

---

## Assessment

Twenty years of SAP S/4HANA, Oracle Cloud, and Dynamics implementations have taught me one lesson above all others: specifications that ignore ERP data realities die on the integration floor. The OMTSF spec suite, taken as a whole, does not ignore those realities. SPEC-005 provides actionable SAP field mappings (LFA1, EINA/EINE, EKKO/EKPO, MARA/MARC) that a competent ABAP developer could use to build a working extractor in a sprint. The `internal` identifier scheme with `authority` scoping (e.g., `sap-prod-100`) is the correct design -- it reflects how ERP systems actually store vendor identity, and it does not pretend that every vendor record arrives with a DUNS or LEI attached. The three-tier validation model (L1 structural, L2 completeness, L3 enrichment) solves the tension I flagged in both prior reviews: ERP data is messy, and a format that rejects files with incomplete metadata will never see production use.

The spec suite has matured substantially since the vision review. The `same_as` edge type in SPEC-003 Section 7 directly addresses the multi-client SAP duplicate vendor problem I deal with on every implementation. The enrichment lifecycle in SPEC-005 Section 5 accurately describes how enterprise data actually moves from internal-only to externally enriched -- this is not theoretical, it is the workflow I have watched play out in at least a dozen MDM programs. The `supplies`, `subcontracts`, and `tolls` edge types map cleanly to purchasing info records, subcontracting POs, and tolling arrangements that are first-class constructs in SAP MM and Oracle Procurement.

That said, several gaps remain that would block or seriously complicate production deployment at enterprise scale. The most consequential is the absence of any delta/patch mechanism. A manufacturer with 40,000 vendors and 200,000 purchasing info records cannot regenerate a complete `.omts` file on every vendor master change. ERP change pointers (SAP CDHDR/CDPOS, Oracle audit columns) produce incremental deltas -- the spec must consume them. The SAP Business Partner model omission (BUT000/BUT0ID) means the spec covers legacy vendor master only, not greenfield S/4HANA implementations where Business Partner is the sole master data object. And while the SAP mappings are solid, the Oracle SCM Cloud and D365 sections remain at field-name level without API endpoint references, making them insufficient for implementation.

---

## Strengths

- **`internal` identifier as first-class citizen.** The spec correctly treats ERP vendor numbers as legitimate identifiers rather than second-class placeholders. The `authority` scoping convention (`sap-mm-prod`, `oracle-scm-us`) maps directly to how multi-instance ERP landscapes are organized.
- **Three-tier validation aligns with ERP data quality reality.** A raw SAP export with only LIFNR identifiers passes L1. Adding VAT from STCD1/STCD2 moves toward L2. GLEIF enrichment reaches L3. This mirrors actual MDM maturity curves.
- **SAP field mappings in SPEC-005 are domain-accurate.** LFA1 to organization, EINA/EINE to supplies edges, EKKO BSART='UB' to subcontracts edges, MARA/MARC to good nodes. These are the correct tables and correct interpretations.
- **Multi-client deduplication guidance is practical.** SPEC-003 Section 8 directly addresses the 5-15% vendor duplication rate I see in every SAP landscape, with the correct recommendation to produce one node per legal entity carrying multiple `internal` identifiers.
- **`same_as` edge type handles the residual deduplication problem.** When an extractor cannot confidently deduplicate, it can declare probable equivalence rather than silently duplicating or silently merging.
- **Enrichment lifecycle accurately models enterprise workflows.** The progression from internal-only to partially enriched to fully enriched (SPEC-005 Section 5) is exactly how MDM enrichment programs work in practice.
- **`supplies`/`subcontracts`/`tolls` edge types map to ERP procurement constructs.** These are not abstract categories -- they correspond to PO types and info record categories that exist in every major ERP.
- **Sensitivity model on identifiers is correct for ERP data.** VAT numbers defaulting to `restricted` and internal IDs defaulting to `restricted` reflects actual enterprise data classification policies.

---

## Concerns

- **[Critical] No delta/patch mechanism for incremental updates.** Full-file re-export is infeasible for enterprise-scale vendor masters. SAP change document tables (CDHDR/CDPOS), Oracle audit columns, and D365 change tracking all produce incremental deltas. The spec has no way to express "add these 3 nodes, modify these 2 edges, remove this 1 node." Every ERP integration I have built in the past decade uses incremental extraction. This is a deployment blocker for any organization with more than ~5,000 vendors.
- **[Critical] SAP Business Partner model (BUT000/BUT0ID) not mapped.** All greenfield S/4HANA implementations use Business Partner as the sole entity master. BUT0ID stores typed identifier keys (TAX1, TAX2, DUNS, etc.) with explicit type codes -- actually a cleaner mapping to OMTSF identifier records than LFA1/STCD1/STCD2. Omitting this means the spec covers only legacy or migrated SAP systems, not new deployments.
- **[Major] Oracle SCM Cloud and D365 mappings are too shallow for implementation.** SPEC-005 Sections 3 and 4 list field names but not REST API endpoints, OData entities, query parameters, or pagination patterns. A developer implementing an Oracle extractor needs to know they are hitting `/fscmRestApi/resources/11.13.18.05/suppliers` with specific query fields, not just that `VENDOR_ID` exists somewhere.
- **[Major] No BOM/`composed_of` edge type.** Manufacturing ERPs center on bills of material (SAP CS/PP modules: STPO/STKO tables; Oracle BOM). Without a `composed_of` edge linking finished goods to components, the spec cannot represent what products are made of. This blocks material traceability for EUDR and embedded emissions calculations for CBAM.
- **[Major] No EDI coexistence positioning.** Enterprises exchange millions of EDI documents daily (EDIFACT ORDERS, DESADV, INVOIC; ANSI X12 850/856/810). The spec must explicitly state how OMTSF relates to EDI: complementary (network topology vs. transactional documents), not competing. Without this, procurement IT teams will see OMTSF as yet another integration burden.
- **[Major] `authority` naming convention for `internal` identifiers is informal.** SPEC-002 Section 5.1 recommends `{system-type}-{instance-id}` but does not enforce it. In practice, if one extractor uses `sap-prod` and another uses `sap-s4h-prd-100`, the same system produces non-comparable authority strings. This undermines intra-file deduplication and cross-file provenance tracking.
- **[Minor] No mapping from SAP purchasing organization (EKORG) or company code (BUKRS) to graph structure.** In multi-org SAP deployments, the same vendor may have different purchasing data across purchasing organizations. The spec should clarify whether these map to separate edges or edge properties.
- **[Minor] STCD1/STCD2 to `vat` scheme mapping oversimplifies.** SAP tax number fields store various identifier types depending on country (Brazilian CNPJ in STCD1, German USt-IdNr in STCD2, US EIN in STCD1). The mapping should note that scheme assignment depends on the country key and tax number category, not the field position.
- **[Minor] No guidance on ERP extraction scheduling or triggering.** Enterprises need to know when to extract: on vendor master change (event-driven via SAP Business Workflow or Oracle Business Events), on a schedule (daily/weekly batch), or on demand. This is operational guidance but belongs in the informative SPEC-005.

---

## Recommendations

1. **(P0) Define a delta/patch envelope specification.** Add a file-level `update_type` field (`snapshot` or `delta`). For delta files, define an operations array supporting `add`, `modify`, and `remove` operations on nodes and edges. Reference nodes/edges by their external identifiers (for cross-file operations) or graph-local IDs (for intra-file references). This is the single highest-priority gap for enterprise adoption.

2. **(P0) Add SAP Business Partner model mapping to SPEC-005.** Map BUT000 (entity header), BUT0ID (identification numbers with typed ID_TYPE keys), BP_CENTRALDATAPERSON, and CDS views (I_BusinessPartner, I_BPIdentification) alongside the existing LFA1 mappings. Note that BUT0ID's typed identification model maps more naturally to OMTSF identifier records than LFA1's positional STCD1/STCD2 fields.

3. **(P1) Expand Oracle SCM Cloud mapping to REST API level.** Reference the Supplier resource (`/fscmRestApi/resources/`), Supplier Sites child resource, and Purchase Order resource. Include key query parameters and pagination guidance. Similarly for D365: reference the VendVendorV2 OData entity and DirPartyTable joins.

4. **(P1) Add `composed_of` edge type to SPEC-001.** Properties: `quantity` (required, number), `unit` (required, string), `valid_from` (required), `valid_to` (optional). Direction: source = component `good`, target = parent `good`. Map to SAP BOM tables (STPO item, STKO header) and Oracle BOM_STRUCTURES_B.

5. **(P1) Publish explicit EDI coexistence statement in SPEC-005.** State that OMTSF represents supply network topology (who supplies whom, corporate structure, facility locations), while EDI handles transactional document exchange (purchase orders, advance shipping notices, invoices). They are complementary: EDI transaction data can confirm or update OMTSF supply edges, and OMTSF network context can enrich EDI partner resolution.

6. **(P1) Formalize the `authority` naming convention for `internal` identifiers.** Define a normative pattern: `{system-type}-{environment}-{instance}` where system-type is from a recommended vocabulary (`sap`, `oracle-scm`, `d365`, `ariba`), environment is `prod`/`test`/`dev`, and instance is the system-specific discriminator (SAP client number, Oracle business unit ID). Enforce format in L2 validation.

7. **(P2) Add STCD1/STCD2 disambiguation guidance to SPEC-005.** Note that SAP tax number fields are country-dependent: scheme assignment should be derived from LAND1 (country key) combined with the tax number type configuration in table T005-TAXBS, not assumed to be `vat` universally.

8. **(P2) Add extraction scheduling guidance to SPEC-005.** Document event-driven extraction (SAP change documents via CDHDR/CDPOS or Business Workflow, Oracle Business Events), scheduled batch extraction (ABAP report, Oracle BI Publisher), and on-demand extraction patterns. Note that delta extraction depends on the delta/patch specification from Recommendation 1.

---

## Cross-Expert Notes

- **To Graph Modeling Expert:** The `composed_of` edge type I am requesting creates material composition subgraphs that interact with your merge algebra. When two files both declare that "Product X is composed of Component Y," the merge engine must handle BOM-level deduplication. I recommend we co-define the edge identity predicate for `composed_of` edges, since BOM quantity differences across files (one supplier says 3 kg, another says 3.2 kg) should produce a conflict record, not parallel edges.

- **To Regulatory Compliance Expert:** The delta/patch mechanism has a direct regulatory implication you should review. When a supplier is removed from a supply network (a `remove` operation in a delta file), this constitutes a change to the due diligence scope under CSDDD Article 8. Delta files should probably carry a `reason` field on remove operations to distinguish "supplier terminated" from "data correction."

- **To Security & Privacy Expert:** Per the R2 panel report Cross-Domain Interaction #7, I want to reinforce that delta files are more sensitive than snapshots. A delta showing "3 new Chinese suppliers added this week" is more intelligence-dense than a 40,000-node snapshot. The delta/patch specification must inherit disclosure scope constraints, and I would recommend that delta files default to `restricted` sensitivity unless explicitly overridden.

- **To Standards Expert:** The EDI coexistence positioning I am requesting should reference specific EDI message types (EDIFACT PARTIN for partner information, PRODAT for product data) and note where OMTSF graph data can be derived from EDI streams. This bridges the gap between the transactional EDI world and the structural OMTSF world -- a mapping you are well positioned to formalize.

- **To Procurement Expert:** The `authority` naming convention formalization directly affects multi-ERP deduplication workflows. If your organization runs SAP for direct materials and Oracle for indirect, the same supplier appears with `internal:sap-prod-100:V-10234` and `internal:oracle-scm-us:SUP-4872`. Deduplication depends on consistent, parseable authority strings. I recommend we jointly define the naming convention in a way that procurement teams can enforce during extractor configuration.

- **To Supply Chain Expert:** Your request for quantitative properties on supply edges (volume, capacity, spend share) aligns with what ERP systems actually store. SAP purchasing info records (EINE) carry planned delivery time, minimum order quantity, and standard price. Oracle PO lines carry unit price and quantity. These should map to recommended (not required) properties on `supplies` edges, populated during extraction when available.
