# OMTS Excel Templates

Two Excel templates for importing supply-chain data into the [Open Multi-Tier Supply-chain Format (OMTS)](../../spec/graph-data-model.md).

## Which template should I use?

| Template | Best for | Sheets |
|----------|----------|--------|
| **Supplier List** (`omts-supplier-list-template.xlsx`) | Quick supplier onboarding, procurement lists, LKSG/CSDDD reporting | 1 sheet |
| **Full Import** (`omts-import-template.xlsx`) | Complete supply-chain modelling with facilities, goods, attestations, corporate structure | 11 sheets |

**Start with the Supplier List** if you just need to list your suppliers with basic metadata. Move to the Full Import template when you need to model facilities, certifications, ownership structures, or consignments.

## Supplier List Template

### Getting started

1. Open `omts-supplier-list-template.xlsx`
2. Fill in the metadata area (rows 1-2): your organization name, snapshot date, and disclosure scope
3. Add one row per supplier relationship starting at row 5
4. Hover over any column header to see a tooltip describing the expected format

### Column reference

| Column | Header | Required | Description |
|--------|--------|----------|-------------|
| A | supplier_name | Yes | Legal name of the supplier |
| B | supplier_id | No | Dedup key — rows with the same value collapse to one org node |
| C | jurisdiction | No | ISO 3166-1 alpha-2 country code (e.g. GB, DE, CN) |
| D | tier | No | 1 = direct, 2 = sub-supplier, 3 = sub-sub-supplier (default: 1) |
| E | parent_supplier | No | For tier 2/3: name or supplier_id of the tier N-1 supplier |
| F | business_unit | No | Internal BU managing this relationship |
| G | commodity | No | HS/CN code or description (e.g. 7318.15) |
| H | valid_from | No | ISO 8601 date (YYYY-MM-DD) |
| I | annual_value | No | Annual spend (numeric) |
| J | value_currency | No | ISO 4217 currency (EUR, USD, GBP) |
| K | contract_ref | No | Contract or MSA reference |
| L | lei | No | Legal Entity Identifier (20 chars, ISO 17442) |
| M | duns | No | D-U-N-S Number (9 digits) |
| N | vat | No | VAT or tax ID |
| O | vat_country | No | Country that issued the VAT number |
| P | internal_id | No | Your internal vendor number |
| Q | risk_tier | No | critical, high, medium, or low |
| R | kraljic_quadrant | No | strategic, leverage, bottleneck, or non-critical |
| S | approval_status | No | approved, conditional, pending, blocked, or phase-out |
| T | notes | No | Free text (not imported into graph) |

### Multi-BU and dedup

The same supplier can appear on multiple rows with different `business_unit` values. Use `supplier_id` to ensure they collapse to a single organization node in the graph:

```
supplier_name     | supplier_id | business_unit | commodity | risk_tier
Bolt Supplies Ltd | bolt-001    | Procurement   | 7318.15   | low
Bolt Supplies Ltd | bolt-001    | Engineering   | 7318.16   | medium
```

This produces one organization node (`bolt-001`) with two supply-relationship edges, each carrying its own risk classification.

### Tiered suppliers

Use the `tier` and `parent_supplier` columns to model sub-suppliers:

```
supplier_name         | tier | parent_supplier
Bolt Supplies Ltd     | 1    |
Yorkshire Steel Works | 2    | Bolt Supplies Ltd
```

`parent_supplier` can reference either a `supplier_name` or `supplier_id` from another row.

### Conversion

```bash
omtsf import-supplier-list omts-supplier-list-example.xlsx -o output.omts
```

## Full Import Template

### Getting started

1. Open `omts-import-template.xlsx`
2. Fill in the **Metadata** sheet (snapshot_date is required)
3. Add organizations to the **Organizations** sheet
4. Add relationships in **Supply Relationships** and/or **Corporate Structure**
5. Optionally fill in Facilities, Goods, Attestations, Consignments, Persons, Same As, and Identifiers
6. Hover over any column header to see a tooltip describing the expected format

### Sheet overview

| Sheet | Purpose | Key required fields |
|-------|---------|-------------------|
| Metadata | File-level settings | snapshot_date |
| Organizations | Legal entities | name |
| Facilities | Physical locations | name |
| Goods | Products and materials | name |
| Attestations | Certifications and audits | name, attestation_type, valid_from |
| Consignments | Batches and shipments (CBAM/EUDR) | name |
| Supply Relationships | Supply-chain edges | type, supplier_id, buyer_id, valid_from |
| Corporate Structure | Ownership and control edges | type, subsidiary_id, parent_id, valid_from |
| Persons | Beneficial owners (confidential) | name |
| Same As | Entity deduplication assertions | entity_a, entity_b |
| Identifiers | Additional identifiers | node_id, scheme, value |

### Node IDs

Every entity sheet has an `id` column. If left blank, the importer auto-generates an ID from the name. Use explicit IDs when you need to reference entities from other sheets (e.g. `supplier_id` in Supply Relationships must match an `id` in Organizations).

### Edge direction

- **Supply Relationships**: `supplier_id` is the source (who supplies), `buyer_id` is the target (who buys)
- **Corporate Structure**: `subsidiary_id` is the child entity, `parent_id` is the parent entity

### Conversion

```bash
omtsf import-excel omts-import-example.xlsx -o output.omts
```

## Common conventions

### Dates

All dates use ISO 8601 format: **YYYY-MM-DD** (e.g. `2026-01-15`).

### Country codes

Use ISO 3166-1 alpha-2 codes: `GB`, `DE`, `CN`, `US`, `SE`, etc.

### Currency codes

Use ISO 4217 codes: `EUR`, `USD`, `GBP`, `CNY`, etc.

### Identifiers

| Scheme | Format | Example |
|--------|--------|---------|
| LEI | 20-character alphanumeric (ISO 17442) | `5493006MHB84DD0ZWV18` |
| DUNS | 9 digits | `234567890` |
| GLN | 13 digits (GS1) | `5060012340001` |
| VAT | Country-specific | `DE123456789` |
| National registry | Country-specific + GLEIF RA code | `HRB86891` / `RA000548` |

### Required vs optional fields

- Yellow-highlighted cells in the template indicate required fields
- Dropdown menus appear for columns with constrained value sets
- Hover over column headers to see detailed descriptions

## Regenerating templates

The templates are generated by `generate_template.py`. To regenerate after modifying the script:

```bash
python3 templates/excel/generate_template.py
```

This produces four files:
- `omts-import-template.xlsx` — empty multi-sheet template
- `omts-import-example.xlsx` — multi-sheet template with Acme-Bolt example data
- `omts-supplier-list-template.xlsx` — empty single-sheet supplier list
- `omts-supplier-list-example.xlsx` — supplier list with multi-BU example data
