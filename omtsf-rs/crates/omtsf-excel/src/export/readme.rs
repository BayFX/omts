/// Writes the README sheet with field definitions and usage instructions.
use rust_xlsxwriter::{Format, Worksheet, XlsxError};

use crate::error::ExportError;
use crate::export::style::header_format;

fn w(ws: &mut Worksheet, row: u32, col: u16, val: &str) -> Result<(), ExportError> {
    ws.write(row, col, val)
        .map(|_| ())
        .map_err(|e: XlsxError| ExportError::ExcelWrite {
            detail: e.to_string(),
        })
}

fn wf(ws: &mut Worksheet, row: u32, col: u16, val: &str, fmt: &Format) -> Result<(), ExportError> {
    ws.write_with_format(row, col, val, fmt)
        .map(|_| ())
        .map_err(|e: XlsxError| ExportError::ExcelWrite {
            detail: e.to_string(),
        })
}

/// Writes the README sheet into `ws`.
pub fn write_readme(ws: &mut Worksheet) -> Result<(), ExportError> {
    let bold = Format::new().set_bold();
    let header_fmt = header_format();

    let mut row: u32 = 0;

    wf(ws, row, 0, "OMTSF Excel Export", &bold)?;
    row += 1;

    w(
        ws,
        row,
        0,
        "This workbook was exported from an OMTSF .omts supply-chain graph file.",
    )?;
    row += 2;

    wf(ws, row, 0, "Sheet Overview", &bold)?;
    row += 1;

    let sheet_descriptions: &[(&str, &str)] = &[
        (
            "Metadata",
            "File header fields: snapshot_date, reporting_entity, disclosure_scope",
        ),
        (
            "Organizations",
            "Organization nodes with inline identifier columns (lei, duns, nat_reg, vat, internal)",
        ),
        ("Facilities", "Facility nodes"),
        ("Goods", "Good / commodity nodes"),
        ("Persons", "Person nodes (omitted in public-scope files)"),
        (
            "Attestations",
            "Attestation nodes merged with attested_by edge data",
        ),
        ("Consignments", "Consignment nodes"),
        (
            "Supply Relationships",
            "Supply-family edges (supplies, subcontracts, tolls, distributes, brokers, operates, produces, sells_to)",
        ),
        (
            "Corporate Structure",
            "Corporate-family edges (ownership, legal_parentage, operational_control, beneficial_ownership, former_identity)",
        ),
        (
            "Same As",
            "same_as edges indicating two nodes represent the same real-world entity",
        ),
        (
            "Identifiers",
            "Additional identifier records not mapped to inline columns on node sheets",
        ),
        ("README", "This sheet"),
    ];

    wf(ws, row, 0, "Sheet", &header_fmt)?;
    wf(ws, row, 1, "Description", &header_fmt)?;
    row += 1;

    for (sheet, desc) in sheet_descriptions {
        w(ws, row, 0, sheet)?;
        w(ws, row, 1, desc)?;
        row += 1;
    }

    row += 1;
    wf(ws, row, 0, "Key Field Definitions", &bold)?;
    row += 1;

    wf(ws, row, 0, "Field", &header_fmt)?;
    wf(ws, row, 1, "Description", &header_fmt)?;
    row += 1;

    let field_defs: &[(&str, &str)] = &[
        ("id", "Unique node or edge identifier within this file"),
        (
            "type",
            "Node or edge type (e.g. organization, facility, supplies)",
        ),
        ("name", "Human-readable display name"),
        (
            "jurisdiction",
            "ISO 3166-1 alpha-2 country code of registration",
        ),
        (
            "status",
            "Lifecycle status (active, dissolved, merged, suspended)",
        ),
        ("lei", "Legal Entity Identifier (20 chars, ISO 17442)"),
        ("duns", "D-U-N-S number"),
        ("nat_reg_value", "National registration number value"),
        ("nat_reg_authority", "Issuing authority for nat_reg"),
        ("vat_value", "VAT number value"),
        ("vat_country", "Country code for VAT authority"),
        ("internal_id", "Internal system identifier"),
        ("internal_system", "Name of the internal system"),
        (
            "supplier_id",
            "Source node ID in a supply relationship (the supplier)",
        ),
        (
            "buyer_id",
            "Target node ID in a supply relationship (the buyer)",
        ),
        (
            "subsidiary_id",
            "Source node ID in a corporate relationship (the subsidiary)",
        ),
        (
            "parent_id",
            "Target node ID in a corporate relationship (the parent)",
        ),
        ("entity_a", "First entity in a same_as relationship"),
        ("entity_b", "Second entity in a same_as relationship"),
        (
            "confidence",
            "Data quality confidence: verified, reported, inferred, estimated",
        ),
        ("valid_from", "Start date of validity in YYYY-MM-DD format"),
        (
            "valid_to",
            "End date of validity in YYYY-MM-DD format (leave blank for open-ended)",
        ),
        ("attested_entity_id", "Node ID of the entity being attested"),
        ("scope", "Scope of the attestation relationship"),
    ];

    for (field, desc) in field_defs {
        w(ws, row, 0, field)?;
        w(ws, row, 1, desc)?;
        row += 1;
    }

    ws.set_column_width(0, 25.0)
        .map(|_| ())
        .map_err(|e: XlsxError| ExportError::ExcelWrite {
            detail: e.to_string(),
        })?;
    ws.set_column_width(1, 70.0)
        .map(|_| ())
        .map_err(|e: XlsxError| ExportError::ExcelWrite {
            detail: e.to_string(),
        })?;

    Ok(())
}
