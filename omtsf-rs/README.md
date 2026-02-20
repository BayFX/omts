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
omtsf subgraph <file> <node_id>... Extract the induced subgraph for a set of nodes
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

- **L1 (Structural Integrity)** -- JSON schema conformance, referential integrity, identifier format.
- **L2 (Completeness)** -- Recommended fields present, external identifiers populated.
- **L3 (Enrichment)** -- Cross-reference checks against external registries (LEI, GLEIF RA list).

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

Extracts the induced subgraph for a given set of node IDs. Output includes all edges whose source and target are both in the selected set.

| Option | Description |
|--------|-------------|
| `--expand` | Include neighbors up to N hops from the specified nodes (default: 0) |

### `init`

Scaffolds a new minimal `.omts` file and writes it to stdout.

| Option | Description |
|--------|-------------|
| `--example` | Generate a realistic example file instead of a minimal skeleton |

## Build

```
cargo build --release
```

Binary is at `target/release/omtsf`.

## License

Apache-2.0
