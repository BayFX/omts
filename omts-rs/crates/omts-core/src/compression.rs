//! zstd compression and decompression layer.
//!
//! Implements the compression layer defined in SPEC-007 Section 6.
//!
//! # WASM Compatibility
//!
//! This module requires the `compression` feature (enabled by default).
//! The `zstd` crate uses C bindings and does not compile for `wasm32-unknown-unknown`.
//! WASM targets should depend on `omts-core` with `default-features = false`.
//! Full WASM compatibility will be addressed in T-057.

use std::io::Read;

/// Error returned by compression and decompression operations.
#[derive(Debug)]
pub enum CompressionError {
    /// The compression operation failed.
    CompressionFailed(std::io::Error),
    /// The decompression operation failed.
    DecompressionFailed(std::io::Error),
    /// The decompressed output would exceed the caller-specified size limit.
    SizeLimitExceeded {
        /// The maximum number of bytes the caller permitted.
        max_size: usize,
    },
}

impl std::fmt::Display for CompressionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompressionError::CompressionFailed(e) => write!(f, "compression failed: {e}"),
            CompressionError::DecompressionFailed(e) => write!(f, "decompression failed: {e}"),
            CompressionError::SizeLimitExceeded { max_size } => write!(
                f,
                "decompressed data exceeds maximum allowed size of {max_size} bytes"
            ),
        }
    }
}

impl std::error::Error for CompressionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CompressionError::CompressionFailed(e) => Some(e),
            CompressionError::DecompressionFailed(e) => Some(e),
            CompressionError::SizeLimitExceeded { .. } => None,
        }
    }
}

/// Compresses `data` using zstd at the default compression level.
///
/// The output starts with the zstd magic bytes `0x28 0xB5 0x2F 0xFD`, enabling
/// format detection per SPEC-007 Section 6.3.
pub fn compress_zstd(data: &[u8]) -> Result<Vec<u8>, CompressionError> {
    zstd::encode_all(data, 0).map_err(CompressionError::CompressionFailed)
}

/// Decompresses zstd-compressed `data`, enforcing a size cap against decompression bombs.
///
/// `max_size` is the maximum number of bytes the decompressed payload may contain.
/// If the actual decompressed size would exceed `max_size`, the function returns
/// [`CompressionError::SizeLimitExceeded`] after reading `max_size + 1` bytes,
/// without loading the remainder of the stream into memory.
///
/// On success, callers should apply encoding detection per SPEC-007 Section 2
/// to determine whether the decompressed payload is JSON or CBOR.
pub fn decompress_zstd(data: &[u8], max_size: usize) -> Result<Vec<u8>, CompressionError> {
    let decoder = zstd::Decoder::new(data).map_err(CompressionError::DecompressionFailed)?;

    // Reading max_size + 1 bytes lets us distinguish "exactly max_size" from "more than
    // max_size" without buffering the full stream first.
    let limit = (max_size as u64).saturating_add(1);
    let mut limited = decoder.take(limit);
    let mut output = Vec::new();
    limited
        .read_to_end(&mut output)
        .map_err(CompressionError::DecompressionFailed)?;

    if output.len() > max_size {
        return Err(CompressionError::SizeLimitExceeded { max_size });
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]

    use super::*;
    use crate::encoding::{Encoding, detect_encoding};

    #[test]
    fn round_trip_json() {
        let json = br#"{"omts_version":"1.0","nodes":[],"edges":[]}"#;
        let compressed = compress_zstd(json).expect("compress");
        assert_eq!(
            detect_encoding(&compressed).expect("detect compressed"),
            Encoding::Zstd
        );
        let decompressed = decompress_zstd(&compressed, 4096).expect("decompress");
        assert_eq!(decompressed, json);
        assert_eq!(
            detect_encoding(&decompressed).expect("detect decompressed"),
            Encoding::Json
        );
    }

    #[test]
    fn compressed_output_starts_with_zstd_magic() {
        let data = b"test payload";
        let compressed = compress_zstd(data).expect("compress");
        assert_eq!(&compressed[..4], &[0x28, 0xB5, 0x2F, 0xFD]);
    }

    #[test]
    fn size_limit_exceeded() {
        let data = vec![b'x'; 1024];
        let compressed = compress_zstd(&data).expect("compress");
        let result = decompress_zstd(&compressed, 100);
        assert!(matches!(
            result,
            Err(CompressionError::SizeLimitExceeded { max_size: 100 })
        ));
    }

    #[test]
    fn size_limit_exact_boundary_succeeds() {
        let data = b"hello";
        let compressed = compress_zstd(data).expect("compress");
        decompress_zstd(&compressed, data.len()).expect("exactly at limit should succeed");
    }

    #[test]
    fn size_limit_one_under_boundary_fails() {
        let data = b"hello";
        let compressed = compress_zstd(data).expect("compress");
        let result = decompress_zstd(&compressed, data.len() - 1);
        assert!(matches!(
            result,
            Err(CompressionError::SizeLimitExceeded { .. })
        ));
    }

    #[test]
    fn empty_input_round_trip() {
        let data: &[u8] = b"";
        let compressed = compress_zstd(data).expect("compress empty");
        let decompressed = decompress_zstd(&compressed, 0).expect("decompress empty");
        assert!(decompressed.is_empty());
    }

    #[test]
    fn invalid_data_returns_error() {
        let result = decompress_zstd(b"this is not valid zstd", 4096);
        assert!(matches!(
            result,
            Err(CompressionError::DecompressionFailed(_))
        ));
    }

    #[test]
    fn error_display_compression_failed() {
        let e = CompressionError::CompressionFailed(std::io::Error::other("test"));
        assert!(e.to_string().contains("compression failed"));
    }

    #[test]
    fn error_display_decompression_failed() {
        let e = CompressionError::DecompressionFailed(std::io::Error::other("test"));
        assert!(e.to_string().contains("decompression failed"));
    }

    #[test]
    fn error_display_size_limit_exceeded() {
        let e = CompressionError::SizeLimitExceeded { max_size: 42 };
        let s = e.to_string();
        assert!(s.contains("42"));
    }
}
