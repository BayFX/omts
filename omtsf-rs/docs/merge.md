# omtsf-cli Technical Specification: Merge Engine

**Status:** Draft
**Date:** 2026-02-20

---

## 1. Purpose

This document specifies the design of the merge engine in `omtsf-core`. The merge engine implements SPEC-003: given two or more `.omts` files describing overlapping supply chain subgraphs, it produces a single graph that is the deduplicated union. The engine must guarantee commutativity, associativity, and idempotency (SPEC-003 S5) so that independent parties merging overlapping files in any order converge on an identical result.

The merge engine is the most algorithmically dense component in `omtsf-core`. It combines union-find for identity resolution, predicate-based candidate detection with indexed lookups, deterministic property conflict recording, and post-merge validation. This specification covers data structures, algorithms, ordering guarantees, and testing strategy.

---

## 2. Union-Find for Identity Resolution

### 2.1 Data Structure

Node identity resolution uses a union-find (disjoint set) structure. Each node from the concatenated input receives a slot in a flat `Vec<usize>` parent array, indexed by a dense node ordinal assigned during concatenation. The structure uses both path compression (path halving variant) and union-by-rank to achieve near-constant amortized time per operation.

```rust
pub struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<u8>,
}

impl UnionFind {
    pub fn new(n: usize) -> Self { ... }
    pub fn find(&mut self, mut x: usize) -> usize { ... }
    pub fn union(&mut self, a: usize, b: usize) { ... }
}
```

Path compression uses the iterative path-halving technique: during `find`, each visited node is linked directly to its grandparent (`self.parent[x] = self.parent[self.parent[x]]`). This avoids stack depth concerns on large graphs while still achieving the inverse-Ackermann amortized bound. Union-by-rank breaks ties deterministically: when ranks are equal, the **lower ordinal** becomes the root. This ensures that `find` returns the same representative regardless of operation ordering, which is critical for the commutativity guarantee (SPEC-003 S5).

**Why union-find over alternative approaches.** Graph-based connected component algorithms (BFS/DFS) require materializing the full match graph before computing components, which is wasteful when matches arrive incrementally from the identifier index. Union-find processes each match pair as it is discovered in O(alpha(n)) amortized time with no auxiliary graph. The deterministic tie-breaking rule (lower ordinal wins) eliminates the need for a post-hoc canonicalization step that would be required with BFS/DFS-based components.

### 2.2 Feeding Identity Predicates into Union-Find

Before union-find operations begin, the engine builds an **identifier index**: a `HashMap<CanonicalId, Vec<usize>>` mapping each canonical identifier string to the list of node ordinals carrying that identifier. Construction is O(total identifiers) with a single pass over all nodes.

For each canonical identifier key that maps to two or more nodes, the engine evaluates the full identity predicate (scheme match, value match, authority match, temporal compatibility per SPEC-003 S2) for each pair. Pairs that satisfy the predicate are unioned. The `internal` scheme is excluded from the index entirely -- `internal` identifiers are never inserted into the map.

Temporal compatibility is evaluated pairwise: for two identifier records sharing the same `(scheme, value, authority)` tuple, the engine checks whether their validity intervals overlap. If both carry `valid_to` and the earlier `valid_to` precedes the later `valid_from`, the pair is rejected. If either record omits temporal fields entirely, compatibility is assumed. An explicit `valid_to: null` (JSON null, representing no-expiry) is treated as open-ended and never causes incompatibility.

ANNULLED LEIs are excluded at index-construction time. The `is_lei_annulled` function checks whether an LEI identifier's `extra` map contains `"entity_status": "ANNULLED"` (case-sensitive, matching GLEIF's convention). If L2 enrichment data is unavailable, the function returns `false` (no false-positive exclusions).

After all pairwise unions complete, the union-find implicitly represents the transitive closure (SPEC-003 S4 step 3): if node X matches Y via one identifier and Y matches Z via another, `find(X) == find(Z)`.

**File-scoped node IDs.** Node IDs are file-local: two files can both contain a node named `"org-1"` referring to entirely different entities. The pipeline resolves each edge's source/target through the per-file ID map of the file that owns that edge, not through a global map that would silently clobber entries from later files.

---

## 3. Identity Predicates

### 3.1 Node Identity Predicate

The node identity predicate is a pure function over two identifier records (SPEC-003 S2):

```rust
pub fn identifiers_match(a: &Identifier, b: &Identifier) -> bool {
    if a.scheme == "internal" || b.scheme == "internal" {
        return false;                          // Rule 1: internal excluded
    }
    if a.scheme != b.scheme { return false; }  // Rule 2: scheme match
    if a.value.trim() != b.value.trim() {      // Rule 3: value match (trimmed)
        return false;
    }
    // Rule 4: authority check
    if a.authority.is_some() || b.authority.is_some() {
        match (&a.authority, &b.authority) {
            (Some(aa), Some(ba)) => {
                if !aa.eq_ignore_ascii_case(ba) { return false; }
            }
            _ => return false,  // one present, one absent
        }
    }
    temporal_compatible(&a, &b)                // Rule 5: temporal overlap
}
```

The predicate is symmetric by construction: every comparison is symmetric (string equality, case-insensitive equality, interval overlap). This symmetry is the foundation of the commutativity guarantee.

### 3.2 Edge Identity Predicate

Edge identity is evaluated after node merge groups are resolved (SPEC-003 S3). Two edges are merge candidates when:

1. Their source nodes belong to the same union-find group
2. Their target nodes belong to the same union-find group
3. Their `type` fields are equal
4. They share an external identifier (same predicate as nodes), OR they lack external identifiers and their merge-identity properties match per the type-specific table in SPEC-003 S3.1

`same_as` edges are excluded: they are never merged with other edges.

The `edge_identity_properties_match` function encodes the SPEC-003 S3.1 table as an exhaustive match over `EdgeTypeTag`:

| Edge Type | Identity Properties |
|-----------|-------------------|
| `ownership` | `percentage`, `direct` |
| `operational_control` | `control_type` |
| `legal_parentage` | `consolidation_basis` |
| `former_identity` | `event_type`, `effective_date` |
| `beneficial_ownership` | `control_type`, `percentage` |
| `supplies` | `commodity`, `contract_ref` |
| `subcontracts` | `commodity`, `contract_ref` |
| `tolls` | `commodity` |
| `distributes` | `service_type` |
| `brokers` | `commodity` |
| `operates` | *(type + endpoints suffice)* |
| `produces` | *(type + endpoints suffice)* |
| `composed_of` | *(type + endpoints suffice)* |
| `sells_to` | `commodity`, `contract_ref` |
| `attested_by` | `scope` |
| `same_as` | *(never matched)* |
| Extension | *(type + endpoints suffice)* |

Floating-point properties (`percentage`) use bitwise comparison (`to_bits()`) rather than IEEE 754 equality. This ensures `NaN != NaN` (two edges with NaN percentages are distinct) and avoids the -0.0 == +0.0 trap.

### 3.3 Performance: Indexing and Hashing

The canonical identifier string (SPEC-002 S4) serves as the hash key for the identifier index. The `CanonicalId` type is a newtype over `String` that enforces percent-encoding of colons (`%3A`), percent signs (`%25`), newlines (`%0A`), and carriage returns (`%0D`) at construction time. Hashing uses the default `SipHash-1-3` provided by the standard library `HashMap`.

For authority-required schemes (`nat-reg`, `vat`), the canonical form is `{scheme}:{authority}:{value}`. For all other schemes, it is `{scheme}:{value}`, and any authority field present on the identifier is excluded from the canonical string. This ensures that identifier deduplication and index lookups are consistent with the spec's identity semantics.

Edge candidate detection uses a composite key `EdgeCompositeKey`:

```rust
pub struct EdgeCompositeKey {
    pub source_rep: usize,   // find(source_ordinal)
    pub target_rep: usize,   // find(target_ordinal)
    pub edge_type: EdgeTypeTag,
}
```

This groups edges by resolved endpoints and type in O(total edges). Pairwise comparison within each bucket handles identifier and property matching. The `build_edge_candidate_index` function constructs this index in a single pass, using per-file node-ordinal lookups to correctly resolve file-local node IDs.

---

## 4. Property Merge Strategy

### 4.1 Conflict Resolution

When a merge group contains nodes (or edges) with differing values for the same property, the engine does not pick a winner. Instead it records all distinct values with provenance (SPEC-003 S4):

```rust
pub struct Conflict {
    pub field: String,
    pub values: Vec<ConflictEntry>,
}

pub struct ConflictEntry {
    pub value: serde_json::Value,
    pub source_file: String,
}
```

The `_conflicts` array is appended to the merged node at the top level (for nodes) or inside `properties` (for edges). Conflict entries are sorted by `source_file` lexicographically, then by the JSON-serialized form of `value`, ensuring deterministic output. Validators MUST NOT reject files containing `_conflicts` (SPEC-003 S4).

The `merge_scalars` function implements the core comparison: given N `(Option<T>, source_file)` pairs, it serializes each present value to `serde_json::Value` for comparison, returning `ScalarMergeResult::Agreed(value)` if all present values are JSON-equal, or `ScalarMergeResult::Conflict(entries)` with one deduplicated entry per `(source_file, value)` pair.

### 4.2 Per-Property-Type Merge Functions

- **Scalar properties (name, jurisdiction, status, etc.):** If all source values are equal, retain the value. If they differ, omit the property from the merged output and record a conflict. The `resolve_scalar_merge` helper wraps `merge_scalars` and produces the `(Option<T>, Option<Conflict>)` pair.
- **Identifiers array:** Set union via `merge_identifiers`. Deduplicate by `CanonicalId` string (first occurrence wins). Sort the merged array by canonical string in lexicographic UTF-8 byte order (SPEC-003 S4).
- **Labels array:** Set union of `{key, value}` pairs via `merge_labels`. Sort by `key` ascending, then `value` ascending, with absent values (`None`) sorting before present values (SPEC-001 S8.4, SPEC-003 S4).
- **Graph-local `id`:** Assigned sequentially by the engine (`n-0`, `n-1`, ...) after sorting groups by their lowest canonical identifier. The ID is opaque and file-local.
- **Unknown/extra fields:** Preserved from the representative node in each merge group. The `extra` serde map on nodes and edges carries through unknown fields for round-trip fidelity (SPEC-001 S2.2).

### 4.3 Provenance Tracking

The merged file header includes a `merge_metadata` object (SPEC-003 S6):

```rust
pub struct MergeMetadata {
    pub source_files: Vec<String>,
    pub reporting_entities: Vec<String>,
    pub timestamp: String,           // ISO 8601
    pub merged_node_count: usize,
    pub merged_edge_count: usize,
    pub conflict_count: usize,
}
```

When source files declare different `reporting_entity` values, the merged header omits `reporting_entity` and records all values in `merge_metadata.reporting_entities`. When all files agree on the same `reporting_entity`, the merged header preserves it. The `source_files` list and `reporting_entities` are sorted lexicographically and deduplicated.

---

## 5. Determinism Guarantees

Every merge output must be byte-identical for the same set of inputs regardless of argument order or grouping. The following invariants enforce this:

1. **Identifier sort order.** After merge, each node's `identifiers` array is sorted by canonical string in lexicographic UTF-8 byte order (SPEC-003 S4). This is the primary ordering mechanism.

2. **Node output order.** Merged nodes are emitted sorted by their lowest canonical identifier string. Nodes with no external identifiers are sorted by their representative ordinal as a fallback. The sort key is `(min_canonical_id, representative_ordinal)`.

3. **Edge output order.** Edges are sorted by `(source_node_canonical, target_node_canonical, type_string, lowest_edge_canonical_id, representative_ordinal)`. Source and target canonicals are the lowest canonical identifier of the merged source/target node groups.

4. **Conflict array order.** Conflicts within a node are sorted by `field` name. Values within a conflict are sorted by `(source_file, json_value_as_string)`.

5. **Label sort order.** Labels sorted by `(key, value)` with absent values before present.

6. **`source_files` in `merge_metadata`.** Sorted lexicographically and deduplicated.

7. **Union-find representative stability.** The lower-ordinal-wins tie-breaking rule in `UnionFind::union` ensures the representative of each group is deterministic regardless of union call order.

These orderings collectively guarantee that `merge(A, B)` and `merge(B, A)` produce byte-identical JSON output (after normalizing the randomly-generated `file_salt` and wall-clock `timestamp`). Implementations can verify determinism by comparing the SHA-256 digest of the serialized output with `file_salt` and `timestamp` zeroed.

---

## 6. Algebraic Property Tests

The three algebraic guarantees -- commutativity, associativity, idempotency (SPEC-003 S5) -- are the highest-priority test targets. Property-based testing with `proptest` is the primary verification strategy. Tests live in `omtsf-core/tests/merge_properties.rs`.

### 6.1 Graph Generation Strategy

The `proptest` strategy generates small `.omts` graphs with controlled identifier overlap:

```rust
fn arb_omts_file() -> impl Strategy<Value = OmtsFile> {
    // Nodes: 1..=6, each with a unique DUNS identifier from a shared pool.
    // Edges: 0..=20 supplies edges, deduplicated by (src, tgt), no self-loops.
    // Salt: drawn from {SALT_A, SALT_B, SALT_C}.
}
```

The identifier pool uses only DUNS identifiers (9-digit numeric, no check-digit rule) to avoid the complexity of generating valid LEI (MOD 97-10) or GLN (GS1 mod-10) values. Six distinct DUNS values provide ample overlap between generated files.

Each node in a file receives a unique identifier from the pool (slot `i` for node `i`), ensuring that `merge(A, A)` collapses each twin pair into exactly one merged node. Edges are deduplicated by `(src, tgt)` within each file to prevent false negatives in the idempotency check.

### 6.2 Test Properties

**Commutativity:**
```rust
proptest! {
    fn merge_is_commutative(a in arb_omts_file(), b in arb_omts_file()) {
        let ab = merge(&[a.clone(), b.clone()]).expect("merge(A,B)");
        let ba = merge(&[b, a]).expect("merge(B,A)");
        prop_assert_eq!(stable_hash(&ab.file), stable_hash(&ba.file));
    }
}
```

**Associativity:**
```rust
proptest! {
    fn merge_is_associative(
        a in arb_omts_file(), b in arb_omts_file(), c in arb_omts_file(),
    ) {
        let ab_c = merge(&[merge(&[a.clone(), b.clone()])?.file, c.clone()])?;
        let a_bc = merge(&[a, merge(&[b, c])?.file])?;
        prop_assert_eq!(stable_hash(&ab_c.file), stable_hash(&a_bc.file));
    }
}
```

**Idempotency:**
```rust
proptest! {
    fn merge_is_idempotent(a in arb_omts_file()) {
        let aa = merge(&[a.clone(), a.clone()]).expect("merge(A,A)");
        assert_structurally_equal(&a, &aa.file);
    }
}
```

Idempotency uses structural comparison rather than byte equality because the merge engine reassigns graph-local `id` values and generates a fresh `file_salt`. The `assert_structurally_equal` function compares:
- **Node partition:** The set of canonical-identifier-sets across nodes must be identical. Each node in the original matches exactly one node in the merged output by its identifier set.
- **Edge connectivity:** The set of `(src_cid_set, tgt_cid_set, type)` triples must be identical, resolving endpoints through canonical identifiers rather than graph-local IDs.

The `stable_hash` function used by commutativity and associativity tests serializes the file to JSON, zeros out `file_salt` and `merge_metadata.timestamp` (both non-deterministic), and returns the SHA-256 hex digest.

### 6.3 Regression Fixtures

In addition to property tests, `omtsf-cli/tests/` contains hand-crafted `.omts` fixture pairs covering:
- Disjoint graphs (no merge candidates)
- Full overlap (identical files)
- Partial overlap with conflicting properties
- Transitive merge chains (A-B via LEI, B-C via DUNS)
- `same_as` edges at each confidence level
- Temporal incompatibility preventing merge
- ANNULLED LEI exclusion
- Large merge groups triggering safety warnings
- Colliding graph-local node IDs across files (same `"org-1"` string, different entities)

---

## 7. `same_as` Handling

### 7.1 Integration with Union-Find

`same_as` edges (SPEC-003 S7) are processed after the identifier-based union-find pass. The engine scans all edges of type `same_as` and evaluates each against the configured confidence threshold via `SameAsThreshold`:

```rust
pub enum SameAsThreshold {
    Definite,   // only "definite" edges honoured (default)
    Probable,   // "definite" and "probable" honoured
    Possible,   // all three levels honoured
}
```

The `honours` method on `SameAsThreshold` maps the `confidence` property string to an acceptance decision. Absent confidence is treated as `"possible"` (the weakest level, per SPEC-003 S7.1 default of `"probable"` for the property but conservative treatment for absent fields). Unrecognized confidence strings are also treated as `"possible"`.

When a `same_as` edge is honoured, the engine calls `uf.union(src_ord, tgt_ord)` on the existing union-find structure. Because union-find inherently computes transitive closure, no separate closure step is needed: if A `same_as` B and B `same_as` C, all three end up in the same group after two union operations.

The `apply_same_as_edges` function extracts the confidence string from `properties.extra["confidence"]`, falling back to `edge.extra["confidence"]`, to accommodate both the properties-wrapper and top-level serialization patterns.

### 7.2 Cycle Considerations

`same_as` is semantically symmetric (SPEC-003 S7.1) and forms an undirected equivalence relation. Cycles in `same_as` edges (A->B, B->C, C->A) are not problematic -- they are redundant unions that union-find handles idempotently. `same_as` edges themselves are never merged or deduplicated. They are retained in the output as advisory provenance records, with their `source` and `target` references rewritten to the merged node IDs.

### 7.3 Interaction with Identifier-Based Merge

`same_as` unions are applied after identifier-based unions. This ordering is logically irrelevant (union-find is order-independent) but allows the engine to report which merge groups were formed by identifiers alone versus which were extended by `same_as` edges. Honoured `same_as` edges are collected and returned from `apply_same_as_edges` for this purpose. This distinction supports merge provenance auditing (SPEC-003 S6).

---

## 8. The Eight-Step Merge Procedure

The `merge_with_config` function in `merge_pipeline.rs` orchestrates the full SPEC-003 S4 procedure:

1. **Concatenate** all nodes from all input files into a flat `Vec<Node>`, tracking `node_origins[i]` (which file contributed node `i`) and building per-file `HashMap<&str, usize>` maps from local node IDs to global ordinals.

2. **Build identifier index** from the concatenated nodes, filtering out `internal` scheme and ANNULLED LEIs. For each canonical key mapping to 2+ nodes, evaluate the pairwise identity predicate and union matching pairs.

3. **Apply `same_as` edges** to the union-find, gated by `MergeConfig::same_as_threshold`. This extends the transitive closure with advisory equivalence assertions.

4. **Check merge-group safety limits.** Compute group sizes; emit `MergeWarning::OversizedMergeGroup` for any group exceeding `MergeConfig::group_size_limit` (default: 50 nodes per SPEC-003 S4.1).

5. **Merge each node group.** For each union-find equivalence class: union identifiers (deduplicated, sorted by canonical string), union labels (deduplicated, sorted by key then value), merge scalar properties (agree or record conflict), assign a deterministic new node ID.

6. **Rewrite edge references.** Concatenate all edges, resolve each edge's source/target through its owning file's per-file ID map to the global ordinal, then to the union-find representative, then to the new merged node ID. Build the edge candidate index using `EdgeCompositeKey`.

7. **Deduplicate edges.** For each edge candidate bucket, run pairwise `edges_match` and build a second union-find for edges. Merge each edge group's identifiers and labels; retain the representative edge's scalar properties.

8. **Emit output file** with merged nodes (sorted by canonical key), merged edges (sorted by source/target/type/identifier canonical), fresh `file_salt`, latest `snapshot_date`, `merge_metadata`, and run L1 validation. Return `MergeError::PostMergeValidationFailed` if any L1 rule fails.

---

## 9. Post-Merge Validation and Graph Invariants

After the merge procedure completes, the engine runs L1 validation on the merged output (SPEC-003 S5.1). The implementation must not emit a file that violates L1 rules. Key checks:

- **No duplicate node IDs.** The engine assigns sequential IDs (`n-0`, `n-1`, ...), so this is enforced by construction.
- **All edge references resolve.** Edge source/target rewriting uses the union-find representative mapped through `rep_to_new_id`, guaranteeing resolution. Edges with dangling references (source or target node not found in any file) are silently dropped.
- **Identifier uniqueness per node.** The deduplication in step 5 (union by canonical string) prevents duplicate identifier records.
- **Graph type constraints** (SPEC-001 S9.5) are preserved because the merge engine does not change node types or edge types.

L2 and L3 rules are re-evaluated and reported as warnings. Two L3 rules are merge-specific:

- **L3-MRG-01:** For each node, sum inbound `ownership` edge `percentage` values (considering temporal overlap). Warn if the sum exceeds 100.
- **L3-MRG-02:** Extract the `legal_parentage` subgraph and verify it forms a forest (no directed cycles). Report cycles if detected.

### 9.1 Merge-Group Safety Limits

After transitive closure, the engine computes the size of each merge group. If any group exceeds `MergeConfig::group_size_limit` (default: 50), the engine emits a `MergeWarning::OversizedMergeGroup` identifying the group's representative ordinal, the group size, and the configured limit. Warning order is deterministic (sorted by representative ordinal). This helps operators detect false-positive cascades where a single erroneous identifier match pulls unrelated entities into a single group (SPEC-003 S4.1).
