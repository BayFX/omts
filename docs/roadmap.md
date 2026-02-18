# OMTSF Reference Implementation Roadmap

**Status:** Draft
**Date:** 2026-02-18

---

## Overview

The OMTSF specification suite defines the file format, identity model, merge semantics, and privacy controls for supply chain graph exchange. A reference implementation is essential for validating the specification, enabling adoption, and providing a foundation for downstream tooling.

The planned reference implementation is **`omtsf-rs`**, a Rust crate providing core OMTSF functionality with WebAssembly (WASM) bindings for browser and Node.js deployment.

## Repository

**Planned repository:** `omtsf-rs` (to be created)

**Language:** Rust (stable toolchain)

**License:** Apache-2.0

## Phases

### Phase 1: L1 Validator

**Goal:** A command-line tool and library that validates `.omts` files against all L1 structural integrity rules.

**Scope:**
- Parse `.omts` JSON files (streaming parser for large files via `serde_json`)
- Validate all L1 rules from SPEC-001 (L1-GDM-01 through L1-GDM-04, graph type constraints), SPEC-002 (L1-EID-01 through L1-EID-11), and SPEC-004 (L1-SDI-01 through L1-SDI-03)
- Validate against the JSON Schema (`schema/omts-v0.1.0.schema.json`)
- Output structured validation results (JSON) with rule identifiers
- CLI binary: `omtsf validate <file.omts>`

**Key crate dependencies:**
- `serde`, `serde_json` — JSON parsing with `#[serde(tag = "type")]` for node/edge types
- `jsonschema` — JSON Schema validation
- `clap` — CLI argument parsing

### Phase 2: Merge Engine

**Goal:** A library implementing the merge procedure (SPEC-003) with full algebraic guarantees.

**Scope:**
- Union-find (disjoint set) data structure for merge candidate grouping
- Identity predicate evaluation (SPEC-003, Section 2) with temporal compatibility
- Transitive closure computation
- Merge-group size warnings (Section 4.1 advisory limits)
- Conflict detection and `_conflicts` array generation
- Merge provenance (`merge_metadata`) generation
- CLI: `omtsf merge <file1.omts> <file2.omts> -o <merged.omts>`

### Phase 3: WASM Bindings

**Goal:** Compile the validator and merge engine to WebAssembly for browser and Node.js use.

**Scope:**
- `wasm-bindgen` bindings for validator and merge engine
- npm package: `@omtsf/core`
- Browser-compatible: validate and merge `.omts` files client-side
- TypeScript type definitions generated from JSON Schema

## Non-Goals (for reference implementation)

- ERP connectors (SAP, Oracle, D365 extractors are separate projects)
- Graph visualization (UI is a separate concern)
- L3 enrichment (requires external data sources)
- Database backends (reference impl is file-based)

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for contribution guidelines.
