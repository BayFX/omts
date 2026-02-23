/// Excel export orchestration.
///
/// Builds a multi-sheet `.xlsx` workbook from an [`OmtsFile`] and writes the
/// bytes to any `Write` sink. The sheet layout mirrors the reference template
/// so that exported workbooks can be re-imported without changes.
///
/// # Sheet order
///
/// 1. README
/// 2. Metadata
/// 3. Organizations
/// 4. Facilities
/// 5. Goods
/// 6. Persons
/// 7. Attestations
/// 8. Consignments
/// 9. Supply Relationships
/// 10. Corporate Structure
/// 11. Same As
/// 12. Identifiers
use std::io::Write;

use rust_xlsxwriter::{Workbook, XlsxError};

use omts_core::file::OmtsFile;

use crate::error::ExportError;

mod edges;
mod identifiers;
mod metadata;
mod nodes;
mod readme;
mod style;
pub mod supplier_list;

/// Exports an [`OmtsFile`] to Excel `.xlsx` format.
///
/// The workbook bytes are written to `writer`. The writer is flushed after
/// all bytes are written.
///
/// Boundary ref nodes are omitted; they are import-time artifacts that have
/// no meaning in the exported workbook.
///
/// # Errors
///
/// Returns [`ExportError`] if the workbook cannot be built or written.
pub fn export_excel<W: Write>(file: &OmtsFile, mut writer: W) -> Result<(), ExportError> {
    let mut wb = Workbook::new();

    // Add all sheets in template order. Each sheet name is used later to look
    // up the worksheet reference for writing.
    for name in [
        "README",
        "Metadata",
        "Organizations",
        "Facilities",
        "Goods",
        "Persons",
        "Attestations",
        "Consignments",
        "Supply Relationships",
        "Corporate Structure",
        "Same As",
        "Identifiers",
    ] {
        wb.add_worksheet()
            .set_name(name)
            .map_err(|e: XlsxError| ExportError::ExcelWrite {
                detail: e.to_string(),
            })?;
    }

    // Write each sheet in a separate scope so the mutable borrow of the
    // worksheet is released before the next worksheet_from_name call.
    {
        let ws = get_ws(&mut wb, "README")?;
        readme::write_readme(ws)?;
    }
    {
        let ws = get_ws(&mut wb, "Metadata")?;
        metadata::write_metadata(ws, file)?;
    }
    {
        let ws = get_ws(&mut wb, "Organizations")?;
        nodes::write_organizations(ws, &file.nodes)?;
    }
    {
        let ws = get_ws(&mut wb, "Facilities")?;
        nodes::write_facilities(ws, &file.nodes)?;
    }
    {
        let ws = get_ws(&mut wb, "Goods")?;
        nodes::write_goods(ws, &file.nodes)?;
    }
    {
        let ws = get_ws(&mut wb, "Persons")?;
        nodes::write_persons(ws, &file.nodes)?;
    }
    {
        let ws = get_ws(&mut wb, "Attestations")?;
        nodes::write_attestations(ws, &file.nodes)?;
    }
    {
        let ws = get_ws(&mut wb, "Consignments")?;
        nodes::write_consignments(ws, &file.nodes)?;
    }
    {
        let ws = get_ws(&mut wb, "Supply Relationships")?;
        edges::write_supply_relationships(ws, &file.edges)?;
    }
    {
        let ws = get_ws(&mut wb, "Corporate Structure")?;
        edges::write_corporate_structure(ws, &file.edges)?;
    }
    {
        let ws = get_ws(&mut wb, "Same As")?;
        edges::write_same_as(ws, &file.edges)?;
    }
    {
        // Write attested_by data back into the Attestations sheet.
        let att_row_map = edges::build_attestation_row_map(&file.nodes);
        let ws = get_ws(&mut wb, "Attestations")?;
        edges::write_attested_by_back(ws, &file.edges, &att_row_map)?;
    }
    {
        let ws = get_ws(&mut wb, "Identifiers")?;
        identifiers::write_identifiers(ws, &file.nodes)?;
    }

    let xlsx_bytes = wb
        .save_to_buffer()
        .map_err(|e: XlsxError| ExportError::ExcelWrite {
            detail: e.to_string(),
        })?;

    writer.write_all(&xlsx_bytes).map_err(|e| ExportError::Io {
        detail: e.to_string(),
    })?;
    writer.flush().map_err(|e| ExportError::Io {
        detail: e.to_string(),
    })?;

    Ok(())
}

fn get_ws<'a>(
    wb: &'a mut Workbook,
    name: &str,
) -> Result<&'a mut rust_xlsxwriter::Worksheet, ExportError> {
    wb.worksheet_from_name(name)
        .map_err(|e: XlsxError| ExportError::ExcelWrite {
            detail: format!("worksheet {name:?}: {e}"),
        })
}
