# OMTSF Specification: Entity Identification

**Spec:** OMTSF-SPEC-001
**Status:** Draft
**Date:** 2026-02-17
**Revision:** 2 (post-panel review)
**License:** This specification is licensed under [CC-BY-4.0](https://creativecommons.org/licenses/by/4.0/). Code artifacts in this repository are licensed under Apache 2.0.
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

**Unknown fields:** Conformant parsers MUST preserve unknown fields in identifier records during round-trip serialization. Unknown fields MUST NOT cause validation failure at any level. This ensures forward compatibility when future spec versions add fields (e.g., `confidence`, `verification`).

### 3.3 Graph-Local Edge Identity

Every edge in an `.omts` file MUST have an `id` field containing a string that is unique within that file. This supports the directed labeled multigraph model: multiple edges of the same type between the same pair of nodes are permitted and distinguished by their independent IDs.

Edges MAY carry an optional `identifiers` array with the same structure as node identifiers, enabling cross-file merge of edges when needed.

### 3.4 Canonical Identifier String Format

Each identifier record has a **canonical string form** used for sorting, hashing, and deterministic comparison:

- For schemes requiring `authority`: `{scheme}:{authority}:{value}`
- For schemes without `authority`: `{scheme}:{value}`

Examples:
- `lei:5493006MHB84DD0ZWV18`
- `nat-reg:RA000548:HRB86891`
- `vat:DE:DE123456789`
- `internal:sap-mm-prod:V-100234`
- `duns:081466849`

**Encoding rules:**
- All components are UTF-8 encoded
- The colon (`:`, U+003A) is the delimiter
- If an `authority` or `value` contains a literal colon, it MUST be percent-encoded as `%3A`
- If an `authority` or `value` contains a literal percent sign, it MUST be percent-encoded as `%25`

This canonical form is used in boundary reference hashing (Section 10.3) and merge identity comparison (Section 9.1).

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

**LEI Registration Status and Lifecycle:**

LEIs have a registration status maintained by GLEIF. The following statuses affect OMTSF processing:

| LEI Status | Meaning | OMTSF Merge Behavior | Validation |
|------------|---------|---------------------|------------|
| `ISSUED` | Active, annually renewed | Normal merge candidate | -- |
| `LAPSED` | Failed to renew; entity still exists | Still valid for merge. The entity is unchanged; only the registration fee is unpaid. | L2 warning |
| `RETIRED` | Voluntarily retired by the entity | Still valid for merge for historical data. Producers SHOULD set `valid_to` on the identifier. | L2 warning |
| `MERGED` | Entity merged into another; successor LEI exists | Still valid for merge. Producers SHOULD create a `former_identity` edge linking the retired-LEI node to the successor-LEI node, with `event_type: "merger"`. | L2 warning |
| `ANNULLED` | Issued in error or fraudulently | MUST NOT be used for merge. Treat as invalid. | L2 error |

The GLEIF database provides explicit successor relationships for MERGED LEIs via the `SuccessorEntity` field. Tooling that performs Level 3 enrichment SHOULD retrieve successor LEI data and generate `former_identity` edges automatically.

#### `duns` -- DUNS Number

- **Authority:** Dun & Bradstreet
- **Format:** 9-digit numeric string.
- **Validation:** MUST match `^[0-9]{9}$`.
- **`authority` field:** Not required.
- **Coverage:** ~500 million entities worldwide. Broadest single-system coverage. Includes branches, divisions, and sole proprietorships.
- **Data availability:** Proprietary. Free to obtain a number; expensive to query data or hierarchy. OMTSF files MAY contain DUNS numbers (they are just strings), but enrichment/validation requires D&B data access.
- **Note:** D&B's corporate hierarchy (Family Tree) is a premium product. OMTSF represents hierarchy via edge types (Section 6), not via the identifier scheme.

**DUNS Branch/HQ Disambiguation:**

D&B assigns separate DUNS numbers to different structural levels of the same legal entity. The D&B Family Tree model defines:

| D&B Level | Description | OMTSF Mapping |
|-----------|-------------|---------------|
| **Global Ultimate** | Topmost entity in the corporate family | `organization` node. Link to subsidiaries via `legal_parentage` or `ownership` edges. |
| **Domestic Ultimate** | Topmost entity within a single country | `organization` node. Link to Global Ultimate via `legal_parentage` edge. |
| **Parent** | Direct legal parent of a subsidiary | `organization` node. Link via `legal_parentage` edge. |
| **Headquarters** | Main office of a company with branches | `organization` node. The HQ DUNS is the primary identifier for the legal entity. |
| **Branch** | A physical location or division of an entity | `facility` node. The branch DUNS identifies the location, not a separate legal entity. |

**Key guidance for producers:**

- A single legal entity may hold multiple DUNS numbers (HQ + branches). The HQ DUNS identifies the entity; branch DUNS numbers identify its locations.
- When a DUNS number identifies a branch, it SHOULD be assigned to a `facility` node, not an `organization` node.
- Merge engines SHOULD be aware that two nodes with different DUNS numbers may represent the same legal entity (one HQ, one branch). Level 3 validation MAY flag this by querying D&B's Family Tree linkage.
- When an ERP system stores only a single DUNS number and it is unclear whether it is an HQ or branch DUNS, producers SHOULD assign it to an `organization` node and note the ambiguity.

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
- **Validation:** `authority` field is REQUIRED and MUST contain a valid GLEIF Registration Authority (RA) code from the OMTSF-maintained RA list snapshot (see Section 4.4). `value` format validation is authority-specific and MAY be deferred to Level 2 validation.
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
| `RA000602` | Division of Corporations | Delaware, US |
| `RA000631` | Secretary of State | California, US |

The full GLEIF RA list contains 700+ registration authorities and is available at `https://www.gleif.org/en/about-lei/code-lists/gleif-registration-authorities-list`.

#### `vat` -- VAT / Tax Identification Number

- **Authority:** National tax authorities
- **Format:** Varies by jurisdiction. EU VAT numbers are prefixed by a 2-letter country code.
- **Validation:** `authority` field is REQUIRED and MUST contain a valid ISO 3166-1 alpha-2 country code. Format validation is country-specific and MAY be deferred to Level 2 validation.
- **`authority` field:** Required. ISO 3166-1 alpha-2 country code (e.g., `DE`, `GB`, `US`).
- **Sensitivity:** Default sensitivity for `vat` identifiers is `restricted`. Producers SHOULD explicitly set sensitivity. Validators MUST NOT reject a file for omitting `vat` identifiers.

**Privacy note:** Tax IDs are legally protected data in most jurisdictions. OMTSF files containing `vat` identifiers with `sensitivity: "confidential"` are subject to the selective disclosure rules in Section 10.

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

### 4.3 Scheme Governance Process

The identifier scheme vocabulary is a controlled registry that requires governance to evolve without fragmenting the ecosystem.

**Adding a new core scheme** requires:
1. A written proposal submitted as a pull request to the OMTSF repository, including: scheme code, issuing authority, format specification, validation rules, coverage estimate, data availability assessment, and at least one production deployment demonstrating use.
2. A 30-day public review period.
3. Approval by the OMTSF Technical Steering Committee (TSC) via lazy consensus (no objection within the review period) or explicit majority vote if objections are raised.

**Criteria for core scheme inclusion:**
- The scheme MUST have a publicly available specification.
- The identifier values MUST NOT be encumbered by intellectual property restrictions that prevent their inclusion in OMTSF files.
- The scheme MUST have demonstrated coverage of a meaningful population of supply chain entities (suggested threshold: 100,000+ entities or regulatory mandate).
- The issuing authority MUST be identifiable and operational.

**Promoting an extension scheme to core** follows the same process as adding a new scheme. Regulatory mandate (e.g., a regulation effectively requiring a particular identifier) is a sufficient basis for promotion.

**Deprecating a core scheme** requires:
1. A written rationale documenting why the scheme should be deprecated (e.g., issuing authority dissolved, scheme superseded).
2. A 90-day notice period.
3. Deprecated schemes remain recognized by validators for at least 2 major spec versions after deprecation.

### 4.4 GLEIF RA List Versioning

The `nat-reg` scheme depends on the GLEIF Registration Authority code list, which is maintained by GLEIF and updated periodically. To decouple OMTSF validation from GLEIF's publication timing:

1. The OMTSF project MUST maintain a versioned snapshot of the GLEIF RA list in the repository (e.g., `data/gleif-ra-list-2026Q1.csv`).
2. Each spec revision MUST reference a specific snapshot version (e.g., "based on GLEIF RA list retrieved 2026-01-15").
3. Snapshots SHOULD be updated quarterly, aligned with GLEIF's publication cadence.
4. **Validator behavior for unknown RA codes:** Validators encountering an `authority` value not present in the referenced snapshot SHOULD emit a warning but MUST NOT reject the file. This ensures that newly added RA codes do not break validation between snapshot updates.
5. The snapshot update process follows the standard pull request workflow and does not require TSC approval.

**Current reference:** GLEIF RA list retrieved 2026-01-15 (700+ registration authorities).

---

## 5. Entity Type Taxonomy

OMTSF distinguishes four core node types. This separation addresses the panel finding (m15) that the vision conflates facilities with organizations, and the regulatory requirement (CSDDD, AMLD) to trace ownership to natural persons.

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

### 5.4 `person`

A natural person relevant to the supply chain graph, typically as a beneficial owner, director, or authorized representative. This node type addresses CSDDD and EU Anti-Money Laundering Directive (AMLD 5/6) requirements for tracing ownership to natural persons.

**Typical identifiers:** National ID (via `nat-reg`), internal reference codes. LEI and DUNS do not apply to natural persons.

**Properties:**
- `name` (required): Full name of the person
- `jurisdiction` (recommended): ISO 3166-1 alpha-2 country code of nationality or primary residence
- `role` (optional): Free-text description of the person's role (e.g., "Ultimate Beneficial Owner", "Director")

**Privacy constraints:**
- All identifiers on `person` nodes default to `sensitivity: "confidential"` regardless of scheme-level defaults. Producers MAY override to `restricted` where legally permitted.
- `person` nodes MUST be omitted entirely (not replaced with boundary references) when generating files with `disclosure_scope: "public"`. This reflects GDPR data minimization requirements.
- Producers MUST assess whether including `person` nodes complies with applicable data protection law (GDPR, CCPA, etc.) before generating files.

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

### 6.5 `beneficial_ownership`

An ownership or control relationship between a `person` node and an `organization` node. This edge type supports the EU CSDDD and AMLD 5/6 requirements for identifying ultimate beneficial owners (UBOs).

| Property | Required | Type | Description |
|----------|----------|------|-------------|
| `percentage` | No | number (0--100) | Ownership percentage. Omit if unknown. |
| `control_type` | Yes | enum | One of: `voting_rights`, `capital`, `other_means`, `senior_management` |
| `direct` | No | boolean | `true` if direct ownership, `false` if through intermediary entities. Default: `true`. |
| `valid_from` | Yes | ISO 8601 date | |
| `valid_to` | No | ISO 8601 date | |

**Direction convention:** `source` = `person` (the beneficial owner), `target` = `organization` (the entity owned/controlled).

**Determining UBO status:** Under AMLD, a person is a UBO if they hold >25% of shares or voting rights, or exercise control through other means. OMTSF records the raw ownership data; the 25% threshold determination is a tooling concern.

**Privacy:** `beneficial_ownership` edges inherit the sensitivity constraints of `person` nodes. They default to `sensitivity: "confidential"` and MUST be omitted from files with `disclosure_scope: "public"`.

---

## 7. Supply Relationship Edge Types

The corporate hierarchy edges in Section 6 describe corporate structure. This section defines the edge types for commercial and operational supply relationships -- the actual supply chain. These are as important for interoperability as entity identification: two parties modeling the same supply relationship must use compatible edge types for merge to produce correct results.

### 7.1 `supplies`

A direct commercial supply relationship: one entity sells goods or services to another.

| Property | Required | Type | Description |
|----------|----------|------|-------------|
| `valid_from` | Yes | ISO 8601 date | Start of the supply relationship |
| `valid_to` | No | ISO 8601 date | End of the supply relationship. `null` = ongoing. |
| `commodity` | No | string | HS code or free-text description of what is supplied |
| `contract_ref` | No | string | Reference to a contract or purchase agreement |

**Direction convention:** `source` = supplier, `target` = buyer.

**Regulatory relevance:** A `supplies` edge between two `organization` nodes constitutes a "direct business relationship" under CSDDD Article 3(e) and a "direct supplier" under LkSG Section 2(7).

### 7.2 `subcontracts`

A delegated production relationship: one entity contracts another to perform production work on its behalf. The subcontractor produces goods or performs services that the contracting entity delivers to its own customer.

| Property | Required | Type | Description |
|----------|----------|------|-------------|
| `valid_from` | Yes | ISO 8601 date | |
| `valid_to` | No | ISO 8601 date | |
| `commodity` | No | string | What is subcontracted |
| `contract_ref` | No | string | Subcontracting agreement reference |

**Direction convention:** `source` = subcontractor, `target` = contracting entity.

**Regulatory relevance:** Subcontracting relationships create indirect supply chain exposure. Under LkSG Section 9, substantiated knowledge of human rights violations at a subcontractor triggers due diligence obligations.

### 7.3 `tolls`

A tolling or processing arrangement: one entity provides raw materials to another, which processes them and returns the finished or semi-finished product. The material owner retains ownership throughout.

| Property | Required | Type | Description |
|----------|----------|------|-------------|
| `valid_from` | Yes | ISO 8601 date | |
| `valid_to` | No | ISO 8601 date | |
| `commodity` | No | string | Material being tolled/processed |

**Direction convention:** `source` = toll processor, `target` = material owner.

### 7.4 `distributes`

A logistics or distribution relationship: one entity provides warehousing, transportation, or distribution services for another's goods.

| Property | Required | Type | Description |
|----------|----------|------|-------------|
| `valid_from` | Yes | ISO 8601 date | |
| `valid_to` | No | ISO 8601 date | |
| `service_type` | No | enum | `warehousing`, `transport`, `fulfillment`, `other` |

**Direction convention:** `source` = logistics provider, `target` = goods owner.

### 7.5 `brokers`

An intermediary relationship: one entity arranges transactions between buyers and sellers without taking possession of the goods.

| Property | Required | Type | Description |
|----------|----------|------|-------------|
| `valid_from` | Yes | ISO 8601 date | |
| `valid_to` | No | ISO 8601 date | |
| `commodity` | No | string | What is brokered |

**Direction convention:** `source` = broker, `target` = entity on whose behalf the broker acts.

### 7.6 `operates`

An operational relationship between an `organization` node and a `facility` node.

| Property | Required | Type | Description |
|----------|----------|------|-------------|
| `valid_from` | Yes | ISO 8601 date | |
| `valid_to` | No | ISO 8601 date | |

**Direction convention:** `source` = organization, `target` = facility.

### 7.7 `produces`

A production relationship between a `facility` node and a `good` node, indicating that the facility produces or processes that good.

| Property | Required | Type | Description |
|----------|----------|------|-------------|
| `valid_from` | Yes | ISO 8601 date | |
| `valid_to` | No | ISO 8601 date | |

**Direction convention:** `source` = facility, `target` = good.

---

## 8. Attestation and Certification Model

Supply chain due diligence regulations require documentary evidence: EUDR demands due diligence statements per consignment, LkSG requires documented risk analysis and preventive measures, CSDDD requires stakeholder consultations and remediation plans, and buyers routinely require ISO/SA8000/SMETA certifications from suppliers. This section defines the model for attaching such evidence to the graph.

### 8.1 `attestation` Node Type

An attestation is a document, certificate, audit result, or due diligence statement that is linked to one or more entities or facilities.

**Properties:**

| Property | Required | Type | Description |
|----------|----------|------|-------------|
| `name` | Yes | string | Name or title of the attestation |
| `attestation_type` | Yes | enum | One of: `certification`, `audit`, `due_diligence_statement`, `self_declaration`, `other` |
| `standard` | No | string | The standard or framework (e.g., `SA8000`, `ISO 14001`, `SMETA`, `EUDR-DDS`) |
| `issuer` | No | string | Name of the issuing/certifying body |
| `valid_from` | Yes | ISO 8601 date | Date the attestation became effective |
| `valid_to` | No | ISO 8601 date | Expiration date. `null` = no expiration. |
| `outcome` | No | enum | `pass`, `conditional_pass`, `fail`, `pending`, `not_applicable` |
| `reference` | No | string | Document reference number or URI |

**Identifiers:** Attestation nodes MAY carry an `identifiers` array (e.g., an EUDR due diligence statement number, an internal audit ID).

### 8.2 `attested_by` Edge Type

Links an entity, facility, or good to an attestation.

| Property | Required | Type | Description |
|----------|----------|------|-------------|
| `scope` | No | string | What aspect the attestation covers (e.g., "working conditions", "deforestation-free", "carbon emissions") |

**Direction convention:** `source` = the entity/facility/good being attested, `target` = the `attestation` node.

### 8.3 Usage Examples

A facility with SA8000 certification:
```json
{
  "id": "att-sa8000-bolt",
  "type": "attestation",
  "name": "SA8000 Certification - Bolt Sheffield Plant",
  "attestation_type": "certification",
  "standard": "SA8000:2014",
  "issuer": "Social Accountability International",
  "valid_from": "2025-06-01",
  "valid_to": "2028-05-31",
  "outcome": "pass"
}
```

An EUDR due diligence statement:
```json
{
  "id": "att-eudr-dds-001",
  "type": "attestation",
  "name": "EUDR Due Diligence Statement #DDS-2026-00142",
  "attestation_type": "due_diligence_statement",
  "standard": "EUDR-DDS",
  "valid_from": "2026-01-15",
  "outcome": "pass",
  "reference": "DDS-2026-00142"
}
```

---

## 9. Merge Semantics

Merge is the operation of combining two or more `.omts` files that describe overlapping portions of a supply network into a single coherent graph. The vision describes this as "concatenating and deduplicating lists." This section defines what deduplication means.

### 9.1 Identity Predicate for Nodes

Two nodes from different files are **merge candidates** if and only if they share at least one external identifier record where all of the following hold:

1. `scheme` values are equal (case-sensitive string comparison)
2. `value` values are equal (case-sensitive string comparison after normalization: leading/trailing whitespace trimmed, for numeric-only schemes leading zeros are significant)
3. If `authority` is present in **either** record, `authority` values MUST be equal (case-insensitive string comparison)

The `internal` scheme is explicitly excluded: `internal` identifiers NEVER satisfy the identity predicate across files, because they are scoped to their issuing system.

### 9.2 Identity Predicate for Edges

Two edges from different files are **merge candidates** if all of the following hold:

1. Their resolved source nodes are merge candidates (or the same node post-merge)
2. Their resolved target nodes are merge candidates (or the same node post-merge)
3. Their `type` values are equal
4. They share at least one external identifier (if identifiers are present on edges), OR they have no external identifiers and their core properties are equal (same `type`, same resolved endpoints, same non-temporal properties)

This definition supports the multigraph model: two edges with the same type and endpoints but different properties (e.g., two distinct supply contracts) are NOT merge candidates unless they share an explicit external identifier.

### 9.3 Merge Procedure

Given files A and B:

1. **Concatenate** all nodes from A and B into a single list.
2. **Identify** merge candidate pairs using the identity predicate (Section 9.1).
3. **Compute transitive closure** of merge candidates. If node X is a merge candidate with node Y (via identifier I1), and node Y is a merge candidate with node Z (via identifier I2), then X, Y, and Z are all merged into a single node. This is required because the same real-world entity may carry different identifiers in different files (e.g., LEI in file A, DUNS in file B, both LEI and DUNS in file C).
4. **Merge** each candidate group:
   - The merged node retains the **union** of all identifier records from all sources.
   - For each property present in multiple source nodes:
     - If values are equal: retain the value.
     - If values differ: the merger MUST record both values with their provenance (source file, reporting entity). Conflict resolution is a tooling concern.
   - The merged node's graph-local `id` is assigned by the merger (it is an arbitrary file-local string).
5. **Rewrite** all edge source/target references to use the merged node IDs.
6. **Identify** merge candidate edge pairs using the edge identity predicate (Section 9.2).
7. **Deduplicate** edges that are merge candidates, merging their properties as with nodes.
8. **Retain** all non-duplicate edges.

### 9.4 Algebraic Properties

For the decentralized merge model to work -- where different parties independently merge overlapping files without coordination -- the merge operation MUST satisfy the following algebraic properties:

**Commutativity:** `merge(A, B) = merge(B, A)`. The order in which two files are provided to a merge operation MUST NOT affect the result. This is satisfied by the identity predicate (symmetric) and the union-based merge procedure.

**Associativity:** `merge(merge(A, B), C) = merge(A, merge(B, C))`. Three-file merge MUST produce the same result regardless of grouping. This is satisfied by the transitive closure computation in step 3: the final merge graph is determined by the full set of identifier overlap relationships, not by the order in which they are discovered.

**Idempotency:** `merge(A, A) = A`. Merging a file with itself MUST produce an equivalent graph (same nodes, edges, identifiers, and properties; graph-local IDs may differ).

**Implementation note:** The transitive closure requirement means merge implementations SHOULD use a union-find (disjoint set) data structure for efficient merge candidate grouping. This operates in O(n * α(n)) time, where α is the inverse Ackermann function (effectively constant).

### 9.5 Merge Provenance

To support post-merge auditability, the merged file SHOULD include a `merge_metadata` section in the file header recording:

- Source file identifiers (file hash or filename)
- Merge timestamp
- Number of nodes and edges merged
- Number of property conflicts detected

---

## 10. Identifier Sensitivity and Selective Disclosure

Supply chain graphs contain competitively sensitive information. The identifier model interacts with selective disclosure in two ways: individual identifier redaction and whole-node redaction.

### 10.1 Identifier Sensitivity Levels

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

### 10.2 Disclosure Scope

Files MAY declare a `disclosure_scope` in the file header to indicate the intended audience:

| Scope | Meaning |
|-------|---------|
| `internal` | For use within the originating organization only |
| `partner` | Shared with direct trading partners |
| `public` | Shared without restriction |

When `disclosure_scope` is declared:
- If `disclosure_scope` is `public`: the file MUST NOT contain identifiers with `sensitivity: "confidential"` or `sensitivity: "restricted"`. `person` nodes MUST NOT be present.
- If `disclosure_scope` is `partner`: the file MUST NOT contain identifiers with `sensitivity: "confidential"`.

Validators MUST enforce these constraints at Level 1 when `disclosure_scope` is present.

### 10.3 Boundary References (Redacted Nodes)

When a node is redacted in a subgraph projection (the file represents only a portion of the full graph), the redacted node is replaced with a **boundary reference**: a minimal node stub that preserves graph connectivity without revealing the entity's identity.

A boundary reference node:
- Has `type` set to `boundary_ref`
- Has a single identifier with `scheme` set to `opaque`
- The `value` of the opaque identifier is computed as follows:

**Hash computation:**

1. Collect all `public` identifiers on the original node.
2. Compute the canonical string form of each identifier (Section 3.4).
3. Sort the canonical strings lexicographically by UTF-8 byte order.
4. Join the sorted strings with a newline delimiter (`0x0A`).
5. If the resulting string is **non-empty**: `value` = hex-encoded `SHA-256(joined_string_bytes || file_salt_bytes)`
6. If the resulting string is **empty** (the node has no `public` identifiers): `value` = hex-encoded random 32-byte token generated by a CSPRNG. This ensures that each restricted-only entity produces a unique boundary reference, preventing the collision where all such entities would otherwise hash to the same value.

**`file_salt`** is a 32-byte value generated by a cryptographically secure pseudorandom number generator (CSPRNG, e.g., `/dev/urandom`, `getrandom(2)`, `crypto.getRandomValues()`). It is included in the file header as a 64-character lowercase hexadecimal string.

**Test vectors:**

Given identifiers:
- `lei:5493006MHB84DD0ZWV18` (public)
- `duns:081466849` (public)
- `vat:DE:DE123456789` (restricted, excluded from hash)

And `file_salt` = `0x00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff`:

1. Public canonical strings: `duns:081466849`, `lei:5493006MHB84DD0ZWV18`
2. Sorted: `duns:081466849`, `lei:5493006MHB84DD0ZWV18`
3. Joined: `duns:081466849\nlei:5493006MHB84DD0ZWV18`
4. Hash input: UTF-8 bytes of joined string || raw salt bytes
5. `value` = `SHA-256(hash_input)` hex-encoded

This design prevents enumeration attacks: an adversary cannot hash known LEIs to discover whether a specific entity appears in the redacted graph, because the salt is file-specific.

---

## 11. Validation Rules

Validation is tiered per the panel recommendation (P1-4). The identifier model defines validation rules at each level.

### 11.1 Level 1 -- Structural Integrity

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
| L1-ID-08 | For `lei` scheme: `value` MUST match `^[A-Z0-9]{18}[0-9]{2}$` and MUST pass MOD 97-10 check digit verification |
| L1-ID-09 | For `duns` scheme: `value` MUST match `^[0-9]{9}$` |
| L1-ID-10 | For `gln` scheme: `value` MUST match `^[0-9]{13}$` and MUST pass GS1 mod-10 check digit verification |
| L1-ID-11 | `valid_from` and `valid_to`, if present, MUST be valid ISO 8601 date strings |
| L1-ID-12 | If both `valid_from` and `valid_to` are present, `valid_from` MUST be less than or equal to `valid_to` |
| L1-ID-13 | `sensitivity`, if present, MUST be one of `public`, `restricted`, `confidential` |
| L1-ID-14 | `boundary_ref` nodes MUST have exactly one identifier with `scheme: "opaque"` |
| L1-ID-15 | No two identifier records on the same node may have identical `scheme`, `value`, and `authority` |
| L1-ID-16 | Edge `type` MUST be a recognized edge type from Sections 6, 7, or 8, or an extension type using reverse-domain notation |
| L1-ID-17 | If `disclosure_scope` is declared, sensitivity constraints (Section 10.2) MUST be satisfied |

### 11.2 Level 2 -- Completeness

These rules SHOULD be satisfied. Violations produce warnings, not errors.

| Rule | Description |
|------|-------------|
| L2-ID-01 | Every `organization` node SHOULD have at least one external identifier (scheme other than `internal`) |
| L2-ID-02 | Every `facility` node SHOULD be connected to an `organization` node via an edge or the `operator` property |
| L2-ID-03 | Temporal fields (`valid_from`, `valid_to`) SHOULD be present on all identifier records |
| L2-ID-04 | `ownership` edges SHOULD have `valid_from` set |
| L2-ID-05 | `nat-reg` authority values SHOULD be valid GLEIF RA codes per the current snapshot (Section 4.4) |
| L2-ID-06 | `vat` authority values SHOULD be valid ISO 3166-1 alpha-2 country codes |
| L2-ID-07 | `lei` values with LAPSED, RETIRED, or MERGED status (when detectable) SHOULD produce a warning |
| L2-ID-08 | `lei` values with ANNULLED status SHOULD produce an error |

### 11.3 Level 3 -- Enrichment

These rules require external data sources and are intended for enrichment tooling, not mandatory validation.

| Rule | Description |
|------|-------------|
| L3-ID-01 | `lei` values SHOULD be verifiable against the GLEIF public database (entity exists and status is not ANNULLED) |
| L3-ID-02 | `nat-reg` values SHOULD be cross-referenceable with the authority's registry |
| L3-ID-03 | The sum of inbound `ownership` `percentage` values to any single node (for overlapping validity periods) SHOULD NOT exceed 100 |
| L3-ID-04 | `legal_parentage` edges SHOULD form a forest (no cycles in the parentage subgraph) |
| L3-ID-05 | If a node has both `lei` and `nat-reg` identifiers, they SHOULD be consistent with GLEIF Level 1 cross-reference data |
| L3-ID-06 | For MERGED LEIs, a `former_identity` edge to the successor entity SHOULD be present |
| L3-ID-07 | DUNS numbers on `organization` nodes SHOULD be HQ-level DUNS, not branch DUNS |

---

## 12. Standards Mapping

This section documents how OMTSF entity identification relates to existing standards, per panel recommendation P0-11.

### 12.1 Identifier Systems

| OMTSF Scheme | Standard | Relationship |
|-------------|----------|-------------|
| `lei` | ISO 17442 | **Reuses.** OMTSF adopts LEI as-is. Format validation follows ISO 17442 check digit rules. |
| `duns` | D&B proprietary | **References.** OMTSF references DUNS as an identifier scheme. No dependency on D&B data products. |
| `gln` | GS1 General Specifications | **Reuses.** OMTSF adopts GLN format and check digit rules from GS1. |
| `nat-reg` | ISO 17442-2 (GLEIF RA list) | **Reuses.** OMTSF uses GLEIF's Registration Authority code list for jurisdiction qualification. |
| `vat` | ISO 3166-1 (country codes) | **Reuses** ISO 3166-1 alpha-2 for jurisdiction qualification. |

### 12.2 Data Models

| OMTSF Concept | Related Standard | Relationship |
|---------------|-----------------|-------------|
| Directed labeled property multigraph | ISO/IEC 39075 (GQL) Property Graph Model | **Aligns with.** OMTSF adopts the same conceptual model: nodes and edges with independent identity, labels (types), and properties. |
| Identifier scheme qualification | ISO 6523 (ICD), UN/CEFACT UNTDID 3055 | **Informed by.** OMTSF's scheme-qualified identifier pattern follows the same principle as ISO 6523 International Code Designator and UNTDID code list 3055. |
| Corporate hierarchy | GLEIF Level 2 relationship data | **Extends.** OMTSF includes GLEIF Level 2's accounting consolidation concept (`legal_parentage`) and extends it with `ownership` (including minority stakes), `operational_control`, `beneficial_ownership`, and `former_identity`. |
| Identifier URI format | GS1 EPC URI, GS1 Digital Link | **Compatible with.** OMTSF's `scheme:value` format can be mechanically converted to/from GS1 EPC URIs (e.g., `gln:0614141000036` <-> `urn:epc:id:sgln:0614141.00001.0`). |
| Composite identifier model | PEPPOL Participant Identifiers | **Informed by.** PEPPOL's `{scheme}:{identifier}` pattern (with ISO 6523 ICD scheme codes) directly influenced OMTSF's design. |

### 12.3 Regulatory Alignment

| Regulation | Entity Identification Requirement | OMTSF Coverage |
|-----------|----------------------------------|---------------|
| EU CSDDD | Identify business partners, value chain entities, and beneficial owners | `organization` nodes with external identifiers; `ownership`, `legal_parentage`, and `beneficial_ownership` edges; `person` nodes for UBOs |
| EUDR | Identify operators, traders, and geolocated production plots; due diligence statements | `organization` nodes (operators/traders) + `facility` nodes with `geo` coordinates; `attestation` nodes for DDS |
| German LkSG | Identify direct and indirect suppliers; documented risk analysis | Full graph with `supplies` and `subcontracts` edge types; `attestation` nodes for risk analysis documentation |
| US UFLPA | Map supply chains to identify entities in Xinjiang region | `organization` and `facility` nodes with `jurisdiction` and `geo` properties |
| EU CBAM | Identify installations and operators for carbon reporting | `facility` nodes (installations) linked to `organization` nodes (operators) via `operates` edges |
| EU AMLD 5/6 | Identify ultimate beneficial owners (natural persons) | `person` nodes linked to `organization` nodes via `beneficial_ownership` edges |

---

## 13. ERP Integration Mapping

This section provides reference mappings for how entity identifiers in common ERP systems correspond to OMTSF identifier records and edge types.

### 13.1 SAP S/4HANA

#### Node Derivation (Vendor Master)

| SAP Field | Table/Structure | OMTSF Mapping |
|-----------|----------------|---------------|
| `LIFNR` (Vendor Number) | `LFA1` | `scheme: "internal"`, `authority: "{sap_system_id}"` |
| `STCD1` (Tax Number 1) | `LFA1` | `scheme: "vat"`, `authority` from `LAND1` (country key) |
| `STCD2` (Tax Number 2) | `LFA1` | `scheme: "vat"`, `authority` from `LAND1` |
| Custom DUNS field | `LFA1` (via append structure) | `scheme: "duns"` |
| `NAME1`--`NAME4` | `LFA1` | Node `name` property |
| `LAND1` (Country Key) | `LFA1` | Node `jurisdiction` property |
| `EKORG` (Purchasing Org) | `LFM1` | Context for `internal` authority scoping |

#### Edge Derivation (Supply Relationships)

| SAP Table | Structure | OMTSF Mapping |
|-----------|-----------|---------------|
| `EINA` / `EINE` (Purchasing Info Record) | Vendor-material relationship | `supplies` edge from vendor `organization` to buyer `organization`, with `commodity` from material group |
| `EKKO` (PO Header) + `EKPO` (PO Item) | Purchase order | `supplies` edge (if no info record exists). Derive from `EKKO-LIFNR` (vendor) and `EKKO-BUKRS` (company code). |
| `EKKO-BSART` (PO Type) | Document type `UB` = subcontracting | `subcontracts` edge (when PO type indicates subcontracting) |
| `MARA` / `MARC` (Material Master) | Material → `good` node | `good` node with `scheme: "internal"`, `authority: "{sap_system_id}"`, `value` from `MATNR` |
| `RSEG` (Invoice Document) | Invoice line to vendor | Confirms `supplies` edge; provides volume/quantity data for edge properties |

#### Deduplication Note

In multi-client SAP landscapes, the same legal entity may appear as different `LIFNR` values across clients. The `authority` field on `internal` identifiers SHOULD include the client number (e.g., `sap-prod-100`, `sap-prod-200`) to distinguish these. See Section 14.1 for intra-file deduplication guidance.

### 13.2 Oracle SCM Cloud

| Oracle Field | Object | OMTSF Mapping |
|-------------|--------|---------------|
| `VENDOR_ID` | Supplier | `scheme: "internal"`, `authority: "{oracle_instance}"` |
| `VENDOR_SITE_ID` | Supplier Site | Separate `facility` node with `internal` identifier |
| `TAX_REGISTRATION_NUMBER` | Supplier | `scheme: "vat"`, `authority` from country |
| `DUNS_NUMBER` | Supplier | `scheme: "duns"` |
| `VENDOR_NAME` | Supplier | Node `name` property |
| `PO_HEADERS_ALL` + `PO_LINES_ALL` | Purchase orders | `supplies` edge derivation (vendor → buying org) |

### 13.3 Microsoft Dynamics 365

| D365 Field | Entity | OMTSF Mapping |
|-----------|--------|---------------|
| `VendAccount` | VendTable | `scheme: "internal"`, `authority: "{d365_instance}"` |
| `TaxRegistrationId` | VendTable | `scheme: "vat"`, `authority` from country |
| `DunsNumber` | DirPartyTable | `scheme: "duns"` |
| `Name` | DirPartyTable | Node `name` property |

---

## 14. Producer Guidance

This section provides guidance for producers (systems or processes that generate `.omts` files) on common challenges.

### 14.1 Intra-File Deduplication

ERP systems frequently contain duplicate records for the same real-world entity. In a typical SAP S/4HANA system with 20,000+ vendors, 5--15% are duplicates (same legal entity, different `LIFNR`). Producers MUST address this to avoid polluting the graph with duplicate nodes.

**Recommended approach:**

1. **Before export**, identify vendor records that represent the same legal entity. Two records are candidates for deduplication if they share any external identifier (`duns`, `lei`, `nat-reg`, `vat`) or if fuzzy name matching with address comparison produces high confidence.
2. **Produce one `organization` node per distinct legal entity**, carrying all `internal` identifiers from each source record. For example, if vendor `V-100` and `V-200` in SAP both represent Acme GmbH, produce a single node with two `internal` identifiers:
   ```json
   {
     "id": "org-acme",
     "type": "organization",
     "name": "Acme GmbH",
     "identifiers": [
       { "scheme": "internal", "value": "V-100", "authority": "sap-prod-100" },
       { "scheme": "internal", "value": "V-200", "authority": "sap-prod-200" },
       { "scheme": "duns", "value": "081466849" }
     ]
   }
   ```
3. **If deduplication is not feasible** (e.g., the producer cannot determine with sufficient confidence that two records represent the same entity), produce separate nodes and declare equivalence using a `same_as` edge:
   ```json
   {
     "id": "edge-sa-001",
     "type": "same_as",
     "source": "org-acme-v100",
     "target": "org-acme-v200",
     "properties": {
       "confidence": "probable",
       "basis": "name_match"
     }
   }
   ```
   The `same_as` edge type is advisory: merge engines MAY use it to combine nodes but are not required to.

### 14.2 Identifier Enrichment Lifecycle

Files typically begin with minimal identifiers (internal ERP codes only) and are enriched over time as external identifiers are obtained. This section defines the conceptual model for that progression.

**Enrichment levels:**

| Level | Description | Typical Identifiers | Merge Capability |
|-------|-------------|--------------------|--------------------|
| **Internal-only** | Raw ERP export | `internal` only | No cross-file merge possible |
| **Partially enriched** | Some external IDs obtained | `internal` + one of (`duns`, `nat-reg`, `vat`) | Cross-file merge possible where identifiers overlap |
| **Fully enriched** | Multiple external IDs verified | `internal` + `lei` + `nat-reg` + `vat` (+ `duns` where available) | High-confidence cross-file merge |

**Enrichment workflow:**

1. **Export:** Producer generates an `.omts` file from ERP data. Nodes carry `internal` identifiers and whatever external identifiers the ERP already holds (typically `vat` and sometimes `duns`).
2. **Match:** An enrichment tool takes the internal-only nodes and attempts to resolve them to external identifiers using available data sources (GLEIF, OpenCorporates, D&B, national registries).
3. **Augment:** The enrichment tool adds external identifiers to the nodes, preserving the original `internal` identifiers.
4. **Re-export:** The enriched file is written. It now passes Level 2 completeness checks (L2-ID-01).

**Important:** Enrichment MUST NOT remove or modify existing identifiers. It is an additive process. The original `internal` identifiers are preserved for reconciliation with the source system.

**Validation level alignment:**
- A file with only `internal` identifiers is valid at Level 1 (structural integrity).
- A file where most `organization` nodes have at least one external identifier satisfies Level 2 (completeness).
- A file where identifiers have been verified against authoritative sources satisfies Level 3 (enrichment).

---

## 15. Serialization Example

A complete minimal `.omts` file fragment demonstrating the entity identification model, including supply relationship edges, attestation, and person/beneficial ownership:

```json
{
  "omtsf_version": "0.1.0",
  "snapshot_date": "2026-02-17",
  "file_salt": "a1b2c3d4e5f67890a1b2c3d4e5f67890a1b2c3d4e5f67890a1b2c3d4e5f67890",
  "disclosure_scope": "partner",
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
    },
    {
      "id": "att-sa8000",
      "type": "attestation",
      "name": "SA8000 Certification",
      "attestation_type": "certification",
      "standard": "SA8000:2014",
      "issuer": "Social Accountability International",
      "valid_from": "2025-06-01",
      "valid_to": "2028-05-31",
      "outcome": "pass"
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
        "valid_to": null,
        "commodity": "7318.15"
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
      "source": "org-acme",
      "target": "org-bolt",
      "properties": {
        "percentage": 51.0,
        "valid_from": "2019-04-01"
      }
    },
    {
      "id": "edge-005",
      "type": "attested_by",
      "source": "fac-bolt-sheffield",
      "target": "att-sa8000",
      "properties": {
        "scope": "working conditions"
      }
    }
  ]
}
```

---

## 16. Open Questions

1. ~~**Canonical string format for identifiers.**~~ **Resolved.** Canonical format is `scheme:authority:value` (authority omitted when not required by the scheme). See Section 3.4.

2. ~~**Minimum identifier requirement.**~~ **Resolved.** External identifiers remain a Level 2 (completeness) recommendation (L2-ID-01), not a Level 1 structural requirement. This preserves the adoption ramp for ERP-only exports and is consistent with the design principle that internal identifiers are first-class.

3. **Edge merge strategy.** Should edge identity for cross-file merge use independent edge identifiers (requiring explicit IDs on edges), or a composite key of (resolved source, resolved target, type, properties hash)? The current spec supports both but does not mandate edge identifiers for merge.

4. **`same_as` edge semantics.** Should `same_as` edges be transitive? If node A `same_as` node B and node B `same_as` node C, does that imply A `same_as` C? This has implications for merge engines that consume intra-file equivalence declarations.

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
