# Open Source Strategy Expert Review: OMTS Spec Suite (Post-Panel Revision)

**Reviewer:** Open Source Strategy & Governance Lead
**Date:** 2026-02-18
**Scope:** Full spec suite (SPEC-001 through SPEC-006) + TSC Charter + LICENSE, post-panel-revision state
**Specs Reviewed:**
- OMTS-SPEC-001: Graph Data Model (normative)
- OMTS-SPEC-002: Entity Identification (normative)
- OMTS-SPEC-003: Merge Semantics (normative)
- OMTS-SPEC-004: Selective Disclosure (normative)
- OMTS-SPEC-005: ERP Integration Guide (informative)
- OMTS-SPEC-006: Standards Mapping (informative)
- `docs/governance/tsc-charter.md`
- `LICENSE` (MIT, Copyright BayFX 2026)

---

## Assessment

The OMTS specification suite has matured considerably since my initial review. The project has addressed the majority of the critical technical findings from the 11-expert panel: edge property serialization is now normatively resolved with a `"properties"` wrapper convention (SPEC-001 Section 2.1), advisory size limits are defined (SPEC-001 Section 9.4), the `composed_of` edge type and `consignment` node type address the BOM and lot-level traceability gaps, `verification_status` and `data_quality` metadata satisfy the confidence/provenance concern, the file integrity mechanism in SPEC-004 Section 6 covers SHA-256 digests and optional digital signatures, and the SPEC-004 test vectors now include expected SHA-256 outputs. The open questions in SPEC-003 have been resolved normatively. The `nat-reg` default sensitivity has been changed to `restricted`. These were the right priorities and the execution is clean.

From an open source governance perspective, however, the project's infrastructure gaps remain largely unchanged. The TSC Charter is well-constructed -- it represents some of the best early-stage governance scaffolding I have seen in a project at this maturity level. But the charter is scaffolding, not load-bearing structure. There is still no CONTRIBUTING.md, no DCO or CLA, no code of conduct, no reference implementation, no conformance test suite, and no public roadmap. The LICENSE file still reads `Copyright (c) 2026 BayFX` under MIT, covering both specifications and future code without differentiation. These are the gaps that determine whether OMTS becomes a multi-stakeholder standard or remains a single-company project with good documentation. The specification text is now strong enough that the bottleneck has shifted from "is the spec good enough?" to "can anyone outside BayFX participate?"

The adoption flywheel has no moving parts yet. Comparable projects that succeeded at OMTS's stage -- CloudEvents reached CNCF sandbox within months of its initial spec, OpenTelemetry launched with both a governance committee and reference SDKs in multiple languages, and Open Supply Hub built its community around a working platform with CC-BY-4.0 data licensing -- all established contributor infrastructure before or concurrent with their specification work. OMTS has six polished specifications and zero pathways for external participation. The informative specs (SPEC-005, SPEC-006) are strong adoption-enabling assets, particularly the SAP S/4HANA Business Partner mapping and the ISO 6523 ICD table, but they need a reference implementation to validate against and a contribution process to attract the ERP integration engineers who would build real extractors.

---

## Strengths

- **Panel findings addressed with discipline.** The post-panel commit resolves 15+ critical and major findings across all normative specs. The edge property wrapper, size limits, `composed_of` edge, `consignment` node, `verification_status`, `data_quality`, file integrity, test vectors, temporal merge predicate, attestation lifecycle states, and `nat-reg` sensitivity default were all addressed. This demonstrates responsive governance even in the absence of a formal TSC.
- **TSC Charter is production-grade.** The 5-9 member composition with balance requirements (Section 3.1), the lazy consensus with concrete review periods (Section 4.1), the bootstrap-to-permanent transition with a 6-month deadline (Section 7), the conflict-of-interest clause (Section 6), and the two-thirds supermajority for charter amendments (Section 8) follow established patterns from OASIS and CNCF governance models.
- **Scheme governance process (SPEC-002 Section 5.3) prevents vocabulary bloat.** Requiring a production deployment for core scheme inclusion, a 30-day public review, and TSC approval via lazy consensus is disciplined. The 90-day deprecation notice with 2-major-version grace period protects existing implementations. This is better governance than many OASIS technical committees achieve.
- **Normative/informative split is correct and well-executed.** SPEC-005 and SPEC-006 as informative guides provide essential implementation context without creating normative barriers. The ERP mapping tables in SPEC-005 are detailed enough to be directly actionable for SAP S/4HANA implementations.
- **Extension mechanism via reverse-domain notation** (SPEC-001 Section 8, SPEC-002 Section 5.2) enables community experimentation without TSC approval cycles. This is the pattern that enabled CloudEvents protocol bindings to proliferate before the spec reached 1.0.
- **Tiered validation (L1/L2/L3) enables incremental adoption.** A producer can ship structurally valid files on day one and enrich toward L2/L3 compliance over time. This directly reduces the adoption barrier.
- **GLEIF RA list snapshot strategy** (SPEC-002 Section 5.4) decouples OMTS validation from an external dependency -- a pragmatic decision that avoids the failure mode where GLEIF updates break all validators.
- **Cross-spec references are explicit and tabular.** Every spec declares prerequisites and downstream dependencies. This is frequently missing in early-stage multi-document standards.

---

## Concerns

- **[Critical] No CONTRIBUTING.md or DCO/CLA.** This remains the single largest barrier to community formation. Enterprise legal departments will not approve engineer contributions to a project without a DCO (lightweight, used by Linux kernel, CNCF projects) or CLA (heavyweight, used by OASIS, Apache). The TSC Charter defines who decides but no document defines how external contributors participate. Without this, the project cannot grow beyond its founders. Every week without a contribution process is a week potential contributors move on.
- **[Critical] Single-company copyright with MIT for specifications.** The LICENSE file assigns copyright to BayFX under MIT. For code, MIT is acceptable. For specifications intended as a multi-stakeholder standard, MIT is an anti-pattern because it permits proprietary forking without attribution. A competitor could fork the spec, modify it incompatibly, and call it "OMTS-compatible" with no obligation to acknowledge the original work. CC-BY-4.0 for specifications (requiring attribution) is the industry standard pattern (used by OpenAPI, AsyncAPI, CloudEvents under CNCF charter). Apache 2.0 for code provides an explicit patent grant that MIT lacks -- relevant because the merge algorithm in SPEC-003 and the boundary reference hashing in SPEC-004 could theoretically be subject to patent claims. The copyright notice should read "Copyright OMTS Contributors" to signal multi-stakeholder intent; BayFX retains credit as originator in project history.
- **[Critical] No conformance test suite or fixtures.** The specs now define 30+ validation rules with well-specified behavior (L1-GDM-01 through L1-GDM-04, L1-EID-01 through L1-EID-11, L1-SDI-01 through L1-SDI-03) and four test vectors in SPEC-004. But there are no `.omts` fixture files in the repository for L1 rule validation. Without a machine-executable test suite, conformance claims are unverifiable and two implementations will diverge on edge cases. This is the highest-leverage ecosystem enablement artifact: every third-party implementation needs test data.
- **[Major] No reference implementation or public roadmap.** The vision document describes `omts-rs` as a Rust reference implementation with WASM compilation. Six specs and a TSC charter later, there is no code, no repository, and no stated timeline. The post-panel revisions added significant complexity (temporal merge predicates, file integrity with optional signatures, `composed_of` edges, `consignment` nodes) that can only be validated through implementation. A specification without a reference implementation cannot validate its own feasibility.
- **[Major] No conformance clause definitions.** The specs define validation rules but do not define what "conformant producer," "conformant consumer," or "conformant validator" means as formal roles with testable obligations. For example: must a conformant consumer accept files with unknown extension edge types? (SPEC-001 Section 8.2 says yes, but this is not stated in a conformance clause.) Must a conformant producer include `file_salt`? (SPEC-001 Section 2 says yes, but there is no conformance section collecting these obligations.) Without these definitions, interoperability claims remain informal.
- **[Major] No adoption wedge identified.** SPEC-006 Section 3 maps six regulations (CSDDD, EUDR, LkSG, UFLPA, CBAM, AMLD) but there is no stated strategy for which regulation drives the first real-world deployment. The project should name a first use case with a target timeline. My recommendation remains German LkSG direct supplier reporting for automotive or chemical companies: LkSG is in force, it requires structured supply chain data, German enterprises have high standards adoption culture, and SPEC-005 already covers SAP S/4HANA which dominates German enterprise procurement.
- **[Minor] No code of conduct.** Standard expectation for open source projects seeking enterprise and foundation credibility. The Contributor Covenant (used by CNCF, Eclipse, and most modern open source projects) is a low-effort, high-signal addition.
- **[Minor] No RFC or specification change process for normative changes.** The TSC Charter (Section 2) defines authority over specific areas but does not describe the process for proposing new normative content (e.g., a new edge type in SPEC-001 or a change to merge behavior in SPEC-003). Is it a PR with a 30-day review? A separate RFC document? The scheme governance process in SPEC-002 Section 5.3 is well-defined; the equivalent process for spec-level changes is not.

---

## Recommendations

1. **(P0) Create CONTRIBUTING.md with DCO.** Define the contribution workflow (fork, branch, PR), adopt the Developer Certificate of Origin (DCO sign-off per commit), document how external contributors interact with TSC governance, and specify the review process for normative vs. informative changes. This is the prerequisite for community formation and eventual foundation hosting.

2. **(P0) Separate spec and code licensing.** Move specifications in `spec/` to CC-BY-4.0 (attribution required, share-alike not required). Plan Apache 2.0 for code repositories (explicit patent grant). Update the copyright notice to "Copyright OMTS Contributors." Add a `LICENSE-SPEC` file for the specifications and retain `LICENSE` for code. BayFX retains originator credit in the git history and project documentation.

3. **(P0) Publish conformance test fixtures.** For each L1 validation rule across SPEC-001, SPEC-002, and SPEC-004, produce at minimum one valid `.omts` file and one invalid `.omts` file with the expected validation result. Ship these as `tests/fixtures/l1-gdm-01-valid.omts`, `tests/fixtures/l1-gdm-01-invalid-dup-node-id.omts`, etc. This is the single highest-leverage ecosystem enablement action.

4. **(P1) Define formal conformance clauses.** Add a "Conformance" section to SPEC-001 that defines conformant producer, conformant consumer, and conformant validator roles. Example: "A conformant consumer MUST accept any file passing all L1 rules across SPEC-001 through SPEC-004, MUST preserve unknown fields during round-trip serialization, and MUST NOT reject files containing unknown extension types."

5. **(P1) Name the adoption wedge and publish a roadmap.** Pick one regulation and one industry vertical: German LkSG direct supplier reporting for automotive. Publish a public roadmap with milestones: (a) reference implementation parses and validates L1 rules, (b) SAP S/4HANA reference extractor produces valid `.omts` files, (c) first external organization produces a valid file from their own ERP data.

6. **(P1) Publish the `omts-rs` repository.** Even an empty repository with a README stating scope, planned crate structure, and a timeline signals commitment. The reference implementation is both a proof of feasibility and the seed of the ecosystem -- language bindings, CI integrations, and conformance tooling all depend on it.

7. **(P2) Evaluate foundation hosting.** Once the permanent TSC is constituted and 2-3 external contributors are active, evaluate hosting under the Linux Foundation, OASIS, or Eclipse Foundation. Foundation hosting provides legal infrastructure (CLA management, trademark protection), credibility with enterprise adopters, and event/marketing support. The CNCF model (used by CloudEvents, OpenTelemetry) is particularly relevant given the planned Rust/WASM toolchain.

8. **(P2) Plan language bindings.** Python (via PyO3 from `omts-rs`) and TypeScript (via WASM) are the highest-priority targets. Python covers the data engineering and analytics ecosystem; TypeScript covers the web tooling and browser validation use case described in the vision. Publish as separate repositories under the same governance umbrella.

9. **(P2) Establish a community extension scheme registry.** The reverse-domain extension mechanism (SPEC-001 Section 8, SPEC-002 Section 5.2) enables uncoordinated experimentation, but a lightweight registry (even a simple YAML file in the repository) of known extension schemes, edge types, and node types would prevent duplication and help the TSC identify candidates for promotion to core.

---

## Cross-Expert Notes

- **To Standards Expert:** The scheme governance process (SPEC-002 Section 5.3) is solid but should be generalized to cover all normative spec changes, not just scheme vocabulary. Consider whether OMTS's TSC process should align with OASIS Open Project Rules if foundation hosting is pursued -- OASIS requires entity CLAs and specification non-assertion covenants that the current MIT license does not provide.

- **To Systems Engineering Expert:** The conformance test suite plan (Recommendation 3) should produce machine-readable test results (e.g., TAP or JUnit XML) from the start. This enables CI integration for third-party implementations and automated conformance badges -- both critical for ecosystem trust. The test fixtures should be designed so that `omts-rs` can consume them as golden files.

- **To Enterprise Integration Expert:** The adoption wedge recommendation (LkSG + German automotive) directly leverages the SAP S/4HANA Business Partner mapping in SPEC-005 Section 2.4. A reference SAP extractor producing valid `.omts` files would be the single most impactful ecosystem deliverable. I recommend we collaborate on a "Quick Start: LkSG Supplier Export from SAP" guide as the first adoption-facing document.

- **To Security & Privacy Expert:** The licensing concern (Recommendation 2) intersects with the file integrity model in SPEC-004 Section 6. If files carry digital signatures and attestations, the legal framework under which those attestations are made matters. Apache 2.0's explicit patent grant protects implementors from patent claims on the validation and hashing algorithms. MIT provides no patent grant.

- **To Regulatory Compliance Expert:** The adoption wedge (LkSG first, CSDDD second) is driven by regulatory timeline: LkSG is in force today, CSDDD transposition begins in 2026. OMTS's regulatory alignment table (SPEC-006 Section 3) is a strong asset for the adoption pitch. If OMTS can demonstrate value for LkSG reporting, CSDDD adoption follows because the data model already covers CSDDD requirements.

- **To Graph Modeling Expert:** The TSC Charter (Section 2) now explicitly includes merge semantics (SPEC-003) in the scope of TSC authority and notes that changes require a major version increment. This addresses the governance-merge stability concern from the panel report. Merge semantics should be treated as a stability guarantee once v1.0 is declared -- any breaking change to identity predicates or transitive closure behavior must follow the 90-day extended review period.
