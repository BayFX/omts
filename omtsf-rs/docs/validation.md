# omtsf-core Technical Specification: Validation Engine

**Status:** Draft
**Date:** 2026-02-20

---

## 1. Purpose

This document specifies the architecture of the three-level validation engine in `omtsf-core`. The engine implements the conformant validator requirements from SPEC-001 Section 11.3: all L1 rules produce errors, L2 rules produce warnings, L3 rules produce informational findings, and unknown fields or extension types never cause rejection.

The engine lives entirely in `omtsf-core`. It operates on parsed data model types (`&OmtsFile`), not raw JSON. Parse errors are a separate concern handled at the deserialization boundary.

---

## 2. Diagnostic Types

Every validation finding is a `Diagnostic`:

```rust
pub struct Diagnostic {
    pub rule_id: RuleId,
    pub severity: Severity,
    pub location: Location,
    pub message: String,
}

pub enum Severity {
    Error,    // L1 -- structural violation, file is non-conformant
    Warning,  // L2 -- semantic concern, file is conformant but incomplete
    Info,     // L3 -- enrichment observation from external data
}
```

`RuleId` is an enum with one variant per spec-defined rule. Every diagnostic carries a machine-readable identifier that maps directly to the spec (e.g., `L1-GDM-03`, `L2-EID-05`). The enum has a `code(&self) -> &'static str` method returning the hyphenated string form for serialized output.

```rust
#[non_exhaustive]
pub enum RuleId {
    // SPEC-001 L1
    L1_GDM_01, L1_GDM_02, L1_GDM_03, L1_GDM_04, L1_GDM_05, L1_GDM_06,
    // SPEC-002 L1
    L1_EID_01, L1_EID_02, L1_EID_03, L1_EID_04, L1_EID_05,
    L1_EID_06, L1_EID_07, L1_EID_08, L1_EID_09, L1_EID_10, L1_EID_11,
    // SPEC-004 L1
    L1_SDI_01, L1_SDI_02,
    // SPEC-001 L2
    L2_GDM_01, L2_GDM_02, L2_GDM_03, L2_GDM_04,
    // SPEC-002 L2
    L2_EID_01, L2_EID_02, L2_EID_03, L2_EID_04,
    L2_EID_05, L2_EID_06, L2_EID_07, L2_EID_08,
    // L3 (SPEC-002, SPEC-003)
    L3_EID_01, L3_EID_02, L3_EID_03, L3_EID_04, L3_EID_05,
    L3_MRG_01, L3_MRG_02,
    // Internal errors and extension rules
    Internal,
    Extension(String),
}

impl RuleId {
    pub fn code(&self) -> &str {
        match self {
            Self::L1_GDM_01 => "L1-GDM-01",
            Self::L1_GDM_02 => "L1-GDM-02",
            // ... exhaustive match for all variants
            Self::Internal => "INTERNAL",
            Self::Extension(s) => s.as_str(),
        }
    }
}
```

The enum is `#[non_exhaustive]` so that adding new spec-defined rules in future versions does not break downstream callers who match on it.

### 2.1 Location Tracking

```rust
pub enum Location {
    Header { field: &'static str },
    Node { node_id: String, field: Option<String> },
    Edge { edge_id: String, field: Option<String> },
    Identifier { node_id: String, index: usize, field: Option<String> },
    Global,
}
```

`node_id` and `edge_id` are the graph-local `id` values from the file, not internal indices. The optional `field` narrows to a specific property (e.g., `"source"` on an edge for a dangling reference). `Identifier` locations include the array index so the user can locate the exact entry.

### 2.2 Collected Results

```rust
pub struct ValidationResult {
    pub diagnostics: Vec<Diagnostic>,
}

impl ValidationResult {
    pub fn has_errors(&self) -> bool;
    pub fn errors(&self) -> impl Iterator<Item = &Diagnostic>;
    pub fn warnings(&self) -> impl Iterator<Item = &Diagnostic>;
    pub fn infos(&self) -> impl Iterator<Item = &Diagnostic>;
    /// True if and only if zero diagnostics have `Severity::Error`.
    pub fn is_conformant(&self) -> bool;
}
```

The engine always collects all diagnostics -- it never fails fast. A file with 50 L1 violations returns all 50.

---

## 3. Rule Registry Architecture

### 3.1 Rule Trait

Each validation rule implements a common trait:

```rust
pub trait ValidationRule {
    fn id(&self) -> RuleId;
    fn level(&self) -> Level;
    fn severity(&self) -> Severity;
    fn check(&self, ctx: &ValidationContext, diags: &mut Vec<Diagnostic>);
}

pub enum Level { L1, L2, L3 }
```

Rules push zero or more diagnostics into the `diags` vector. A rule that finds nothing wrong pushes nothing. The `ctx: &ValidationContext` parameter bundles the parsed graph together with pre-computed lookup indices (Section 3.4). Rules never touch raw JSON.

### 3.2 Registry and Dispatch

The registry is a `Vec<Box<dyn ValidationRule>>` built at initialization. It is not a plugin system; rules are compiled into `omtsf-core`. The registry is constructed by a factory function:

```rust
pub fn build_registry(config: &ValidationConfig) -> Vec<Box<dyn ValidationRule>>;
```

`ValidationConfig` controls which levels are active:

```rust
pub struct ValidationConfig {
    pub run_l1: bool,  // always true in a conformant validator
    pub run_l2: bool,  // default: true
    pub run_l3: bool,  // default: false (requires external data)
    pub external_data: Option<Box<dyn ExternalDataSource>>,
}
```

Dispatch is a linear walk over the registry. Each rule's `check` method is called once with the full context. Rules are stateless and independent -- ordering within a level does not matter. There is no dependency graph between rules, no priority system, no early-exit. If profiling later shows hot spots, individual rules can be optimized internally without changing the dispatch model.

### 3.3 Extensibility

Extension rules can be added by implementing `ValidationRule` and appending to the registry. The trait is public. Extension rules use `RuleId::Extension(String)` to carry their own identifiers. Extension rules MUST NOT use the `L1-*`, `L2-*`, or `L3-*` prefixes -- those are reserved for spec-defined rules.

### 3.4 Validation Context

Multiple rules need the same lookup structures. A `ValidationContext` is computed once before dispatch begins:

```rust
pub struct ValidationContext<'a> {
    pub file: &'a OmtsFile,
    pub node_by_id: HashMap<&'a str, &'a Node>,
    pub edge_by_id: HashMap<&'a str, &'a Edge>,
    pub node_ids: HashSet<&'a str>,
    pub edge_ids: HashSet<&'a str>,
}
```

The context is constructed from the parsed `OmtsFile` and passed by shared reference to every rule. It is immutable for the duration of the validation pass.

---

## 4. Validation Levels

### 4.1 L1 -- Structural Integrity (Errors)

L1 rules enforce the MUST constraints from SPEC-001, SPEC-002, and SPEC-004. A file that violates any L1 rule is non-conformant. The complete L1 rule set:

**Graph Data Model (SPEC-001 Section 9.1):**

| Rule | Check |
|------|-------|
| L1-GDM-01 | Every node has a non-empty `id`, unique within the file |
| L1-GDM-02 | Every edge has a non-empty `id`, unique within the file |
| L1-GDM-03 | Every edge `source` and `target` references an existing node `id` |
| L1-GDM-04 | Edge `type` is a recognized core type, `same_as`, or reverse-domain extension |
| L1-GDM-05 | `reporting_entity` if present references an existing organization node `id` |
| L1-GDM-06 | Edge source/target node types match the permitted types table (SPEC-001 Section 9.5). Extension edge types are exempt. |

**Entity Identification (SPEC-002 Section 6.1):**

| Rule | Check |
|------|-------|
| L1-EID-01 | Every identifier has a non-empty `scheme` |
| L1-EID-02 | Every identifier has a non-empty `value` |
| L1-EID-03 | `authority` is present and non-empty when scheme is `nat-reg`, `vat`, or `internal` |
| L1-EID-04 | `scheme` is a core scheme or reverse-domain extension |
| L1-EID-05 | LEI matches `^[A-Z0-9]{18}[0-9]{2}$` and passes MOD 97-10 check digit |
| L1-EID-06 | DUNS matches `^[0-9]{9}$` |
| L1-EID-07 | GLN matches `^[0-9]{13}$` and passes GS1 mod-10 check digit |
| L1-EID-08 | `valid_from` and `valid_to` if present are valid ISO 8601 dates (`YYYY-MM-DD`) |
| L1-EID-09 | `valid_from` <= `valid_to` when both present |
| L1-EID-10 | `sensitivity` if present is `public`, `restricted`, or `confidential` |
| L1-EID-11 | No duplicate `{scheme, value, authority}` tuple on the same node |

**Selective Disclosure (SPEC-004 Section 6.1):**

| Rule | Check |
|------|-------|
| L1-SDI-01 | `boundary_ref` nodes have exactly one identifier with scheme `opaque` |
| L1-SDI-02 | If `disclosure_scope` is declared, sensitivity constraints are satisfied: `public` scope forbids `restricted` and `confidential` identifiers and `person` nodes; `partner` scope forbids `confidential` identifiers |

**Implementation note:** L1-GDM-01 and L1-GDM-02 build the `node_by_id` and `edge_by_id` maps in the `ValidationContext`. Duplicate ids emit a diagnostic; the first occurrence wins. L1-GDM-03 and L1-GDM-05 use these maps for reference resolution. L1-GDM-06 looks up source/target node types against the permitted-types table (SPEC-001 Section 9.5). Extension edge types (matching `^[a-z][a-z0-9]*(\.[a-z][a-z0-9]*)+$`) skip this check.

### 4.2 L2 -- Semantic Completeness (Warnings)

L2 rules enforce SHOULD constraints. They flag likely modeling errors or missing data without rejecting the file.

**Graph Data Model (SPEC-001 Section 9.2):**

| Rule | Check |
|------|-------|
| L2-GDM-01 | Every `facility` node connects to an `organization` via an edge or `operator` property |
| L2-GDM-02 | `ownership` edges have `valid_from` set |
| L2-GDM-03 | `organization`/`facility` nodes and `supplies`/`subcontracts`/`tolls` edges carry `data_quality` |
| L2-GDM-04 | If any `supplies` edge carries `tier`, the file declares `reporting_entity` |

**Entity Identification (SPEC-002 Section 6.2):**

| Rule | Check |
|------|-------|
| L2-EID-01 | Every `organization` node has at least one non-`internal` identifier |
| L2-EID-02 | Temporal fields (`valid_from`, `valid_to`) present on all identifier records |
| L2-EID-03 | `nat-reg` authority values are valid GLEIF RA codes per snapshot |
| L2-EID-04 | `vat` authority values are valid ISO 3166-1 alpha-2 country codes |
| L2-EID-05 | LEI values with LAPSED/RETIRED/MERGED status produce a warning |
| L2-EID-06 | LEI values with ANNULLED status produce an error-severity warning |
| L2-EID-07 | Identifiers on reassignable schemes (`duns`, `gln`) carry temporal fields |
| L2-EID-08 | Identifiers with `verification_status: "verified"` also carry `verification_date` |

L2 rules are included in the registry when `config.run_l2` is true (the default).

### 4.3 L3 -- Enrichment (Info)

L3 rules cross-reference external data sources and are off by default. The engine does not perform HTTP calls -- L3 rules receive external data through an injected trait:

```rust
pub trait ExternalDataSource {
    fn lei_status(&self, lei: &str) -> Option<LeiRecord>;
    fn nat_reg_lookup(&self, authority: &str, value: &str) -> Option<NatRegRecord>;
}
```

The CLI wires in a concrete implementation; WASM consumers provide their own adapter.

L3 rules:

| Rule | Check |
|------|-------|
| L3-EID-01 | LEI values are verifiable against the GLEIF database |
| L3-EID-02 | `nat-reg` values are cross-referenceable with the authority's registry |
| L3-EID-03 | When a node has both `lei` and `nat-reg`, the GLEIF Level 1 record matches |
| L3-EID-04 | For MERGED LEIs, a `former_identity` edge to the successor entity is present |
| L3-EID-05 | DUNS numbers on `organization` nodes are HQ-level DUNS, not branch |
| L3-MRG-01 | Sum of `ownership.percentage` edges into any org node does not exceed 100 |
| L3-MRG-02 | `legal_parentage` edges form a forest (no cycles) -- detected via topological sort |

L3-MRG-02 extracts the subgraph of `legal_parentage` edges and runs a topological sort. A cycle produces an Info diagnostic listing the node ids in the cycle.

---

## 5. Check Digit Implementations

These are pure functions in a `check_digits` module within `omtsf-core`. They operate on `&str` and return `bool`. They do not allocate.

### 5.1 MOD 97-10 (ISO 7064) for LEI

**Input:** `&str` of length 20, already confirmed to match `^[A-Z0-9]{18}[0-9]{2}$` by the regex check in L1-EID-05.

**Algorithm:**

1. Convert each character to its numeric value: digits 0-9 stay as-is, letters A=10, B=11, ..., Z=35.
2. Concatenate all numeric values into a single large integer representation. Because the result can exceed 128 bits, compute the modulus incrementally: maintain a running `u64` remainder. For each character, append its one-or-two digit numeric value by multiplying the accumulator by 10 (for values 0-9) or 100 (for values 10-35) and adding, then taking mod 97.
3. The final remainder must equal 1.

**Output:** `bool`. True if the check digit is valid.

**Error cases:** Assumes the regex pre-check has passed. L1-EID-05 calls the regex check first and only proceeds to MOD 97-10 on match.

**Test vectors (from SPEC-002 Section 5.1):**

| LEI | Expected |
|-----|----------|
| `5493006MHB84DD0ZWV18` | valid |
| `5493006MHB84DD0ZWV19` | invalid (wrong check digit) |
| `549300TRUWO2CD2G5692` | valid (GLEIF's own LEI) |

### 5.2 GS1 Mod-10 for GLN

**Input:** `&str` of length 13, already confirmed to match `^[0-9]{13}$`.

**Algorithm:**

1. Number positions 1 through 13 from left to right.
2. Apply alternating weights starting from the rightmost position (position 13): position 13 weight 1, position 12 weight 3, position 11 weight 1, position 10 weight 3, and so on alternating.
3. Sum the weighted products of positions 1 through 12 (all except the check digit at position 13).
4. Check digit = `(10 - (sum mod 10)) mod 10`.
5. Compare computed check digit against the actual digit at position 13.

**Output:** `bool`. True if the check digit matches.

**Implementation note:** The function operates on ASCII bytes directly (`b'0'..=b'9'`), converting to digit values via byte subtraction (`byte - b'0'`). No integer parsing is needed.

**Test vectors (from SPEC-001 Section 10):**

| GLN | Expected |
|-----|----------|
| `5060012340001` | valid |
| `5060012340002` | invalid (wrong check digit) |
| `0000000000000` | valid (edge case: all zeros) |

---

## 6. Error Handling Strategy

Three categories of errors exist. They are distinct types and must not be conflated.

### 6.1 Parse Errors

Produced by `serde_json` deserialization: malformed JSON, missing required fields, wrong types. Parse errors prevent validation from running. They are reported as a `ParseError` wrapping `serde_json::Error` with byte offset or line/column.

Parse errors are not `Diagnostic` values. They are a separate variant in the top-level result type:

```rust
pub enum ValidateOutput {
    ParseFailed(ParseError),
    Validated(ValidationResult),
}

pub struct ParseError {
    pub source: serde_json::Error,
    pub context: Option<String>,
}
```

The `context` field carries additional information when available (e.g., which field was being deserialized).

### 6.2 Validation Errors

These are `Diagnostic` values with `Severity::Error`. They mean the file parsed successfully but violates one or more L1 rules. The file is non-conformant. Validation warnings (`Severity::Warning`) and info findings (`Severity::Info`) are also `Diagnostic` values at lower severities.

A parse failure returns `ValidateOutput::ParseFailed`; a successful parse returns `ValidateOutput::Validated` containing zero or more diagnostics across all three severity levels.

### 6.3 Internal Errors

Bugs in the validator itself -- index out of bounds, unexpected `None`, logic errors. These must never be swallowed. In debug builds they panic. In release builds they produce a diagnostic with `RuleId::Internal` and `Severity::Error`, with a message asking the user to report the bug. The validator continues if possible.

### 6.4 Unknown Fields and Extensions

Per SPEC-001 Section 11.3, the validator MUST NOT reject files based on unknown fields, extension edge types, or unrecognized `data_quality` values. Serde deserialization uses `#[serde(flatten)]` with `serde_json::Map<String, Value>` catch-all fields on every struct to capture extensions. `#[serde(deny_unknown_fields)]` is never used anywhere in the type hierarchy.

Extension edge types (matching the reverse-domain pattern) bypass L1-GDM-04 and L1-GDM-06. Extension identifier schemes bypass format validation in L1-EID-04 through L1-EID-07. Unknown `data_quality.confidence` values are silently preserved.

---

## 7. CLI Integration

The CLI's `validate` command calls into `omtsf-core` and maps the result:

| Outcome | stderr | stdout | Exit code |
|---------|--------|--------|-----------|
| Parse failure | Parse error message | nothing | 2 |
| L1 errors present | All diagnostics | nothing | 1 |
| Only L2/L3 findings | All diagnostics | nothing | 0 |
| Clean | "Valid." | nothing | 0 |

Diagnostics are formatted one-per-line to stderr. The default format is human-readable:

```
[E] L1-GDM-03 edge "edge-042": target "node-999" does not reference an existing node
[W] L2-EID-01 node "org-acme": organization has no external identifiers
[I] L3-MRG-01 node "org-bolt": inbound ownership percentages sum to 112%
```

A `--format json` flag emits each diagnostic as a JSON object (one per line, NDJSON) for machine consumption:

```json
{"rule":"L1-GDM-03","severity":"error","location":{"type":"edge","edge_id":"edge-042","field":"target"},"message":"target \"node-999\" does not reference an existing node"}
```

The `--level` flag controls which levels to run: `--level l1` runs only L1, `--level l1,l2` runs L1 and L2 (the default), `--level l1,l2,l3` runs all three. L1 is always included; specifying `--level l2` implies L1.
