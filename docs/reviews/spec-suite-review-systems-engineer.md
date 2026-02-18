# Expert Panel Review: OMTSF Specification Suite

**Reviewer:** Systems Engineering Expert, Senior Systems Engineer (Rust)
**Specs Reviewed:** OMTSF-SPEC-001 through OMTSF-SPEC-006 (all Draft, Revision as of 2026-02-18)
**Review Date:** 2026-02-18

---

## Assessment

The OMTSF specification suite is well-decomposed for implementation. The six-spec structure maps almost directly to a Rust workspace: a `omtsf-model` crate for SPEC-001 types, `omtsf-ident` for SPEC-002 scheme validation and canonical formatting, `omtsf-merge` for SPEC-003 union-find and transitive closure, `omtsf-disclosure` for SPEC-004 boundary ref hashing and redaction, and thin crate facades for the CLI and WASM targets. The flat adjacency-list serialization in SPEC-001 is ideal for streaming JSON parsing with `serde` -- nodes and edges are top-level arrays, which means a SAX-style parser can validate and build the graph in a single pass without buffering the entire file. The choice of JSON over a binary format has a real performance cost for million-node graphs, but the specification wisely avoids encoding binary alternatives at this stage, leaving room for a future `application/omts+cbor` content type without breaking the data model.

The merge algebra in SPEC-003 is formally correct -- commutativity, associativity, and idempotency are the right properties, and the union-find recommendation is the right data structure. I have implemented union-find with path compression and union-by-rank many times; the spec's description is accurate and the O(n * alpha(n)) characterization is correct. The transitive closure requirement is the single most important correctness constraint in the suite, and I am glad it is stated explicitly rather than left as an implementation detail. The boundary reference hashing in SPEC-004 is cryptographically sound: SHA-256 with a CSPRNG salt, newline-delimited sorted canonical identifiers, and explicit handling of the empty-identifier case to avoid collisions. The test vector in Section 4 is essential for cross-implementation conformance testing.

Where the spec falls short from an implementation perspective is in the areas that matter most for parsing untrusted input: there are no maximum lengths, no maximum cardinalities, and no formal grammar. A parser that faithfully implements the spec as written is vulnerable to denial-of-service via a 10 GB `identifiers` array on a single node, or a node ID that is a 500 MB string. The edge properties structure is inconsistent between the schema tables (flat properties) and the serialization example (nested under a `"properties"` key), which will cause every implementation to make an assumption the spec does not resolve. These are not theoretical concerns -- they are the first things a fuzzer will find.

---

## Strengths

- **Clean separation of concerns across specs.** SPEC-001 through SPEC-004 are normative with clear dependency ordering. SPEC-005 and SPEC-006 are explicitly informative. This maps directly to crate boundaries in a Rust workspace and avoids circular dependencies.
- **Flat adjacency-list serialization.** Streaming-friendly. A `serde` `Deserialize` impl can process nodes and edges as they arrive without needing to buffer the entire document. Enables memory-mapped I/O for large files.
- **Deterministic canonical identifier format (SPEC-002, Section 4).** The `scheme:authority:value` format with percent-encoding for colons is unambiguous, sortable, and hashable. This is the kind of specification detail that prevents implementation divergence.
- **Test vector for boundary reference hashing (SPEC-004, Section 4).** Critical for cross-implementation conformance. The inclusion of a concrete hash input example with mixed sensitivity levels prevents ambiguity in the hashing pipeline.
- **Union-find recommendation with correct complexity analysis (SPEC-003, Section 5).** The spec does not just say "compute transitive closure" -- it names the right data structure and the right time complexity. This prevents implementers from accidentally using O(n^2) approaches.
- **Check digit algorithms fully specified (SPEC-002, Appendix A).** MOD 97-10 for LEI and mod-10 for GLN are described step-by-step. No external reference needed to implement validation.
- **Extension mechanism uses reverse-domain notation.** Avoids namespace collisions without a central registry. Validators can distinguish core types from extensions syntactically.
- **`internal` scheme excluded from cross-file merge by design.** This prevents a class of false-merge bugs where two different ERP systems happen to use the same vendor number.

---

## Concerns

- **[Critical] No maximum cardinalities or string length limits anywhere in the spec.** The `identifiers` array has no maximum length. Node `id` has no maximum length. Property values have no maximum length. The `nodes` and `edges` arrays have no maximum count. A conformant parser that accepts all valid files must also accept a file with a single node carrying 10 million identifiers, or a node ID that is 1 GB of UTF-8. This is a denial-of-service vector for any parser processing untrusted input. Every production implementation will impose limits -- but without spec-defined limits, those limits will diverge, and a file valid in one implementation will be rejected by another.
- **[Critical] Edge property structure is ambiguous.** SPEC-001 Section 5-7 define edge properties as flat fields (e.g., `percentage`, `valid_from` directly on the edge object), but the serialization example in Section 10 wraps them in a `"properties"` object. This is a breaking ambiguity: `edge.percentage` vs `edge.properties.percentage` are different JSON paths. Every implementation must choose one interpretation, and two implementations that choose differently will produce incompatible files.
- **[Major] No JSON Schema, no formal grammar, no ABNF.** The spec defines types in prose tables. There is no machine-readable schema that a parser generator can consume. This means every implementation hand-codes its own type definitions, introducing divergence risk. A JSON Schema for SPEC-001 would serve as both a validation tool and a canonical type reference. For Rust, a `schemars`-annotated type hierarchy would generate the schema from the implementation.
- **[Major] No streaming or chunked format for large graphs.** The spec defines a single JSON document. JSON requires the entire document to be syntactically complete before any value is accessible (unlike NDJSON or JSON Lines). For million-node graphs, this means multi-gigabyte files that must be fully buffered or parsed with a streaming JSON parser. The spec should either define a streaming variant (e.g., NDJSON with one record per line) or explicitly acknowledge the single-document limitation and recommend chunking strategies.
- **[Major] Merge conflict representation is unstructured (SPEC-003, Section 4, step 4).** When node properties conflict, the spec says "the merger MUST record both values with their provenance." But the structure of this conflict record is not defined. Without a defined schema for conflicts, every implementation will invent its own, making merged files non-interoperable when conflicts exist.
- **[Major] `file_salt` validation is incomplete.** SPEC-001 says 64-character lowercase hexadecimal. But it does not specify validation behavior: must a parser reject uppercase hex? Must it reject a salt of all zeros? A CSPRNG requirement is stated for generation but is unverifiable at parse time. The spec should define what a validator checks (format only) versus what a producer must guarantee (CSPRNG quality).
- **[Minor] ISO 8601 date format is underspecified.** The spec says "ISO 8601 date" but ISO 8601 permits many formats: `2026-02-18`, `20260218`, `2026-W08-3`, `2026-049`. The serialization example uses `YYYY-MM-DD`. The spec should mandate the `YYYY-MM-DD` profile explicitly, or parsers will diverge on which ISO 8601 representations they accept.
- **[Minor] `geo` field on `facility` nodes references both point coordinates and GeoJSON geometry but provides no schema for either.** A parser encountering `{"lat": 53.38, "lon": -1.47}` vs `{"type": "Polygon", "coordinates": [...]}` must handle two entirely different structures in the same field. This should either be two separate fields or a discriminated union with a `type` tag.
- **[Minor] No `Content-Type` or magic bytes.** The `.omts` extension is the only way to identify a file. There is no MIME type registration, no magic number at the start of the file, and no JSON field ordering requirement that would allow early identification. A `Content-Type: application/vnd.omtsf+json` registration would help HTTP-based tooling.
- **[Minor] Percent-encoding in canonical identifier format (SPEC-002, Section 4) only covers colons and percent signs.** If a value contains a newline (the boundary ref hash delimiter), the hash computation in SPEC-004 becomes ambiguous. The canonical format should either prohibit newlines in identifier components or mandate their percent-encoding.

---

## Recommendations

1. **[P0] Resolve the edge property structure ambiguity.** Choose one of: (a) properties are flat fields on the edge object alongside `id`, `type`, `source`, `target`; or (b) properties are nested under a `"properties"` key. Update all schema tables and the serialization example to be consistent. This is blocking for any implementation work.

2. **[P0] Define maximum cardinalities and string lengths.** At minimum: max node/edge count per file (suggest 10 million), max identifiers per node (suggest 100), max string length for IDs and property values (suggest 64 KB). These can be generous -- the point is to give parsers a contractual upper bound so they can pre-allocate or reject before OOM.

3. **[P1] Publish a JSON Schema for SPEC-001.** Machine-readable schema enables automated validation, code generation, and eliminates prose ambiguity. For Rust, derive the schema from `schemars` on the reference implementation types so it stays in sync.

4. **[P1] Mandate `YYYY-MM-DD` as the sole ISO 8601 date profile.** Add a validation rule (L1) requiring dates to match `^\d{4}-\d{2}-\d{2}$` with valid calendar dates.

5. **[P1] Define the merge conflict record schema.** A `conflicts` array on merged nodes with `{property, values: [{value, source_file, reporting_entity}], resolution}` structure. Without this, merged files containing conflicts are not interoperable.

6. **[P1] Specify `file_salt` validation precisely.** L1 validation: must match `^[0-9a-f]{64}$` (lowercase hex, exactly 64 characters). Producer requirement: must be generated by a CSPRNG. Document that validators cannot verify CSPRNG quality.

7. **[P1] Add a concrete hash output to the SPEC-004 test vector.** The current test vector describes the input construction but omits the expected SHA-256 output. Without the final hex string, implementers cannot verify their hashing pipeline end-to-end.

8. **[P2] Define a streaming variant for large files.** NDJSON (one JSON object per line, with a header line followed by node lines followed by edge lines) would allow streaming parsing and per-record validation without buffering the entire file.

9. **[P2] Register `application/vnd.omtsf+json` as a MIME type.** Enables HTTP content negotiation, browser-based tooling, and file type detection without relying on file extensions.

10. **[P2] Extend percent-encoding in canonical identifier format to cover newlines.** Add `\n` -> `%0A` and `\r` -> `%0D` to the encoding rules in SPEC-002, Section 4, to prevent ambiguity with the newline delimiter in SPEC-004 boundary ref hashing.

---

## Cross-Expert Notes

**To Graph Modeling Expert:** The transitive closure requirement in SPEC-003 is the correctness lynchpin of the entire merge system. From an implementation perspective, the union-find recommendation is exactly right, but the spec should also specify that the union-find operates on canonical identifier strings (SPEC-002, Section 4), not on raw `(scheme, value, authority)` tuples. This avoids ambiguity about comparison normalization and enables a single `HashMap<String, usize>` as the index structure.

**To Data Format Expert:** The edge property nesting ambiguity (flat vs. `"properties"` wrapper) is the single most implementation-blocking issue in the suite. The serialization example in SPEC-001, Section 10 uses a wrapper; the schema tables in Sections 5-7 do not. If the wrapper is intentional, it creates a clean separation between structural fields (`id`, `type`, `source`, `target`) and domain fields, which maps well to a Rust enum with a `#[serde(flatten)]` inner struct. If flat is intended, the `serde` model is simpler but the JSON is harder to validate without a schema.

**To Security & Privacy Expert:** The boundary reference hash construction is sound, but the test vector is incomplete -- it describes the input pipeline but does not provide the expected output hash. For a cryptographic construction, the expected output is the single most important part of a test vector. I would also note that `SHA-256(data || salt)` is a length-extension-vulnerable construction. While this is not exploitable in this context (the attacker does not learn the intermediate hash state), using `HMAC-SHA-256(salt, data)` would be more conservative and is equally implementable in WASM via the `ring` or `sha2` crates.

**To Enterprise Integration Expert:** For the SAP and Oracle ERP mappings in SPEC-005, the most valuable addition from an implementation perspective would be sample SQL queries or CDS view definitions that produce the exact JSON structure defined in SPEC-001. Prose field mappings are useful for understanding, but executable queries eliminate ambiguity for the engineer writing the extractor.

**To Open Source Strategy Expert:** The conformance test suite (referenced in the R2 panel report as P1-23) is the single highest-leverage artifact for ecosystem health. I would recommend structuring it as a directory of `.omts` files with companion `.expected` files (JSON describing expected validation results), runnable by any implementation. In Rust, this becomes a `#[test]` that walks the directory and asserts outcomes, and any other language can do the same. This is how `html5lib-tests` and `JSON Schema Test Suite` drive cross-implementation conformance.
