/// Excel import for the OMTS reference implementation.
///
/// This crate reads a multi-sheet `.xlsx` workbook and produces a valid
/// [`omts_core::OmtsFile`]. The `calamine` dependency is confined to this
/// crate and does not bleed into `omts-core` or `omts-cli`.
///
/// # Sheet layout
///
/// | Sheet | Purpose |
/// |---|---|
/// | README | Human-readable instructions (skipped) |
/// | Metadata | `snapshot_date`, `reporting_entity`, `disclosure_scope`, defaults |
/// | Organizations | Organization nodes with inline identifier columns |
/// | Facilities | Facility nodes |
/// | Goods | Good nodes |
/// | Persons | Person nodes |
/// | Attestations | Attestation nodes + `attested_by` edge generation |
/// | Consignments | Consignment nodes |
/// | Supply Relationships | `supplies`-family edges |
/// | Corporate Structure | `ownership`-family edges |
/// | Same As | `same_as` edges |
/// | Identifiers | Additional identifier records (merged with inline columns) |
///
/// # Two-pass parse
///
/// 1. Collect all nodes from node sheets into an ID map.
/// 2. Resolve edge source/target references against that map.
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::{Read, Seek};

use calamine::{Reader, Xlsx, open_workbook_from_rs};

use omts_core::enums::DisclosureScope;
use omts_core::file::OmtsFile;
use omts_core::generate_file_salt;
use omts_core::newtypes::{NodeId, SemVer};
use omts_core::validation::{ValidationConfig, validate};

mod edges;
pub mod error;
pub mod export;
mod identifiers;
mod metadata;
mod nodes;
mod sheet;
mod slug;
mod supplier_list;

pub use error::{ExportError, ImportError};
pub use export::export_excel;
pub use export::supplier_list::export_supplier_list;

/// The OMTS version string embedded in all imported files.
const OMTS_VERSION: &str = "1.0.0";

/// Detected variant of an Excel workbook template.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExcelVariant {
    /// Full multi-sheet template (Organizations, Facilities, etc.)
    Full,
    /// Simplified single-sheet "Supplier List" template.
    SupplierList,
}

/// Detects which template variant a workbook uses by inspecting sheet names.
///
/// - If "Supplier List" sheet exists → `SupplierList`
/// - If "Organizations" sheet exists → `Full`
/// - Otherwise → error
///
/// # Errors
///
/// Returns [`ImportError::MissingSheet`] if neither expected sheet is found.
pub fn detect_excel_variant<R: Read + Seek>(
    workbook: &Xlsx<R>,
) -> Result<ExcelVariant, ImportError> {
    let names = workbook.sheet_names();
    if names.iter().any(|s| s == "Supplier List") {
        Ok(ExcelVariant::SupplierList)
    } else if names.iter().any(|s| s == "Organizations") {
        Ok(ExcelVariant::Full)
    } else {
        Err(ImportError::MissingSheet {
            sheet: "Organizations or Supplier List".to_owned(),
        })
    }
}

/// Imports an Excel workbook and returns a valid [`OmtsFile`].
///
/// The reader must be positioned at the start of a valid `.xlsx` file.
/// A fresh CSPRNG `file_salt` is always generated.
///
/// The template variant is auto-detected by inspecting sheet names:
/// - "Supplier List" sheet → simplified single-sheet import
/// - "Organizations" sheet → full multi-sheet import
///
/// L1 validation runs on the constructed graph before returning. If any L1
/// errors are found, [`ImportError::ValidationFailed`] is returned and no
/// output is produced. L2 warnings are collected but do not block output.
///
/// # Errors
///
/// Returns [`ImportError`] for:
/// - Missing required sheets or columns
/// - Invalid cell values
/// - Unresolved edge references
/// - Person nodes present with `disclosure_scope: "public"`
/// - L1 validation failures
/// - Excel file I/O errors
pub fn import_excel<R: Read + Seek>(
    reader: R,
    authority: Option<&str>,
) -> Result<OmtsFile, ImportError> {
    let workbook: Xlsx<R> =
        open_workbook_from_rs(reader).map_err(|e: calamine::XlsxError| ImportError::ExcelRead {
            detail: e.to_string(),
        })?;

    let variant = detect_excel_variant(&workbook)?;
    match variant {
        ExcelVariant::Full => import_full_excel(workbook),
        ExcelVariant::SupplierList => supplier_list::import_supplier_list(workbook, authority),
    }
}

/// Imports a full multi-sheet workbook.
fn import_full_excel<R: Read + Seek>(mut workbook: Xlsx<R>) -> Result<OmtsFile, ImportError> {
    let sheet_names: Vec<String> = workbook.sheet_names().clone();

    let metadata_sheet = get_sheet(&mut workbook, &sheet_names, "Metadata")?;
    let meta = metadata::parse_metadata(&metadata_sheet)?;

    let orgs_sheet = get_sheet(&mut workbook, &sheet_names, "Organizations")?;
    let facilities_sheet = get_sheet(&mut workbook, &sheet_names, "Facilities")?;
    let goods_sheet = get_sheet(&mut workbook, &sheet_names, "Goods")?;
    let persons_sheet = get_sheet(&mut workbook, &sheet_names, "Persons")?;
    let attestations_sheet = get_sheet(&mut workbook, &sheet_names, "Attestations")?;
    let consignments_sheet = get_sheet(&mut workbook, &sheet_names, "Consignments")?;
    let supply_rel_sheet = get_sheet(&mut workbook, &sheet_names, "Supply Relationships")?;
    let corp_struct_sheet = get_sheet(&mut workbook, &sheet_names, "Corporate Structure")?;
    let same_as_sheet = get_sheet(&mut workbook, &sheet_names, "Same As")?;
    let identifiers_sheet = get_sheet(&mut workbook, &sheet_names, "Identifiers")?;

    let mut inline_identifiers: HashMap<String, Vec<omts_core::types::Identifier>> = HashMap::new();

    let mut graph_nodes = nodes::parse_all_nodes(
        &orgs_sheet,
        &facilities_sheet,
        &goods_sheet,
        &persons_sheet,
        &attestations_sheet,
        &consignments_sheet,
        &meta,
        &mut inline_identifiers,
    )?;

    let sheet_identifiers = identifiers::parse_identifiers_sheet(&identifiers_sheet)?;
    identifiers::merge_identifiers_onto_nodes(
        &mut graph_nodes,
        &inline_identifiers,
        &sheet_identifiers,
    );

    // Build the set of all node IDs for reference validation.
    let node_id_set: HashSet<String> = graph_nodes.iter().map(|n| n.id.to_string()).collect();

    // Check for person nodes with public disclosure scope before producing output.
    if meta.disclosure_scope == Some(DisclosureScope::Public) {
        let has_persons = graph_nodes.iter().any(|n| {
            matches!(
                &n.node_type,
                omts_core::enums::NodeTypeTag::Known(omts_core::enums::NodeType::Person)
            )
        });
        if has_persons {
            return Err(ImportError::PersonNodesWithPublicScope);
        }
    }

    let graph_edges = edges::parse_all_edges(
        &supply_rel_sheet,
        &corp_struct_sheet,
        &same_as_sheet,
        &attestations_sheet,
        &node_id_set,
    )?;

    let file_salt = generate_file_salt().map_err(|e| ImportError::ExcelRead {
        detail: format!("CSPRNG failure: {e}"),
    })?;

    let omts_version = SemVer::try_from(OMTS_VERSION).map_err(|e| ImportError::ExcelRead {
        detail: format!("internal: invalid version string: {e}"),
    })?;

    let reporting_entity = meta
        .reporting_entity
        .as_deref()
        .and_then(|id| NodeId::try_from(id).ok());

    let omts_file = OmtsFile {
        omts_version,
        snapshot_date: meta.snapshot_date,
        file_salt,
        disclosure_scope: meta.disclosure_scope,
        previous_snapshot_ref: None,
        snapshot_sequence: None,
        reporting_entity,
        nodes: graph_nodes,
        edges: graph_edges,
        extra: BTreeMap::new(),
    };

    // Run L1 validation; collect errors and block output on failures.
    let config = ValidationConfig {
        run_l1: true,
        run_l2: false,
        run_l3: false,
    };
    let result = validate(&omts_file, &config, None);
    let errors: Vec<String> = result
        .diagnostics
        .iter()
        .filter(|d| matches!(d.severity, omts_core::validation::Severity::Error))
        .map(|d| d.message.clone())
        .collect();
    if !errors.is_empty() {
        return Err(ImportError::ValidationFailed {
            diagnostics: errors,
        });
    }

    Ok(omts_file)
}

/// Retrieves a sheet by name from the workbook.
///
/// Returns [`ImportError::MissingSheet`] if the sheet is not present.
fn get_sheet<R: Read + Seek>(
    workbook: &mut Xlsx<R>,
    sheet_names: &[String],
    name: &str,
) -> Result<calamine::Range<calamine::Data>, ImportError> {
    if !sheet_names.iter().any(|s| s == name) {
        return Err(ImportError::MissingSheet {
            sheet: name.to_owned(),
        });
    }
    workbook
        .worksheet_range(name)
        .map_err(|e| ImportError::ExcelRead {
            detail: format!("failed to read sheet {name:?}: {e}"),
        })
}
