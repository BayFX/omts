# omtsf-cli

Rust command-line tool for working with `.omts` files.

## Commands

```
omtsf validate <file>              Validate an .omts file (L1/L2/L3)
omtsf merge <file>...              Merge two or more .omts files
omtsf redact <file>                Redact a file for a target disclosure scope
omtsf inspect <file>               Print summary statistics for a graph
omtsf diff <a> <b>                 Compute a structural diff between two files
omtsf convert <file>               Re-serialize an .omts file (normalize whitespace, key ordering)
omtsf reach <file> <node_id>       List all nodes reachable from a source node
omtsf path <file> <from> <to>      Find paths between two nodes
omtsf subgraph <file> [node_id...] Extract induced subgraph by node IDs and/or selectors
omtsf init                         Scaffold a new minimal .omts file
```

### Global Flags

| Flag | Description |
|------|-------------|
| `-f`, `--format` | Output format: `human` (default) or `json` |
| `-q`, `--quiet` | Suppress all stderr output except errors |
| `-v`, `--verbose` | Increase stderr verbosity (timing, rule counts, file metadata) |
| `--max-file-size` | Maximum input file size in bytes (default: 256 MB, env: `OMTSF_MAX_FILE_SIZE`) |
| `--no-color` | Disable ANSI color codes in human output (env: `NO_COLOR`) |

All commands accept `-` as the file path to read from stdin.

---

### `validate`

Runs the three-level validation defined in SPEC-001 Section 9:

- **L1 (Structural Integrity):** JSON schema conformance, referential integrity, identifier format.
- **L2 (Completeness):** Recommended fields present, external identifiers populated.
- **L3 (Enrichment):** Cross-reference checks against external registries (LEI, GLEIF RA list).

Exit code 0 on success, non-zero on failure. Diagnostics to stderr.

| Option | Description |
|--------|-------------|
| `--level` | Maximum validation level to run: `1`, `2` (default), or `3` |

### `merge`

Implements the merge procedure from SPEC-003. Accepts two or more `.omts` files, resolves node identity via composite external identifiers, and writes a merged graph to stdout. Honors `same_as` edges and merge-group safety limits.

| Option | Description |
|--------|-------------|
| `--strategy` | Merge strategy: `union` (default) or `intersect` |

### `redact`

Applies the selective disclosure rules from SPEC-004. Given a target `disclosure_scope`, replaces nodes and edge properties that exceed the scope's sensitivity threshold with `boundary_ref` placeholders.

| Option | Description |
|--------|-------------|
| `--scope` | Target disclosure scope: `public`, `partner`, or `internal` (required) |

### `inspect`

Prints a human-readable summary: node counts by type, edge counts by type, identifier scheme coverage, disclosure scope, snapshot date.

### `diff`

Compares two `.omts` files structurally. Reports added/removed/modified nodes and edges by graph-local ID.

| Option | Description |
|--------|-------------|
| `--ids-only` | Only report added/removed/changed IDs, no property-level detail |
| `--summary-only` | Only print the summary statistics line |
| `--node-type` | Restrict diff to nodes of this type (repeatable) |
| `--edge-type` | Restrict diff to edges of this type (repeatable) |
| `--ignore-field` | Exclude this property from comparison (repeatable) |

### `convert`

Re-serializes an `.omts` file, normalizing whitespace and key ordering.

| Option | Description |
|--------|-------------|
| `--pretty` | Pretty-print JSON output with 2-space indentation (default) |
| `--compact` | Emit minified JSON with no extraneous whitespace |

### `reach`

Lists all nodes reachable from a source node via directed edges. Useful for upstream/downstream supply chain traversal.

| Option | Description |
|--------|-------------|
| `--depth` | Maximum traversal depth (default: unlimited) |
| `--direction` | Traversal direction: `outgoing` (default), `incoming`, or `both` |

### `path`

Finds paths between two nodes. Reports one or more simple paths through the graph.

| Option | Description |
|--------|-------------|
| `--max-paths` | Maximum number of paths to report (default: 10) |
| `--max-depth` | Maximum path length in edges (default: 20) |

### `subgraph`

Extracts the induced subgraph for seed nodes selected by explicit IDs, property-based selector flags, or both. Output includes all edges whose source and target are both in the selected set.

| Option | Description |
|--------|-------------|
| `--node-type` | Match nodes of this type (repeatable) |
| `--edge-type` | Match edges of this type (repeatable) |
| `--label` | Match elements with this label key or key=value pair (repeatable) |
| `--identifier` | Match nodes with this identifier scheme or scheme:value pair (repeatable) |
| `--jurisdiction` | Match nodes by ISO 3166-1 alpha-2 country code (repeatable) |
| `--name` | Case-insensitive substring match on node name (repeatable) |
| `--expand` | Include neighbors up to N hops from the seed set (default: 0) |
| `--to` | Output encoding: `json` (default) or `cbor` |
| `--compress` | Compress output with zstd |

### `init`

Scaffolds a new minimal `.omts` file and writes it to stdout.

| Option | Description |
|--------|-------------|
| `--example` | Generate a realistic example file instead of a minimal skeleton |

## Performance

Benchmarked with [Criterion](https://github.com/bheisler/criterion.rs) on
deterministic test data from 141 elements (S) to 2.2 million elements (Huge, 500 MB).

| Operation | S (141 elem) | M (1.5K) | L (5.9K) | XL (15K) | Huge (2.2M) |
|-----------|---:|---:|---:|---:|---:|
| JSON deserialize | 162 us | 1.84 ms | 11.4 ms | 32.8 ms | 4.53 s |
| CBOR decode | 163 us | 1.82 ms | 8.49 ms | 27.3 ms | 3.92 s |
| Graph build | 29 us | 293 us | 1.40 ms | 4.43 ms | 1.59 s |
| Validate (L1+L2+L3) | 59 us | 747 us | 3.80 ms | 14.7 ms | 5.01 s |
| Diff (identical) | 316 us | 3.60 ms | 17.4 ms | 70.3 ms | -- |
| Merge (disjoint, 2 files) | 1.12 ms | 15.5 ms | 82.6 ms | -- | -- |
| All paths (depth 10) | 47 us | 11.6 ms | -- | -- | -- |
| Shortest path (root to leaf) | 6.8 us | 88 us | 365 us | 1.01 ms | 455 ms |
| Selector match (label) | 991 ns | 10.1 us | 68.1 us | 239 us | 82.5 ms |

**Highlights:**

- CBOR is 21% smaller than JSON and 26-36% faster for both encode and decode
- All-paths query: 16.6x speedup from backtracking DFS with bitset cycle detection
- Selector matching: 3-4x speedup from pre-computed lowercase patterns
- Full validation of a 500 MB supply chain graph (2.2M elements) in 5 seconds
- Parse-and-build round-trip for 500 MB: 6.1 seconds

For full results including scaling analysis and per-optimization breakdowns, see
[BENCHMARK_RESULTS.md](crates/omtsf-bench/BENCHMARK_RESULTS.md).

## Build

```
cargo build --release
```

Binary is at `target/release/omtsf`.

## License

Apache-2.0
