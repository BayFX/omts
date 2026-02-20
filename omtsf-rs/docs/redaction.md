# omtsf-core Technical Specification: Selective Disclosure / Redaction Engine

**Status:** Draft
**Date:** 2026-02-20

---

## 1. Purpose

This document specifies the selective disclosure and redaction engine in `omtsf-core`. The engine transforms an `.omts` file from a higher-trust scope (typically `internal`) to a lower-trust scope (`partner` or `public`) by removing sensitive identifiers, stripping confidential edge properties, replacing out-of-scope nodes with boundary references, and omitting person nodes and beneficial ownership edges where required. Invoked via `omtsf redact <file> --scope <partner|public>`, it emits a valid `.omts` on stdout that must pass L1 validation.

---

## 2. Sensitivity Classification

### 2.1 Identifier Sensitivity Defaults

Every identifier record carries a `sensitivity` field (SPEC-002 Section 3). When the field is absent, the engine applies scheme-based defaults:

| Scheme | Default Sensitivity |
|--------|-------------------|
| `lei` | `public` |
| `duns` | `public` |
| `gln` | `public` |
| `nat-reg` | `restricted` |
| `vat` | `restricted` |
| `internal` | `restricted` |
| Any unrecognized scheme | `public` |

An explicit `sensitivity` value on the identifier record always overrides the scheme default.

The resolution function `effective_sensitivity` implements this cascade:

```rust
pub fn effective_sensitivity(identifier: &Identifier, node_type: &NodeTypeTag) -> Sensitivity {
    // 1. Explicit override always wins.
    if let Some(explicit) = &identifier.sensitivity {
        return explicit.clone();
    }
    // 2. Person-node rule (Section 2.2).
    if let NodeTypeTag::Known(NodeType::Person) = node_type {
        return Sensitivity::Confidential;
    }
    // 3. Scheme-based default.
    scheme_default(&identifier.scheme)
}
```

### 2.2 Person Node Override

All identifiers on `person` nodes default to `confidential` regardless of scheme defaults. An explicit override to `restricted` is permitted; an override to `public` is not (validators should flag this as semantically suspect, but the engine respects what the file declares).

### 2.3 Edge Property Sensitivity Defaults

Edge properties carry sensitivity analogous to identifiers. Defaults:

| Property | Default |
|----------|---------|
| `contract_ref` | `restricted` |
| `annual_value` | `restricted` |
| `value_currency` | `restricted` |
| `volume` | `restricted` |
| `volume_unit` | `public` |
| `percentage` (on `ownership`) | `public` |
| `percentage` (on `beneficial_ownership`) | `confidential` |
| All other properties | `public` |

Producers may override defaults via a `_property_sensitivity` object inside `properties`. The engine reads this object first when determining per-property sensitivity.

Resolution order: (1) consult `_property_sensitivity` override map on the edge; (2) fall through to the default table, where `percentage` dispatches on edge type.

---

## 3. Disclosure Scopes and Filtering Rules

The engine accepts a target scope and applies the following constraints:

### 3.1 `partner` Scope

- **Identifiers:** Remove all identifiers with effective sensitivity `confidential`. Retain `public` and `restricted`.
- **Edge properties:** Remove properties with effective sensitivity `confidential`. Retain `restricted` and `public`. The `_property_sensitivity` object is retained (it may be useful to the partner).
- **Person nodes:** Retain, but their identifiers are filtered per the rules above (all default to `confidential`, so most will be stripped).
- **Beneficial ownership edges:** Retain, but property filtering applies (`percentage` defaults to `confidential` on these edges).
- **Nodes with no remaining identifiers:** Retained with an empty `identifiers` array. Not replaced with a boundary reference.

### 3.2 `public` Scope

- **Identifiers:** Remove all identifiers with effective sensitivity `confidential` OR `restricted`. Retain only `public`.
- **Edge properties:** Remove properties with effective sensitivity `confidential` or `restricted`. The `_property_sensitivity` object itself is omitted entirely.
- **Person nodes:** Omit entirely. Not replaced with boundary references -- they are simply absent from the output graph.
- **Beneficial ownership edges:** Omit entirely.
- **Edges referencing omitted nodes:** Any edge whose `source` or `target` references an omitted `person` node is itself omitted. The engine must not produce dangling edge references.

### 3.3 `internal` Scope

No filtering. The file is emitted as-is. This is a no-op path, useful for validation-only workflows.

---

## 4. Boundary Reference Hashing

When the producer designates a node for redaction (the node falls outside the subgraph being exported), the engine replaces it with a `boundary_ref` node. The opaque identifier value is computed deterministically from the node's public identifiers and the file salt.

### 4.1 Algorithm

1. Collect all identifiers on the original node whose effective sensitivity is `public`.
2. Compute the canonical string form of each identifier per SPEC-002 Section 4. The canonical form is `{scheme}:{value}` for schemes without a required authority, or `{scheme}:{authority}:{value}` for schemes requiring authority (`nat-reg`, `vat`). Colons within authority or value fields are percent-encoded as `%3A`; percent signs as `%25`; newlines as `%0A`; carriage returns as `%0D`.
3. Sort the canonical strings lexicographically by UTF-8 byte order. This is a plain byte-wise comparison -- no Unicode collation.
4. Join the sorted strings with a single newline byte (`0x0A`).
5. If the joined string is **non-empty**: concatenate the UTF-8 bytes of the joined string with the raw 32 bytes of `file_salt` (decoded from hex). Compute SHA-256 over this concatenation. The boundary reference value is the lowercase hex encoding of the 32-byte digest.
6. If the joined string is **empty** (the node has zero public identifiers): generate 32 random bytes from a CSPRNG and hex-encode them. The result must be a 64-character lowercase hexadecimal string.

### 4.2 Rust Implementation Notes

Use `sha2` (pure Rust, WASM-compatible) for SHA-256, `getrandom` for CSPRNG (delegates to `crypto.getRandomValues` on wasm32), and a hand-written hex codec to avoid external crate dependencies. Do not use `ring` or `openssl` -- both have C/asm components that break wasm compilation.

```rust
use sha2::{Digest, Sha256};

pub fn boundary_ref_value(
    public_ids: &[CanonicalId],
    salt: &[u8; 32],
) -> Result<String, BoundaryHashError> {
    if public_ids.is_empty() {
        // Random path: 32 CSPRNG bytes, hex-encoded.
        let mut buf = [0u8; 32];
        getrandom::getrandom(&mut buf).map_err(BoundaryHashError::CsprngFailure)?;
        return Ok(hex_encode(&buf));
    }

    // Deterministic path: sort, join, hash with salt.
    let mut canonicals: Vec<&str> = public_ids.iter().map(CanonicalId::as_str).collect();
    canonicals.sort_unstable(); // UTF-8 byte-order sort

    let joined = canonicals.join("\n");

    let mut hasher = Sha256::new();
    hasher.update(joined.as_bytes());
    hasher.update(salt.as_slice());

    Ok(hex_encode(&hasher.finalize()))
}
```

### 4.3 Test Vectors

All vectors use `file_salt` = `0x00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff`.

**TV1: Multiple public identifiers, one restricted (excluded)**

Input identifiers:
- `lei:5493006MHB84DD0ZWV18` (public)
- `duns:081466849` (public)
- `vat:DE:DE123456789` (restricted, excluded from hash)

Canonical strings (public only): `duns:081466849`, `lei:5493006MHB84DD0ZWV18`

Sorted: `duns:081466849`, `lei:5493006MHB84DD0ZWV18`

Joined: `duns:081466849\nlei:5493006MHB84DD0ZWV18`

SHA-256 = `e8798687b081da98b7cd1c4e5e2423bd3214fbab0f1f476a2dcdbf67c2e21141`

**TV2: Single identifier**

Input identifiers:
- `lei:5493006MHB84DD0ZWV18` (public)

Canonical string: `lei:5493006MHB84DD0ZWV18`

Joined: `lei:5493006MHB84DD0ZWV18`

SHA-256 = `7849e55c4381ba852a2ada50f15e58d871de085893b7be8826f75560854c78c8`

**TV3: Identifier requiring percent-encoding**

Input identifiers:
- `nat-reg:RA000548:HRB%3A86891` (public -- registry number contains a literal colon, percent-encoded per SPEC-002 Section 4)

Canonical string: `nat-reg:RA000548:HRB%3A86891`

Joined: `nat-reg:RA000548:HRB%3A86891`

SHA-256 = `7b33571d3bba150f4dfd9609c38b4f9acc9a3a8dbfa3121418a35264562ca5d9`

**TV4: No public identifiers (random token)**

Input identifiers:
- `internal:sap-prod:V-100234` (restricted, excluded)
- `vat:DE:DE123456789` (restricted, excluded)

No public identifiers exist. Value = hex-encoded 32-byte CSPRNG token. Output must be a 64-character lowercase hexadecimal string. This value is non-deterministic; tests verify format only.

### 4.4 Salt Handling

The `file_salt` is a 64-character lowercase hex string in the JSON header, decoded to a 32-byte array before hashing. The salt must match `^[0-9a-f]{64}$` (SPEC-001 Section 9.4); the decoder rejects uppercase hex digits, matching the `FileSalt` newtype invariant. If decoding fails, the engine rejects the file before redaction begins.

When `omtsf init` generates a new file, it produces a fresh salt from the platform CSPRNG. Fresh salt per export prevents cross-file correlation of boundary references.

---

## 5. Node Classification and Redaction Decisions

The engine classifies each node in the input graph into one of three dispositions:

| Disposition | Meaning |
|-------------|---------|
| **Retain** | Node appears in output, possibly with filtered identifiers and properties. |
| **Replace** | Node is replaced with a `boundary_ref` stub. Original identifiers, name, and properties are stripped. |
| **Omit** | Node is removed entirely. All edges referencing it are also removed. |

Classification rules by node type and target scope:

| Node Type | `partner` Scope | `public` Scope |
|-----------|----------------|---------------|
| `organization` | Retain or Replace (producer choice) | Retain or Replace |
| `facility` | Retain or Replace | Retain or Replace |
| `good` | Retain or Replace | Retain or Replace |
| `consignment` | Retain or Replace | Retain or Replace |
| `attestation` | Retain or Replace | Retain or Replace |
| `person` | Retain (identifiers filtered) | **Omit** |
| `boundary_ref` | Pass through | Pass through |

The Retain-vs-Replace decision for non-person nodes is a producer choice -- `omtsf redact` accepts a set of node IDs to retain, and everything outside that set is replaced with boundary references. Existing `boundary_ref` nodes pass through unconditionally.

The classification is a two-step process: `classify_node` returns the base disposition (only `person` in `public` scope yields `Omit`), then the caller promotes `Retain` to `Replace` for nodes absent from the retain set:

```rust
pub fn classify_node(node: &Node, target_scope: &DisclosureScope) -> NodeAction {
    match target_scope {
        DisclosureScope::Internal | DisclosureScope::Partner => NodeAction::Retain,
        DisclosureScope::Public => match &node.node_type {
            NodeTypeTag::Known(NodeType::Person) => NodeAction::Omit,
            _ => NodeAction::Retain,
        },
    }
}

// Caller applies producer choice:
let action = match classify_node(node, &scope) {
    NodeAction::Omit => NodeAction::Omit,
    _ => {
        let is_bref = matches!(&node.node_type, NodeTypeTag::Known(NodeType::BoundaryRef));
        if is_bref || retain_ids.contains(&node.id) { NodeAction::Retain }
        else { NodeAction::Replace }
    }
};
```

### 5.1 Boundary Reference Node Structure

A replaced node becomes:

```json
{
  "id": "<original-node-id>",
  "type": "boundary_ref",
  "identifiers": [
    {
      "scheme": "opaque",
      "value": "<computed-hash-or-random-token>"
    }
  ]
}
```

The `id` preserves the original graph-local ID so existing edge references remain valid. The node carries exactly one `opaque` identifier (L1-SDI-01).

---

## 6. Edge Handling During Redaction

### 6.1 Boundary-Crossing Edges

An edge connecting a retained node to a replaced (boundary_ref) node is preserved -- this is the primary purpose of boundary references.

### 6.2 Both Endpoints Replaced

When both endpoints are replaced with boundary references, the edge is **omitted**. An edge between two opaque stubs leaks relationship existence with no informational value.

### 6.3 Edges Referencing Omitted Nodes

When either endpoint references an omitted node, the edge is omitted.

### 6.4 Edge Type Filtering

In `public` scope, `beneficial_ownership` edges are unconditionally omitted regardless of endpoint disposition (SPEC-004 Section 5).

### 6.5 Property Stripping on Retained Edges

For edges that survive the filtering pass, properties are stripped according to the target scope's sensitivity threshold:

- **`partner` scope:** Remove properties with effective sensitivity `confidential`.
- **`public` scope:** Remove properties with effective sensitivity `confidential` or `restricted`. Also remove the `_property_sensitivity` object.

If stripping removes a property that would normally be required, the edge remains valid -- redacted files may have sparser property sets than internal files. The classification function evaluates rules in priority order:

```rust
pub fn classify_edge(
    edge: &Edge,
    source_action: &NodeAction,
    target_action: &NodeAction,
    target_scope: &DisclosureScope,
) -> EdgeAction {
    // Section 6.4: beneficial_ownership unconditionally omitted in public scope.
    if matches!(target_scope, DisclosureScope::Public) {
        if let EdgeTypeTag::Known(EdgeType::BeneficialOwnership) = &edge.edge_type {
            return EdgeAction::Omit;
        }
    }
    // Section 6.3: either endpoint omitted.
    if matches!(source_action, NodeAction::Omit) || matches!(target_action, NodeAction::Omit) {
        return EdgeAction::Omit;
    }
    // Section 6.2: both endpoints replaced.
    if matches!(source_action, NodeAction::Replace)
        && matches!(target_action, NodeAction::Replace)
    {
        return EdgeAction::Omit;
    }
    // Section 6.1 + both-Retain case.
    EdgeAction::Retain
}
```

---

## 7. Output Validation

### 7.1 Post-Redaction Invariants

The engine runs a validation pass on the output before emitting it. The following invariants must hold:

1. **No dangling edges.** Every edge `source` and `target` must reference a node `id` present in the output.
2. **Boundary ref structure.** Every `boundary_ref` node has exactly one identifier with `scheme: "opaque"` (L1-SDI-01).
3. **Scope consistency.** If the output declares `disclosure_scope`, the sensitivity constraints from SPEC-004 Section 3 must be satisfied (L1-SDI-02). The engine sets `disclosure_scope` on the output header to match the target scope.
4. **No person nodes in public output.** If the target scope is `public`, the output must contain zero nodes with `type: "person"`.
5. **No beneficial_ownership edges in public output.** If the target scope is `public`, the output must contain zero edges with `type: "beneficial_ownership"`.
6. **Salt preserved.** The output file retains the original `file_salt`. Boundary reference hashes are only meaningful relative to the salt that produced them.

The engine invokes L1 validation on the output and returns `RedactError::InvalidOutput` on failure. A post-redaction validation failure indicates a bug in the engine, not in the input.

### 7.2 Boundary Reference Consistency

If the same original node is referenced by multiple edges, the engine must produce exactly one boundary_ref node for it, not multiple copies. The hash computation is deterministic (same public identifiers + same salt = same hash), so this is a correctness check on the engine's node deduplication logic.

For the CSPRNG path (no public identifiers), the engine generates the random token once per node and reuses it for all references. The implementation pre-computes all values into a `HashMap<NodeId, String>` before building output nodes:

```rust
let mut boundary_ref_values: HashMap<NodeId, String> = HashMap::new();
for node in &file.nodes {
    if !matches!(node_actions.get(&node.id), Some(NodeAction::Replace)) {
        continue;
    }
    let public_ids: Vec<CanonicalId> = /* collect public identifiers */;
    let hash = boundary_ref_value(&public_ids, &salt)?;
    boundary_ref_values.insert(node.id.clone(), hash);
}
```

---

## 8. Security Considerations

### 8.1 Graph Structure Leakage

Boundary references preserve graph topology. An adversary can count boundary_ref nodes, observe their degree, and infer structural properties of the hidden graph (e.g., a high in-degree boundary_ref on `supplies` edges reveals a hub supplier). This is inherent to the design: boundary references exist to preserve connectivity for downstream analysis. Producers concerned about topology leakage should omit edges to boundary refs entirely (producing a disconnected subgraph). The rule that edges between two replaced nodes are omitted (Section 6.2) partially mitigates this by hiding internal connectivity of the redacted portion.

### 8.2 Salt Entropy

The file salt must have at least 256 bits of CSPRNG entropy. A weak salt enables precomputation: an adversary could hash all ~2.5M LEIs in the GLEIF database and match boundary references to entities. The `getrandom` crate provides the correct entropy source per platform (Linux: `getrandom(2)`, macOS: `getentropy(2)`, Windows: `BCryptGenRandom`, WASM: `crypto.getRandomValues`).

The salt is visible in the header -- its purpose is anti-enumeration, not secrecy. It forces O(N) work per file where N is the candidate identifier count, and with 2^256 possible salts, rainbow tables are infeasible. Fresh salt per file prevents cross-file correlation of redacted entities.

### 8.3 Timing Side-Channels

The SHA-256 vs. CSPRNG branch is observable via timing but is not a meaningful threat: the branch condition is already visible in the output (deterministic hashes are reproducible; random tokens are not). The `sha2` pure-Rust implementation avoids secret-dependent branches in the compression function.

### 8.4 Threat Model

**Trusted:** The producer (the entity running `omtsf redact`). The producer has access to the full unredacted graph and decides which nodes to retain, replace, or omit.

**Untrusted:** The recipient of the redacted file. The recipient may attempt to:

- **Reverse boundary reference hashes** to recover entity identities. Mitigated by the salt: the adversary must guess both the entity identifier(s) and verify against the salt. With a fresh salt per file, precomputed tables are useless.
- **Correlate boundary references across files** from the same producer. Mitigated by fresh salt per file. Two exports of the same graph will produce different hashes for the same redacted entity.
- **Infer sensitive properties from graph structure.** Partially mitigated by the edge omission rules (Section 6.2, 6.3, 6.4). Topology leakage at the boundary remains (Section 8.1).
- **Tamper with the file** to inject false data. Out of scope for the redaction engine; integrity is addressed by digital signatures, which are a separate concern.

**Out of scope:** Compromised producers (adversary already has the unredacted graph) and access control (the engine transforms files, it does not gate access).

### 8.5 CSPRNG Failure Mode

If the platform CSPRNG is unavailable, `getrandom` returns an error propagated as `BoundaryHashError::CsprngFailure`. The engine must never fall back to a weaker random source -- a failed redaction is preferable to predictable boundary references.
