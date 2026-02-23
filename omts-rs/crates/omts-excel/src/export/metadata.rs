/// Writes the Metadata sheet with file header fields.
///
/// The Metadata sheet uses a key/value layout (column A = field name,
/// column B = value), mirroring the import-side format so that exported
/// workbooks can be re-imported without changes.
use rust_xlsxwriter::{Worksheet, XlsxError};

use omts_core::file::OmtsFile;

use crate::error::ExportError;
use crate::export::style::{set_column_widths, write_header_row};

/// Writes all known header fields to the Metadata sheet.
pub fn write_metadata(ws: &mut Worksheet, file: &OmtsFile) -> Result<(), ExportError> {
    write_header_row(ws, &["Field", "Value"])?;
    set_column_widths(ws, &[(0, 24.0), (1, 40.0)])?;

    let kv = |ws: &mut Worksheet, row: u32, key: &str, val: &str| -> Result<(), ExportError> {
        ws.write(row, 0, key)
            .map(|_| ())
            .map_err(|e: XlsxError| ExportError::ExcelWrite {
                detail: e.to_string(),
            })?;
        ws.write(row, 1, val)
            .map(|_| ())
            .map_err(|e: XlsxError| ExportError::ExcelWrite {
                detail: e.to_string(),
            })
    };

    let snapshot_date = file.snapshot_date.to_string();
    kv(ws, 1, "snapshot_date", &snapshot_date)?;

    let reporting_entity = file
        .reporting_entity
        .as_ref()
        .map(std::string::ToString::to_string)
        .unwrap_or_default();
    kv(ws, 2, "reporting_entity", &reporting_entity)?;

    let disclosure_scope = file
        .disclosure_scope
        .as_ref()
        .map(disclosure_scope_str)
        .unwrap_or_default();
    kv(ws, 3, "disclosure_scope", disclosure_scope)?;

    let omts_version = file.omts_version.to_string();
    kv(ws, 4, "omts_version", &omts_version)?;

    Ok(())
}

fn disclosure_scope_str(scope: &omts_core::enums::DisclosureScope) -> &'static str {
    match scope {
        omts_core::enums::DisclosureScope::Internal => "internal",
        omts_core::enums::DisclosureScope::Partner => "partner",
        omts_core::enums::DisclosureScope::Public => "public",
    }
}
