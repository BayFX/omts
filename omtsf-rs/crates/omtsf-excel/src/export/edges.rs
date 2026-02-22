/// Writes edge sheets: Supply Relationships, Corporate Structure, Same As.
///
/// Also writes back `attested_by` edges into the Attestations sheet rows as
/// `attested_entity_id` and `scope` columns.
///
/// Supply Relationships uses domain columns: `supplier_id` (source) / `buyer_id`
/// (target). Corporate Structure uses: `subsidiary_id` (source) / `parent_id`
/// (target). These conventions mirror the import-side column names.
use std::collections::HashMap;

use rust_xlsxwriter::{Worksheet, XlsxError};

use omtsf_core::enums::{EdgeType, EdgeTypeTag};
use omtsf_core::structures::{Edge, Node};

use crate::error::ExportError;
use crate::export::style::{set_column_widths, write_header_row};

fn ws(worksheet: &mut Worksheet, row: u32, col: u16, val: &str) -> Result<(), ExportError> {
    worksheet
        .write(row, col, val)
        .map(|_| ())
        .map_err(|e: XlsxError| ExportError::ExcelWrite {
            detail: e.to_string(),
        })
}

fn wf64(worksheet: &mut Worksheet, row: u32, col: u16, val: f64) -> Result<(), ExportError> {
    worksheet
        .write(row, col, val)
        .map(|_| ())
        .map_err(|e: XlsxError| ExportError::ExcelWrite {
            detail: e.to_string(),
        })
}

fn wu32(worksheet: &mut Worksheet, row: u32, col: u16, val: u32) -> Result<(), ExportError> {
    worksheet
        .write(row, col, val)
        .map(|_| ())
        .map_err(|e: XlsxError| ExportError::ExcelWrite {
            detail: e.to_string(),
        })
}

/// Writes the Supply Relationships sheet.
///
/// Returns the number of data rows written.
pub fn write_supply_relationships(
    worksheet: &mut Worksheet,
    edges: &[Edge],
) -> Result<u32, ExportError> {
    write_header_row(
        worksheet,
        &[
            "id",
            "type",
            "supplier_id",
            "buyer_id",
            "valid_from",
            "valid_to",
            "commodity",
            "tier",
            "volume",
            "volume_unit",
            "annual_value",
            "value_currency",
            "contract_ref",
            "share_of_buyer_demand",
        ],
    )?;
    set_column_widths(
        worksheet,
        &[
            (0, 24.0),
            (1, 16.0),
            (2, 24.0),
            (3, 24.0),
            (4, 14.0),
            (5, 14.0),
            (6, 20.0),
            (7, 8.0),
            (8, 12.0),
            (9, 14.0),
            (10, 14.0),
            (11, 16.0),
            (12, 20.0),
            (13, 20.0),
        ],
    )?;

    let mut row: u32 = 1;
    for edge in edges {
        if is_supply_edge(&edge.edge_type) {
            write_supply_row(worksheet, row, edge)?;
            row += 1;
        }
    }
    Ok(row - 1)
}

fn is_supply_edge(t: &EdgeTypeTag) -> bool {
    matches!(
        t,
        EdgeTypeTag::Known(EdgeType::Supplies)
            | EdgeTypeTag::Known(EdgeType::Subcontracts)
            | EdgeTypeTag::Known(EdgeType::Tolls)
            | EdgeTypeTag::Known(EdgeType::Distributes)
            | EdgeTypeTag::Known(EdgeType::Brokers)
            | EdgeTypeTag::Known(EdgeType::Operates)
            | EdgeTypeTag::Known(EdgeType::Produces)
            | EdgeTypeTag::Known(EdgeType::ComposedOf)
            | EdgeTypeTag::Known(EdgeType::SellsTo)
    )
}

fn write_supply_row(worksheet: &mut Worksheet, row: u32, edge: &Edge) -> Result<(), ExportError> {
    ws(worksheet, row, 0, &edge.id.to_string())?;
    ws(worksheet, row, 1, edge.edge_type.as_str())?;
    ws(worksheet, row, 2, &edge.source.to_string())?;
    ws(worksheet, row, 3, &edge.target.to_string())?;

    let p = &edge.properties;
    ws(
        worksheet,
        row,
        4,
        p.valid_from
            .as_ref()
            .map(std::string::ToString::to_string)
            .as_deref()
            .unwrap_or(""),
    )?;

    let valid_to_str: String = match &p.valid_to {
        Some(Some(d)) => d.to_string(),
        _ => String::new(),
    };
    ws(worksheet, row, 5, &valid_to_str)?;
    ws(worksheet, row, 6, p.commodity.as_deref().unwrap_or(""))?;

    if let Some(tier) = p.tier {
        wu32(worksheet, row, 7, tier)?;
    }
    if let Some(vol) = p.volume {
        wf64(worksheet, row, 8, vol)?;
    }

    ws(worksheet, row, 9, p.volume_unit.as_deref().unwrap_or(""))?;

    if let Some(av) = p.annual_value {
        wf64(worksheet, row, 10, av)?;
    }

    ws(
        worksheet,
        row,
        11,
        p.value_currency.as_deref().unwrap_or(""),
    )?;
    ws(worksheet, row, 12, p.contract_ref.as_deref().unwrap_or(""))?;

    if let Some(sbd) = p.share_of_buyer_demand {
        wf64(worksheet, row, 13, sbd)?;
    }

    Ok(())
}

/// Writes the Corporate Structure sheet.
///
/// Returns the number of data rows written.
pub fn write_corporate_structure(
    worksheet: &mut Worksheet,
    edges: &[Edge],
) -> Result<u32, ExportError> {
    write_header_row(
        worksheet,
        &[
            "id",
            "type",
            "subsidiary_id",
            "parent_id",
            "valid_from",
            "valid_to",
            "percentage",
            "direct",
            "consolidation_basis",
        ],
    )?;
    set_column_widths(
        worksheet,
        &[
            (0, 24.0),
            (1, 22.0),
            (2, 24.0),
            (3, 24.0),
            (4, 14.0),
            (5, 14.0),
            (6, 12.0),
            (7, 10.0),
            (8, 22.0),
        ],
    )?;

    let mut row: u32 = 1;
    for edge in edges {
        if is_corp_edge(&edge.edge_type) {
            write_corp_row(worksheet, row, edge)?;
            row += 1;
        }
    }
    Ok(row - 1)
}

fn is_corp_edge(t: &EdgeTypeTag) -> bool {
    matches!(
        t,
        EdgeTypeTag::Known(EdgeType::Ownership)
            | EdgeTypeTag::Known(EdgeType::LegalParentage)
            | EdgeTypeTag::Known(EdgeType::OperationalControl)
            | EdgeTypeTag::Known(EdgeType::BeneficialOwnership)
            | EdgeTypeTag::Known(EdgeType::FormerIdentity)
    )
}

fn write_corp_row(worksheet: &mut Worksheet, row: u32, edge: &Edge) -> Result<(), ExportError> {
    ws(worksheet, row, 0, &edge.id.to_string())?;
    ws(worksheet, row, 1, edge.edge_type.as_str())?;
    ws(worksheet, row, 2, &edge.source.to_string())?;
    ws(worksheet, row, 3, &edge.target.to_string())?;

    let p = &edge.properties;
    ws(
        worksheet,
        row,
        4,
        p.valid_from
            .as_ref()
            .map(std::string::ToString::to_string)
            .as_deref()
            .unwrap_or(""),
    )?;

    let valid_to_str: String = match &p.valid_to {
        Some(Some(d)) => d.to_string(),
        _ => String::new(),
    };
    ws(worksheet, row, 5, &valid_to_str)?;

    if let Some(pct) = p.percentage {
        wf64(worksheet, row, 6, pct)?;
    }

    if let Some(direct) = p.direct {
        ws(worksheet, row, 7, if direct { "true" } else { "false" })?;
    }

    ws(
        worksheet,
        row,
        8,
        p.consolidation_basis
            .as_ref()
            .map(consolidation_basis_str)
            .unwrap_or(""),
    )?;

    Ok(())
}

/// Writes the Same As sheet.
///
/// Returns the number of data rows written.
pub fn write_same_as(worksheet: &mut Worksheet, edges: &[Edge]) -> Result<u32, ExportError> {
    write_header_row(
        worksheet,
        &["id", "entity_a", "entity_b", "confidence", "basis"],
    )?;
    set_column_widths(
        worksheet,
        &[(0, 24.0), (1, 24.0), (2, 24.0), (3, 14.0), (4, 30.0)],
    )?;

    let mut row: u32 = 1;
    for edge in edges {
        if matches!(&edge.edge_type, EdgeTypeTag::Known(EdgeType::SameAs)) {
            write_same_as_row(worksheet, row, edge)?;
            row += 1;
        }
    }
    Ok(row - 1)
}

fn write_same_as_row(worksheet: &mut Worksheet, row: u32, edge: &Edge) -> Result<(), ExportError> {
    ws(worksheet, row, 0, &edge.id.to_string())?;
    ws(worksheet, row, 1, &edge.source.to_string())?;
    ws(worksheet, row, 2, &edge.target.to_string())?;

    let confidence = edge
        .extra
        .get("confidence")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    ws(worksheet, row, 3, confidence)?;

    let basis = edge
        .extra
        .get("basis")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    ws(worksheet, row, 4, basis)?;

    Ok(())
}

/// Builds a map from attestation node ID → data row index (1-based) in the
/// Attestations sheet.
pub fn build_attestation_row_map(nodes: &[Node]) -> HashMap<String, u32> {
    use omtsf_core::enums::NodeTypeTag;
    let mut map = HashMap::new();
    let mut row: u32 = 1;
    for node in nodes {
        if matches!(
            &node.node_type,
            NodeTypeTag::Known(omtsf_core::enums::NodeType::Attestation)
        ) {
            map.insert(node.id.to_string(), row);
            row += 1;
        }
    }
    map
}

/// Writes `attested_by` relationship data back into the Attestations sheet.
///
/// For each `attested_by` edge, looks up the target (attestation node) in
/// `att_row_map` and writes the source (attested entity) and scope into
/// columns 10 and 11 of the matching row.
pub fn write_attested_by_back(
    worksheet: &mut Worksheet,
    edges: &[Edge],
    att_row_map: &HashMap<String, u32>,
) -> Result<(), ExportError> {
    for edge in edges {
        if !matches!(&edge.edge_type, EdgeTypeTag::Known(EdgeType::AttestedBy)) {
            continue;
        }
        let att_node_id = edge.target.to_string();
        if let Some(&att_data_row) = att_row_map.get(&att_node_id) {
            ws(worksheet, att_data_row, 10, &edge.source.to_string())?;
            ws(
                worksheet,
                att_data_row,
                11,
                edge.properties.scope.as_deref().unwrap_or(""),
            )?;
        }
    }
    Ok(())
}

fn consolidation_basis_str(s: &omtsf_core::enums::ConsolidationBasis) -> &'static str {
    match s {
        omtsf_core::enums::ConsolidationBasis::Ifrs10 => "ifrs10",
        omtsf_core::enums::ConsolidationBasis::UsGaapAsc810 => "us_gaap_asc810",
        omtsf_core::enums::ConsolidationBasis::Other => "other",
        omtsf_core::enums::ConsolidationBasis::Unknown => "unknown",
    }
}
