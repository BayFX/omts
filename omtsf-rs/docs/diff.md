# omtsf-core Technical Specification: Diff Engine

**Status:** Draft
**Date:** 2026-02-20

---

## 1. Purpose

This document specifies the structural diff engine in `omtsf-core`. The engine compares two parsed `.omts` files and produces a typed description of what changed: which nodes and edges were added, removed, or modified, and which properties within matched elements differ.

The diff engine reuses the merge identity predicates defined in SPEC-003 to determine correspondence between elements across files. Two nodes that would be merge candidates are treated as "the same entity observed differently." Everything that fails to match is an addition or deletion.

The engine lives in `omtsf-core` and operates on deserialized `&OmtsFile` values. It has no filesystem dependency. The CLI's `omtsf diff <a> <b>` command handles file I/O, calls the library, and formats the output.

---

## 2. Matching Algorithm

### 2.1 Node Matching

Node matching applies the SPEC-003 Section 2 identity predicate. Two nodes from file A and file B are a **matched pair** if they share at least one external identifier where:

1. `scheme` values are equal (case-sensitive), excluding `internal`
2. `value` values are equal (case-sensitive, whitespace-trimmed)
3. `authority` values are equal (case-insensitive) when present in either record
4. Temporal validity periods overlap (or at least one is open-ended)

The algorithm proceeds as follows:

1. Build an index from canonical identifier strings (`{scheme}:{value}` or `{scheme}:{authority}:{value}`) to node references, one index per file. Exclude identifiers with `scheme: "internal"`.
2. For each identifier key present in both indices, record the pair `(node_a, node_b)` as a candidate match.
3. Compute the transitive closure of candidate matches. If node A1 matches B1 via one identifier and A1 matches B2 via another, then A1 is matched to both B1 and B2. This mirrors merge behavior and uses the same union-find approach described in `merge.md` Section 2.
4. If a transitive closure group contains more than one node from the same file, emit a warning. This indicates ambiguity (two nodes in file A map to the same entity in file B). The diff engine reports the group but does not attempt to resolve it. Each node in the group is reported as matched, with a diagnostic noting the ambiguity.
5. Unmatched nodes in A are **deletions**. Unmatched nodes in B are **additions**.

### 2.2 Edge Matching

Edge matching applies the SPEC-003 Section 3 identity predicate, evaluated after node matching is complete. Two edges are a **matched pair** if:

1. Their source nodes are matched (i.e., belong to the same match group)
2. Their target nodes are matched
3. Their `type` values are equal
4. They share an external identifier, OR they lack external identifiers and their merge-identity properties (per the SPEC-003 Section 3.1 table) are equal

The merge-identity property table from SPEC-003 is reused directly:

| Edge Type | Identity Properties (beyond type + endpoints) |
|-----------|-----------------------------------------------|
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
| `same_as` | *(never matched -- always unique)* |

When multiple edges in one file match a single edge in the other (e.g., two `supplies` edges with the same commodity and endpoints), the engine pairs them by order of appearance and reports excess edges as additions or deletions.

Unmatched edges in A are **deletions**. Unmatched edges in B are **additions**.

---

## 3. Property Comparison

For each matched pair of nodes or edges, the engine compares properties field by field.

### 3.1 Scalar Properties

Scalar fields (`name`, `jurisdiction`, `status`, `percentage`, `direct`, etc.) are compared by value equality. A change is recorded when the value in A differs from the value in B, or when a field is present in one but absent in the other.

**Semantic equivalence for dates.** Date fields are compared as calendar dates, not as strings. `"2026-02-19"` and `"2026-2-19"` are semantically equivalent (though the latter is technically non-conformant per SPEC-001 Section 2.1, the diff engine normalizes before comparison to avoid false positives on formatting variation).

**Numeric comparison.** Numeric fields are compared by value, not by string representation. `51.0` and `51` are equal. Floating-point comparison uses an epsilon of `1e-9` for fields like `percentage`, `quantity`, and `volume`.

### 3.2 Identifiers Array

The `identifiers` array is compared as a set, keyed by the canonical string form (`{scheme}:{value}` or `{scheme}:{authority}:{value}`). Identifiers present in B but not A are additions. Identifiers present in A but not B are deletions. Identifiers present in both are compared field by field for changes to `valid_from`, `valid_to`, `sensitivity`, `verification_status`, and `verification_date`.

### 3.3 Labels Array

The `labels` array is compared as a set of `{key, value}` pairs. Labels present in B but not A are additions. Labels present in A but not B are deletions. Because a label's identity is the full `{key, value}` tuple, a change in value for a given key appears as a deletion of the old pair and an addition of the new one.

### 3.4 Nested Objects

The `data_quality` object and the `geo` object on facility nodes are compared field by field, following the same scalar comparison rules. A missing object in one side and a present object in the other is recorded as an addition or deletion of the entire object.

The edge `properties` wrapper is transparent to the diff engine. The engine compares the logical properties of the edge, not the JSON nesting structure.

---

## 4. Diff API

### 4.1 Core Types

```rust
pub struct DiffResult {
    pub nodes: NodesDiff,
    pub edges: EdgesDiff,
    pub warnings: Vec<String>,
}

pub struct NodesDiff {
    pub added: Vec<NodeRef>,
    pub removed: Vec<NodeRef>,
    pub modified: Vec<NodeDiff>,
}

pub struct EdgesDiff {
    pub added: Vec<EdgeRef>,
    pub removed: Vec<EdgeRef>,
    pub modified: Vec<EdgeDiff>,
}

pub struct NodeDiff {
    pub id_a: String,
    pub id_b: String,
    pub node_type: String,
    pub matched_by: Vec<String>,    // canonical identifier strings that caused the match
    pub property_changes: Vec<PropertyChange>,
    pub identifier_changes: IdentifierSetDiff,
    pub label_changes: LabelSetDiff,
}

pub struct EdgeDiff {
    pub id_a: String,
    pub id_b: String,
    pub edge_type: String,
    pub property_changes: Vec<PropertyChange>,
    pub identifier_changes: IdentifierSetDiff,
    pub label_changes: LabelSetDiff,
}

pub struct PropertyChange {
    pub field: String,
    pub old_value: Option<Value>,
    pub new_value: Option<Value>,
}

pub struct IdentifierSetDiff {
    pub added: Vec<CanonicalId>,
    pub removed: Vec<CanonicalId>,
    pub modified: Vec<IdentifierFieldDiff>,
}

pub struct LabelSetDiff {
    pub added: Vec<Label>,
    pub removed: Vec<Label>,
}
```

`NodeRef` and `EdgeRef` are lightweight references carrying the element's graph-local `id`, `type`, and `name` (for nodes) to support readable output without cloning entire elements. `Value` is `serde_json::Value` -- the engine does not interpret property values beyond the comparison rules in Section 3.

### 4.2 Library Entry Point

```rust
pub fn diff(a: &OmtsFile, b: &OmtsFile) -> DiffResult;
pub fn diff_filtered(a: &OmtsFile, b: &OmtsFile, filter: &DiffFilter) -> DiffResult;
```

`diff` compares all nodes and edges. `diff_filtered` accepts a filter to restrict the comparison:

```rust
pub struct DiffFilter {
    pub node_types: Option<HashSet<String>>,   // only diff these node types; None = all
    pub edge_types: Option<HashSet<String>>,   // only diff these edge types; None = all
    pub ignore_fields: HashSet<String>,        // skip these property names during comparison
}
```

Filtering by node type also filters edges: when node types are restricted, edges whose source or target has a filtered-out type are excluded from the diff.

### 4.3 Summary Statistics

```rust
pub struct DiffSummary {
    pub nodes_added: usize,
    pub nodes_removed: usize,
    pub nodes_modified: usize,
    pub nodes_unchanged: usize,
    pub edges_added: usize,
    pub edges_removed: usize,
    pub edges_modified: usize,
    pub edges_unchanged: usize,
}

impl DiffResult {
    pub fn summary(&self, total_nodes_a: usize, total_edges_a: usize,
                   total_nodes_b: usize, total_edges_b: usize) -> DiffSummary;
    pub fn is_empty(&self) -> bool; // true iff no additions, removals, or modifications
}
```

The `summary` method accepts total counts from both input files so it can compute `nodes_unchanged` and `edges_unchanged` (matched pairs with zero property changes). `is_empty` is the predicate the CLI uses to choose between exit code 0 (identical) and exit code 1 (differences found).

---

## 5. Output Formats

### 5.1 Human-Readable (Default)

The default output is a text format inspired by unified diff, adapted for graph structures. Each changed element gets a block. Header lines identify the element; body lines show property changes.

```
--- a/node org-bolt (organization) "Bolt Supplies Ltd"
+++ b/node org-bolt-v2 (organization) "Bolt Supplies Ltd"
  matched by: duns:234567890
  jurisdiction: GB  (unchanged)
~ name: "Bolt Supplies Ltd" -> "Bolt Supplies Limited"
+ identifier: gln:5060012340001
- label: {com.acme.risk-tier: low}
+ label: {com.acme.risk-tier: medium}

+ node org-newco (organization) "NewCo Holdings AG"

- edge edge-004 (ownership) org-acme -> org-bolt
  percentage: 51.0, direct: true

~ edge edge-001 (supplies) org-bolt -> org-acme
  ~ commodity: "7318.15" -> "7318.15.90"
  + tier: 1

=== Summary ===
Nodes:  1 added, 0 removed, 1 modified, 3 unchanged
Edges:  0 added, 1 removed, 1 modified, 2 unchanged
```

Prefixes: `+` = added, `-` = removed, `~` = modified, no prefix = context. Node and edge headers use the graph-local `id` from the respective file. Modified elements show both IDs when they differ.

### 5.2 JSON (Machine-Readable)

With `--format json`, the CLI serializes the `DiffResult` directly as JSON to stdout. The structure mirrors the Rust types in Section 4.1. Property values are serialized as their JSON types. The output is a single JSON object (not NDJSON), suitable for programmatic consumption.

```json
{
  "summary": {
    "nodes_added": 1,
    "nodes_removed": 0,
    "nodes_modified": 1,
    "nodes_unchanged": 3,
    "edges_added": 0,
    "edges_removed": 1,
    "edges_modified": 1,
    "edges_unchanged": 2
  },
  "nodes": {
    "added": [...],
    "removed": [...],
    "modified": [...]
  },
  "edges": {
    "added": [...],
    "removed": [...],
    "modified": [...]
  },
  "warnings": []
}
```

---

## 6. CLI Integration

The `omtsf diff <a> <b>` command reads two `.omts` files, parses both into `OmtsFile` values, calls `diff` (or `diff_filtered` if filters are provided), and writes the result to stdout.

| Flag | Effect |
|------|--------|
| `--format text` | Human-readable output (default) |
| `--format json` | JSON output |
| `--summary-only` | Print only the summary line, no per-element details |
| `--node-type <type>` | Restrict diff to nodes of this type (repeatable) |
| `--edge-type <type>` | Restrict diff to edges of this type (repeatable) |
| `--ignore-field <field>` | Exclude this property from comparison (repeatable) |

**Direction convention.** File A is the baseline ("before"), file B is the target ("after"). Additions are elements present in B but not A. Deletions are elements present in A but not B. This matches `git diff <old> <new>` semantics.

**Exit codes:**

| Code | Meaning |
|------|---------|
| 0 | Diff computed successfully, files are identical |
| 1 | Diff computed successfully, files differ |
| 2 | One or both files failed to parse |

This follows the `diff(1)` convention: exit code 1 means "differences found," not "error."

---

## 7. Edge Cases

**Version mismatch.** If the two files declare different `omtsf_version` values, the engine emits a warning but proceeds. Property comparison still works -- unknown fields in either file are captured as `Value` and compared structurally.

**Empty files.** A file with zero nodes and zero edges is valid. Diffing it against a populated file produces all nodes and edges as additions (or deletions, depending on argument order).

**Boundary refs.** `boundary_ref` nodes are matched by their `opaque` scheme identifier, like any other node. Property comparison applies to whatever fields are present on the stub.

**`same_as` edges.** Per the merge-identity table, `same_as` edges are never matched across files. Every `same_as` edge in A appears as a deletion, and every `same_as` edge in B appears as an addition. This is intentional: `same_as` edges are intra-file assertions with no cross-file identity.

**Header fields.** The diff engine compares graph elements (nodes and edges), not file header fields. Changes to `snapshot_date`, `file_salt`, `reporting_entity`, or other header fields are outside the scope of the structural diff. The CLI MAY report header differences as a separate informational section in the human-readable output, but this is a formatting concern, not an engine concern.

**Extension types.** Nodes and edges with extension types (reverse-domain notation) participate in matching and comparison like any core type. Extension edge types have no entry in the merge-identity property table, so they fall back to identifier-only matching. When two extension edges share endpoints, type, and no identifiers, they are treated as unmatched (one deletion, one addition) rather than silently paired.
