# OMTSF Specification: Entity Identification

**Spec:** OMTSF-SPEC-001
**Status:** Draft
**Date:** 2026-02-17
**Addresses:** C1, C8, C15, M3, M5, P0-1, P1-20, P2-15

---

## 1. Problem Statement

Entity identification is the load-bearing foundation of the OMTSF architecture. Without a defined identifier strategy, the merge-by-concatenation model described in the vision is theoretical: if two parties export files using different identifiers for the same legal entity, merge produces duplicates instead of a unified graph.

No single global business identifier exists:

- **LEI** (Legal Entity Identifier) covers ~2.7 million entities, skewed toward financial institutions. Open and free to query, but costs $50--200/year per entity to obtain. Does not cover facilities or unregistered entities.
- **DUNS** (Dun & Bradstreet) covers ~500 million entities -- the broadest coverage -- but is proprietary. Hierarchy data is a premium product. Redistribution is restricted by license.
- **GLN** (GS1 Global Location Number) covers locations and parties within the GS1 membership base (~2 million companies). Requires GS1 membership. No comprehensive public registry.
- **National company registry numbers** are authoritative within their jurisdiction but use incompatible formats across ~200 countries. The US has no federal registry; Germany fragments by court. A number is only meaningful paired with its jurisdiction.
- **Tax IDs** (VAT, EIN, TIN) have high coverage but are legally confidential in most jurisdictions. Using them as primary keys in exchanged files raises GDPR and privacy concerns.

The consequence: any specification that mandates a single identifier scheme excludes the majority of supply chain participants. The solution is a composite identifier model that treats all schemes as peers.

---

## 2. Design Principles

**No single mandatory scheme.** The format MUST NOT require any single proprietary or paid identifier system. An entity with only an internal ERP vendor number is as representable as one with an LEI.

**Composite identity.** Every entity node carries an array of zero or more external identifiers from multiple schemes. The more identifiers an entity carries, the higher the probability of successful cross-file merge.

**Graph-local vs. external identity.** File-local IDs (used for edge source/target references within a single file) are structurally distinct from external identifiers (used for cross-file merge). They serve different purposes and MUST NOT be conflated.

**Scheme-qualified identifiers.** Every identifier declares its scheme. A bare number is meaningless; `duns:081466849` is unambiguous.

**Internal identifiers are first-class.** ERP vendor numbers, buyer-assigned supplier codes, and other system-local IDs are the most common identifiers in practice. They MUST be representable without requiring translation to a global scheme.

**Sensitivity-aware.** Some identifiers (tax IDs, internal codes) carry privacy or confidentiality constraints. The identifier model supports sensitivity classification to enable selective redaction.

**Temporally valid.** Identifiers change over time. Companies re-register, merge, acquire new LEIs, or lose DUNS numbers. The model supports temporal validity on every identifier.

---

## 3. Identifier Model

### 3.1 Graph-Local Node Identity

Every node in an `.omts` file MUST have an `id` field containing a string that is unique within that file. This ID is used solely for edge source/target references within the same file. It carries no semantic meaning and MUST NOT be used for cross-file merge or entity resolution.

Producers MAY use any string format for graph-local IDs: UUIDs, sequential integers, human-readable slugs, or opaque tokens. Validators MUST only enforce uniqueness within the file.

### 3.2 External Identifier Structure

Each node carries an optional `identifiers` array. Each entry is an **identifier record** with the following fields:

| Field | Required | Type | Description |
|-------|----------|------|-------------|
| `scheme` | Yes | string | Identifier scheme code from the controlled vocabulary (Section 4) |
| `value` | Yes | string | The identifier value within that scheme |
| `authority` | Conditional | string | Issuing authority or jurisdiction qualifier. Required for `nat-reg`, `vat`, and `internal` schemes. |
| `valid_from` | No | string (ISO 8601 date) | Date this identifier became effective for this entity |
| `valid_to` | No | string (ISO 8601 date) | Date this identifier ceased to be valid for this entity. `null` means currently valid. |
| `sensitivity` | No | enum | One of `public`, `restricted`, `confidential`. Default: `public`. |

**Rationale for `authority` as conditional:** Some schemes are globally unambiguous (LEI is always issued by a GLEIF-accredited LOU; DUNS is always issued by D&B). Others require disambiguation: a national registry number is meaningless without its jurisdiction, a VAT number needs its country, and an internal ID needs its issuing system.

### 3.3 Graph-Local Edge Identity

Every edge in an `.omts` file MUST have an `id` field containing a string that is unique within that file. This supports the directed labeled multigraph model: multiple edges of the same type between the same pair of nodes are permitted and distinguished by their independent IDs.

Edges MAY carry an optional `identifiers` array with the same structure as node identifiers, enabling cross-file merge of edges when needed.

---

## 4. Identifier Scheme Vocabulary

### 4.1 Core Schemes

Conformant OMTSF validators MUST recognize the following schemes and enforce their format validation rules.

#### `lei` -- Legal Entity Identifier

- **Standard:** ISO 17442
- **Authority:** GLEIF (Global Legal Entity Identifier Foundation)
- **Format:** 20-character alphanumeric string. Characters 1--18 are the entity-specific part (alphanumeric). Characters 19--20 are check digits (numeric).
- **Validation:** MUST match `^[A-Z0-9]{18}[0-9]{2}$`. MUST pass MOD 97-10 check digit verification (ISO 7064).
- **`authority` field:** Not required. The issuing LOU can be derived from the LEI itself via the GLEIF API.
- **Coverage:** ~2.7 million entities worldwide. Strong in financial services, growing in supply chain due to regulatory mandates (EU CSDDD, MiFID II).
- **Data availability:** 100% open. Full database downloadable from GLEIF at no cost. Includes Level 1 (entity data) and Level 2 (corporate hierarchy via accounting consolidation relationships).

#### `duns` -- DUNS Number

- **Authority:** Dun & Bradstreet
- **Format:** 9-digit numeric string.
- **Validation:** MUST match `^[0-9]{9}$`.
- **`authority` field:** Not required.
- **Coverage:** ~500 million entities worldwide. Broadest single-system coverage. Includes branches, divisions, and sole proprietorships.
- **Data availability:** Proprietary. Free to obtain a number; expensive to query data or hierarchy. OMTSF files MAY contain DUNS numbers (they are just strings), but enrichment/validation requires D&B data access.
- **Note:** D&B's corporate hierarchy (Family Tree) is a premium product. OMTSF represents hierarchy via edge types (Section 6), not via the identifier scheme.

#### `gln` -- Global Location Number

- **Standard:** GS1 General Specifications
- **Authority:** GS1 (federated via ~115 national Member Organizations)
- **Format:** 13-digit numeric string.
- **Validation:** MUST match `^[0-9]{13}$`. MUST pass GS1 mod-10 check digit (last digit).
- **`authority` field:** Not required. The GS1 Company Prefix embedded in the GLN identifies the issuing MO.
- **Coverage:** Used by 2+ million GS1 member companies. Strong in retail, FMCG, healthcare. Weaker in mining, heavy industry.
- **Note:** GLN can identify legal entities, functional entities, or physical locations. OMTSF disambiguates via node type (`organization` vs. `facility`), not via the identifier scheme.

#### `nat-reg` -- National Company Registry

- **Authority:** Government company registries (e.g., UK Companies House, German Handelsregister, French RCS)
- **Format:** Varies by jurisdiction.
- **Validation:** `authority` field is REQUIRED and MUST contain a valid GLEIF Registration Authority (RA) code from the RA list maintained by GLEIF (ISO 17442-2). `value` format validation is authority-specific and MAY be deferred to Level 2 validation.
- **`authority` field:** Required. Contains the GLEIF RA code (e.g., `RA000585` for UK Companies House, `RA000548` for German Handelsregister).
- **Coverage:** Collectively comprehensive for all formally registered entities within their jurisdictions.

**Common authority codes:**

| RA Code | Registry | Jurisdiction |
|---------|----------|-------------|
| `RA000585` | Companies House | United Kingdom |
| `RA000548` | Handelsregister | Germany |
| `RA000525` | Registre du Commerce (SIREN) | France |
| `RA000665` | Kamer van Koophandel | Netherlands |
| `RA000476` | National Tax Board (houjin bangou) | Japan |
| `RA000553` | Ministry of Corporate Affairs (CIN) | India |

The full GLEIF RA list contains 700+ registration authorities and is available at `https://www.gleif.org/en/about-lei/code-lists/gleif-registration-authorities-list`.

#### `vat` -- VAT / Tax Identification Number

- **Authority:** National tax authorities
- **Format:** Varies by jurisdiction. EU VAT numbers are prefixed by a 2-letter country code.
- **Validation:** `authority` field is REQUIRED and MUST contain a valid ISO 3166-1 alpha-2 country code. Format validation is country-specific and MAY be deferred to Level 2 validation.
- **`authority` field:** Required. ISO 3166-1 alpha-2 country code (e.g., `DE`, `GB`, `US`).
- **Sensitivity:** Default sensitivity for `vat` identifiers is `restricted`. Producers SHOULD explicitly set sensitivity. Validators MUST NOT reject a file for omitting `vat` identifiers.

**Privacy note:** Tax IDs are legally protected data in most jurisdictions. OMTSF files containing `vat` identifiers with `sensitivity: "confidential"` are subject to the selective disclosure rules in Section 9.

#### `internal` -- System-Local Identifier

- **Authority:** The issuing system (ERP, procurement platform, internal database)
- **Format:** Opaque string. No format constraints beyond non-empty.
- **Validation:** `authority` field is REQUIRED and MUST be a non-empty string identifying the issuing system.
- **`authority` field:** Required. Free-form string identifying the source system. Recommended convention: `{system-type}-{instance-id}` (e.g., `sap-mm-prod`, `oracle-scm-us`, `ariba-network`).
- **Merge behavior:** `internal` identifiers NEVER trigger cross-file merge. They are scoped to their issuing system and are meaningful only within that context.

### 4.2 Extension Schemes

Conformant validators MAY recognize additional schemes. Extension scheme codes MUST use one of the following patterns to avoid collision with future core schemes:

- **Reverse-domain notation:** `com.example.supplier-id`, `org.gs1.sgln`
- **Known extension codes:**

| Scheme Code | Name | Notes |
|-------------|------|-------|
| `org.opencorporates` | OpenCorporates | Value is `{jurisdiction}/{number}` (e.g., `gb/07228507`) |
| `org.refinitiv.permid` | Refinitiv PermID | Numeric identifier |
| `org.iso.isin` | ISIN | 12-character alphanumeric, ISO 6166 |
| `org.gs1.gtin` | Global Trade Item Number | 8, 12, 13, or 14 digits |

Validators encountering an unrecognized scheme code MUST NOT reject the file. Unknown schemes are passed through without format validation.

---

## 5. Entity Type Taxonomy

OMTSF distinguishes three core node types. This separation addresses the panel finding (m15) that the vision conflates facilities with organizations.

### 5.1 `organization`

A legal entity: a company, non-governmental organization, government body, or other formally registered entity with legal standing.

**Typical identifiers:** LEI, DUNS (headquarters), national registry number, VAT number.

**Properties:**
- `name` (required): Legal name of the entity
- `jurisdiction` (recommended): ISO 3166-1 alpha-2 country code of incorporation or primary registration
- `status` (optional): `active`, `dissolved`, `merged`, `suspended`

### 5.2 `facility`

A physical location: a factory, warehouse, farm, mine, port, or office.

**Typical identifiers:** GLN, DUNS (branch), internal site code.

**Properties:**
- `name` (required): Name or label of the facility
- `operator` (recommended): Graph-local `id` of the `organization` node that operates this facility (also representable as an edge)
- `address` (optional): Structured or free-text address
- `geo` (optional): WGS 84 coordinates (`lat`, `lon`) or GeoJSON geometry for polygon boundaries (relevant for EUDR land parcel traceability)

### 5.3 `good`

A product, material, commodity, or service that flows through the supply network.

**Typical identifiers:** GTIN, HS/CN commodity code, internal SKU, CAS number (chemicals).

**Properties:**
- `name` (required): Name or description of the good
- `commodity_code` (optional): HS or CN code
- `unit` (optional): Unit of measure (e.g., `kg`, `mt`, `pcs`)

---

## 6. Corporate Hierarchy Edge Types

The vision document's graph model requires edge types that represent corporate structure, not just commercial supply relationships. This addresses critical finding C15 (corporate hierarchy absent from data model) and aligns with GLEIF Level 2 relationship data while extending beyond it.

### 6.1 `ownership`

An equity ownership relationship between two `organization` nodes.

| Property | Required | Type | Description |
|----------|----------|------|-------------|
| `percentage` | Yes | number (0--100) | Ownership percentage. `0` indicates a known relationship where percentage is unknown. |
| `direct` | No | boolean | `true` if direct ownership, `false` if indirect. Default: `true`. |
| `valid_from` | Yes | ISO 8601 date | Date this ownership relationship became effective |
| `valid_to` | No | ISO 8601 date | Date this relationship ended. `null` = current. |

**Validation (Level 3):** The sum of all inbound `ownership` edges to a single node with overlapping validity periods SHOULD NOT exceed 100%.

### 6.2 `operational_control`

Operational control without equity ownership. Covers franchises, management contracts, tolling arrangements, and licensed manufacturing.

| Property | Required | Type | Description |
|----------|----------|------|-------------|
| `control_type` | Yes | enum | One of: `franchise`, `management`, `tolling`, `licensed_manufacturing`, `other` |
| `valid_from` | Yes | ISO 8601 date | |
| `valid_to` | No | ISO 8601 date | |

### 6.3 `legal_parentage`

Direct legal parent-subsidiary relationship. Maps to GLEIF Level 2 `IS_DIRECTLY_CONSOLIDATED_BY`.

| Property | Required | Type | Description |
|----------|----------|------|-------------|
| `valid_from` | Yes | ISO 8601 date | |
| `valid_to` | No | ISO 8601 date | |
| `consolidation_basis` | No | enum | `ifrs10`, `us_gaap_asc810`, `other`, `unknown` |

**Direction convention:** Edge points from child (subsidiary) to parent. `source` = subsidiary, `target` = parent.

### 6.4 `former_identity`

Represents identity transformation events: mergers, acquisitions, renames, and demergers.

| Property | Required | Type | Description |
|----------|----------|------|-------------|
| `event_type` | Yes | enum | One of: `merger`, `acquisition`, `rename`, `demerger`, `spin_off` |
| `effective_date` | Yes | ISO 8601 date | Date the event took effect |
| `description` | No | string | Human-readable description of the event |

**Direction convention:** Edge points from the predecessor entity to the successor entity. `source` = old identity, `target` = new/surviving identity.

---

## 7. Merge Semantics

Merge is the operation of combining two or more `.omts` files that describe overlapping portions of a supply network into a single coherent graph. The vision describes this as "concatenating and deduplicating lists." This section defines what deduplication means.

### 7.1 Identity Predicate for Nodes

Two nodes from different files are **merge candidates** if and only if they share at least one external identifier record where all of the following hold:

1. `scheme` values are equal (case-sensitive string comparison)
2. `value` values are equal (case-sensitive string comparison after normalization: leading/trailing whitespace trimmed, for numeric-only schemes leading zeros are significant)
3. If `authority` is present in **either** record, `authority` values MUST be equal (case-insensitive string comparison)

The `internal` scheme is explicitly excluded: `internal` identifiers NEVER satisfy the identity predicate across files, because they are scoped to their issuing system.

### 7.2 Identity Predicate for Edges

Two edges from different files are **merge candidates** if all of the following hold:

1. Their resolved source nodes are merge candidates (or the same node post-merge)
2. Their resolved target nodes are merge candidates (or the same node post-merge)
3. Their `type` values are equal
4. They share at least one external identifier (if identifiers are present on edges), OR they have no external identifiers and their core properties are equal (same `type`, same resolved endpoints, same non-temporal properties)

This definition supports the multigraph model: two edges with the same type and endpoints but different properties (e.g., two distinct supply contracts) are NOT merge candidates unless they share an explicit external identifier.

### 7.3 Merge Procedure

Given files A and B:

1. **Concatenate** all nodes from A and B into a single list.
2. **Identify** merge candidate pairs using the identity predicate (Section 7.1).
3. **Merge** each candidate pair:
   - The merged node retains the **union** of all identifier records from both sources.
   - For each property present in both source nodes:
     - If values are equal: retain the value.
     - If values differ: the merger MUST record both values with their provenance (source file, reporting entity). Conflict resolution is a tooling concern.
   - The merged node's graph-local `id` is assigned by the merger (it is an arbitrary file-local string).
4. **Rewrite** all edge source/target references to use the merged node IDs.
5. **Identify** merge candidate edge pairs using the edge identity predicate (Section 7.2).
6. **Deduplicate** edges that are merge candidates, merging their properties as with nodes.
7. **Retain** all non-duplicate edges.

### 7.4 Merge Provenance

To support post-merge auditability, the merged file SHOULD include a `merge_metadata` section in the file header recording:

- Source file identifiers (file hash or filename)
- Merge timestamp
- Number of nodes and edges merged
- Number of property conflicts detected

---

## 8. Identifier Sensitivity and Selective Disclosure

Supply chain graphs contain competitively sensitive information. The identifier model interacts with selective disclosure in two ways: individual identifier redaction and whole-node redaction.

### 8.1 Identifier Sensitivity Levels

| Level | Meaning | Behavior in Subgraph Projection |
|-------|---------|-------------------------------|
| `public` | No restrictions on sharing | Always included |
| `restricted` | Share only with direct trading partners | MAY be omitted in files shared beyond direct partners |
| `confidential` | Do not share outside the originating organization | MUST be omitted in any file shared externally |

Default sensitivity by scheme:
- `lei`: `public`
- `duns`: `public`
- `gln`: `public`
- `nat-reg`: `public`
- `vat`: `restricted`
- `internal`: `restricted`

Producers MAY override defaults by setting `sensitivity` explicitly on any identifier record.

### 8.2 Boundary References (Redacted Nodes)

When a node is redacted in a subgraph projection (the file represents only a portion of the full graph), the redacted node is replaced with a **boundary reference**: a minimal node stub that preserves graph connectivity without revealing the entity's identity.

A boundary reference node:
- Has `type` set to `boundary_ref`
- Has a single identifier with `scheme` set to `opaque`
- The `value` of the opaque identifier is `SHA-256(canonical_identifiers || file_salt)` where:
  - `canonical_identifiers` is the sorted, concatenated string of all `public` identifiers on the original node in the format `scheme:authority:value` (with empty authority omitted)
  - `file_salt` is a random 32-byte value included in the file header, unique per file generation
- Has no other properties

This design prevents enumeration attacks: an adversary cannot hash known LEIs to discover whether a specific entity appears in the redacted graph, because the salt is file-specific.

---

## 9. Validation Rules

Validation is tiered per the panel recommendation (P1-4). The identifier model defines validation rules at each level.

### 9.1 Level 1 -- Structural Integrity

These rules MUST pass for a file to be considered structurally valid.

| Rule | Description |
|------|-------------|
| L1-ID-01 | Every node MUST have an `id` field containing a non-empty string unique within the file |
| L1-ID-02 | Every edge MUST have an `id` field containing a non-empty string unique within the file |
| L1-ID-03 | Every edge `source` and `target` MUST reference an existing node `id` in the same file |
| L1-ID-04 | Every identifier record MUST have a non-empty `scheme` field |
| L1-ID-05 | Every identifier record MUST have a non-empty `value` field |
| L1-ID-06 | For schemes requiring `authority` (`nat-reg`, `vat`, `internal`), the `authority` field MUST be present and non-empty |
| L1-ID-07 | `scheme` MUST be either a core scheme code or a valid extension scheme code (reverse-domain notation) |
| L1-ID-08 | For `lei` scheme: `value` MUST match `^[A-Z0-9]{18}[0-9]{2}$` |
| L1-ID-09 | For `duns` scheme: `value` MUST match `^[0-9]{9}$` |
| L1-ID-10 | For `gln` scheme: `value` MUST match `^[0-9]{13}$` |
| L1-ID-11 | `valid_from` and `valid_to`, if present, MUST be valid ISO 8601 date strings |
| L1-ID-12 | If both `valid_from` and `valid_to` are present, `valid_from` MUST be less than or equal to `valid_to` |
| L1-ID-13 | `sensitivity`, if present, MUST be one of `public`, `restricted`, `confidential` |
| L1-ID-14 | `boundary_ref` nodes MUST have exactly one identifier with `scheme: "opaque"` |
| L1-ID-15 | No two identifier records on the same node may have identical `scheme`, `value`, and `authority` |

### 9.2 Level 2 -- Completeness

These rules SHOULD be satisfied. Violations produce warnings, not errors.

| Rule | Description |
|------|-------------|
| L2-ID-01 | Every `organization` node SHOULD have at least one external identifier (scheme other than `internal`) |
| L2-ID-02 | Every `facility` node SHOULD be connected to an `organization` node via an edge or the `operator` property |
| L2-ID-03 | Temporal fields (`valid_from`, `valid_to`) SHOULD be present on all identifier records |
| L2-ID-04 | `ownership` edges SHOULD have `valid_from` set |
| L2-ID-05 | `lei` values SHOULD pass MOD 97-10 check digit verification |
| L2-ID-06 | `gln` values SHOULD pass GS1 mod-10 check digit verification |
| L2-ID-07 | `nat-reg` authority values SHOULD be valid GLEIF RA codes |
| L2-ID-08 | `vat` authority values SHOULD be valid ISO 3166-1 alpha-2 country codes |

### 9.3 Level 3 -- Enrichment

These rules require external data sources and are intended for enrichment tooling, not mandatory validation.

| Rule | Description |
|------|-------------|
| L3-ID-01 | `lei` values SHOULD be verifiable against the GLEIF public database (entity exists and is not retired) |
| L3-ID-02 | `nat-reg` values SHOULD be cross-referenceable with the authority's registry |
| L3-ID-03 | The sum of inbound `ownership` `percentage` values to any single node (for overlapping validity periods) SHOULD NOT exceed 100 |
| L3-ID-04 | `legal_parentage` edges SHOULD form a forest (no cycles in the parentage subgraph) |
| L3-ID-05 | If a node has both `lei` and `nat-reg` identifiers, they SHOULD be consistent with GLEIF Level 1 cross-reference data |

---

## 10. Standards Mapping

This section documents how OMTSF entity identification relates to existing standards, per panel recommendation P0-11.

### 10.1 Identifier Systems

| OMTSF Scheme | Standard | Relationship |
|-------------|----------|-------------|
| `lei` | ISO 17442 | **Reuses.** OMTSF adopts LEI as-is. Format validation follows ISO 17442 check digit rules. |
| `duns` | D&B proprietary | **References.** OMTSF references DUNS as an identifier scheme. No dependency on D&B data products. |
| `gln` | GS1 General Specifications | **Reuses.** OMTSF adopts GLN format and check digit rules from GS1. |
| `nat-reg` | ISO 17442-2 (GLEIF RA list) | **Reuses.** OMTSF uses GLEIF's Registration Authority code list for jurisdiction qualification. |
| `vat` | ISO 3166-1 (country codes) | **Reuses** ISO 3166-1 alpha-2 for jurisdiction qualification. |

### 10.2 Data Models

| OMTSF Concept | Related Standard | Relationship |
|---------------|-----------------|-------------|
| Directed labeled property multigraph | ISO/IEC 39075 (GQL) Property Graph Model | **Aligns with.** OMTSF adopts the same conceptual model: nodes and edges with independent identity, labels (types), and properties. |
| Identifier scheme qualification | ISO 6523 (ICD), UN/CEFACT UNTDID 3055 | **Informed by.** OMTSF's scheme-qualified identifier pattern follows the same principle as ISO 6523 International Code Designator and UNTDID code list 3055. |
| Corporate hierarchy | GLEIF Level 2 relationship data | **Extends.** OMTSF includes GLEIF Level 2's accounting consolidation concept (`legal_parentage`) and extends it with `ownership` (including minority stakes), `operational_control`, and `former_identity`. |
| Identifier URI format | GS1 EPC URI, GS1 Digital Link | **Compatible with.** OMTSF's `scheme:value` format can be mechanically converted to/from GS1 EPC URIs (e.g., `gln:0614141000036` ↔ `urn:epc:id:sgln:0614141.00001.0`). |
| Composite identifier model | PEPPOL Participant Identifiers | **Informed by.** PEPPOL's `{scheme}:{identifier}` pattern (with ISO 6523 ICD scheme codes) directly influenced OMTSF's design. |

### 10.3 Regulatory Alignment

| Regulation | Entity Identification Requirement | OMTSF Coverage |
|-----------|----------------------------------|---------------|
| EU CSDDD | Identify business partners and entities in the value chain | `organization` nodes with external identifiers; `ownership` and `legal_parentage` edges for corporate structure |
| EUDR | Identify operators, traders, and geolocated production plots | `organization` nodes (operators/traders) + `facility` nodes with `geo` coordinates |
| German LkSG | Identify direct and indirect suppliers | Full graph model with multi-tier node and edge representation |
| US UFLPA | Map supply chains to identify entities in Xinjiang region | `organization` and `facility` nodes with `jurisdiction` and `geo` properties |
| EU CBAM | Identify installations and operators for carbon reporting | `facility` nodes (installations) linked to `organization` nodes (operators) via edges |

---

## 11. ERP Integration Mapping

This section provides reference mappings for how entity identifiers in common ERP systems correspond to OMTSF identifier records.

### 11.1 SAP S/4HANA

| SAP Field | Table/Structure | OMTSF Mapping |
|-----------|----------------|---------------|
| `LIFNR` (Vendor Number) | `LFA1` | `scheme: "internal"`, `authority: "{sap_system_id}"` |
| `STCD1` (Tax Number 1) | `LFA1` | `scheme: "vat"`, `authority` from `LAND1` (country key) |
| `STCD2` (Tax Number 2) | `LFA1` | `scheme: "vat"`, `authority` from `LAND1` |
| Custom DUNS field | `LFA1` (via append structure) | `scheme: "duns"` |
| `NAME1`--`NAME4` | `LFA1` | Node `name` property |
| `LAND1` (Country Key) | `LFA1` | Node `jurisdiction` property |
| `EKORG` (Purchasing Org) | `LFM1` | Context for `internal` authority scoping |

### 11.2 Oracle SCM Cloud

| Oracle Field | Object | OMTSF Mapping |
|-------------|--------|---------------|
| `VENDOR_ID` | Supplier | `scheme: "internal"`, `authority: "{oracle_instance}"` |
| `VENDOR_SITE_ID` | Supplier Site | Separate `facility` node with `internal` identifier |
| `TAX_REGISTRATION_NUMBER` | Supplier | `scheme: "vat"`, `authority` from country |
| `DUNS_NUMBER` | Supplier | `scheme: "duns"` |
| `VENDOR_NAME` | Supplier | Node `name` property |

### 11.3 Microsoft Dynamics 365

| D365 Field | Entity | OMTSF Mapping |
|-----------|--------|---------------|
| `VendAccount` | VendTable | `scheme: "internal"`, `authority: "{d365_instance}"` |
| `TaxRegistrationId` | VendTable | `scheme: "vat"`, `authority` from country |
| `DunsNumber` | DirPartyTable | `scheme: "duns"` |
| `Name` | DirPartyTable | Node `name` property |

---

## 12. Serialization Example

A complete minimal `.omts` file fragment demonstrating the entity identification model:

```json
{
  "omtsf_version": "0.1.0",
  "snapshot_date": "2026-02-17",
  "file_salt": "a1b2c3d4e5f6...",
  "nodes": [
    {
      "id": "org-acme",
      "type": "organization",
      "name": "Acme Manufacturing GmbH",
      "jurisdiction": "DE",
      "identifiers": [
        { "scheme": "lei", "value": "5493006MHB84DD0ZWV18" },
        { "scheme": "nat-reg", "value": "HRB86891", "authority": "RA000548" },
        { "scheme": "vat", "value": "DE123456789", "authority": "DE", "sensitivity": "restricted" },
        { "scheme": "duns", "value": "081466849" },
        { "scheme": "internal", "value": "V-100234", "authority": "sap-mm-prod", "sensitivity": "restricted" }
      ]
    },
    {
      "id": "org-bolt",
      "type": "organization",
      "name": "Bolt Supplies Ltd",
      "jurisdiction": "GB",
      "identifiers": [
        { "scheme": "nat-reg", "value": "07228507", "authority": "RA000585" },
        { "scheme": "duns", "value": "234567890" }
      ]
    },
    {
      "id": "fac-bolt-sheffield",
      "type": "facility",
      "name": "Bolt Sheffield Plant",
      "identifiers": [
        { "scheme": "gln", "value": "5060012340001" },
        { "scheme": "internal", "value": "SITE-SHF-01", "authority": "bolt-erp" }
      ],
      "geo": { "lat": 53.3811, "lon": -1.4701 }
    },
    {
      "id": "good-steel-bolts",
      "type": "good",
      "name": "M10 Steel Hex Bolts",
      "identifiers": [
        { "scheme": "org.gs1.gtin", "value": "05060012340018" }
      ],
      "commodity_code": "7318.15"
    }
  ],
  "edges": [
    {
      "id": "edge-001",
      "type": "supplies",
      "source": "org-bolt",
      "target": "org-acme",
      "properties": {
        "valid_from": "2023-01-15",
        "valid_to": null
      }
    },
    {
      "id": "edge-002",
      "type": "operates",
      "source": "org-bolt",
      "target": "fac-bolt-sheffield",
      "properties": {
        "valid_from": "2018-06-01"
      }
    },
    {
      "id": "edge-003",
      "type": "produces",
      "source": "fac-bolt-sheffield",
      "target": "good-steel-bolts",
      "properties": {
        "valid_from": "2020-03-01"
      }
    },
    {
      "id": "edge-004",
      "type": "ownership",
      "source": "org-bolt",
      "target": "org-acme",
      "properties": {
        "percentage": 0,
        "valid_from": "2023-01-15",
        "description": "No ownership relationship; included for completeness"
      }
    }
  ]
}
```

---

## 13. Open Questions

These questions are flagged for resolution during panel review of this specification:

1. **Canonical string format for identifiers.** Should the canonical compact representation be `scheme:value` (PEPPOL-style) or `scheme:authority:value` (structured)? The compact form is needed for content hashing and canonical encoding (per P1-16). This spec currently uses the structured object form; the compact string form is needed for boundary reference hashing and deterministic comparisons.

2. **Minimum identifier requirement.** Should Level 1 (structural) validation require at least one external identifier per `organization` node, or should this remain a Level 2 (completeness) recommendation? Requiring it at Level 1 would catch data quality problems early but would block files from ERP systems that only have vendor numbers.

3. **Edge merge strategy.** Should edge identity for cross-file merge use independent edge identifiers (requiring explicit IDs on edges), or a composite key of (resolved source, resolved target, type, properties hash)? The current spec supports both but does not mandate edge identifiers for merge.

---

## Appendix A: Check Digit Algorithms

### A.1 LEI Check Digit (MOD 97-10, ISO 7064)

1. Replace each letter with its numeric equivalent: A=10, B=11, ..., Z=35.
2. Move the first 4 characters (2-letter prefix + 2-digit check digits) to the end.
3. Compute the integer value modulo 97.
4. Result MUST equal 1.

### A.2 GS1 Check Digit (Mod-10)

For a 13-digit GLN `d1 d2 d3 ... d13`:
1. Multiply odd-positioned digits (d1, d3, d5, ..., d11) by 1 and even-positioned digits (d2, d4, d6, ..., d12) by 3.
2. Sum all products.
3. `d13` = (10 - (sum mod 10)) mod 10.
