# Expert Panel Review: OMTS Specification Suite (R2)

**Reviewer:** Systems Engineering Expert, Senior Systems Engineer (Rust)
**Specs Reviewed:** OMTS-SPEC-001 through OMTS-SPEC-006 (all Draft, as of 2026-02-18)
**Review Date:** 2026-02-18
**Review Round:** R2 (post-panel-findings remediation)

---

## Assessment

The specification suite has materially improved since R1. The two implementation-blocking issues I flagged -- edge property serialization ambiguity (C2) and missing size limits (C3) -- have been resolved. SPEC-001, Section 2.1 now mandates the `"properties"` wrapper for edge domain fields and explicitly separates structural fields (`id`, `type`, `source`, `target`) from domain fields. This maps cleanly to a Rust `serde` model: a flat struct for structural fields with `#[serde(flatten)]` on an inner `Properties` enum discriminated by edge type. The advisory size limits in Section 9.4 (1M nodes, 5M edges, 50 identifiers/node, 10K string length, 64-char salt) give parsers the contractual bounds they need for pre-allocation and DoS mitigation. The `file_salt` regex `^[0-9a-f]{64}$` is now explicit. The ISO 8601 date format is mandated as `YYYY-MM-DD` in Section 2.1. These were all P0/P1 items and they are all resolved.

SPEC-004 now includes four test vectors with concrete SHA-256 outputs: the multi-identifier case, the single-identifier case, a percent-encoded identifier case, and the no-public-identifiers random-token case. This is sufficient for cross-implementation conformance testing of the boundary reference pipeline. The percent-encoding rules in SPEC-002, Section 4 now cover newlines (`%0A`) and carriage returns (`%0D`) in addition to colons and percent signs, closing the hash ambiguity I flagged. The SPEC-003 merge conflict record now has a concrete `_conflicts` array structure with `field`, `values[].value`, and `values[].source_file`. This is parseable and interoperable.

The remaining gaps are not blocking for a reference implementation but matter for production hardening. The most significant from a Rust/WASM perspective: (1) no machine-readable JSON Schema yet, meaning the type hierarchy must be hand-coded from prose; (2) no streaming or chunked format for graphs exceeding single-document JSON limits; and (3) the `file_integrity` mechanism in SPEC-004 Section 6 introduces a chicken-and-egg problem for content hashing that needs precise canonical serialization rules. These are P1-tier items that should land before v1.0 but do not block prototyping.

---

## Strengths

- **Edge property wrapper resolved (SPEC-001, Section 2.1).** The `"properties"` wrapper is now normative with a clear serialization example. Structural fields are top-level; domain fields nest inside `properties`. This maps to a clean `serde` model with `#[serde(tag = "type")]` on the edge enum and `#[serde(flatten)]` for properties. No more ambiguity.
- **Advisory size limits defined (SPEC-001, Section 9.4).** 1M nodes, 5M edges, 50 identifiers/node, 10K string field length. These are generous enough for real workloads and tight enough for parser safety. The 10x rejection threshold for untrusted input is pragmatic.
- **First-key requirement for file detection (SPEC-001, Section 2.1).** `"omts_version"` must be the first JSON key. This enables file type sniffing without full parse -- a `serde_json::StreamDeserializer` can read the first token and bail if it is not `omts_version`. Smart and cheap.
- **Four concrete test vectors in SPEC-004, Section 4.** Multi-identifier, single-identifier, percent-encoded, and random-token paths. The expected SHA-256 outputs are included. This is a conformance test suite for boundary reference hashing out of the box.
- **Percent-encoding now covers newlines (SPEC-002, Section 4).** Colons, percent signs, `\n`, and `\r` are all encoded. The canonical identifier format is now unambiguous as a hash input, even when values contain the SPEC-004 join delimiter.
- **Merge conflict record schema defined (SPEC-003, Section 4, step 4).** The `_conflicts` array with `field` and `values[].value`/`values[].source_file` is parseable and deterministic. Implementations can now produce interoperable merged files containing conflicts.
- **`file_salt` validation is precise.** `^[0-9a-f]{64}$` is in the advisory size limits table and the SPEC-004 description. Producer obligation (CSPRNG) is separated from validator obligation (regex match).
- **Temporal compatibility in merge predicate (SPEC-003, Section 2).** The identity predicate now checks validity period overlap, preventing false merges from reassigned DUNS/GLN numbers. The fallback for records missing temporal fields maintains backward compatibility.
- **`data_quality` metadata on all nodes and edges (SPEC-001, Section 8.3).** The `confidence` enum (`verified`, `reported`, `inferred`, `estimated`) with optional `source` and `last_verified` addresses the provenance gap. This is informational and does not burden L1 validation.
- **`composed_of` edge type (SPEC-001, Section 6.8).** BOM decomposition is now a first-class edge type with `quantity` and `unit`, mapping directly to ERP BOM structures.
- **File integrity mechanism (SPEC-004, Section 6).** SHA-256 content hash with optional Ed25519/ECDSA-P256 signatures. The `sha2`, `ed25519-dalek`, and `p256` crates all compile to `wasm32-unknown-unknown`, making this fully WASM-compatible.

---

## Concerns

- **[Major] No machine-readable JSON Schema.** This was flagged in R1 (C1) and remains unresolved. Every Rust implementation will hand-code `serde` structs from prose tables. A JSON Schema (draft 2020-12) would enable `schemars`-based validation, code generation for non-Rust implementations, and VS Code intellisense for `.omts` files. Without it, cross-implementation type divergence is probable.
- **[Major] `file_integrity` content hash requires canonical serialization.** SPEC-004, Section 6 says "serialize the file without the `file_integrity` field" and hash the UTF-8 bytes. But JSON serialization is not canonical: key ordering, whitespace, numeric representation, and Unicode escaping all vary across implementations. Two serializers producing semantically identical JSON will produce different SHA-256 hashes. This needs either (a) RFC 8785 (JCS) as the canonical form, or (b) explicit rules for the hash-input serialization (sorted keys, no whitespace, no Unicode escapes for non-control characters). Without this, content hashes are implementation-specific and non-portable.
- **[Major] No streaming format for files exceeding advisory limits.** The spec acknowledges 1M nodes / 5M edges as advisory limits. At ~500 bytes per node and ~200 bytes per edge, a max-advisory file is ~1.5 GB of JSON. Standard `serde_json::from_reader` will buffer this entirely; streaming requires `serde_json::StreamDeserializer` or `struson`, but the single-document JSON structure (one root object with `nodes` and `edges` arrays) means you cannot begin processing edges until the entire `nodes` array is parsed. An NDJSON variant or a structure where `nodes` and `edges` are separate top-level arrays in a streaming-friendly position would help.
- **[Minor] `geo` field on `facility` remains unstructured.** SPEC-001, Section 4.2 allows both `{"lat": ..., "lon": ...}` and GeoJSON geometry in the same `geo` field. In `serde`, this requires `#[serde(untagged)]` deserialization, which tries each variant in order and produces poor error messages on malformed input. A discriminated union with a `type` field, or separate `geo_point` and `geo_polygon` fields, would be more ergonomic for typed deserialization.
- **[Minor] `consignment` node `quantity` is `number`, not `number | null`.** SPEC-001, Section 4.6 defines `quantity` as `number` (optional). In JSON, a missing field and a `null` field are different. The spec should clarify whether `"quantity": null` is valid or only field omission is permitted. For `serde`, this is `Option<f64>` vs `#[serde(skip_serializing_if = "Option::is_none")]` -- a small but meaningful distinction for round-trip fidelity.
- **[Minor] No MIME type registration.** Still relevant for HTTP-based tooling and WASM browser integration. `application/vnd.omts+json` would enable `Content-Type` negotiation and proper browser handling.

---

## Recommendations

1. **[P1] Publish normative JSON Schema (draft 2020-12).** Derive from a `schemars`-annotated Rust reference implementation to keep schema and types in sync. This is the single highest-leverage artifact for ecosystem interoperability. Blocks non-Rust implementations from starting with confidence.

2. **[P1] Specify canonical serialization for `file_integrity` content hash.** Either adopt RFC 8785 (JCS) or define minimal canonicalization rules: lexicographic key ordering, no insignificant whitespace, no Unicode escapes for characters above U+001F, IEEE 754 double serialization per ES2015. Without this, `content_hash` values are not portable across implementations.

3. **[P1] Clarify `null` vs. absent semantics for optional fields.** Add a normative statement: optional fields MAY be omitted or set to `null`; validators MUST treat both as equivalent. This ensures `serde`'s `#[serde(default, skip_serializing_if = "Option::is_none")]` behavior matches the spec.

4. **[P2] Define a streaming variant for large graphs.** Recommend NDJSON: first line is the file header (version, salt, scope), subsequent lines are node objects, followed by edge objects. A sentinel line or content-type header distinguishes nodes from edges. This enables `serde_json::StreamDeserializer` to process records one at a time.

5. **[P2] Structure `geo` as a discriminated union.** Either require a `type` field (`"point"` or `"geojson"`) or split into `geo_point` (lat/lon only) and `geo_shape` (GeoJSON). Removes `#[serde(untagged)]` ambiguity.

6. **[P2] Register MIME type `application/vnd.omts+json`.** Enables HTTP content negotiation, `wasm-bindgen` fetch integration, and browser-based validation tooling.

---

## Cross-Expert Notes

**To Data Format Expert:** The `file_integrity` content hash in SPEC-004, Section 6 is the most pressing format engineering gap. Without canonical serialization rules, two conformant implementations computing `SHA-256` over the "same" file will produce different hashes due to JSON serialization non-determinism (key order, whitespace, numeric precision). If RFC 8785 (JCS) is too heavyweight, a minimal subset (sorted keys, no whitespace, no unnecessary Unicode escapes) suffices. The `serde_json` crate does not guarantee key order for `HashMap`-backed structures, so this has direct implementation impact -- we would need `BTreeMap` or `IndexMap` with `preserve_order`.

**To Security & Privacy Expert:** The Ed25519 and ECDSA-P256 signature algorithms in SPEC-004, Section 6 are fully implementable in WASM. The `ed25519-dalek` and `p256` crates from the RustCrypto project compile to `wasm32-unknown-unknown` without system dependencies. The `sha2` crate handles SHA-256 in WASM as well. However, the spec should clarify what exactly is signed: is it the `content_hash` hex string bytes, or the raw SHA-256 digest bytes? For Ed25519, the signature is over a message -- the spec should state that the message is the raw 32-byte SHA-256 digest, not its hex encoding.

**To Graph Modeling Expert:** The advisory size limits (1M nodes, 5M edges) have direct implications for the union-find data structure in SPEC-003. A `Vec<usize>`-backed union-find with path compression and union-by-rank handles 1M elements in well under a second (benchmarks show ~3.5M operations/second on commodity hardware). The canonical identifier `HashMap<String, usize>` index for the identity predicate will dominate memory at this scale -- roughly 100 bytes per identifier entry (key + hash map overhead), so 50M identifiers (50 per node * 1M nodes at max) would consume ~5 GB. Implementations should use a `StringInterner` or arena allocator to reduce per-string overhead.

**To Enterprise Integration Expert:** The SAP Business Partner mapping in SPEC-005, Section 2.4 is now adequate for implementation. The `BUT0ID` to OMTS scheme mapping table is directly translatable to a Rust `match` statement on `IDTYPE`. For the reference implementation extractor, I would recommend a `TryFrom<But0idRecord>` impl that produces an `IdentifierRecord` with proper scheme discrimination. The tax number disambiguation table in Section 2.5 is also actionable -- each row maps to a pattern match on `(LAND1, field_name)`.

**To Open Source Strategy Expert:** The conformance test vectors in SPEC-004 are the nucleus of a test suite. For Rust, these become `#[test]` functions in the `omts-disclosure` crate that parse the test vector inputs and assert SHA-256 output equality. The recommended test suite structure: a `tests/fixtures/` directory with `.omts` files and companion `.expected.json` files describing expected validation outcomes per L1/L2 rule. Any language can consume these fixtures. This pattern is proven by the JSON Schema Test Suite and html5lib-tests projects.
