/// Shared formatting helpers for Excel export.
///
/// Header formatting, column widths, and data validation dropdowns that match
/// the reference template styling.
use rust_xlsxwriter::{Format, FormatBorder, Worksheet, XlsxError};

use crate::error::ExportError;

/// Builds the bold header format used for all sheet header rows.
pub fn header_format() -> Format {
    Format::new()
        .set_bold()
        .set_border_bottom(FormatBorder::Medium)
        .set_background_color(0xD9E1F2)
}

/// Sets widths for a list of columns by index.
///
/// `widths` is a slice of `(col_index, width_in_characters)` pairs.
pub fn set_column_widths(ws: &mut Worksheet, widths: &[(u16, f64)]) -> Result<(), ExportError> {
    for &(col, width) in widths {
        ws.set_column_width(col, width)
            .map(|_| ())
            .map_err(|e: XlsxError| ExportError::ExcelWrite {
                detail: e.to_string(),
            })?;
    }
    Ok(())
}

/// Writes the header row with the bold header format.
///
/// `headers` is a slice of `&str` column names to write starting at column 0.
pub fn write_header_row(ws: &mut Worksheet, headers: &[&str]) -> Result<(), ExportError> {
    let fmt = header_format();
    for (col, &name) in headers.iter().enumerate() {
        ws.write_with_format(0, col as u16, name, &fmt)
            .map(|_| ())
            .map_err(|e: XlsxError| ExportError::ExcelWrite {
                detail: e.to_string(),
            })?;
    }
    Ok(())
}
