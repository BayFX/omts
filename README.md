# OMTSF: Open Multi-Tier Supply Format

An open exchange format for supply chain graph data. OMTSF represents supply networks as directed graphs of typed nodes and typed edges, serialized as self-contained `.omts` (JSON) files that can be validated, merged, redacted, and shared across organizational boundaries.

**Version:** 0.0.1 (draft)
**License:** Specs [CC-BY-4.0](spec/LICENSE) | Code [Apache-2.0](LICENSE)

## The Problem

Supply chain data is trapped. Every organization holds a partial view of a shared network, encoded in proprietary formats, internal schemas, and spreadsheets. There is no common way to export a supply network and hand it to another party such that both sides can read, validate, and merge it without manual translation.

Regulations increasingly require multi-tier visibility. EUDR, LkSG/CSDDD, CBAM, and beneficial ownership directives all demand structured proof of what is upstream. The tooling to analyze supply chains exists. The missing piece is a file format that lets the data reach it.

## What OMTSF Is

An `.omts` file is a self-contained JSON document describing a supply chain as a graph. Nodes represent the entities (organizations, facilities, goods, persons, attestations, consignments) and edges represent the relationships between them: who supplies whom, who owns whom, what certifications cover which facilities, and how goods flow through the network.

```json
{
  "omtsf_version": "0.0.1",
  "snapshot_date": "2026-02-19",
  "file_salt": "a1b2c3d4...64 hex chars...",
  "nodes": [
    { "id": "org-acme", "type": "organization", "name": "Acme Corp",
      "external_ids": [{ "scheme": "lei", "value": "5493006MHB84DD0ZWV18" }] },
    { "id": "org-bolt", "type": "organization", "name": "Bolt Fasteners Ltd" },
    { "id": "fac-bolt", "type": "facility", "name": "Bolt Sheffield Plant",
      "geo": { "lat": 53.38, "lon": -1.47 } }
  ],
  "edges": [
    { "id": "e-001", "type": "supplies", "source": "org-bolt", "target": "org-acme",
      "properties": { "commodity": "7318.15", "tier": 1 } },
    { "id": "e-002", "type": "operates", "source": "fac-bolt", "target": "org-bolt" }
  ]
}
```

The format is designed around five principles:

- **The file is a graph, stored flat.** Nodes and edges are flat lists with no nesting. Merging files from different parties is list concatenation and deduplication, not tree reconciliation.
- **Validation is not optional.** A valid `.omts` file passes structural and semantic checks. Edges must reference existing nodes. Identifiers must be well-formed. Recipients can trust the structure without inspecting every field.
- **The format is the contract.** If two systems both produce valid `.omts` files, those files are compatible. No bilateral mapping tables, no integration projects.
- **Data stays local.** Validation, analysis, and transformation all run locally. Supply chain data never needs to leave the machine.
- **The spec is open and vendor-neutral.** No single company owns the format. The specification and reference implementation are open source.

## How It Can Be Used

OMTSF addresses real regulatory and operational scenarios:

- **EUDR due diligence.** Model origin cooperatives, plantations with geolocation, and Due Diligence Statements as attestation nodes. Prove deforestation-free sourcing with a single file.
- **LkSG/CSDDD multi-tier mapping.** Document supplier hierarchies across tiers with risk assessments attached as attestation nodes.
- **Multi-ERP consolidation.** Export supplier masters from SAP, Oracle, and Dynamics as `.omts` files. Merge them using composite identifiers (LEI, DUNS, VAT numbers) to produce a single deduplicated supplier graph.
- **Beneficial ownership transparency.** Map corporate structures with ownership percentages, legal parentage, and person nodes for UBOs, governed by privacy rules.
- **CBAM embedded emissions.** Track installation-level emissions data on consignment nodes linked to producing facilities.
- **Selective disclosure.** Share supply chain structure with auditors or partners while redacting commercially sensitive identities behind salted-hash boundary references.

## The Data Model

### Node Types

| Type | Description |
|------|-------------|
| `organization` | A legal entity: company, NGO, government body |
| `facility` | A physical location: factory, warehouse, farm, mine, port |
| `good` | A product, material, commodity, or service |
| `person` | A natural person (beneficial owner, director) |
| `attestation` | A certificate, audit result, or due diligence statement |
| `consignment` | A batch, lot, or shipment of goods |
| `boundary_ref` | A placeholder replacing a redacted node (preserves graph structure) |

### Edge Types

| Category | Types |
|----------|-------|
| Corporate hierarchy | `ownership`, `operational_control`, `legal_parentage`, `former_identity`, `beneficial_ownership` |
| Supply relationships | `supplies`, `subcontracts`, `sells_to`, `distributes`, `brokers`, `tolls` |
| Operational links | `operates`, `produces`, `composed_of` |
| Attestation | `attested_by` |

Nodes carry external identifiers (LEI, DUNS, GLN, VAT, national registrations) that enable cross-file entity resolution during merge.

## Specifications

| Spec | Title | Scope |
|------|-------|-------|
| [SPEC-001](spec/graph-data-model.md) | Graph Data Model | File structure, node types, edge types, validation rules |
| [SPEC-002](spec/entity-identification.md) | Entity Identification | Identifier schemes, composite identifiers, check digit validation |
| [SPEC-003](spec/merge-semantics.md) | Merge Semantics | Multi-file merge procedure, identity predicates, `same_as` edges |
| [SPEC-004](spec/selective-disclosure.md) | Selective Disclosure | Sensitivity levels, redaction rules, boundary references |
| [SPEC-005](spec/erp-integration.md) | ERP Integration | Export mappings for SAP, Oracle, Dynamics 365 (informative) |
| [SPEC-006](spec/standards-mapping.md) | Standards Mapping | Regulatory alignment: EUDR, LkSG, CSDDD, CBAM, AMLD (informative) |
| [SPEC-007](spec/serialization-bindings.md) | Serialization Bindings | JSON and CBOR encoding, zstd compression, encoding detection |

## Rust Reference Implementation (`omtsf-rs`)

The canonical tooling for working with `.omts` files is a Rust library and CLI. It parses, validates, merges, redacts, diffs, and queries supply chain graphs. The core library compiles to WebAssembly for browser-based use.

### CLI Commands

```
omtsf validate <file>              Validate against the spec (L1/L2/L3)
omtsf merge <file>...              Merge two or more files into a single graph
omtsf redact <file> --scope <s>    Redact for a target disclosure scope
omtsf diff <a> <b>                 Structural diff between two files
omtsf inspect <file>               Print summary statistics
omtsf convert <file>               Re-serialize (normalize whitespace/key ordering)
omtsf reach <file> <node>          List all reachable nodes from a source
omtsf path <file> <from> <to>      Find paths between two nodes
omtsf subgraph <file> <nodes>...   Extract the induced subgraph for a set of nodes
omtsf query <file>                 Search nodes/edges by type, label, identifier
omtsf extract-subchain <file>      Extract subgraph matching selector criteria
omtsf init                         Scaffold a new minimal .omts file
```

### Key Capabilities

- **Three-level validation.** L1 (structural integrity), L2 (semantic completeness), L3 (cross-reference enrichment including cycle detection).
- **Merge with entity resolution.** Combines files from different sources using composite external identifiers. Handles overlapping and disjoint graphs.
- **Selective redaction.** Replaces sensitive nodes with boundary references at `public` or `partner` disclosure scopes. Person nodes and confidential identifiers are stripped automatically.
- **Graph queries.** Reachability analysis, shortest path, all-paths enumeration, ego graphs, and induced subgraph extraction. Supports edge-type filtering and directional traversal.
- **Structural diff.** Compares two files and reports added, removed, and modified nodes and edges. Supports type-based and field-based filtering.
- **CBOR and compression.** CBOR encoding produces files 21% smaller than JSON and decodes 26-36% faster. Zstd compression supported. Automatic encoding detection on load.
- **Selector-based queries.** Query nodes and edges by type, label, identifier, or jurisdiction. Extract subchains by selector match with configurable expansion hops.
- **WASM-compatible core.** The `omtsf-core` library has no I/O dependencies and compiles to `wasm32-unknown-unknown` for client-side browser tooling.

### Performance

Benchmarked on supply chain graphs from 141 elements (28 KB) to 2.2 million elements (500 MB):

| Operation | Small (141 elem) | Large (5.9K elem) | Huge (2.2M elem) |
|-----------|------------------|--------------------|-------------------|
| JSON parse | 162 us | 11.4 ms | 4.53 s |
| CBOR decode | 163 us | 8.49 ms | 3.92 s |
| Graph build | 29 us | 1.40 ms | 1.59 s |
| Validate L1+L2+L3 | 59 us | 3.80 ms | 5.01 s |
| Merge (disjoint) | 1.12 ms | 82.6 ms | - |
| Structural diff | 316 us | 17.4 ms | - |
| Reachability | 4.5 us | 234 us | 455 ms |
| Selector match | 991 ns | 68.1 us | 82.5 ms |

CBOR files are 21% smaller than JSON and decode 26-36% faster. Full
validation of a 500 MB graph completes in roughly 5 seconds.

### Build

```
cd omtsf-rs
cargo build --release
```

See [omtsf-rs/README.md](omtsf-rs/README.md) for the full command reference.

## Repository Layout

```
spec/                 Normative and informative specifications
schema/               JSON Schema for .omts files
tests/fixtures/       Validation test fixtures
usecases/             Example use case descriptions
omtsf-rs/             Rust reference implementation (CLI + library)
  crates/omtsf-core/    Core library (parsing, validation, merge, graph, WASM-safe)
  crates/omtsf-cli/     CLI binary
  crates/omtsf-wasm/    WASM bindings
  crates/omtsf-bench/   Benchmarks and supply chain generator
docs/                 Vision, governance, reviews, roadmap
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Specifications are licensed under [CC-BY-4.0](spec/LICENSE). Code is licensed under [Apache-2.0](LICENSE).
