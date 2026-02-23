# Project Guidelines

## Repository Structure

- `/spec/` — Normative and informative specification documents (main artifact)
  - `graph-data-model.md` (SPEC-001, foundation)
  - `entity-identification.md` (SPEC-002)
  - `merge-semantics.md` (SPEC-003)
  - `selective-disclosure.md` (SPEC-004)
  - `erp-integration.md` (SPEC-005, informative)
  - `standards-mapping.md` (SPEC-006, informative)
  - `serialization-bindings.md` (SPEC-007)
- `/omts-rs/` — Rust reference implementation (CLI + library)
  - `crates/omts-core` — Graph model, parsing, validation, merge logic (WASM-compatible, no I/O)
  - `crates/omts-cli` — CLI interface (`omts` binary), handles all I/O
  - `crates/omts-excel` — Excel import/export support
  - `crates/omts-wasm` — WASM bindings (thin wrapper around omts-core)
  - `crates/omts-bench` — Benchmarks
  - `docs/` — Technical specification for the implementation (data-model, validation, merge, diff, query, redaction, cli-interface, graph-engine, tasks)
  - `tests/fixtures/` — `.omts` test fixtures for integration tests (JSON, CBOR, Zstd variants)
  - `README.md` — Command overview and build instructions
- `/schema/` — JSON Schema definitions
  - `omts-v0.1.0.schema.json`
- `/templates/excel/` — Excel import templates and examples
  - `omts-import-template.xlsx`, `omts-import-example.xlsx`
  - `omts-supplier-list-template.xlsx`, `omts-supplier-list-example.xlsx`
  - `generate_template.py` — Python script to regenerate templates
- `/tests/fixtures/` — Shared `.omts` test fixture files (valid and invalid cases)
- `/docs/` — Reviews, vision, roadmap, and auxiliary project documentation
  - `vision.md`, `roadmap.md`
  - `reviews/` — Expert panel reviews
  - `governance/` — TSC charter and governance docs
- `/usecases/` — Use case documentation
- `/.github/workflows/ci.yml` — CI pipeline

## Git Commits

- No `Co-Authored-By` trailer
- Short commit messages, senior engineer style (e.g., "add expert-panel skill", "fix validation edge case")

## Rust Development (omts-rs/)

### Quick Reference

| Command | What it does |
|---------|-------------|
| `just` | Type-check the workspace (default) |
| `just fmt` | Format all code |
| `just fmt-check` | Check formatting without changes |
| `just lint` | Run clippy with `-D warnings` |
| `just test` | Run all tests |
| `just build` | Build debug binaries |
| `just wasm-check` | Verify omts-core compiles to WASM |
| `just deny` | Run cargo-deny license/advisory checks |
| `just ci` | Full pipeline: fmt-check, lint, test, doc, wasm-check, deny |
| `just pre-commit` | Fast subset: fmt-check, lint, test |

### Workspace Rules

- **No `unsafe`** — `unsafe_code` is denied workspace-wide
- **No `unwrap()` / `expect()` / `panic!()` / `todo!()` / `unimplemented!()`** in production code — use `Result` + `?` instead
- **Exhaustive matches required** — `wildcard_enum_match_arm` is denied; always match every variant
- **No `dbg!()` macro** — remove before committing
- **WASM safety** — `omts-core` additionally denies `print_stdout` and `print_stderr`; all I/O belongs in `omts-cli`
- Test files may use `#![allow(clippy::expect_used)]` at the file level

### Where to Put Code

| Crate | Purpose | Notes |
|-------|---------|-------|
| `crates/omts-core` | Graph model, parsing, validation, merge logic | No I/O, no stdout/stderr, WASM-compatible |
| `crates/omts-cli` | CLI interface (`omts` binary) | Uses clap, handles all I/O |
| `crates/omts-excel` | Excel import/export | Depends on omts-core |
| `crates/omts-wasm` | WASM bindings | Thin wrapper around omts-core |
| `crates/omts-bench` | Benchmarks | Performance testing |
| `crates/omts-core/tests/` | Integration tests for core | `#![allow(clippy::expect_used)]` permitted |
| `omts-rs/tests/fixtures/` | `.omts` test fixture files | JSON/CBOR/Zstd formats |
| `tests/fixtures/` | Shared `.omts` test fixtures | Valid and invalid cases, shared across crates |

### Code Style

- **File size limit**: keep `.rs` files under 800 lines — if longer, split into modules
- **Public interfaces**: `///` doc comments on all public types, traits, functions, methods, and enum variants — write for `cargo doc` readers
- **Inline comments**: only when explaining *why*, never *what* — if the code needs a comment to explain what it does, rewrite the code
- **No commented-out code** — delete it, git has history
- **No section-separator comments** (e.g., `// --- helpers ---`) — use modules instead

### Before Submitting

1. `just pre-commit` passes (fmt-check + lint + test)
2. New public types/functions have doc comments
3. New enum variants are handled in all match arms
4. Error types use `Result<T, E>` — never panic on bad input

### Error Handling

- Define error enums in the module that produces them
- Implement `std::fmt::Display` and `std::error::Error`
- Use `?` for propagation; callers decide how to handle errors
- CLI maps errors to user-friendly messages and exit codes

### Newtype Pattern

Prefer newtypes for domain identifiers to prevent mixing up plain strings:

```rust
pub struct NodeId(String);
pub struct EdgeId(String);
```
