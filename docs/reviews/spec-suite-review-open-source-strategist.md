# Open Source Strategy Expert Review: OMTSF Spec Suite

**Reviewer:** Open Source Strategy & Governance Lead
**Date:** 2026-02-18
**Scope:** Full spec suite (SPEC-001 through SPEC-006) + TSC Charter + LICENSE
**Specs Reviewed:**
- OMTSF-SPEC-001: Graph Data Model (normative)
- OMTSF-SPEC-002: Entity Identification (normative)
- OMTSF-SPEC-003: Merge Semantics (normative)
- OMTSF-SPEC-004: Selective Disclosure (normative)
- OMTSF-SPEC-005: ERP Integration Guide (informative)
- OMTSF-SPEC-006: Standards Mapping (informative)
- `docs/governance/tsc-charter.md`
- `LICENSE` (MIT, Copyright BayFX 2026)

---

## Assessment

The OMTSF project has made substantial governance progress since the vision review, where I flagged the absence of any governance structure as critical. The TSC Charter (`docs/governance/tsc-charter.md`) directly addresses that finding: it establishes a 5-9 member committee, defines lazy consensus with concrete review periods (30 days for additions, 90 days for deprecations), specifies a bootstrap process with a 6-month deadline to constitute a permanent TSC after v1.0, and includes a conflict-of-interest clause. The scheme governance process in SPEC-002 Section 5.3 is particularly well designed -- it requires a production deployment before core scheme inclusion, which prevents the vocabulary from bloating with theoretical additions. The 6-spec decomposition into 4 normative and 2 informative documents is structurally sound and follows the pattern of successful standards (e.g., how W3C separates normative specs from implementation guides).

However, the project still has critical gaps in the contributor-facing infrastructure that will determine whether anyone outside BayFX ever contributes. There is no CONTRIBUTING.md, no Developer Certificate of Origin (DCO) or Contributor License Agreement (CLA), no conformance test suite, no reference implementation, and no code of conduct. The LICENSE file still reads `Copyright (c) 2026 BayFX` with MIT applied to everything -- spec and code alike. MIT for specifications is a known anti-pattern because it permits proprietary forks with no attribution obligation on derivative specifications (as opposed to CC-BY-4.0, which requires attribution). The copyright holder remains a single company, which sends the wrong signal to enterprise adopters evaluating whether to invest engineering effort in a format they do not control.

The adoption flywheel is not yet spinning. There is no identified first-mover use case, no public roadmap for a reference implementation, no language binding plan, and no conformance test suite plan. The informative specs (SPEC-005, SPEC-006) are solid foundational work for adoption -- the ERP mapping guide in particular will reduce the "how do I actually use this?" barrier -- but they need a reference implementation to validate against. A spec without a conformance test suite is a suggestion, not a standard.

---

## Strengths

- **TSC Charter is well-structured.** The bootstrap process (founders act as interim TSC, must constitute permanent TSC within 6 months of v1.0) is realistic and prevents indefinite single-company control. The 2-year renewable terms and removal-for-inactivity clause are standard good practice.
- **Scheme governance in SPEC-002 Section 5.3 is best-in-class for a project at this stage.** Requiring a production deployment for core scheme inclusion, a 30-day review for additions, and a 90-day notice for deprecations with 2-major-version grace period is disciplined and defensible.
- **The normative/informative split is correct.** Keeping ERP mapping (SPEC-005) and standards mapping (SPEC-006) as informative prevents them from becoming normative barriers to adoption while still providing essential implementation guidance.
- **Extension mechanism via reverse-domain notation** (SPEC-001 Section 8, SPEC-002 Section 5.2) is the right choice. It prevents namespace collision, allows experimentation without TSC approval, and follows the pattern proven by Java packages and Android permissions.
- **Tiered validation levels (L1/L2/L3)** across all normative specs enable incremental adoption. An organization can produce a structurally valid file on day one and enrich toward L2/L3 over time. This directly addresses the "strict validation blocks adoption" concern from the vision review.
- **The GLEIF RA list snapshot strategy** (SPEC-002 Section 5.4) decouples OMTSF validation from an external dependency's publication schedule -- a pragmatic decision that avoids the "GLEIF updated their list and broke all validators" failure mode.
- **Cross-spec references are explicit and consistent.** Every spec declares its prerequisites and downstream dependencies in a table. This is table-stakes for multi-document standards but frequently missing in early-stage projects.

---

## Concerns

- **[Critical] No CONTRIBUTING.md or DCO/CLA.** Without a contribution process, the project cannot accept external contributions with legal clarity. Enterprise legal departments will not approve their engineers contributing to a project that lacks a DCO (lightweight) or CLA (heavyweight). This is the single largest barrier to community formation. The TSC Charter defines who decides, but there is no document defining how anyone outside the founders participates.
- **[Critical] No conformance test suite or plan for one.** The specs define 30+ validation rules across L1/L2/L3 (e.g., L1-GDM-01 through L1-GDM-04, L1-EID-01 through L1-EID-11, L1-SDI-01 through L1-SDI-02) but there are no test vectors beyond the single boundary reference example in SPEC-004 Section 4. Without a test suite, conformance claims are unverifiable. Two implementations could both claim compliance and produce incompatible results.
- **[Critical] Single-company copyright with MIT for specifications.** The LICENSE file assigns copyright to BayFX. MIT permits proprietary forking without attribution. For code this is fine, but for a specification intended as a multi-stakeholder standard, this creates a governance credibility problem. A large enterprise evaluating OMTSF adoption will ask: "What stops BayFX from forking the spec into a proprietary version?" The answer under MIT is: nothing. CC-BY-4.0 for specs (requiring attribution) and Apache 2.0 for code (explicit patent grant) is the industry standard pattern (used by OpenAPI, AsyncAPI, CloudEvents).
- **[Major] No reference implementation or public roadmap for one.** The vision document describes `omtsf-rs` as a Rust reference implementation, but six specs later there is no code, no repository for the reference implementation, and no stated timeline. A specification without a reference implementation cannot validate its own feasibility. The merge semantics in SPEC-003 (transitive closure, union-find, commutativity/associativity/idempotency) are mathematically precise but untested in code. Edge cases will surface only during implementation.
- **[Major] No conformance clause definitions.** The specs define validation rules but do not define what it means to be a "conformant producer," "conformant consumer," or "conformant validator." SPEC-001 Section 9 and SPEC-002 Section 6 define rules, but there is no conformance clause stating: "A conformant producer MUST generate files that pass all L1 rules. A conformant consumer MUST accept files that pass all L1 rules and MUST preserve unknown fields during round-trip." Without these definitions, interoperability claims are informal.
- **[Major] No adoption wedge identified.** The standards mapping in SPEC-006 lists six regulations (CSDDD, EUDR, LkSG, UFLPA, CBAM, AMLD) and shows coverage, but there is no stated strategy for which regulation drives the first real-world deployment. A spec that tries to serve all six regulations simultaneously will ship none. The project needs a named first use case (e.g., "LkSG direct supplier reporting for German automotive OEMs") with a target timeline.
- **[Minor] No code of conduct.** Standard expectation for open source projects seeking enterprise and foundation credibility. Its absence signals the project has not yet invested in community norms.
- **[Minor] No RFC or specification change process beyond the TSC Charter.** The TSC Charter defines decision-making for scheme vocabulary changes but does not describe the process for proposing normative changes to the specs themselves (e.g., adding a new edge type to SPEC-001, modifying the merge procedure in SPEC-003). Is that a PR + 30-day review? A separate RFC document? Unstated.
- **[Minor] SPEC-003 Section 10 contains open questions in a normative spec.** Open Question #1 (edge merge strategy) is unresolved. Shipping a normative spec with "Open Questions" undermines confidence that the spec is implementation-ready. These should be resolved or explicitly deferred to a future version with a note that implementations MAY vary on this point.

---

## Recommendations

1. **(P0) Create CONTRIBUTING.md with DCO.** Define the contribution workflow (fork, branch, PR), adopt the Developer Certificate of Origin (used by Linux kernel, CNCF projects), and document how external contributors interact with the TSC governance process. This unblocks community formation and is a prerequisite for foundation hosting.

2. **(P0) Separate spec and code licensing.** Move specifications to CC-BY-4.0. Move code (when the reference implementation exists) to Apache 2.0 for the explicit patent grant. Update the COPYRIGHT notice to reflect multi-stakeholder intent (e.g., "Copyright OMTSF Contributors" rather than "Copyright BayFX"). BayFX retains credit as the originating organization in the project history; the copyright assignment signals that the project belongs to its community.

3. **(P0) Publish a conformance test suite plan with initial test vectors.** For each L1 validation rule, produce at minimum: one valid input, one invalid input, and the expected validation result. Ship these as `.omts` fixture files in the repository (e.g., `tests/fixtures/l1-eid-05-valid.omts`, `tests/fixtures/l1-eid-05-invalid-checkdigit.omts`). This is the single highest-leverage action for ecosystem enablement because every third-party implementation needs test data.

4. **(P1) Define conformance clauses.** Add a "Conformance" section to SPEC-001 defining conformant producer, conformant consumer, and conformant validator roles with testable obligations. Example: "A conformant consumer MUST accept any file that passes all L1 rules across SPEC-001 through SPEC-004. A conformant consumer MUST preserve unknown fields and unknown extension types during round-trip serialization."

5. **(P1) Name the adoption wedge.** Pick one regulation and one industry vertical as the first deployment target. My recommendation: German LkSG direct supplier reporting for automotive or chemical companies, because (a) LkSG is already in force, (b) it requires structured supply chain data, (c) the German market has high standards adoption culture, and (d) the ERP mapping guide (SPEC-005) already covers SAP, which dominates German enterprise.

6. **(P1) Publish a reference implementation roadmap.** At minimum: a public repository for `omtsf-rs`, a stated milestone for "parses and validates a valid .omts file against all L1 rules," and a timeline. The reference implementation is both a proof of feasibility and the seed of the ecosystem (language bindings, CI integrations, and tooling all depend on it).

7. **(P1) Resolve open questions in SPEC-003 before declaring the spec draft-final.** Open Question #1 (edge merge strategy) directly affects implementors. Either resolve it with a normative answer or explicitly state that edge-level merge identity is implementation-defined in this version, with a note that a future version will normatively specify it.

8. **(P2) Evaluate foundation hosting.** Once the permanent TSC is constituted and 2-3 external contributors are active, evaluate hosting under the Linux Foundation, OASIS, or Eclipse Foundation. Foundation hosting provides legal infrastructure (CLA management, trademark protection), credibility with enterprise adopters, and event/marketing support. The 6-month bootstrap deadline in the TSC Charter provides a natural decision point.

9. **(P2) Plan language bindings.** Python (via PyO3) and TypeScript (via WASM) are the highest-priority targets because they cover the data engineering and web tooling ecosystems respectively. Publish these as separate repositories under the same governance umbrella.

---

## Cross-Expert Notes

- **To Standards Expert:** The scheme governance process (SPEC-002 Section 5.3) is well-designed from an open source perspective but should cross-reference ISO/IEC JTC 1 or OASIS maintenance procedures if OMTSF ever seeks formal standards recognition. The TSC lazy consensus model aligns with IETF rough consensus but lacks the explicit "humming" escalation path -- the 14-day objection resolution window (TSC Charter Section 4.2) partially fills this gap.

- **To Systems Engineering Expert:** The conformance test suite plan (Recommendation 3) should be designed to produce machine-readable test results from the start. This enables CI integration for third-party implementations and automated conformance badges -- both critical for ecosystem trust.

- **To Enterprise Integration Expert:** The adoption wedge recommendation (LkSG + German automotive) directly leverages your ERP mapping work in SPEC-005. The SAP S/4HANA mapping is the highest-value section because SAP's market share in German enterprise is >70% for procurement. I recommend we collaborate on a "Quick Start: LkSG Supplier Export from SAP" guide as the first adoption-facing document.

- **To Security & Privacy Expert:** The copyright and licensing concern (Recommendation 2) intersects with your integrity model. If files carry signatures or attestations, the legal framework under which those attestations are made matters. Apache 2.0's explicit patent grant protects implementors from patent claims on the validation algorithms. MIT does not.

- **To Regulatory Compliance Expert:** The adoption wedge (LkSG first, CSDDD second) is driven by regulatory timeline: LkSG is in force today, CSDDD transposition deadlines begin in 2026. If OMTSF can demonstrate value for LkSG reporting, CSDDD adoption follows naturally because the data model already covers CSDDD requirements (SPEC-006 Section 3).

- **To Graph Modeling Expert:** The open question in SPEC-003 on edge merge strategy is a governance risk as much as a technical one. If two implementations choose different edge identity strategies during the open-question period, they will produce incompatible merge results. This is exactly the kind of interoperability fracture that a test suite prevents. Resolving this question normatively is a prerequisite for the conformance test suite covering merge behavior.
