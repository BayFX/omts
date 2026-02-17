# Expert Panel Report: OMTSF Vision Document

**Date:** 2026-02-17
**Document Reviewed:** `docs/vision.md`
**Panel Size:** 11 experts

---

## Panel Chair Summary

The OMTSF vision document presents a well-motivated, architecturally sound foundation for an open supply chain data interchange format. All eleven panelists agree that the problem diagnosis is correct -- supply chain data is trapped in proprietary silos, and a portable, vendor-neutral file format is the right intervention. The core design decisions -- a flat adjacency list of typed nodes and typed edges, goods as first-class graph objects, strict validation, local-only processing, and self-contained files -- received universal praise. These choices reflect genuine insight into both the technical requirements of graph serialization and the operational realities of multi-party supply chain data exchange.

However, the panel identified a critical structural gap that threatens the viability of the entire project: **the vision defers entity identification to the spec phase, but entity identity is the load-bearing foundation on which every other design decision rests**. Nine of eleven panelists independently flagged this as a critical or high-priority concern. Without a defined identifier strategy, the merge semantics cannot be specified, the graph model is incomplete, ERP integration is blocked, and regulatory compliance is unachievable. This is the single most important finding of this review.

Beyond identity, the panel converged on four additional systemic gaps: (1) no temporal dimension in the data model, flagged by five experts as essential for regulatory compliance and supply chain reality; (2) no governance model despite claims of vendor neutrality, flagged by the standards and open source experts as an adoption blocker; (3) no integrity or authenticity mechanism for files, flagged by the security, serialization, and regulatory experts; and (4) an underspecified formal graph model that leaves ambiguity about multigraph support, edge identity, and merge semantics.

The panel also surfaced productive tensions. The "strict validation" principle, praised by standards and security experts, was challenged by procurement and ERP experts who note that real-world supplier data is always incomplete. The resolution -- tiered validation levels -- was independently proposed by three panelists. Similarly, the deferral of domain-specific fields was seen as disciplined by some and as dangerously aggressive by others who argue that certain fields (geolocation, commodity codes, entity identifiers) are so universally required by regulation that they belong in the core schema.

---

## Panel Composition

| Name | Role | Key Focus Area |
|------|------|---------------|
| Dr. Amara Osei | Supply Chain Visibility & Risk Analyst | Multi-tier visibility, temporal modeling, regulatory data needs |
| Marcus Lindgren | Chief Procurement Officer | Supplier adoption, ERP feasibility, operational cost |
| Dr. Kenji Nakamura | Standards & Interoperability Specialist | GS1/ISO/UN alignment, identifier strategy, governance |
| Sofia Petrova | Senior Systems Engineer (Rust) | Implementation architecture, WASM, parsing safety, performance |
| Prof. Elena Varga | Graph Data Modeling & Algorithm Specialist | Formal graph model, merge semantics, serialization round-trip |
| Rajesh Krishnamurthy | Enterprise Systems Architect | ERP integration, master data mapping, delta updates |
| Dr. Isabelle Moreau | Regulatory Compliance Advisor | CSDDD, EUDR, LkSG, UFLPA, attestation, audit trails |
| Dr. Tomasz Kowalski | Data Format Architect | Serialization, schema evolution, file structure, compression |
| Danielle Okafor | Open Source Strategy & Governance Lead | Governance, licensing, adoption strategy, ecosystem |
| Dr. Yuki Tanaka | Data Security & Privacy Architect | Integrity, selective disclosure, threat modeling, local processing |
| Patricia Engstrom | Entity Identification & Corporate Hierarchy Specialist | DUNS/LEI, entity resolution, corporate hierarchy, M&A |

---

## Consensus Findings

These issues were independently raised by multiple experts, lending them the highest confidence:

1. **Entity identification is the #1 critical gap** (Osei, Lindgren, Nakamura, Varga, Krishnamurthy, Moreau, Okafor, Tanaka, Engstrom -- 9 of 11). Without a defined identifier strategy supporting LEI, DUNS, GLN, and composite identifiers, the merge-by-concatenation model is theoretical. This must be resolved before any other spec work.

2. **Temporal dimension is missing from the data model** (Osei, Lindgren, Moreau, Varga, Engstrom -- 5 of 11). Supply chains change constantly. Nodes and edges need `valid_from`/`valid_to` timestamps. Files need snapshot dates. Without temporal metadata, the format cannot support regulatory due diligence (which requires periodic re-assessment) or distinguish current from historical relationships.

3. **No governance model despite "vendor-neutral" claims** (Nakamura, Okafor -- 2 of 11, but both rated Critical). The repository is copyrighted by BayFX under MIT with no governance charter, no TSC, no CLA/DCO, and no IP policy. This gap between stated intent and actual structure will block enterprise and government adoption.

4. **No file integrity or authenticity mechanism** (Tanaka, Kowalski, Moreau -- 3 of 11). Files exchanged between parties need checksums and optional digital signatures. Without these, recipients cannot verify provenance or detect tampering -- a requirement for regulatory submissions.

5. **Merge semantics are underspecified** (Varga, Osei, Krishnamurthy, Nakamura, Engstrom -- 5 of 11). "Concatenate and deduplicate" requires a definition of identity that does not yet exist. The spec must define identity predicates for both nodes and edges, or merge behavior will be implementation-dependent.

6. **Strict validation vs. real-world data quality** (Lindgren, Krishnamurthy -- 2 of 11). ERP data is messy. If the format rejects files with incomplete metadata, adoption is blocked. The resolution: tiered validation levels (structural vs. completeness vs. enrichment).

---

## Critical Issues

Issues rated **[Critical]** by at least one expert. These must be addressed before proceeding.

| # | Issue | Flagged By |
|---|-------|-----------|
| C1 | **No entity identifier strategy** -- No reference to LEI, DUNS, GLN, or any existing identification standard. Merge, regulatory reporting, and ERP integration all depend on this. | Nakamura, Lindgren, Krishnamurthy, Moreau, Engstrom |
| C2 | **No temporal dimension** -- Static graph cannot represent supply chain changes over time. Required for regulatory due diligence and disruption analysis. | Osei, Moreau |
| C3 | **No data quality/confidence signals** -- No provenance metadata (who reported this? how verified?). Validated files give false confidence without factual reliability signals. | Osei, Moreau |
| C4 | **No governance model** -- BayFX copyright, MIT license, no charter, no TSC, no contribution process. Enterprise and government adopters will not build on a single-company standard. | Okafor, Nakamura |
| C5 | **Licensing mismatch** -- MIT for the spec allows forking; no patent grant. Spec should be CC-BY-4.0, code should be Apache 2.0. | Okafor |
| C6 | **No integrity/authenticity mechanism** -- No checksums, content hashes, or digital signatures. Files cannot be verified for tampering or provenance. | Tanaka, Kowalski |
| C7 | **No selective disclosure model** -- Access control dismissed as out of scope, but data compartmentalization is a format concern. Companies cannot share partial graphs without revealing their entire network. | Tanaka |
| C8 | **No formal graph model** -- Vision does not commit to simple graph vs. multigraph. Supply chains inherently require parallel edges (same supplier, different goods). Edge identity model is undefined. | Varga |
| C9 | **No resource limits for parser safety** -- No max file size, node count, or string length. Untrusted input parsing without bounds enables DoS. WASM heap is ~2-4 GB. | Petrova |
| C10 | **No supplier-side authoring strategy** -- Vision assumes systems export files but doesn't address how small suppliers without ERPs will produce `.omts` files. | Lindgren |
| C11 | **No incremental/delta update model** -- Large manufacturers cannot regenerate entire supply network files on every change. Delta extraction from ERPs is standard practice. | Krishnamurthy |
| C12 | **No mapping to existing standards** -- No mention of GS1 EPCIS, UN/CEFACT, ISO/TC 154, or W3C PROV-O. Risks reinventing existing vocabularies. | Nakamura |
| C13 | **Schema evolution deferred but foundational** -- Evolution rules (field numbering, unknown field handling, optionality) must be defined before the schema, not after. | Kowalski |
| C14 | **No dual-format strategy** -- Need both human-readable (JSON) and binary (CBOR/MessagePack) encodings. Single format forces a compromise that satisfies neither use case. | Kowalski |
| C15 | **Corporate hierarchy absent from data model** -- No ownership, control, or parent-subsidiary edges. Required for CSDDD and LkSG regulatory compliance. | Engstrom |

---

## Major Issues

Issues rated **[Major]** by at least one expert.

| # | Issue | Flagged By |
|---|-------|-----------|
| M1 | **Geolocation not mentioned** -- EUDR requires polygon-level coordinates. CBAM requires facility-level location. Must be core, not an extension. | Osei, Moreau |
| M2 | **No relationship type taxonomy** -- Subcontracting, tolling, licensed manufacturing, brokerage all have different regulatory implications. | Osei |
| M3 | **No ERP mapping guidance** -- No reference to SAP vendor master, Oracle supplier attributes, or standard ERP export mechanisms. | Krishnamurthy, Lindgren |
| M4 | **Domain fields deferred too aggressively** -- Commodity codes (HS/UNSPSC), certifications (ISO, SA8000), and country codes are universal regulatory requirements. | Lindgren, Moreau |
| M5 | **Strict validation blocks incremental adoption** -- Need "structurally valid but incomplete" files. | Lindgren, Krishnamurthy |
| M6 | **No conformance clauses** -- No definition of conformant producer, consumer, or validator. "Conformance is sufficient for interoperability" is untestable. | Nakamura |
| M7 | **Extensibility mechanism unspecified** -- Extensions (new node types, edge types, metadata fields) need must-understand vs. may-ignore rules. Without this, format fragments. | Nakamura, Petrova |
| M8 | **Serialization format has deep WASM implications** -- JSON vs. CBOR vs. rkyv affects zero-copy, binary size, and streaming parse. Rust team must co-decide. | Petrova |
| M9 | **No `no_std` support mentioned** -- Core crate should target `#![no_std] + alloc` for true WASM portability. Retrofitting is painful. | Petrova |
| M10 | **"Data stays local" not architecturally enforced** -- CLI could make network calls unless built without network-capable dependencies. Needs verifiable no-network property. | Tanaka |
| M11 | **No threat model** -- No enumeration of adversaries or attack scenarios for file exchange. | Tanaka |
| M12 | **No magic bytes or content-type registration** -- Self-contained file format needs magic bytes for detection and IANA media type registration. | Kowalski |
| M13 | **No content integrity model** -- No checksums or content hashes in file header. | Kowalski |
| M14 | **Compression not designed** -- Block-level vs. whole-file compression affects random access and streaming. Interacts with every other format decision. | Kowalski |
| M15 | **No CLA/DCO** -- No mechanism to ensure contributions are properly licensed. Blocks enterprise contributors and foundation hosting. | Okafor |
| M16 | **No adoption strategy** -- No identified first movers, target use cases, or path from reference implementation to real-world usage. | Okafor |
| M17 | **No ecosystem plan** -- No language bindings, conformance test suite, or certification process for third-party implementations. | Okafor |

---

## Minor Issues

| # | Issue | Flagged By |
|---|-------|-----------|
| m1 | Circular flows (recycling, reverse logistics) unaddressed | Osei |
| m2 | Merge conflict resolution deferred but critical for procurement workflows | Lindgren |
| m3 | No mention of data freshness or temporal validity | Lindgren |
| m4 | No versioning semantics stated (semantic versioning? forward compatibility?) | Nakamura |
| m5 | No error reporting contract (machine-readable vs. human-readable) | Petrova |
| m6 | No fuzzing or adversarial input testing commitment | Petrova |
| m7 | Graph directionality constraints not stated (DAG vs. cyclic) | Varga |
| m8 | "Adjacency list" terminology may confuse graph theory community | Varga |
| m9 | No EDI coexistence positioning | Krishnamurthy |
| m10 | No canonical encoding for deterministic diffing and signatures | Kowalski |
| m11 | No RFC or specification development process | Okafor |
| m12 | No code of conduct or contribution guide | Okafor |
| m13 | No encryption-at-rest envelope for files on shared drives/email | Tanaka |
| m14 | Merge from different trust domains has security implications | Tanaka |
| m15 | "Facilities" conflated with "organizations" -- different entity types | Engstrom |

---

## Consolidated Recommendations

### P0 -- Immediate (before spec work begins)

| # | Recommendation | Originated By |
|---|---------------|--------------|
| P0-1 | **Define composite entity identifier model.** Support LEI, DUNS, GLN, national registry numbers, tax IDs, and internal system IDs. Allow multiple identifiers per node. Define equivalence rules for merge. | Engstrom, Nakamura, Lindgren, Krishnamurthy, Moreau |
| P0-2 | **Introduce temporal metadata.** `valid_from`, `valid_to`, `last_verified`, `snapshot_date` on nodes, edges, and file header. Non-negotiable for regulatory use. | Osei, Moreau, Engstrom |
| P0-3 | **Define data provenance/confidence structure.** `reported_by`, `confidence` (confirmed/reported/inferred/unverified), `verification_method`, `assertion_date` attachable to any node or edge. | Osei, Moreau |
| P0-4 | **Publish governance charter.** TSC, decision-making process, IP policy, contribution process. Consider CNCF governance template or TODO Group principles. | Okafor, Nakamura |
| P0-5 | **Separate spec and code licensing.** Spec: CC-BY-4.0. Code: Apache 2.0 (explicit patent grant). Adopt DCO for contributions. | Okafor |
| P0-6 | **Define formal graph model.** Commit to directed labeled multigraph with attributed nodes and edges, both carrying independent identifiers. Reference ISO GQL (ISO/IEC 39075) Property Graph Model. | Varga |
| P0-7 | **Define resource limits in spec.** Max file size, node count, edge count, string length. Parser must enforce during streaming deserialization, not post-hoc. | Petrova |
| P0-8 | **Define magic bytes and file header structure.** 4-8 byte magic sequence, format version, encoding type, flags field. Cheap now, expensive to change later. | Kowalski |
| P0-9 | **Commit to dual-encoding strategy.** Human-readable (JSON) + binary (CBOR or MessagePack). File header encoding type field distinguishes them. | Kowalski |
| P0-10 | **Define schema evolution rules before defining the schema.** Field identification (name vs. number), optionality rules, unknown field handling. | Kowalski |
| P0-11 | **Publish explicit standards mapping.** For each OMTSF concept, document corresponding GS1, UN/CEFACT, and ISO construct. State whether OMTSF reuses, extends, or diverges. | Nakamura |
| P0-12 | **Define supplier authoring strategy.** Document how small suppliers without ERPs produce valid files. WASM-powered web form outputting `.omts`. | Lindgren |
| P0-13 | **Include Rust implementation team in serialization format decision.** Joint decision matrix: JSON, CBOR, MessagePack, FlatBuffers, rkyv across readability, zero-copy, WASM size, streaming. | Petrova |

### P1 -- Before v1

| # | Recommendation | Originated By |
|---|---------------|--------------|
| P1-1 | **Make geolocation a core node attribute.** WGS 84 coordinates on facility nodes. Polygon geometries for EUDR land parcels. | Osei, Moreau |
| P1-2 | **Add corporate hierarchy as core edge types.** Ownership (with percentage), operational control, legal parentage. Reference GLEIF Level 2 data. | Engstrom |
| P1-3 | **Define relationship type taxonomy.** Direct supply, subcontracting, tolling, licensed manufacturing, brokerage, logistics. | Osei |
| P1-4 | **Implement tiered validation levels.** Level 1: structural (graph integrity). Level 2: completeness (recommended fields). Level 3: enrichment (cross-references). | Lindgren, Krishnamurthy |
| P1-5 | **Add file integrity mechanism.** SHA-256 content hash in file header. Optional COSE Sign1 signature envelope. | Tanaka, Kowalski |
| P1-6 | **Introduce subgraph projection concept.** Valid subset files with boundary markers. Opaque hashed references for redacted nodes. | Tanaka |
| P1-7 | **Specify extension mechanism.** Must-understand vs. may-ignore rules. Namespace-qualified to prevent collision. | Nakamura, Petrova |
| P1-8 | **Design core crate as `#![no_std] + alloc`.** Forces WASM-clean dependencies from day one. | Petrova |
| P1-9 | **Publish ERP integration guides.** Reference mappings for SAP S/4HANA, Oracle SCM Cloud, Microsoft Dynamics 365. | Krishnamurthy, Lindgren |
| P1-10 | **Include minimal regulatory fields in core schema.** ISO 3166 country codes, HS/CN commodity codes, ISIC/NACE sector codes. | Moreau, Lindgren |
| P1-11 | **Define conformance levels.** Producer, consumer, and validator conformance with testable assertion sets. | Nakamura |
| P1-12 | **Commit to continuous fuzz testing.** `cargo-fuzz` targets for every deserialization entry point. CI-integrated with growing corpus. | Petrova |
| P1-13 | **Publish lightweight threat model.** Enumerate: malicious file injection, topology inference, tampering in transit, unauthorized re-sharing. | Tanaka |
| P1-14 | **Enforce no-network in CLI.** Build without network I/O libraries. CI check auditing dependency tree. | Tanaka |
| P1-15 | **Design delta/patch file format.** Lightweight envelope marking nodes/edges as added/modified/removed relative to prior version. | Krishnamurthy |
| P1-16 | **Define canonical encoding for human-readable format.** RFC 8785 (JCS) or custom canonicalization for deterministic diffing and signatures. | Kowalski |
| P1-17 | **Define adoption wedge.** Target EU CSDDD/LkSG compliance reporting with 2-3 concrete early adopters. | Okafor |
| P1-18 | **Plan language bindings.** Python (PyO3), JavaScript/TypeScript, Java on the roadmap. | Okafor |
| P1-19 | **Design extensibility mechanism concurrently with core types.** Prototype typed enum vs. opaque Value map in Rust. Evaluate against serde round-trip, zero-copy, WASM size. | Petrova |
| P1-20 | **Specify identifier equivalence rules for merge.** Shared external identifier = merge candidate. Define matching logic explicitly. | Engstrom |

### P2 -- Future

| # | Recommendation | Originated By |
|---|---------------|--------------|
| P2-1 | Add regulatory mapping layer as first-class extension (EUDR, UFLPA, CBAM markers). | Osei |
| P2-2 | Explicitly state directed graph may contain cycles. Ensure runtime handles cyclic graphs. | Osei, Varga |
| P2-3 | Develop cost-of-adoption model for procurement leaders. | Lindgren |
| P2-4 | Engage GS1 and UN/CEFACT for liaison/review. | Nakamura |
| P2-5 | Define machine-readable error output format (JSON Lines with byte spans). | Petrova |
| P2-6 | Consider hyperedge/n-ary relationship support for composite relationships. | Varga |
| P2-7 | Use precise graph terminology ("node-link representation" not "adjacency list"). | Varga |
| P2-8 | Define explicit EDI coexistence position. | Krishnamurthy |
| P2-9 | Publish regulatory alignment matrix mapping regulations to schema elements. | Moreau |
| P2-10 | Evaluate block-level compression with zstd. Per-section compression for partial access. | Kowalski |
| P2-11 | Evaluate foundation hosting (Linux Foundation, OASIS, Eclipse). | Okafor |
| P2-12 | Establish RFC process for spec changes. | Okafor |
| P2-13 | Specify optional encryption envelope (age or COSE). | Tanaka |
| P2-14 | Add per-node provenance metadata for post-merge auditability. | Tanaka |
| P2-15 | Define canonical entity reference format (`lei:XXX`, `duns:YYY`, `gb-coh:ZZZ`). | Engstrom |
| P2-16 | Register IANA media types (`application/vnd.omtsf+json`, `application/vnd.omtsf+cbor`). | Kowalski |

---

## Cross-Domain Interactions

The most valuable insights from a multi-expert review are the interdependencies between domains. Key interactions identified:

1. **Identity + Graph Model + Merge (Engstrom + Varga + Nakamura):** The identifier strategy determines whether merge is tractable. The formal graph model determines whether edges need independent IDs. Together, these three decisions form a single architectural unit that must be co-designed.

2. **Identity + ERP Integration (Engstrom + Krishnamurthy):** ERP systems use internal vendor numbers. The identifier model must support system-local IDs alongside global identifiers, or enterprise adoption requires a translation layer that defeats the purpose.

3. **Serialization + Rust Implementation (Kowalski + Petrova):** The dual-encoding decision doubles the parser surface and fuzzing requirement. The extensibility mechanism determines Rust type design (enum vs. HashMap). These must be joint decisions.

4. **Temporal Model + Graph Algorithms (Osei + Varga + Moreau):** Temporal validity on edges creates a time-varying graph. Runtime analysis must support temporal queries ("was there a valid path on this date?"). This affects both the data model and the `petgraph` graph structure choice.

5. **Strict Validation + Data Quality (Nakamura + Lindgren + Krishnamurthy):** The tension between strict validation and messy ERP data is resolved by tiered validation levels, but the tier definitions must satisfy both the standards community (who want rigor) and the procurement community (who need gradual onboarding).

6. **Security + Serialization + Regulatory (Tanaka + Kowalski + Moreau):** File integrity (checksums, signatures) requires canonical encoding. Attestation metadata increases per-element payload. Privacy-sensitive fields (auditor names in provenance data) intersect with GDPR. These three domains must coordinate.

7. **Governance + Standards Alignment + Adoption (Okafor + Nakamura + Lindgren):** Enterprise adoption requires governance credibility. Standards body alignment requires governance structure. Supplier adoption requires low-barrier authoring tools. All three feed the adoption flywheel.

8. **Selective Disclosure + Identity (Tanaka + Engstrom):** Hashed identifiers at graph boundaries for redacted nodes are only private if the identifiers are not predictable. The identifier scheme must support opaque, non-reversible references.

---

## Individual Expert Reports

### Dr. Amara Osei -- Supply Chain Visibility & Risk Analyst

#### Assessment

The OMTSF vision accurately diagnoses the core problem I have spent the better part of two decades fighting: supply chain data is fragmented, proprietary, and functionally opaque beyond tier-1. The decision to build a file format rather than yet another platform is strategically sound. In my experience leading transparency programs at a Fortune 100 manufacturer, every platform project eventually hit the same wall -- suppliers refused to onboard to another portal. A portable file that can be emailed, version-controlled, or dropped into a regulatory submission sidesteps that adoption barrier entirely. The flat graph model (typed nodes, typed edges, goods as first-class objects) is the right foundational abstraction for supply networks.

However, the vision as written reflects a clean, structural view of supply chains that does not yet grapple with the messiness of real-world supply data. Supply chains are not static graphs -- they are temporal, probabilistic, and riddled with incomplete information. The vision must acknowledge this uncertainty at the model level, not just as a tooling concern, or the format risks being too idealized for practitioners to trust.

The regulatory alignment story is promising but underspecified. The EU Deforestation Regulation (EUDR) requires geolocation coordinates for plots of land where commodities were produced. The German LkSG requires risk analysis across the entire supply chain. The US UFLPA requires evidence of supply chain mapping for goods from Xinjiang. Each of these has specific data requirements that the format must be capable of carrying.

#### Strengths
- Goods as first-class graph objects -- correctly separates physical network from commodity flows
- Flat adjacency list with merge by concatenation -- reflects how supply chain data actually accumulates
- Data stays local -- non-negotiable for adoption
- Self-contained, version-stamped files -- critical for regulatory submissions
- Graph analysis as a runtime concern -- clean separation

#### Concerns
- **[Critical]** No temporal dimension in the data model
- **[Critical]** No mechanism for data quality or confidence signals
- **[Major]** Geolocation is not mentioned
- **[Major]** No representation of relationship types beyond directed edges
- **[Major]** Merge model underspecified for conflicting partial views
- **[Minor]** Circular flows unaddressed

#### Recommendations
1. (P0) Introduce temporal model: `valid_from`, `valid_to`, `snapshot_date`
2. (P0) Define data provenance and confidence metadata
3. (P1) Make geolocation a core node attribute
4. (P1) Develop canonical identifier strategy
5. (P1) Define relationship type taxonomy
6. (P2) Add regulatory mapping layer
7. (P2) Explicitly state graph may contain cycles

---

### Marcus Lindgren -- Chief Procurement Officer

#### Assessment

The problem statement is accurate and directly reflects what my team fights with daily: fragmented supplier data trapped in SAP MM, Ariba, spreadsheets, and PDF questionnaires. The idea that the missing piece is a file format rather than yet another platform resonates. What we lack is a credible, vendor-neutral interchange format.

That said, the vision is written from an engineering-architecture perspective. What it does not address -- and what will determine adoption or failure -- is the operational reality of who produces these files, what it costs them, and how the format fits into procurement workflows that already exist.

#### Strengths
- Problem diagnosis is correct
- "The format is the contract" eliminates bilateral mapping tables
- Flat adjacency list is pragmatically sound for procurement
- Data stays local is essential for legal/infosec approval
- Goods as first-class objects models how procurement works
- Self-contained file eliminates "which template version?" problem

#### Concerns
- **[Critical]** No discussion of supplier-side authoring
- **[Critical]** No mention of entity identification standards
- **[Major]** Strict validation may block incremental adoption
- **[Major]** No ERP integration strategy
- **[Major]** Domain-specific fields deferred too aggressively
- **[Minor]** Merge semantics deferred but critical for procurement
- **[Minor]** No mention of data freshness or temporal validity

#### Recommendations
1. (P0) Define supplier authoring strategy (WASM-powered web form)
2. (P0) Adopt/reference existing entity identification standards
3. (P1) Introduce validation profiles or levels
4. (P1) Publish ERP integration guides
5. (P1) Include minimal compliance-relevant fields in core schema
6. (P2) Define temporal metadata on nodes and edges
7. (P2) Develop cost-of-adoption model

---

### Dr. Kenji Nakamura -- Standards & Interoperability Specialist

#### Assessment

The existing standards landscape is fragmented: GS1 EPCIS 2.0 handles event-level traceability but not network topology; UN/CEFACT defines document semantics but not graph interchange; ISO 28000-series covers security management but not structural data exchange. There is indeed a gap for a lightweight, graph-based, file-level supply network interchange format. The vision correctly positions OMTSF in that gap.

However, the vision document is conspicuously silent on how OMTSF relates to the standards that already occupy adjacent territory. A format that does not explicitly define its relationship to GS1 identification keys, to LEI for legal entities, and to EPCIS event semantics will either reinvent those vocabularies poorly, or leave each implementer to map ad hoc.

#### Strengths
- Correct problem identification -- gap between event-level and network topology exchange
- Flat adjacency list avoids nested XML structures
- Goods as first-class nodes
- Validation as a first-class concern
- Local-only processing and self-contained document model

#### Concerns
- **[Critical]** No identifier strategy defined or referenced
- **[Critical]** No mapping to or acknowledgment of existing standards (EPCIS, CBV, UN/CEFACT, PROV-O)
- **[Major]** No governance model specified
- **[Major]** Conformance clauses absent from roadmap
- **[Major]** Extensibility model mentioned but unspecified
- **[Minor]** No mention of versioning semantics

#### Recommendations
1. (P0) Define identifier strategy before spec work begins
2. (P0) Publish explicit standards mapping document
3. (P1) Establish governance charter before v1
4. (P1) Define conformance levels in spec
5. (P1) Specify extension mechanism (must-understand vs. may-ignore)
6. (P2) Engage GS1 and UN/CEFACT early

---

### Sofia Petrova -- Senior Systems Engineer (Rust)

#### Assessment

The decision to use a flat adjacency list as the canonical data model is the single most important choice in this document, and it is the right one. A flat list of typed nodes and typed edges maps directly to `Vec<Node>` and `Vec<Edge>` in Rust, which means parsing is a linear scan, validation is a hash-map lookup pass, and serialization is straightforward with serde.

Where the vision needs more specificity is at the boundary between "data model" and "implementation." Several design decisions that will profoundly affect the Rust implementation -- identifier representation, extensibility mechanism, size limits, and error reporting contract -- are deferred to "spec work." These decisions should be made with the implementation constraints in mind.

#### Strengths
- Flat adjacency list eliminates recursive parsing, simplifies ownership, enables zero-copy
- Self-contained document with embedded schema version
- "Data stays local" directly justifies WASM target
- Mandatory validation -- parser and validator co-designed
- Goods as first-class nodes simplifies type system
- Explicit scope boundaries keep implementation surface tractable

#### Concerns
- **[Critical]** No discussion of size limits or resource bounds
- **[Major]** Serialization format decision has deep WASM implications
- **[Major]** No mention of `no_std` or alloc-only support
- **[Major]** Extensibility model deferred but architecturally load-bearing
- **[Minor]** No error reporting contract
- **[Minor]** No fuzzing or adversarial input commitment

#### Recommendations
1. (P0) Define resource limits in spec
2. (P0) Include Rust team in serialization format decision
3. (P1) Design core crate as `#![no_std] + alloc`
4. (P1) Design extensibility concurrently with core types
5. (P1) Commit to continuous fuzz testing from first parser
6. (P2) Define machine-readable error output format

---

### Prof. Elena Varga -- Graph Data Modeling & Algorithm Specialist

#### Assessment

The OMTSF vision articulates a sound core insight: supply chain networks are graphs, and graph exchange requires a well-defined serialization format with formal identity and merge semantics. The decision to model goods as first-class nodes is the single most important graph-modeling choice in the document. It transforms the data model from a simple bipartite graph into a richer structure where a facility node can participate in multiple distinct commodity subgraphs.

However, the vision document leaves critical graph-modeling questions unresolved: multigraph support, edge identity, and the formal type system. The document is silent on whether two edges of the same type can exist between the same node pair, and whether edges are identified by endpoints or by independent IDs.

#### Strengths
- Goods as first-class nodes enables commodity-scoped subgraph extraction
- Flat adjacency list aligns with proven formats (JSON-Graph, GML)
- Self-contained document with embedded schema version
- Validation as first-class concern
- Explicit scope boundaries

#### Concerns
- **[Critical]** No formal graph model definition (simple vs. multigraph)
- **[Critical]** Edge identity model undefined
- **[Major]** "Merge is concatenation plus deduplication" is an oversimplification
- **[Major]** No mention of multigraph or parallel edges
- **[Minor]** No mention of graph directionality constraints (DAG vs. cyclic)
- **[Minor]** "Adjacency list" terminology may cause confusion

#### Recommendations
1. (P0) Define formal graph model (directed labeled multigraph with independent IDs)
2. (P0) Define identity model for nodes and edges
3. (P1) Adopt or adapt existing graph serialization schema (evaluate JSON-Graph)
4. (P1) Specify whether graph is DAG or permits cycles
5. (P1) Define merge semantics at identity level in spec
6. (P2) Consider hyperedge/n-ary relationship support
7. (P2) Use precise graph terminology

---

### Rajesh Krishnamurthy -- Enterprise Systems Architect

#### Assessment

The data *is* trapped. I have seen procurement teams at Fortune 500 manufacturers maintain their tier-2 and tier-3 supplier maps in Excel files emailed between sourcing managers -- not because they lack systems, but because no system speaks the same language. The vision correctly identifies that the missing piece is an interchange format, not another platform.

That said, the vision underestimates the gravity of the ERP integration challenge. If OMTSF cannot be populated from the messy reality of ERP master data -- through IDocs, OData feeds, BAPI extracts, or flat-file exports -- then it will remain an elegant specification that no one uses.

#### Strengths
- Goods as first-class nodes maps to material master / vendor master separation
- Flat adjacency list is critical for ERP export feasibility
- Self-contained document eliminates version mismatch problems
- Local-only processing is non-negotiable for enterprise adoption
- Scope boundaries are well-drawn

#### Concerns
- **[Critical]** No identity model specified
- **[Critical]** No incremental/delta update mechanism
- **[Major]** No mapping guidance to ERP master data structures
- **[Major]** "Strict validation" conflicts with ERP data quality reality
- **[Minor]** No acknowledgment of EDI coexistence

#### Recommendations
1. (P0) Define identity resolution strategy supporting multiple ID systems
2. (P0) Design delta/patch file format
3. (P1) Publish ERP mapping guide alongside spec
4. (P1) Implement tiered validation severity
5. (P2) Define explicit EDI coexistence position

---

### Dr. Isabelle Moreau -- Regulatory Compliance Advisor

#### Assessment

Having spent fourteen years advising companies on due diligence legislation -- from the French Duty of Vigilance Law in 2017 through the EU CSDDD adopted in 2024 -- I can confirm that the absence of a standardized data format is the single largest technical barrier to multi-tier supply chain transparency.

The "goods as first-class nodes" design decision is particularly significant from a regulatory standpoint. The EUDR does not ask "who are your suppliers?" -- it asks "where was this specific commodity produced, and can you trace it lot-by-lot to a geolocated plot of land?"

However, the vision is silent on several data dimensions that are not optional from a compliance perspective: temporal validity, attestation provenance, geolocation, and commodity-specific identification schemes.

#### Strengths
- Goods as first-class nodes supports lot-level commodity traceability (EUDR, UFLPA)
- Self-contained document model aligns with regulatory submission workflows
- Strict validation supports CSDDD Article 8 verification requirements
- Local-only processing removes NDA/confidentiality objections
- Flat graph with merge supports cascading due diligence

#### Concerns
- **[Critical]** No temporal dimension in data model
- **[Critical]** No attestation or provenance model
- **[Major]** Geolocation not mentioned
- **[Major]** No entity identification strategy
- **[Minor]** Domain-specific fields deferred entirely to extensions

#### Recommendations
1. (P0) Define temporal metadata model (valid_from, valid_to, last_verified)
2. (P0) Include attestation metadata in core schema
3. (P1) Support geolocation natively on facility nodes
4. (P1) Adopt established entity identification schemes
5. (P1) Define minimal regulatory-critical fields in core schema
6. (P2) Publish regulatory alignment matrix

---

### Dr. Tomasz Kowalski -- Data Format Architect

#### Assessment

The foundational architectural decision -- a flat adjacency list of typed nodes and typed edges -- is sound. This model maps cleanly to every major serialization format. The flat structure avoids the deep nesting that plagues XML-based supply chain standards.

However, the document is deliberately silent on nearly every decision that falls within my domain. The encoding format constrains the compression strategy; the compression strategy constrains random access; the header structure constrains self-description and forward compatibility. These decisions form a dependency graph of their own and need to be designed as a coherent system.

#### Strengths
- Flat adjacency list is serialization-friendly
- Self-contained file requirement forces good format hygiene
- Goods as first-class nodes produces cleaner, more compressible data
- Explicit scope boundaries keep serialization surface manageable
- Rust + WASM guarantees identical parsing across environments

#### Concerns
- **[Critical]** No dual-format strategy (text + binary)
- **[Critical]** Schema evolution strategy is foundational but deferred
- **[Major]** No magic bytes or content-type registration
- **[Major]** No integrity or authenticity model beyond validation
- **[Major]** Compression mentioned but not designed
- **[Minor]** No canonical encoding

#### Recommendations
1. (P0) Define magic bytes and file header structure
2. (P0) Commit to dual-encoding strategy (JSON + CBOR/MessagePack)
3. (P0) Define schema evolution rules before defining schema
4. (P1) Add content integrity section (SHA-256 + optional COSE signatures)
5. (P1) Specify canonical encoding (RFC 8785 JCS)
6. (P2) Evaluate block-level compression with zstd

---

### Danielle Okafor -- Open Source Strategy & Governance Lead

#### Assessment

The vision document articulates a compelling and well-scoped problem. The technical framing is right. However, the document is almost entirely silent on governance, community process, and adoption strategy. The vision states "no single company owns the format," but the LICENSE file tells a different story: `Copyright (c) 2026 BayFX`, MIT license, no CLA, no governance charter. In my experience, this gap between stated intent and actual project structure is where open standards projects fail.

#### Strengths
- Clear problem framing differentiates from platform approaches
- Sound design principles for enterprise and regulator trust
- Deliberate scope boundaries show disciplined scoping
- WASM strategy supports "data stays local"
- Self-contained file design enables validation without dependencies

#### Concerns
- **[Critical]** No governance model defined
- **[Critical]** Licensing mismatch between spec and code (MIT for spec allows forking)
- **[Major]** No CLA/DCO
- **[Major]** No adoption strategy or identified first movers
- **[Major]** No ecosystem or conformance plan
- **[Minor]** No RFC or specification development process
- **[Minor]** No code of conduct or contribution guide

#### Recommendations
1. (P0) Publish governance charter (TSC, decision-making, roadmap to multi-stakeholder)
2. (P0) Separate spec and code licensing (CC-BY-4.0 for spec, Apache 2.0 for code)
3. (P0) Adopt DCO
4. (P1) Define adoption wedge (EU due diligence regulations)
5. (P1) Publish conformance test suite plan
6. (P1) Plan language bindings (Python, JavaScript/TypeScript)
7. (P2) Evaluate foundation hosting
8. (P2) Establish RFC process

---

### Dr. Yuki Tanaka -- Data Security & Privacy Architect

#### Assessment

The "data stays local" principle and the WASM sandboxing strategy are precisely what I would want to see. The vision's instinct to avoid platforms as the primary exchange mechanism reduces attack surface and sidesteps centralized trust problems.

However, from a security and privacy standpoint, the vision has a significant structural gap: it treats data protection as largely out of scope. A flat adjacency list of an automotive OEM's tier-2 and tier-3 suppliers is competitive intelligence worth millions. The format *must* have at least primitive mechanisms for integrity verification, selective disclosure, and confidentiality.

#### Strengths
- "Data stays local" is the most important security property
- Rust for untrusted input parsing eliminates memory safety exploits
- Self-contained files create natural audit trail
- Flat structure is easier to reason about for redaction
- Strict validation reduces injection risk

#### Concerns
- **[Critical]** No integrity or authenticity mechanism
- **[Critical]** No selective disclosure or compartmentalization model
- **[Major]** "Data stays local" stated but not architecturally enforced
- **[Major]** No threat model articulated
- **[Minor]** No encryption at rest or in transit envelope
- **[Minor]** Merge from different trust domains has security implications

#### Recommendations
1. (P0) Define file integrity mechanism (SHA-256 hash + optional COSE Sign1)
2. (P0) Introduce subgraph projection concept for selective disclosure
3. (P1) Publish lightweight threat model
4. (P1) Enforce no-network in CLI (build without network I/O libraries)
5. (P1) Define data classification guidance
6. (P2) Specify optional encryption envelope (age or COSE)
7. (P2) Add per-node provenance metadata

---

### Patricia Engstrom -- Entity Identification & Corporate Hierarchy Specialist

#### Assessment

From my 17 years working entity resolution at D&B and subsequently consulting for financial regulators, I can confirm that multi-party supply chain data fragmentation is real and painful. The vision's commitment to a flat graph model, strict validation, and self-contained files are architecturally sound.

However, the vision is largely silent on what I consider the single hardest problem: **entity identity**. Entity identification is not a detail to be worked out later -- it is the load-bearing wall of the entire architecture. If two parties export files using different identifiers for the same legal entity, merge falls apart. There is no single global business identifier: DUNS covers ~500M entities but is proprietary; LEI covers ~2.7M entities, skewed toward financial institutions; national registry numbers are jurisdiction-specific; tax IDs are often confidential.

#### Strengths
- Flat graph model is the right call for merging
- Self-contained files with strict validation surface identity problems at creation
- "The format is the contract" eliminates bilateral mapping agreements
- Local-only processing protects sensitive entity data
- Explicit scope boundaries

#### Concerns
- **[Critical]** No entity identifier strategy (no reference to LEI, DUNS, national registries, or composite model)
- **[Critical]** Corporate hierarchy absent from data model (no ownership/control/parentage edges)
- **[Major]** No temporal dimension for entity identity (mergers, acquisitions, dissolutions)
- **[Major]** Merge semantics depend on identity, which is undefined
- **[Minor]** "Facilities" conflated with "organizations"

#### Recommendations
1. (P0) Define composite identifier model (LEI, DUNS, national registry + jurisdiction, tax ID, internal UUID)
2. (P0) Add corporate hierarchy as core edge types (ownership %, control, parentage)
3. (P1) Introduce temporal validity on entity nodes and edges
4. (P1) Specify identifier equivalence rules for merge operations
5. (P2) Define canonical entity reference format (`lei:XXX`, `duns:YYY`)
