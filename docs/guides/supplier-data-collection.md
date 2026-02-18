# Supplier Data Collection Guide

**Status:** Informative
**Date:** 2026-02-18

This guide provides practical recommendations for collecting supply chain data from suppliers, particularly Tier-2+ SMEs who may have limited data management capabilities.

---

## 1. Minimum Viable Data Request (Tier-1 Suppliers)

The following fields represent the minimum data set needed to create a useful OMTSF graph entry for a direct supplier:

| Field | Priority | Source | Notes |
|-------|----------|--------|-------|
| Legal entity name | Required | Supplier | Full legal name as registered |
| Country of incorporation | Required | Supplier / public records | ISO 3166-1 alpha-2 code |
| National registry number | Recommended | Supplier / public records | Company registration number + jurisdiction |
| VAT number | Recommended | Supplier / invoice data | Often already in ERP from invoicing |
| DUNS number | Optional | Supplier / D&B | May already be in ERP |
| LEI | Optional | GLEIF API | Query by name + jurisdiction to find existing LEIs |
| Primary facility address | Recommended | Supplier | Physical address of main production site |
| Facility coordinates | Optional | Geocoding service | Can be derived from address; required for EUDR |

**Data collection method:** A structured questionnaire (spreadsheet or web form) sent during supplier onboarding or annual review. Most Tier-1 suppliers in regulated industries are accustomed to such requests.

---

## 2. Simplified Template for Tier-2+ SMEs

Tier-2 and deeper suppliers are often small or medium enterprises with limited administrative capacity. Demanding the full data set is counterproductive — a simpler template increases response rates.

### Minimum fields for Tier-2+ SMEs:

1. **Company name** (as it appears on invoices)
2. **Country**
3. **What they supply** (free-text product/service description)
4. **Who they supply to** (name of the Tier-1 or intermediary they sell to)

That's it. Four fields. Everything else can be enriched later.

### Why this works:

- **Name + country** is sufficient for OpenCorporates or GLEIF API lookup to find registry numbers and LEIs.
- **Product description** maps to a `supplies` edge with a `commodity` property.
- **Customer name** establishes the graph connectivity.

### Template format:

A simple spreadsheet with columns:

```
| Company Name | Country | Product/Service | Customer Name |
|-------------|---------|-----------------|---------------|
| [fill in]   | [fill in] | [fill in]     | [fill in]     |
```

Provide this as a downloadable CSV or Excel template. Avoid complex formats (XML, JSON) for supplier-facing data collection.

---

## 3. Progressive Enrichment Path

Supply chain data quality improves over time. The following path moves from minimal to comprehensive:

### Stage 1: Name + Country (Day 1)
- Collect during supplier onboarding.
- Sufficient to create `organization` nodes with `name` and `jurisdiction`.
- No cross-file merge capability yet.

### Stage 2: Add One External Identifier (Month 1-3)
- Query GLEIF API (free) for LEI matches on name + jurisdiction.
- Query OpenCorporates (free tier) for national registry numbers.
- Extract VAT numbers from existing invoice data in ERP.
- Result: L2-EID-01 satisfied; cross-file merge now possible.

### Stage 3: Add Facility Data (Month 3-6)
- Request primary production site addresses from key suppliers.
- Geocode addresses for `geo` coordinates.
- Create `facility` nodes linked via `operates` edges.
- Result: EUDR geolocation readiness for relevant commodities.

### Stage 4: Add Tier-2 Visibility (Month 6-12)
- Request Tier-1 suppliers to identify their critical suppliers (Tier-2).
- Use the simplified SME template (Section 2) for Tier-2 data.
- Create `supplies` edges from Tier-2 to Tier-1 organizations.
- Result: Multi-tier graph with basic Tier-2 visibility.

### Stage 5: Full Enrichment (Ongoing)
- Register LEIs for strategic suppliers that lack one.
- License D&B data for corporate hierarchy mapping.
- Verify identifiers against authoritative sources.
- Result: L3 enrichment level; high-confidence cross-file merge.

---

## 4. GDPR Considerations for Person Data

OMTSF `person` nodes (beneficial owners, directors) contain personal data subject to GDPR and equivalent regulations.

### Before collecting person data:

1. **Legal basis.** Identify the legal basis for processing (GDPR Article 6). For supply chain due diligence, legitimate interest (Article 6(1)(f)) is the most common basis. For AMLD compliance, legal obligation (Article 6(1)(c)) applies.

2. **Data minimization.** Collect only the minimum personal data required. For beneficial ownership under AMLD, this is: name, nationality, percentage of ownership/control, and nature of control. Do not collect addresses, dates of birth, or government ID numbers unless legally required.

3. **Sensitivity classification.** All person data in OMTSF defaults to `sensitivity: "confidential"` (OMTSF-SPEC-004, Section 5). This means:
   - Person nodes are omitted from files with `disclosure_scope: "public"`.
   - Person identifiers are never included in boundary reference hashes.
   - `beneficial_ownership` edges inherit confidential sensitivity.

4. **Supplier notification.** When collecting UBO data from suppliers, inform them of the purpose, legal basis, retention period, and their data subject rights. A standard privacy notice template for supply chain due diligence is recommended.

5. **Cross-border transfers.** Person data in OMTSF files shared across jurisdictions must comply with GDPR Chapter V (international transfers). Standard Contractual Clauses (SCCs) or adequacy decisions are the typical mechanisms.

### Practical recommendations:

- **Do not ask SME suppliers for UBO data unless legally required.** Most CSDDD and LkSG obligations do not require UBO identification for every supplier.
- **Use public UBO registers where available.** Many EU member states maintain public beneficial ownership registers (per AMLD 5). Query these instead of burdening suppliers.
- **Set retention periods.** Delete person data when the supply relationship ends and the legal retention period expires.
- **Separate person data from entity data in collection workflows.** This makes it easier to apply different retention and access policies.

---

## 5. Data Quality Tips

- **Prefer structured identifiers over free-text names.** A VAT number is unambiguous; "Acme GmbH" vs "ACME Manufacturing GmbH" creates merge failures.
- **Validate at collection time.** Check LEI checksums, DUNS format (9 digits), and VAT format before accepting data.
- **Record the source.** Use the `data_quality.source` field to track where each piece of data came from (e.g., `supplier-questionnaire`, `gleif-api`, `invoice-data`).
- **Timestamp verification.** Use the `verification_date` field to record when identifiers were last verified. Stale data is better than no data, but consumers should know how fresh it is.
