//! Integration tests for `omtsf export`.
#![allow(clippy::expect_used)]

use std::path::PathBuf;
use std::process::Command;

/// Path to the compiled `omtsf` binary.
fn omtsf_bin() -> PathBuf {
    let mut path = std::env::current_exe().expect("current exe");
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.push("omtsf");
    path
}

/// Path to a valid `.omts` fixture file.
fn omts_fixture(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../../tests/fixtures/valid");
    path.push(name);
    path
}

/// Path to an Excel fixture file.
fn excel_fixture(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../../tests/fixtures/excel");
    path.push(name);
    path
}

#[test]
fn export_minimal_exits_0() {
    let tmp = tempfile::NamedTempFile::new().expect("temp file");
    let out_path = tmp.path().with_extension("xlsx");

    let out = Command::new(omtsf_bin())
        .args([
            "export",
            omts_fixture("minimal.omts").to_str().expect("path"),
            "-o",
            out_path.to_str().expect("output path"),
        ])
        .output()
        .expect("run omtsf export");

    assert_eq!(
        out.status.code(),
        Some(0),
        "expected exit 0; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn export_writes_xlsx_file() {
    let tmp = tempfile::NamedTempFile::new().expect("temp file");
    let out_path = tmp.path().with_extension("xlsx");

    let out = Command::new(omtsf_bin())
        .args([
            "export",
            omts_fixture("minimal.omts").to_str().expect("path"),
            "-o",
            out_path.to_str().expect("output path"),
        ])
        .output()
        .expect("run omtsf export");

    assert_eq!(out.status.code(), Some(0));
    assert!(
        out_path.exists(),
        "expected output file to be created: {}",
        out_path.display()
    );

    let size = std::fs::metadata(&out_path).expect("metadata").len();
    assert!(size > 0, "output .xlsx file must not be empty");
}

#[test]
fn export_stdout_is_empty_when_output_flag_used() {
    let tmp = tempfile::NamedTempFile::new().expect("temp file");
    let out_path = tmp.path().with_extension("xlsx");

    let out = Command::new(omtsf_bin())
        .args([
            "export",
            omts_fixture("minimal.omts").to_str().expect("path"),
            "-o",
            out_path.to_str().expect("output path"),
        ])
        .output()
        .expect("run omtsf export");

    assert_eq!(out.status.code(), Some(0));
    assert!(
        out.stdout.is_empty(),
        "stdout must be empty when -o is used; got: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn export_without_output_flag_exits_2() {
    // Excel output is binary and cannot be streamed to stdout.
    let out = Command::new(omtsf_bin())
        .args([
            "export",
            omts_fixture("minimal.omts").to_str().expect("path"),
        ])
        .output()
        .expect("run omtsf export (no -o)");

    assert_eq!(
        out.status.code(),
        Some(2),
        "expected exit 2 without -o; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn export_nonexistent_input_exits_2() {
    let tmp = tempfile::NamedTempFile::new().expect("temp file");
    let out_path = tmp.path().with_extension("xlsx");

    let out = Command::new(omtsf_bin())
        .args([
            "export",
            "/tmp/does-not-exist-xyz.omts",
            "-o",
            out_path.to_str().expect("output path"),
        ])
        .output()
        .expect("run omtsf export (missing input)");

    assert_eq!(
        out.status.code(),
        Some(2),
        "expected exit 2 for missing input; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn export_nonexistent_input_prints_error_on_stderr() {
    let tmp = tempfile::NamedTempFile::new().expect("temp file");
    let out_path = tmp.path().with_extension("xlsx");

    let out = Command::new(omtsf_bin())
        .args([
            "export",
            "/tmp/does-not-exist-xyz.omts",
            "-o",
            out_path.to_str().expect("output path"),
        ])
        .output()
        .expect("run omtsf export (missing input)");

    assert!(!out.stderr.is_empty(), "expected error message on stderr");
}

#[test]
fn export_explicit_format_excel_exits_0() {
    let tmp = tempfile::NamedTempFile::new().expect("temp file");
    let out_path = tmp.path().with_extension("xlsx");

    let out = Command::new(omtsf_bin())
        .args([
            "export",
            "--output-format",
            "excel",
            omts_fixture("minimal.omts").to_str().expect("path"),
            "-o",
            out_path.to_str().expect("output path"),
        ])
        .output()
        .expect("run omtsf export --output-format excel");

    assert_eq!(
        out.status.code(),
        Some(0),
        "expected exit 0 with --output-format excel; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn export_full_featured_omts_exits_0() {
    let tmp = tempfile::NamedTempFile::new().expect("temp file");
    let out_path = tmp.path().with_extension("xlsx");

    let out = Command::new(omtsf_bin())
        .args([
            "export",
            omts_fixture("full-featured.omts").to_str().expect("path"),
            "-o",
            out_path.to_str().expect("output path"),
        ])
        .output()
        .expect("run omtsf export full-featured.omts");

    assert_eq!(
        out.status.code(),
        Some(0),
        "expected exit 0 for full-featured fixture; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn export_produces_valid_xlsx_magic_bytes() {
    // A valid .xlsx file starts with the PK ZIP magic bytes (50 4B 03 04).
    let tmp = tempfile::NamedTempFile::new().expect("temp file");
    let out_path = tmp.path().with_extension("xlsx");

    let out = Command::new(omtsf_bin())
        .args([
            "export",
            omts_fixture("minimal.omts").to_str().expect("path"),
            "-o",
            out_path.to_str().expect("output path"),
        ])
        .output()
        .expect("run omtsf export");

    assert_eq!(out.status.code(), Some(0));

    let bytes = std::fs::read(&out_path).expect("read output file");
    assert!(
        bytes.starts_with(&[0x50, 0x4B, 0x03, 0x04]),
        "output must start with PK ZIP magic bytes for a valid .xlsx file"
    );
}

#[test]
fn export_import_example_xlsx_round_trip() {
    // Import the example workbook, export it, verify the output is valid xlsx.
    let import_tmp = tempfile::NamedTempFile::new().expect("temp file");
    let omts_path = import_tmp.path().with_extension("omts");

    let import_out = Command::new(omtsf_bin())
        .args([
            "import",
            excel_fixture("omts-import-example.xlsx")
                .to_str()
                .expect("path"),
            "-o",
            omts_path.to_str().expect("omts output path"),
        ])
        .output()
        .expect("run omtsf import");

    assert_eq!(
        import_out.status.code(),
        Some(0),
        "import must succeed; stderr: {}",
        String::from_utf8_lossy(&import_out.stderr)
    );

    let export_tmp = tempfile::NamedTempFile::new().expect("temp file");
    let xlsx_path = export_tmp.path().with_extension("xlsx");

    let export_out = Command::new(omtsf_bin())
        .args([
            "export",
            omts_path.to_str().expect("omts path"),
            "-o",
            xlsx_path.to_str().expect("xlsx output path"),
        ])
        .output()
        .expect("run omtsf export");

    assert_eq!(
        export_out.status.code(),
        Some(0),
        "export must succeed; stderr: {}",
        String::from_utf8_lossy(&export_out.stderr)
    );

    let bytes = std::fs::read(&xlsx_path).expect("read xlsx");
    assert!(
        bytes.starts_with(&[0x50, 0x4B, 0x03, 0x04]),
        "round-trip output must be a valid xlsx file"
    );
}
