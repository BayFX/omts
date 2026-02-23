# Expert Panel Review: Data Format Perspective

**Reviewer:** Data Format Expert, Data Format Architect
**Scope:** Full spec suite (SPEC-001 through SPEC-006)
**Date:** 2026-02-18

---

## Assessment

The OMTS spec suite makes a sensible foundational choice: JSON as the initial serialization format, flat adjacency list as the conceptual model, and a clean separation between graph-local identity and external identity. For an interchange format targeting a heterogeneous ecosystem of ERP systems, procurement tools, and regulatory submissions, the decision to prioritize human readability and tooling ubiquity over raw throughput is defensible at this stage. The format is approachable, debuggable, and requires no specialized libraries to parse -- all critical for early adoption in an ecosystem where the first barrier is getting anyone to produce a file at all.

However, the spec suite has significant gaps in format-level engineering that will become blocking problems as adoption grows. There is no formal schema definition (no JSON Schema, no Avro IDL, nothing machine-readable), no file integrity mechanism (no checksums, no content hashes, no magic bytes), no canonical serialization rule for deterministic comparison or signing, and no defined path toward a binary or compressed representation for large graphs. The vision document explicitly identifies serialization format as a spec concern and lists "encoding, compression, human-readable vs binary tradeoffs" as a next step, but none of the six specs address it. SPEC-004 defines a SHA-256 hashing procedure but provides incomplete test vectors (the final hash value is missing), which makes independent implementation untestable. The edge property serialization has a subtle inconsistency: spec tables define properties flat on the edge, but the serialization example wraps them in a `properties` object, creating ambiguity for implementors.

The format also lacks any mechanism for streaming, chunked processing, or random access. For a supply chain graph with 50,000+ nodes (plausible for a mid-sized manufacturer's full supplier network), a single monolithic JSON document is workable but approaching the limits of convenient processing. For 500,000+ nodes (an enterprise with multi-tier visibility), JSON without compression or chunking becomes a serious operational problem. The spec needs to acknowledge these scaling boundaries and at minimum define a compression envelope.

---

## Strengths

- **JSON as the initial wire format** maximizes tooling compatibility. Every language, every platform, every developer can parse it without dependencies. This is the right choice for a v0.1 format that needs adoption above all.
- **Flat adjacency list model** (separate node and edge arrays) is structurally simple, well-suited to merge operations, and avoids the deep nesting that plagues tree-based interchange formats.
- **`omts_version` in the file header** enables schema evolution from day one. Parsers can branch on version before attempting to interpret the rest of the document.
- **`file_salt` as a CSPRNG-generated value** in the header is a sound anti-enumeration measure for the boundary reference hashing in SPEC-004. The 32-byte length is appropriate.
- **Extension mechanism via reverse-domain notation** for custom node types, edge types, and identifier schemes is a proven pattern (Java packages, Android intents, D-Bus interfaces) that avoids namespace collisions without requiring a central registry.
- **Identifier canonical string format** (SPEC-002, Section 4) with defined delimiter and percent-encoding rules provides a deterministic serialization path for hashing and comparison.
- **Three-tier validation model** (L1 structural, L2 completeness, L3 enrichment) cleanly separates what a parser must enforce from what tooling may optionally check, enabling progressive adoption.

---

## Concerns

- **[Critical] No machine-readable schema definition.** There is no JSON Schema, no Avro schema, no Protobuf IDL -- nothing that a validator, code generator, or documentation tool can consume programmatically. The spec is prose tables in Markdown. This means every implementation must hand-translate the prose into validation logic, guaranteeing inconsistencies across implementations. A JSON Schema (draft 2020-12) should be the normative definition of the `.omts` file structure, with the Markdown serving as the human-readable companion.

- **[Critical] No file integrity mechanism.** The format has no checksums, no content hash, no magic bytes. A recipient cannot verify that a file was not truncated, corrupted, or tampered with without fully parsing and validating it. For a format designed to be "handed to another party" (vision document), this is a significant gap. At minimum, a content hash (e.g., SHA-256 of the canonical file content) in a detached manifest or trailer would allow integrity verification before processing.

- **[Critical] Incomplete test vectors in SPEC-004.** The boundary reference hash procedure specifies inputs but omits the expected output hash value. This makes it impossible to verify an independent implementation against the spec. Test vectors without expected outputs are not test vectors; they are worked examples missing the answer.

- **[Major] No canonical JSON serialization rule.** SPEC-003 defines merge as producing a deterministic result (commutativity, associativity, idempotency), and SPEC-004 defines hash computation over identifiers. But the spec does not adopt RFC 8785 (JSON Canonicalization Scheme) or any equivalent. Without a canonical form, two semantically identical files can produce different byte sequences, breaking any hash-based integrity, signature, or comparison mechanism. This is especially problematic for the boundary reference hashing: if the canonical identifier string format feeds into SHA-256, the byte-level encoding of those strings must be unambiguous. The current spec partially addresses this for identifiers (via the canonical string format), but not for the file as a whole.

- **[Major] Edge property serialization inconsistency.** SPEC-001 Section 5/6/7 tables define edge properties as flat fields (e.g., `percentage`, `valid_from`), but the Section 10 serialization example wraps them in a `"properties": {}` object. The spec never explicitly states that serialized edge properties live inside a `properties` wrapper. An implementor reading the tables would produce flat edges; an implementor reading the example would produce wrapped edges. One of these must be normative, and it must be stated explicitly.

- **[Major] No compression envelope.** The vision calls out compression as a spec concern, but no spec addresses it. For large graphs, uncompressed JSON is wasteful (supply chain data is highly repetitive: identical date strings, identical scheme names, identical property keys). A defined compression envelope (e.g., `.omts.zst` for zstandard-compressed files, or a framing format that embeds compression metadata) would give implementations a standard way to handle large files without inventing ad-hoc solutions.

- **[Major] No maximum size constraints.** SPEC-002 defines no max length for identifier values, no max cardinality for identifier arrays on a node, and SPEC-001 defines no max count for nodes or edges. While flexibility is good, unbounded sizes create denial-of-service risks for validators processing untrusted input. Implementors need guidance on reasonable limits (even if they are "SHOULD NOT exceed" rather than hard limits).

- **[Minor] No magic bytes or file signature.** The format relies on JSON structure and the `omts_version` field for identification. Standard practice for interchange formats is a magic byte sequence at offset 0 (e.g., `OMTS` or a specific byte pattern) so that file type detection tools, operating systems, and streaming parsers can identify the format without parsing JSON. This is a low-effort addition with high practical value.

- **[Minor] No `Content-Type` / MIME type registration.** If `.omts` files will be transmitted over HTTP, attached to emails, or stored in content management systems, a registered MIME type (e.g., `application/vnd.omts+json`) enables proper handling by intermediaries. This is an IANA registration process that should be initiated before v1.

- **[Minor] `snapshot_date` is date-only, not datetime.** For organizations that produce multiple snapshots per day (plausible for automated ERP exports), date-only granularity is insufficient. An ISO 8601 datetime with timezone (or UTC offset) would future-proof the field.

---

## Recommendations

1. **[P0] Publish a normative JSON Schema (draft 2020-12) for the `.omts` file structure.** This schema should define the file header, node types (with per-type property validation), edge types (with per-type property validation including the `properties` wrapper), and identifier records. The schema becomes the machine-readable contract; the Markdown spec explains the rationale and semantics that a schema cannot capture. Store it in-repo at `schema/omts-v0.1.0.schema.json`.

2. **[P0] Complete the SPEC-004 test vectors.** Add the expected SHA-256 output for each test case. Add at least three additional test vectors: (a) a node with no public identifiers (random token path), (b) a node with identifiers containing colons requiring percent-encoding, (c) a node with a single public identifier. Without complete test vectors, SPEC-004 is unimplementable for third parties.

3. **[P0] Resolve the edge property serialization ambiguity.** Add an explicit normative statement to SPEC-001 Section 10 (or a new Section 10.1) declaring that edge properties are serialized inside a `"properties"` object. Update the property tables in Sections 5-7 to note that these are logical properties, serialized within the `properties` wrapper. Alternatively, if the intent is flat serialization, update the example.

4. **[P1] Adopt RFC 8785 (JCS) as the canonical JSON serialization.** This is needed for any future integrity, signing, or deterministic comparison feature. Even if signing is out of scope for v0.1, adopting JCS now avoids a breaking change later. Define: "When canonical byte-level representation is required (e.g., for hashing or signing), the file MUST be serialized according to RFC 8785."

5. **[P1] Define a file integrity mechanism.** At minimum, specify that a companion `.omts.sha256` file MAY accompany an `.omts` file containing the hex-encoded SHA-256 hash of the canonical file content. For v1, consider embedding integrity in the format itself (e.g., a `content_hash` field computed over the canonical serialization of nodes and edges, excluding the hash field itself).

6. **[P1] Add magic bytes.** Define the first 4 bytes of a valid `.omts` file. Since the file is JSON and must start with `{`, this could be implemented as a convention: the first key in the JSON object MUST be `"omts_version"`, ensuring that the byte sequence `{"omts_version"` acts as a de facto magic string. Document this as a normative requirement.

7. **[P1] Define a compression envelope.** Specify that `.omts.zst` files are zstandard-compressed `.omts` files. Optionally define a framing format (e.g., a small uncompressed header with magic bytes, version, and compression method, followed by the compressed payload). This gives implementors a standard path for large files.

8. **[P2] Add advisory size limits.** State that nodes arrays SHOULD NOT exceed 1,000,000 entries, identifier arrays per node SHOULD NOT exceed 50 entries, and individual string values SHOULD NOT exceed 10,000 UTF-8 bytes. These are not hard limits but give implementors a basis for buffer sizing and DoS protection.

9. **[P2] Register a MIME type.** File an IANA registration for `application/vnd.omts+json` (and eventually `application/vnd.omts` for a binary variant). This is a bureaucratic process best started early.

10. **[P2] Upgrade `snapshot_date` to datetime.** Change the type from ISO 8601 date to ISO 8601 datetime with mandatory UTC offset (e.g., `2026-02-18T14:30:00Z`). Provide a migration note for existing files: date-only values are interpreted as `T00:00:00Z`.

---

## Cross-Expert Notes

- **To Security & Privacy Expert:** The absence of file integrity (no checksums, no signatures) means there is no way to detect tampering or verify provenance of an `.omts` file. The boundary reference hashing in SPEC-004 protects node identity but not file integrity. Recommend coordinating on whether digital signatures (e.g., JWS detached payload over canonical JSON) should be on the v1 roadmap.

- **To Systems Engineering Expert:** The lack of a machine-readable schema (JSON Schema) directly impacts the Rust reference implementation. A JSON Schema can drive code generation for struct definitions and validation logic via `schemars` or `jsonschema-rs`. Without it, every struct and every validation rule must be hand-coded and manually kept in sync with the prose spec.

- **To Graph Modeling Expert:** The edge property wrapper (`"properties": {}`) is an important serialization detail for graph database import/export. Neo4j, Neptune, and TigerGraph all have opinions about property placement. The wrapper approach is more compatible with labeled property graph tooling that expects a properties map, but this needs to be documented as normative.

- **To Enterprise Integration Expert:** Large ERP exports (SAP with 50k+ vendors) will produce `.omts` files in the tens-of-megabytes range as uncompressed JSON. The ERP integration guide (SPEC-005) should note expected file sizes and reference the compression envelope once defined. Also, streaming JSON parsers (e.g., `ijson` in Python, `serde_json::StreamDeserializer` in Rust) can process the flat array structure without loading the entire file into memory, but only if the spec guarantees that `nodes` and `edges` are top-level arrays (not nested). This guarantee exists implicitly but should be stated explicitly for implementors building streaming pipelines.

- **To Standards Expert:** RFC 8785 (JCS) is an IETF-track specification for canonical JSON. Its adoption here would align OMTS with IETF best practices for JSON-based formats. The IANA MIME type registration should follow RFC 6838 (Media Type Specifications and Registration Procedures).

- **To Open Source Strategy Expert:** A published JSON Schema in the repository is a high-value contribution for the open-source ecosystem. It enables automatic validation in VS Code (via the JSON Language Server), CI/CD pipeline integration, and third-party tooling without depending on the Rust reference implementation. This lowers the barrier to entry for non-Rust ecosystems significantly.
