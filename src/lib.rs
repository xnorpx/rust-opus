//! # opus-head-sys
//!
//! Low-level FFI bindings to the Opus audio codec.
//!
//! ## Safety
//!
//! This crate provides raw FFI bindings. All functions that call into the C
//! library are unsafe. Users should consider using a safe wrapper crate.
//!
//! ## License
//!
//! The Opus codec is licensed under the BSD 3-Clause License.
//! See the NOTICE file for full copyright information.

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::all)]

mod bindings;
pub use bindings::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opus_version() {
        unsafe {
            let version = opus_get_version_string();
            assert!(!version.is_null());
            let version_str = std::ffi::CStr::from_ptr(version);
            let version_string = version_str.to_string_lossy();
            println!("Opus version: {}", version_string);
            assert!(version_string.contains("opus") || version_string.contains("libopus"));
        }
    }

    #[test]
    fn test_encoder_size() {
        unsafe {
            let size_mono = opus_encoder_get_size(1);
            let size_stereo = opus_encoder_get_size(2);
            assert!(size_mono > 0);
            assert!(size_stereo > 0);
            assert!(size_stereo >= size_mono);
        }
    }

    #[test]
    fn test_decoder_size() {
        unsafe {
            let size_mono = opus_decoder_get_size(1);
            let size_stereo = opus_decoder_get_size(2);
            assert!(size_mono > 0);
            assert!(size_stereo > 0);
        }
    }

    #[test]
    fn test_error_string() {
        unsafe {
            let ok_str = opus_strerror(OPUS_OK as i32);
            assert!(!ok_str.is_null());

            let bad_arg_str = opus_strerror(OPUS_BAD_ARG);
            assert!(!bad_arg_str.is_null());
        }
    }

    #[test]
    fn test_multistream_encoder_size() {
        unsafe {
            // 2 channels, 1 stream, 1 coupled stream (stereo)
            let size = opus_multistream_encoder_get_size(1, 1);
            assert!(size > 0);

            // 6 channels (5.1 surround), 4 streams, 2 coupled
            let size_surround = opus_multistream_surround_encoder_get_size(6, 1);
            assert!(size_surround > 0);
        }
    }

    #[test]
    fn test_multistream_decoder_size() {
        unsafe {
            // 2 channels, 1 stream, 1 coupled stream (stereo)
            let size = opus_multistream_decoder_get_size(1, 1);
            assert!(size > 0);
        }
    }

    #[test]
    fn test_projection_encoder_size() {
        unsafe {
            // Ambisonics: 4 channels (first-order ambisonics), mapping_family=3
            let size = opus_projection_ambisonics_encoder_get_size(4, 3);
            assert!(size > 0, "Expected positive size, got {}", size);
        }
    }

    #[test]
    fn test_projection_decoder_size() {
        unsafe {
            // 4 output channels, 2 streams, 1 coupled
            let size = opus_projection_decoder_get_size(4, 2, 1);
            assert!(size > 0);
        }
    }
}
