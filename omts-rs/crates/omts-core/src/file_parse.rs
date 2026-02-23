//! Unified parse pipeline: auto-detect encoding, decompress if needed, parse.
//!
//! Implements SPEC-007 Sections 2, 3, 4, and 6.  Entry point is [`parse_omts`].

use crate::cbor::{CborError, decode_cbor};
#[cfg(feature = "compression")]
use crate::compression::{CompressionError, decompress_zstd};
use crate::encoding::{Encoding, EncodingDetectionError, detect_encoding};
use crate::file::OmtsFile;

/// Error produced by the unified [`parse_omts`] pipeline.
#[derive(Debug)]
pub enum OmtsDecodeError {
    /// The file's encoding could not be detected from its initial bytes.
    EncodingDetection(EncodingDetectionError),
    /// CBOR decoding failed.
    Cbor(CborError),
    /// JSON parsing failed.
    Json(serde_json::Error),
    /// zstd decompression failed.
    #[cfg(feature = "compression")]
    Compression(CompressionError),
    /// The decompressed payload is itself zstd-compressed; nested compression is not supported.
    #[cfg(feature = "compression")]
    NestedCompression,
    /// The input is zstd-compressed but the `compression` feature is not enabled.
    #[cfg(not(feature = "compression"))]
    CompressionNotSupported,
}

impl std::fmt::Display for OmtsDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OmtsDecodeError::EncodingDetection(e) => {
                write!(f, "encoding detection failed: {e}")
            }
            OmtsDecodeError::Cbor(e) => write!(f, "CBOR decode failed: {e}"),
            OmtsDecodeError::Json(e) => write!(f, "JSON parse failed: {e}"),
            #[cfg(feature = "compression")]
            OmtsDecodeError::Compression(e) => write!(f, "decompression failed: {e}"),
            #[cfg(feature = "compression")]
            OmtsDecodeError::NestedCompression => {
                write!(f, "nested zstd compression is not supported")
            }
            #[cfg(not(feature = "compression"))]
            OmtsDecodeError::CompressionNotSupported => {
                write!(f, "zstd-compressed files require the `compression` feature")
            }
        }
    }
}

impl std::error::Error for OmtsDecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            OmtsDecodeError::EncodingDetection(e) => Some(e),
            OmtsDecodeError::Cbor(e) => Some(e),
            OmtsDecodeError::Json(e) => Some(e),
            #[cfg(feature = "compression")]
            OmtsDecodeError::Compression(e) => Some(e),
            #[cfg(feature = "compression")]
            OmtsDecodeError::NestedCompression => None,
            #[cfg(not(feature = "compression"))]
            OmtsDecodeError::CompressionNotSupported => None,
        }
    }
}

/// Parses an `.omts` file from raw bytes.
///
/// Auto-detects encoding per SPEC-007 Section 2, decompresses zstd if needed
/// (SPEC-007 Section 6), then parses as JSON (Section 3) or CBOR (Section 4).
///
/// `max_decompressed` is the maximum number of bytes the decompressed payload
/// may contain.  This guards against decompression bombs and is only consulted
/// when the input is zstd-compressed.
///
/// Returns the parsed [`OmtsFile`] and the innermost [`Encoding`] detected:
/// - Uncompressed files: the encoding of the input bytes directly.
/// - zstd-compressed files: the encoding of the decompressed payload (JSON or CBOR),
///   never [`Encoding::Zstd`].
pub fn parse_omts(
    bytes: &[u8],
    max_decompressed: usize,
) -> Result<(OmtsFile, Encoding), OmtsDecodeError> {
    let encoding = detect_encoding(bytes).map_err(OmtsDecodeError::EncodingDetection)?;
    match encoding {
        Encoding::Zstd => parse_zstd(bytes, max_decompressed),
        Encoding::Cbor => {
            let file = decode_cbor(bytes).map_err(OmtsDecodeError::Cbor)?;
            Ok((file, Encoding::Cbor))
        }
        Encoding::Json => {
            let file = serde_json::from_slice(bytes).map_err(OmtsDecodeError::Json)?;
            Ok((file, Encoding::Json))
        }
    }
}

#[cfg(feature = "compression")]
fn parse_zstd(
    bytes: &[u8],
    max_decompressed: usize,
) -> Result<(OmtsFile, Encoding), OmtsDecodeError> {
    let decompressed =
        decompress_zstd(bytes, max_decompressed).map_err(OmtsDecodeError::Compression)?;
    let inner = detect_encoding(&decompressed).map_err(OmtsDecodeError::EncodingDetection)?;
    match inner {
        Encoding::Zstd => Err(OmtsDecodeError::NestedCompression),
        Encoding::Cbor => {
            let file = decode_cbor(&decompressed).map_err(OmtsDecodeError::Cbor)?;
            Ok((file, Encoding::Cbor))
        }
        Encoding::Json => {
            let file = serde_json::from_slice(&decompressed).map_err(OmtsDecodeError::Json)?;
            Ok((file, Encoding::Json))
        }
    }
}

#[cfg(not(feature = "compression"))]
fn parse_zstd(
    _bytes: &[u8],
    _max_decompressed: usize,
) -> Result<(OmtsFile, Encoding), OmtsDecodeError> {
    Err(OmtsDecodeError::CompressionNotSupported)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;
    use crate::cbor::encode_cbor;

    const MINIMAL_JSON: &[u8] = include_bytes!("../../../tests/fixtures/minimal.omts");

    fn parse_minimal_json_to_file() -> OmtsFile {
        serde_json::from_slice(MINIMAL_JSON).expect("parse minimal fixture")
    }

    /// Plain JSON input is parsed and returns [`Encoding::Json`].
    #[test]
    fn parse_json_input() {
        let (file, encoding) = parse_omts(MINIMAL_JSON, 1024 * 1024).expect("parse json");
        assert_eq!(encoding, Encoding::Json);
        assert_eq!(file.omts_version.to_string(), "0.1.0");
    }

    /// Plain CBOR input is parsed and returns [`Encoding::Cbor`].
    #[test]
    fn parse_cbor_input() {
        let original = parse_minimal_json_to_file();
        let cbor_bytes = encode_cbor(&original).expect("encode cbor");
        let (file, encoding) = parse_omts(&cbor_bytes, 1024 * 1024).expect("parse cbor");
        assert_eq!(encoding, Encoding::Cbor);
        assert_eq!(file, original);
    }

    /// zstd-compressed JSON is decompressed and parsed, returning [`Encoding::Json`].
    #[cfg(feature = "compression")]
    #[test]
    fn parse_zstd_json_input() {
        use crate::compression::compress_zstd;
        let compressed = compress_zstd(MINIMAL_JSON).expect("compress json");
        let (file, encoding) = parse_omts(&compressed, 1024 * 1024).expect("parse zstd+json");
        assert_eq!(encoding, Encoding::Json);
        assert_eq!(file, parse_minimal_json_to_file());
    }

    /// zstd-compressed CBOR is decompressed and parsed, returning [`Encoding::Cbor`].
    #[cfg(feature = "compression")]
    #[test]
    fn parse_zstd_cbor_input() {
        use crate::compression::compress_zstd;
        let original = parse_minimal_json_to_file();
        let cbor_bytes = encode_cbor(&original).expect("encode cbor");
        let compressed = compress_zstd(&cbor_bytes).expect("compress cbor");
        let (file, encoding) = parse_omts(&compressed, 1024 * 1024).expect("parse zstd+cbor");
        assert_eq!(encoding, Encoding::Cbor);
        assert_eq!(file, original);
    }

    /// Unrecognized initial bytes return an encoding detection error.
    #[test]
    fn parse_unrecognized_bytes_returns_error() {
        let bad = [0xFFu8, 0x00, 0x01, 0x02];
        let err = parse_omts(&bad, 1024).expect_err("should fail");
        assert!(
            matches!(err, OmtsDecodeError::EncodingDetection(_)),
            "expected EncodingDetection, got: {err}"
        );
    }

    /// Syntactically invalid JSON (not schema-valid) returns a JSON error.
    #[test]
    fn parse_invalid_json_schema_returns_error() {
        let not_omts = b"{}";
        let err = parse_omts(not_omts, 1024).expect_err("empty object is not valid OmtsFile");
        assert!(
            matches!(err, OmtsDecodeError::Json(_)),
            "expected Json error, got: {err}"
        );
    }

    /// The `Display` impl produces a non-empty message for each variant.
    #[test]
    fn display_encoding_detection_error() {
        let inner = crate::encoding::EncodingDetectionError {
            first_bytes: vec![0xFF],
        };
        let e = OmtsDecodeError::EncodingDetection(inner);
        assert!(!e.to_string().is_empty());
        assert!(e.to_string().contains("encoding"));
    }

    #[test]
    fn display_json_error() {
        let inner: serde_json::Error =
            serde_json::from_str::<serde_json::Value>("!!!!").expect_err("bad json");
        let e = OmtsDecodeError::Json(inner);
        assert!(!e.to_string().is_empty());
    }

    #[test]
    fn display_cbor_error() {
        let e = OmtsDecodeError::Cbor(crate::cbor::CborError::Decode("test".to_owned()));
        assert!(e.to_string().contains("CBOR"));
    }

    /// Size limit is enforced when decompressing zstd-wrapped JSON.
    #[cfg(feature = "compression")]
    #[test]
    fn parse_zstd_enforces_size_limit() {
        use crate::compression::compress_zstd;
        let compressed = compress_zstd(MINIMAL_JSON).expect("compress");
        let err = parse_omts(&compressed, 1).expect_err("should exceed limit");
        assert!(
            matches!(err, OmtsDecodeError::Compression(_)),
            "expected Compression error, got: {err}"
        );
    }

    /// Doubly-compressed input returns [`OmtsDecodeError::NestedCompression`].
    #[cfg(feature = "compression")]
    #[test]
    fn parse_nested_zstd_returns_error() {
        use crate::compression::compress_zstd;
        let inner_compressed = compress_zstd(MINIMAL_JSON).expect("first compress");
        let double_compressed = compress_zstd(&inner_compressed).expect("second compress");
        let err = parse_omts(&double_compressed, 1024 * 1024).expect_err("nested zstd should fail");
        assert!(
            matches!(err, OmtsDecodeError::NestedCompression),
            "expected NestedCompression, got: {err}"
        );
    }

    /// Full fixture survives a zstd+JSON round-trip.
    #[cfg(feature = "compression")]
    #[test]
    fn parse_zstd_full_fixture() {
        use crate::compression::compress_zstd;
        let fixture_json = include_bytes!("../../../tests/fixtures/full-featured.omts");
        let compressed = compress_zstd(fixture_json).expect("compress");
        let (file, encoding) = parse_omts(&compressed, 16 * 1024 * 1024).expect("parse");
        assert_eq!(encoding, Encoding::Json);
        let expected: OmtsFile =
            serde_json::from_slice(fixture_json).expect("parse fixture directly");
        assert_eq!(file, expected);
    }
}
