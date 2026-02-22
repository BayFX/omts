/// Implementation of `omtsf export`.
///
/// Reads an `.omts` file and writes an Excel `.xlsx` workbook (or other
/// supported format in the future) to a file.
///
/// Exit codes:
/// - 0 = success
/// - 1 = export logic failure
/// - 2 = file not found, I/O error, or unknown format
use std::fs::File;
use std::path::Path;

use omtsf_core::file::OmtsFile;

use crate::ExportFormat;
use crate::error::CliError;

/// Runs the `export` command.
///
/// Writes `omts_file` in the specified `format` to `output`.
/// When `output` is `None`, returns an error because binary formats like Excel
/// cannot be streamed to a terminal.
///
/// # Errors
///
/// Returns [`CliError`] on I/O failures or export errors.
pub fn run(
    omts_file: &OmtsFile,
    format: &ExportFormat,
    output: Option<&Path>,
) -> Result<(), CliError> {
    match format {
        ExportFormat::Excel => run_excel(omts_file, output),
    }
}

fn run_excel(omts_file: &OmtsFile, output: Option<&Path>) -> Result<(), CliError> {
    let out_path = output.ok_or_else(|| CliError::InvalidArgument {
        detail: "export --output-format excel requires -o <output.xlsx>".to_owned(),
    })?;

    let file = File::create(out_path).map_err(|e| {
        use std::io::ErrorKind;
        match e.kind() {
            ErrorKind::PermissionDenied => CliError::PermissionDenied {
                path: out_path.to_path_buf(),
            },
            ErrorKind::NotFound
            | ErrorKind::ConnectionRefused
            | ErrorKind::ConnectionReset
            | ErrorKind::HostUnreachable
            | ErrorKind::NetworkUnreachable
            | ErrorKind::ConnectionAborted
            | ErrorKind::NotConnected
            | ErrorKind::AddrInUse
            | ErrorKind::AddrNotAvailable
            | ErrorKind::NetworkDown
            | ErrorKind::BrokenPipe
            | ErrorKind::AlreadyExists
            | ErrorKind::WouldBlock
            | ErrorKind::NotADirectory
            | ErrorKind::IsADirectory
            | ErrorKind::DirectoryNotEmpty
            | ErrorKind::ReadOnlyFilesystem
            | ErrorKind::StaleNetworkFileHandle
            | ErrorKind::InvalidInput
            | ErrorKind::InvalidData
            | ErrorKind::TimedOut
            | ErrorKind::WriteZero
            | ErrorKind::StorageFull
            | ErrorKind::NotSeekable
            | ErrorKind::QuotaExceeded
            | ErrorKind::FileTooLarge
            | ErrorKind::ResourceBusy
            | ErrorKind::ExecutableFileBusy
            | ErrorKind::Deadlock
            | ErrorKind::CrossesDevices
            | ErrorKind::TooManyLinks
            | ErrorKind::ArgumentListTooLong
            | ErrorKind::Interrupted
            | ErrorKind::Unsupported
            | ErrorKind::UnexpectedEof
            | ErrorKind::OutOfMemory
            | ErrorKind::Other
            | _ => CliError::IoError {
                source: out_path.display().to_string(),
                detail: e.to_string(),
            },
        }
    })?;

    omtsf_excel::export_excel(omts_file, file).map_err(|e| CliError::IoError {
        source: out_path.display().to_string(),
        detail: e.to_string(),
    })
}
