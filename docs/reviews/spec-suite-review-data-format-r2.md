# Expert Panel Review: Data Format Perspective (Round 2)

**Reviewer:** Data Format Expert, Data Format Architect
**Scope:** Full spec suite (SPEC-001 through SPEC-006), post-R1 panel remediation
**Date:** 2026-02-18

---

## Assessment

The OMTSF spec suite has undergone substantial format-level remediation since the R1 panel review. The most impactful changes address five of the six Critical findings I originally flagged: the edge property serialization ambiguity is resolved with an explicit `"properties"` wrapper convention and normative language in SPEC-001 Section 2.1; SPEC-004 now provides four complete test vectors including the SHA-256 outputs, the percent-encoding path, and the random-token path; advisory size limits are defined in SPEC-001 Section 9.4 (1M nodes, 5M edges, 50 identifiers/node, 10K string length); a file integrity mechanism with `content_hash`, `algorithm`, optional Ed25519/ECDSA-P256 `signature`, and companion `.omts.sha256` is specified in SPEC-004 Section 6; and the first-key requirement (`"omtsf_version"` MUST be the first key) serves as a pragmatic magic string for file type detection. These are exactly the right remediation priorities.

The format now has a coherent integrity story end-to-end: content hashing for tamper detection, digital signatures for provenance, boundary reference hashing with per-file salt for privacy, and identifier canonical strings for deterministic comparison. The `YYYY-MM-DD`-only date mandate in SPEC-001 Section 2.1 eliminates the ISO 8601 profile ambiguity that plagued the prior draft. The `data_quality` metadata on nodes and edges (SPEC-001 Section 8.3) fills the confidence/provenance gap identified by three panelists. The `consignment` node type and `composed_of` edge type enable the material traceability chain that three separate domain experts converged on.

However, a significant gap remains: there is still no machine-readable schema definition (JSON Schema draft 2020-12). This was the single strongest consensus finding in the R1 panel (flagged by three panelists as Critical) and remains unaddressed. A prose specification cannot serve as the normative structural contract for cross-implementation compatibility. Additionally, the content hash computation in SPEC-004 Section 6 relies on "the file without the `file_integrity` field" -- an exclusion-based approach that is fragile without a canonical serialization rule. RFC 8785 (JSON Canonicalization Scheme) adoption would make the hash computation deterministic and implementor-proof.

---

## Strengths

- **Edge property `"properties"` wrapper is now unambiguous.** SPEC-001 Section 2.1 is explicit: structural fields (`id`, `type`, `source`, `target`) are top-level; all domain properties are nested inside `"properties"`. The tables in Sections 5-7 clarify these are "logical properties." This resolves the implementation-blocking ambiguity.
- **Complete test vectors in SPEC-004.** Four vectors covering the multi-identifier sort-and-hash path, single-identifier path, percent-encoding path, and random-token path. Expected SHA-256 outputs are present. Independently implementable.
- **File integrity mechanism is well-scoped.** SHA-256 content hash with algorithm agility field, optional detached digital signatures (Ed25519, ECDSA-P256), and companion `.omts.sha256` file support. This covers the spectrum from simple checksum verification to cryptographic provenance.
- **First-key requirement acts as magic string.** Mandating `"omtsf_version"` as the first JSON key means the byte sequence `{"omtsf_version"` is detectable at offset 0 without full JSON parsing. Pragmatic and compatible with existing JSON tooling.
- **Advisory size limits provide parser safety guidance.** 1M nodes, 5M edges, 50 identifiers/node, 10K string length, with a 10x grace factor for untrusted input. These are reasonable defaults for memory-bounded implementations.
- **`YYYY-MM-DD`-only date mandate** eliminates the week-date and ordinal-date ambiguity. Clean and unambiguous.
- **Unknown field preservation rule in SPEC-002 Section 3** ensures forward compatibility. Parsers that encounter fields added in future spec versions will round-trip them without data loss.
- **`data_quality` metadata** (`confidence`, `source`, `last_verified`) on all nodes and edges closes the provenance gap without overcomplicating the core schema.

---

## Concerns

- **[Critical] No machine-readable schema definition.** The strongest R1 consensus finding remains unresolved. There is no JSON Schema (draft 2020-12), no Avro IDL, no formal grammar. The entire structural contract exists as Markdown prose tables. Every implementation must hand-translate these tables into validation code, guaranteeing divergence. The JSON Schema is the single highest-leverage artifact for the ecosystem: it simultaneously enables automated validation, code generation (Rust via `schemars`, TypeScript via `json-schema-to-typescript`), VS Code/IDE integration, and conformance testing.

- **[Major] Content hash computation is underspecified for deterministic reproducibility.** SPEC-004 Section 6 defines the content hash as "SHA-256 of the file content without the `file_integrity` field." But JSON serialization is not deterministic: key ordering, whitespace, Unicode escaping, and number formatting can all vary between serializers. Two semantically identical files can produce different content hashes. Without adopting RFC 8785 (JSON Canonicalization Scheme) or an equivalent canonical form, the content hash is only verifiable by the original producer's serializer. This undermines the file integrity mechanism's cross-implementation utility.

- **[Major] No compression envelope.** The advisory size limits acknowledge files up to 1M nodes and 5M edges. At that scale, uncompressed JSON is in the hundreds of megabytes. Supply chain data is highly repetitive (identical date strings, identical scheme names, identical property keys), making it an excellent candidate for dictionary-based compression. The spec should define `.omts.zst` (zstandard) as the standard compressed representation, with a small uncompressed header for magic string detection.

- **[Major] First-key requirement conflicts with RFC 8259.** RFC 8259 Section 4 states: "An object is an unordered collection of zero or more name/value pairs." Mandating that `"omtsf_version"` be the first key imposes ordering on an unordered structure. Many JSON libraries (Python `dict` preserves insertion order post-3.7; JavaScript `JSON.stringify` preserves insertion order) will comply in practice, but some serializers (particularly those built on hash maps) may not. The spec should acknowledge this tension and recommend that producers verify first-key ordering post-serialization, or accept that the detection mechanism is best-effort.

- **[Minor] No `Content-Type` / MIME type registration.** For HTTP transport, email attachment handling, and content management systems, a registered MIME type (`application/vnd.omtsf+json`) enables correct handling by intermediaries. This is an IANA registration process (RFC 6838) best started before v1.0.

- **[Minor] `snapshot_date` remains date-only.** Organizations producing multiple automated ERP exports per day cannot distinguish snapshots. An ISO 8601 datetime with UTC offset would future-proof the field without breaking existing files (date-only values parse as `T00:00:00Z`).

- **[Minor] Signature algorithm field lacks key distribution mechanism.** SPEC-004 Section 6 defines `signature` and `signer` fields but provides no guidance on public key distribution, certificate formats, or trust anchors. The `signer` field is a bare string (LEI or domain name) with no binding to a public key. This is acknowledged as "a tooling concern" but should at least reference established patterns (e.g., JWKS endpoints, X.509 certificate chains, or did:web resolution).

---

## Recommendations

1. **[P0] Publish a normative JSON Schema (draft 2020-12).** This remains the single highest-priority artifact. Define the file header, all node types with per-type property validation, all edge types with the `"properties"` wrapper and per-type property validation, identifier records with conditional `authority` rules, and the `file_integrity` object. Store at `schema/omts-v0.1.0.schema.json`. The Markdown spec explains semantics and rationale; the JSON Schema is the machine-readable structural contract.

2. **[P1] Adopt RFC 8785 (JCS) for canonical JSON when computing `content_hash`.** Amend SPEC-004 Section 6 to state: "Before computing the content hash, the file MUST be serialized in canonical JSON form per RFC 8785 (JSON Canonicalization Scheme), with the `file_integrity` field removed." This makes the hash deterministically reproducible across implementations.

3. **[P1] Define a compression envelope.** Specify that `.omts.zst` files are zstandard-compressed `.omts` files. The first bytes of the uncompressed payload MUST satisfy the first-key requirement. Optionally define `.omts.gz` as an alternative for environments where zstandard is unavailable.

4. **[P1] Document the RFC 8259 key ordering tension.** Add a note to SPEC-001 Section 2.1 acknowledging that JSON objects are formally unordered, and that the first-key requirement depends on serializer behavior. Recommend that producers verify byte-level output. State that consumers SHOULD attempt first-key detection but MUST NOT reject files where `"omtsf_version"` is present but not first.

5. **[P2] Register MIME type `application/vnd.omtsf+json`.** File an IANA registration per RFC 6838. Define `application/vnd.omtsf+json` for JSON-serialized files and reserve `application/vnd.omtsf` for potential future binary variants.

6. **[P2] Add key distribution guidance for digital signatures.** Extend SPEC-004 Section 6 to reference at minimum one concrete key resolution mechanism (e.g., "if `signer` is a domain name, the signing public key MAY be retrieved from `https://{signer}/.well-known/omtsf-keys.json`"). This makes the signature mechanism usable without requiring out-of-band key exchange.

7. **[P2] Consider upgrading `snapshot_date` to datetime.** Provide a migration note: date-only values are interpreted as `T00:00:00Z`. This is a non-breaking extension for existing files.

---

## Cross-Expert Notes

- **To Security & Privacy Expert:** The file integrity mechanism in SPEC-004 Section 6 is structurally sound, but its practical utility depends on canonical JSON serialization. Without RFC 8785, the content hash is only verifiable by the original serializer. If digital signatures are on the v1 roadmap, canonical JSON is a prerequisite -- JWS/COSE detached payloads require deterministic byte sequences.

- **To Systems Engineering Expert:** The JSON Schema I am recommending as P0 directly enables Rust code generation via `schemars` or `jsonschema-rs`. The `"properties"` wrapper resolution means `serde` struct design can proceed with a clean `#[serde(flatten)]` or dedicated inner struct. For streaming parsers, the guarantee that `nodes` and `edges` are top-level arrays (SPEC-001 Section 2) means `serde_json::StreamDeserializer` can process them without buffering the entire file, but only if the compression envelope preserves streaming access (zstandard supports this; gzip does not efficiently).

- **To Graph Modeling Expert:** The `composed_of` edge type and `consignment` node type enable BOM decomposition and lot traceability. The edge property wrapper is now unambiguous. For graph database import (Neo4j, Neptune), the `"properties"` object maps naturally to a property map on the relationship, which is the expected structure for labeled property graph tooling.

- **To Enterprise Integration Expert:** Large ERP exports at the advisory limit (1M nodes) will produce JSON files in the 200-500 MB range. The compression envelope recommendation would reduce this to 20-50 MB with zstandard (typical 10:1 ratio on repetitive JSON). SPEC-005 should reference expected file sizes and the compression path once defined.

- **To Standards Expert:** RFC 8785 (JCS) is IETF-track and would align the content hash mechanism with IETF best practices. The IANA MIME type registration should follow RFC 6838 procedures. The ISO 6523 ICD mapping in SPEC-006 Section 4 is now normative and well-structured.

- **To Open Source Strategy Expert:** The JSON Schema in the repository is the highest-leverage contribution for ecosystem growth. It enables VS Code validation (via JSON Language Server), CI/CD pipeline integration, and third-party language bindings (Python `jsonschema`, TypeScript `ajv`) without depending on the Rust reference implementation. This dramatically lowers the barrier to entry.
