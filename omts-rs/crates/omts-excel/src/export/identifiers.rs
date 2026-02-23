/// Writes the Identifiers sheet for overflow identifier records.
///
/// Only identifiers that were NOT mapped to inline columns on the Organizations
/// sheet are written here. Identifiers for non-organization node types are also
/// always written here.
///
/// Columns: `node_id`, scheme, value, authority, sensitivity, `valid_from`, `valid_to`,
/// `verification_status`.
use rust_xlsxwriter::{Worksheet, XlsxError};

use omts_core::enums::{NodeType, NodeTypeTag};
use omts_core::structures::Node;
use omts_core::types::Identifier;

use crate::error::ExportError;
use crate::export::style::{set_column_widths, write_header_row};

/// Inline identifier schemes written on the Organizations sheet.
///
/// These are excluded from the Identifiers sheet to avoid duplication.
const ORG_INLINE_SCHEMES: &[&str] = &["lei", "duns", "nat-reg", "vat", "internal"];

fn ws(worksheet: &mut Worksheet, row: u32, col: u16, val: &str) -> Result<(), ExportError> {
    worksheet
        .write(row, col, val)
        .map(|_| ())
        .map_err(|e: XlsxError| ExportError::ExcelWrite {
            detail: e.to_string(),
        })
}

/// Writes the Identifiers sheet.
///
/// Returns the number of data rows written.
pub fn write_identifiers(worksheet: &mut Worksheet, nodes: &[Node]) -> Result<u32, ExportError> {
    let headers = [
        "node_id",
        "scheme",
        "value",
        "authority",
        "sensitivity",
        "valid_from",
        "valid_to",
        "verification_status",
    ];
    write_header_row(worksheet, &headers)?;
    set_column_widths(
        worksheet,
        &[
            (0, 24.0),
            (1, 16.0),
            (2, 24.0),
            (3, 20.0),
            (4, 14.0),
            (5, 14.0),
            (6, 14.0),
            (7, 18.0),
        ],
    )?;

    let mut row: u32 = 1;

    for node in nodes {
        if matches!(&node.node_type, NodeTypeTag::Known(NodeType::BoundaryRef)) {
            continue;
        }

        let identifiers = match &node.identifiers {
            Some(ids) if !ids.is_empty() => ids,
            _ => continue,
        };

        let is_org = matches!(&node.node_type, NodeTypeTag::Known(NodeType::Organization));

        for id in identifiers {
            if is_org && ORG_INLINE_SCHEMES.contains(&id.scheme.to_lowercase().as_str()) {
                continue;
            }
            write_identifier_row(worksheet, row, &node.id.to_string(), id)?;
            row += 1;
        }
    }

    Ok(row - 1)
}

fn write_identifier_row(
    worksheet: &mut Worksheet,
    row: u32,
    node_id: &str,
    id: &Identifier,
) -> Result<(), ExportError> {
    ws(worksheet, row, 0, node_id)?;
    ws(worksheet, row, 1, &id.scheme)?;
    ws(worksheet, row, 2, &id.value)?;
    ws(worksheet, row, 3, id.authority.as_deref().unwrap_or(""))?;
    ws(
        worksheet,
        row,
        4,
        id.sensitivity.as_ref().map(sensitivity_str).unwrap_or(""),
    )?;
    ws(
        worksheet,
        row,
        5,
        id.valid_from
            .as_ref()
            .map(std::string::ToString::to_string)
            .as_deref()
            .unwrap_or(""),
    )?;

    let valid_to_str: String = match &id.valid_to {
        Some(Some(d)) => d.to_string(),
        _ => String::new(),
    };
    ws(worksheet, row, 6, &valid_to_str)?;
    ws(
        worksheet,
        row,
        7,
        id.verification_status
            .as_ref()
            .map(verification_status_str)
            .unwrap_or(""),
    )?;

    Ok(())
}

fn sensitivity_str(s: &omts_core::enums::Sensitivity) -> &'static str {
    match s {
        omts_core::enums::Sensitivity::Public => "public",
        omts_core::enums::Sensitivity::Restricted => "restricted",
        omts_core::enums::Sensitivity::Confidential => "confidential",
    }
}

fn verification_status_str(s: &omts_core::enums::VerificationStatus) -> &'static str {
    match s {
        omts_core::enums::VerificationStatus::Verified => "verified",
        omts_core::enums::VerificationStatus::Reported => "reported",
        omts_core::enums::VerificationStatus::Inferred => "inferred",
        omts_core::enums::VerificationStatus::Unverified => "unverified",
    }
}
