# Expert Review: OMTSF Specification Suite (Security & Privacy)

**Reviewer:** Security & Privacy Expert, Data Security & Privacy Architect
**Specs Reviewed:** OMTSF-SPEC-001 through OMTSF-SPEC-006 (all Draft, Revision 1, 2026-02-18)
**Review Date:** 2026-02-18

---

## Assessment

The OMTSF spec suite demonstrates a level of privacy consciousness that is unusual and commendable for a supply chain data format at draft stage. SPEC-004 (Selective Disclosure) is the anchor: it introduces a 3-tier sensitivity model (public/restricted/confidential), a disclosure scope mechanism with Level 1 validator enforcement, and a boundary reference construction that uses salted SHA-256 to prevent entity enumeration in redacted subgraphs. Person nodes default to confidential with mandatory omission from public files -- a GDPR data minimization posture baked into the format itself rather than left as an afterthought. The vision document's "data stays local" principle is reinforced throughout: the tooling model is explicitly offline-first, with WASM compilation enabling browser-based processing with no server uploads. These are the right instincts.

However, the suite has a fundamental structural gap: it provides data classification without data integrity. There is no file-level integrity mechanism -- no checksum, no digital signature envelope, no MAC. A file can be silently modified in transit or at rest with no detection capability. The `disclosure_scope` field is a plain-text declaration with no cryptographic binding, meaning an attacker who intercepts a file can change `disclosure_scope: "internal"` to `disclosure_scope: "partner"` and produce a file that passes all Level 1 validation. Similarly, the `merge_metadata` provenance structure in SPEC-003 carries no authentication; a merged file's claimed provenance is unforgeable only to the extent that the consumer trusts the producer, which in a multi-party supply chain is precisely the trust model that needs strengthening. The boundary reference design, while sound against enumeration, produces different hashes for the same entity across re-exports (fresh salt per file), which makes it impossible to correlate redacted nodes across file versions -- a property that is beneficial for privacy but creates operational friction for consumers tracking entity continuity.

The most consequential privacy deficiency is not in SPEC-004 but in the interaction between SPEC-002 and SPEC-004: `nat-reg` identifiers default to `public` sensitivity, which is reasonable for large corporations but exposes sole proprietorships where the company registration number is directly linkable to a natural person. Under GDPR, a sole proprietor's registry number may constitute personal data (CJEU C-434/16, Nowak), and defaulting it to public creates a compliance risk that producers may not recognize without explicit guidance. This is compounded by the absence of any guidance on transport security -- the specs define what goes into the file but say nothing about how the file should be transmitted, leaving a gap where well-classified data is transmitted over unprotected channels.

## Strengths

- **Sensitivity classification at the identifier level.** Attaching sensitivity to individual identifier records rather than to entire nodes is the right granularity. It allows a single organization node to carry a public LEI, a restricted VAT number, and a confidential internal code simultaneously, with each disclosed or redacted independently.
- **Salted boundary references.** Using a per-file CSPRNG-generated salt in the SHA-256 hash input prevents the rainbow table attack that would otherwise trivially de-anonymize redacted graphs by hashing known LEI values against boundary references. The random token fallback for entities with no public identifiers avoids the collision-to-empty problem.
- **Person node privacy defaults.** Defaulting all person node identifiers to confidential regardless of scheme-level defaults, and requiring full omission (not just redaction) from public files, is a stronger privacy posture than most data exchange formats achieve.
- **Disclosure scope enforcement at L1.** Making disclosure scope violations a structural validation failure rather than a warning means non-conformant files are rejected by any compliant validator, creating a hard enforcement boundary rather than a permissive guideline.
- **Local processing architecture.** The Rust-to-WASM compilation model means supply chain graphs never need to leave the user's machine for validation or analysis. This is a genuine differentiator against platform-based competitors that require data upload.

## Concerns

- **[Critical] No file-level integrity or authenticity mechanism.** There is no checksum, signature, or MAC on the file. A file can be tampered with in transit or at rest -- including modification of `disclosure_scope`, `sensitivity` fields, identifiers, or edge properties -- with no detection by the recipient. For a format designed for inter-organizational exchange of competitively sensitive data, this is a foundational gap. Without integrity, the entire sensitivity/disclosure model is advisory rather than enforceable across trust boundaries.

- **[Critical] `nat-reg` default sensitivity of `public` exposes sole proprietorships.** In many EU jurisdictions, sole proprietorships (Einzelunternehmen, entreprise individuelle) are registered under the owner's personal name. A `nat-reg` identifier for such an entity constitutes personal data under GDPR. Defaulting it to `public` means a producer who does not explicitly override the sensitivity will create a file that, when shared publicly, discloses personal data. This is a GDPR Article 5(1)(c) data minimization violation by design.

- **[Major] `disclosure_scope` has no cryptographic binding.** The disclosure scope is a plain-text header field. An adversary with file access can change `internal` to `public` and produce a file that passes all L1 validation. There is no mechanism to verify that the declared scope matches the producer's intent. Combined with the absence of file-level integrity, this means the disclosure model is enforceable only within systems that trust the file's provenance by out-of-band means.

- **[Major] Merge provenance in SPEC-003 has no trust domain separation.** The `merge_metadata` structure records source file identifiers and timestamps but carries no authentication or provenance chain. When files from multiple organizations are merged, there is no mechanism to attribute data to its originating trust domain, verify that claimed provenance is authentic, or detect if a malicious contributor injected fabricated nodes into a merged graph. In a multi-stakeholder supply chain, this is a vector for supply chain data poisoning.

- **[Major] No transport security guidance.** The specs define data-at-rest sensitivity but provide no guidance on data-in-transit protection. There are no recommendations for encryption, secure transfer protocols, or key management. A file classified as `disclosure_scope: "internal"` with confidential identifiers could be transmitted via unencrypted email or HTTP with no spec-level warning.

- **[Minor] Boundary references are unstable across re-exports.** Fresh salt per file means the same entity's boundary reference hash changes with every export. This is good for anti-correlation across files from different producers, but it prevents a single producer from generating consistent boundary references across successive snapshots of the same subgraph. Consumers cannot track whether a redacted entity in file v2 is the same as a redacted entity in file v1.

- **[Minor] Test vectors in SPEC-004 are incomplete.** The boundary reference test vector provides all intermediate steps but omits the final SHA-256 hash value. An implementer cannot verify their implementation against the spec without computing the expected output independently, which defeats the purpose of a test vector.

## Recommendations

1. **(P0) Add file-level integrity specification.** Define an optional but recommended `file_integrity` header block supporting at minimum: (a) a SHA-256 content digest over the canonical serialization of the file body (nodes + edges), and (b) an optional detached signature field supporting Ed25519 or ECDSA-P256 signatures. This enables recipients to detect tampering and optionally verify producer authenticity. Make integrity checking a Level 1 validation rule when the field is present.

2. **(P0) Change `nat-reg` default sensitivity to `restricted`.** This protects sole proprietorships by default. Producers who know an entity is a large corporation can explicitly override to `public`. The current default optimizes for the common case (large companies) at the expense of the vulnerable case (natural persons operating as sole traders). GDPR requires protecting the vulnerable case by default.

3. **(P1) Add transport security guidance section.** Include a non-normative section in SPEC-004 (or a new SPEC-007) recommending: TLS 1.3 for network transfer, GPG/age encryption for email or file-share transmission, and a recommendation against transmitting files with `disclosure_scope: "internal"` over untrusted channels. This does not need to be normative, but its absence leaves a gap that will be filled by insecure defaults.

4. **(P1) Complete the boundary reference test vectors.** Include the final SHA-256 hash value in the SPEC-004 test vector. Add at least two additional test vectors: one for a node with no public identifiers (random token path) and one for a node with identifiers requiring percent-encoding in the canonical form. Implementers need these to achieve interoperable boundary reference generation.

5. **(P1) Add provenance authentication to merge metadata.** Extend the `merge_metadata` structure to optionally include per-source-file integrity digests and an indicator of the trust domain (e.g., the producing organization's LEI or domain name). This does not require full PKI but creates a minimal chain of custody that can be verified when the source files are available.

6. **(P2) Consider a stable boundary reference mode.** Define an optional mechanism where a producer can supply a persistent salt (e.g., derived from a key they control) instead of a fresh CSPRNG salt, enabling stable boundary references across successive exports of the same subgraph. This would be opted into explicitly, with the spec clearly documenting the tradeoff: stability enables temporal tracking but weakens anti-enumeration if the salt is compromised.

7. **(P2) Add sensitivity override guidance for edge properties.** SPEC-001 defines `contract_ref` on supply edges, which may contain commercially sensitive contract numbers. There is no mechanism to classify edge property sensitivity. Extend the sensitivity model to cover edge-level properties, or at minimum provide normative guidance that `contract_ref` should be omitted from files with `disclosure_scope: "public"`.

## Cross-Expert Notes

- **For Supply Chain Expert:** The tension between fresh-salt-per-file (my preference for anti-enumeration) and boundary reference stability (your need for tracking redacted entities across snapshots) is real and unresolved. My Recommendation 6 proposes an opt-in stable mode as a compromise. I would value your assessment of whether the temporal tracking use case is critical enough to justify the weakened privacy guarantees.

- **For Regulatory Compliance Expert:** The `nat-reg` default sensitivity issue (my Concern 2) directly affects GDPR compliance for any OMTSF deployment touching EU sole proprietorships. Your regulatory perspective on whether `restricted` is a sufficient default, or whether `confidential` is warranted for certain jurisdictions, would strengthen the recommendation.

- **For Entity Identification Expert:** The absence of file-level integrity means that identifier records -- the foundation of merge identity -- can be tampered with undetectably. A malicious actor could add a fabricated LEI to a node, causing it to merge with an unrelated entity in a downstream merge operation. This is a merge poisoning vector that your identity predicate design should account for.

- **For Enterprise Integration Expert:** ERP export pipelines are the primary producer of `.omts` files. Your SAP/Oracle mappings in SPEC-005 should include guidance on which fields are likely to contain data requiring `confidential` or `restricted` sensitivity (e.g., `STCD1`/`STCD2` tax numbers should always map with `sensitivity: "restricted"` at minimum). The ERP integration guide is the right place to embed security defaults.

- **For Standards Expert:** The absence of a file integrity mechanism puts OMTSF behind the baseline that ISO 27001 Annex A control A.8.24 (use of cryptography) would expect for a format designed for inter-organizational data exchange. If OMTSF seeks ISO or OASIS standardization, file integrity will be a prerequisite. Better to design it now than retrofit it later.

- **For Data Format Expert:** The canonical serialization of the file body -- necessary for computing a content digest -- depends on deterministic JSON serialization (key ordering, whitespace normalization, number formatting). If you are defining serialization rules, please ensure they produce a canonical form suitable for hashing. This is a dependency for my Recommendation 1.
