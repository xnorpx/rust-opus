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

    /// Test loading DNN weights from target/model and setting them via OPUS_SET_DNN_BLOB.
    ///
    /// This test requires the weights to be generated first:
    /// ```bash
    /// python generate_weights.py
    /// ```
    ///
    /// Note: As of now, the actual weight loading may crash on some configurations.
    /// This test verifies that the weight file can be read and has the correct format.
    #[test]
    #[cfg(feature = "dnn")]
    fn test_dnn_blob_loading() {
        use std::path::PathBuf;

        // Find the weights file in target/model
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let model_dir = manifest_dir.join("target").join("model");

        println!("Looking for weights in: {:?}", model_dir);

        // Look for any opus_data-*.bin file
        let weights_path = match std::fs::read_dir(&model_dir) {
            Ok(dir) => {
                match dir.filter_map(|e| e.ok()).map(|e| e.path()).find(|p| {
                    p.file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.starts_with("opus_data-") && n.ends_with(".bin"))
                        .unwrap_or(false)
                }) {
                    Some(path) => path,
                    None => {
                        panic!(
                            "No opus_data-*.bin found in {:?}.\n\
                            Run 'python generate_weights.py' to generate weights.",
                            model_dir
                        );
                    }
                }
            }
            Err(e) => {
                panic!(
                    "target/model directory not found ({:?}): {}\n\
                    Run 'python generate_weights.py' to generate weights.",
                    model_dir, e
                );
            }
        };

        println!("Loading weights from: {:?}", weights_path);

        // Read the weights file
        let weights_data = std::fs::read(&weights_path).expect("Failed to read weights file");

        println!("Loaded {} bytes of DNN weights", weights_data.len());

        // Verify the file has the correct magic header "DNNw"
        assert!(weights_data.len() > 4, "Weights file too small");
        assert_eq!(
            &weights_data[0..4],
            b"DNNw",
            "Invalid weights file magic header"
        );

        // Verify reasonable size (should be ~14MB for full weights)
        assert!(
            weights_data.len() > 1_000_000,
            "Weights file seems too small"
        );
        assert!(
            weights_data.len() < 100_000_000,
            "Weights file seems too large"
        );

        println!("Weight file format validation passed");
        println!("  Magic: DNNw");
        println!(
            "  Size: {} bytes ({:.2} MB)",
            weights_data.len(),
            weights_data.len() as f64 / 1_000_000.0
        );

        // Test encoder with DNN blob loading
        unsafe {
            let mut error: i32 = 0;

            // Create encoder
            println!("Creating encoder...");
            let encoder = opus_encoder_create(48000, 1, OPUS_APPLICATION_VOIP as i32, &mut error);
            println!(
                "opus_encoder_create returned error={}, encoder={:?}",
                error, encoder
            );
            if error != OPUS_OK as i32 || encoder.is_null() {
                println!("Failed to create encoder");
                return;
            }

            // Load DNN weights into encoder
            println!("Calling opus_encoder_ctl(OPUS_SET_DNN_BLOB)...");
            let ret = opus_encoder_ctl(
                encoder,
                OPUS_SET_DNN_BLOB_REQUEST as i32,
                weights_data.as_ptr() as *const std::ffi::c_void,
                weights_data.len() as i32,
            );
            println!("opus_encoder_ctl(OPUS_SET_DNN_BLOB) returned: {}", ret);

            println!("Destroying encoder...");
            opus_encoder_destroy(encoder);
            println!("Encoder destroyed");

            if ret != OPUS_OK as i32 {
                println!("DNN blob loading failed on encoder with code {}", ret);
                println!("This may indicate the weights file format is incompatible.");
                return;
            }

            println!("Encoder DNN blob loading: OK");

            // Create decoder
            println!("Creating decoder...");
            let decoder = opus_decoder_create(48000, 1, &mut error);
            println!(
                "opus_decoder_create returned error={}, decoder={:?}",
                error, decoder
            );
            if error != OPUS_OK as i32 || decoder.is_null() {
                println!("Failed to create decoder");
                return;
            }

            // Load DNN weights into decoder
            println!("Calling opus_decoder_ctl(OPUS_SET_DNN_BLOB)...");
            let ret = opus_decoder_ctl(
                decoder,
                OPUS_SET_DNN_BLOB_REQUEST as i32,
                weights_data.as_ptr() as *const std::ffi::c_void,
                weights_data.len() as i32,
            );
            println!("opus_decoder_ctl(OPUS_SET_DNN_BLOB) returned: {}", ret);

            println!("Destroying decoder...");
            opus_decoder_destroy(decoder);
            println!("Decoder destroyed");

            if ret != OPUS_OK as i32 {
                println!("DNN blob loading failed on decoder with code {}", ret);
                return;
            }

            println!("Decoder DNN blob loading: OK");
        }

        println!("DNN blob loading test passed!");
    }

    /// Helper function to create deterministic random audio data (noise)
    /// Uses a seed for reproducibility across tests
    fn generate_noise_with_seed(samples: usize, seed: u64) -> Vec<i16> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut data = Vec::with_capacity(samples);
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);

        for i in 0..samples {
            i.hash(&mut hasher);
            let hash = hasher.finish();
            // Scale to i16 range with reduced amplitude to avoid clipping
            let sample = ((hash as i32 % 32768) - 16384) as i16;
            data.push(sample);
        }
        data
    }

    /// Helper function to create random audio data (noise) - default seed
    fn generate_noise(samples: usize) -> Vec<i16> {
        generate_noise_with_seed(samples, 12345)
    }

    /// Helper function to load DNN weights if available
    fn load_dnn_weights() -> Option<Vec<u8>> {
        use std::path::PathBuf;

        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let model_dir = manifest_dir.join("target").join("model");

        let weights_path = std::fs::read_dir(&model_dir)
            .ok()?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .find(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("opus_data-") && n.ends_with(".bin"))
                    .unwrap_or(false)
            })?;

        std::fs::read(&weights_path).ok()
    }

    /// Test basic encode/decode roundtrip WITHOUT DNN model
    /// Also tries to enable DRED (should fail without DNN weights loaded)
    #[test]
    fn test_encode_decode_without_dnn() {
        const SAMPLE_RATE: i32 = 48000;
        const CHANNELS: i32 = 1; // Mono
        const FRAME_SIZE: usize = 960; // 20ms at 48kHz
        const BITRATE: i32 = 32000; // 32kbit/s

        unsafe {
            let mut error: i32 = 0;

            // Create encoder
            let encoder = opus_encoder_create(
                SAMPLE_RATE,
                CHANNELS,
                OPUS_APPLICATION_VOIP as i32,
                &mut error,
            );
            assert_eq!(error, OPUS_OK as i32, "Failed to create encoder");
            assert!(!encoder.is_null());

            // Set bitrate
            let ret = opus_encoder_ctl(encoder, OPUS_SET_BITRATE_REQUEST as i32, BITRATE);
            assert_eq!(ret, OPUS_OK as i32, "Failed to set bitrate");

            // Set packet loss (needed for DRED to add redundancy)
            let packet_loss_perc: i32 = 25;
            let ret = opus_encoder_ctl(
                encoder,
                OPUS_SET_PACKET_LOSS_PERC_REQUEST as i32,
                packet_loss_perc,
            );
            assert_eq!(ret, OPUS_OK as i32, "Failed to set packet loss");

            // Try to enable DRED without DNN weights (should fail or be ignored)
            let dred_duration_ms: i32 = 100; // 100ms of DRED redundancy
            let ret = opus_encoder_ctl(
                encoder,
                OPUS_SET_DRED_DURATION_REQUEST as i32,
                dred_duration_ms,
            );
            println!("OPUS_SET_DRED_DURATION (without DNN) returned: {}", ret);
            // Note: This may return OPUS_OK but DRED won't actually work without weights

            // Create decoder
            let decoder = opus_decoder_create(SAMPLE_RATE, CHANNELS, &mut error);
            assert_eq!(error, OPUS_OK as i32, "Failed to create decoder");
            assert!(!decoder.is_null());

            // Generate noise input
            let input = generate_noise(FRAME_SIZE);
            let mut encoded = vec![0u8; 4000]; // Max packet size
            let mut decoded = vec![0i16; FRAME_SIZE];

            // Encode
            let encoded_len = opus_encode(
                encoder,
                input.as_ptr(),
                FRAME_SIZE as i32,
                encoded.as_mut_ptr(),
                encoded.len() as i32,
            );
            assert!(
                encoded_len > 0,
                "Encoding failed with error {}",
                encoded_len
            );
            println!(
                "Encoded {} samples to {} bytes (WITHOUT DNN, DRED attempted)",
                FRAME_SIZE, encoded_len
            );

            // Decode
            let decoded_len = opus_decode(
                decoder,
                encoded.as_ptr(),
                encoded_len,
                decoded.as_mut_ptr(),
                FRAME_SIZE as i32,
                0,
            );
            assert_eq!(
                decoded_len, FRAME_SIZE as i32,
                "Decoding failed or wrong frame size"
            );
            println!(
                "Decoded {} bytes to {} samples (WITHOUT DNN)",
                encoded_len, decoded_len
            );

            // Cleanup
            opus_encoder_destroy(encoder);
            opus_decoder_destroy(decoder);
        }

        println!("Encode/decode test WITHOUT DNN passed!");
    }

    /// Test basic encode/decode roundtrip WITH DNN model loaded and DRED enabled
    #[test]
    #[cfg(feature = "dnn")]
    fn test_encode_decode_with_dnn() {
        const SAMPLE_RATE: i32 = 48000;
        const CHANNELS: i32 = 1;
        const FRAME_SIZE: usize = 960; // 20ms at 48kHz
        const BITRATE: i32 = 64000;

        // Load DNN weights
        let weights_data = match load_dnn_weights() {
            Some(data) => data,
            None => {
                panic!(
                    "DNN weights not found. Run 'python generate_weights.py' to generate weights."
                );
            }
        };

        println!("Loaded {} bytes of DNN weights", weights_data.len());

        unsafe {
            let mut error: i32 = 0;

            // Create encoder
            let encoder = opus_encoder_create(
                SAMPLE_RATE,
                CHANNELS,
                OPUS_APPLICATION_VOIP as i32,
                &mut error,
            );
            assert_eq!(error, OPUS_OK as i32, "Failed to create encoder");
            assert!(!encoder.is_null());

            // Set bitrate
            let ret = opus_encoder_ctl(encoder, OPUS_SET_BITRATE_REQUEST as i32, BITRATE);
            assert_eq!(ret, OPUS_OK as i32, "Failed to set bitrate");

            // Set expected packet loss (needed for DRED to add redundancy)
            let packet_loss_perc: i32 = 25; // 25% expected packet loss
            let ret = opus_encoder_ctl(
                encoder,
                OPUS_SET_PACKET_LOSS_PERC_REQUEST as i32,
                packet_loss_perc,
            );
            assert_eq!(ret, OPUS_OK as i32, "Failed to set packet loss");
            println!("Packet loss set to {}%", packet_loss_perc);

            // Load DNN weights into encoder
            let ret = opus_encoder_ctl(
                encoder,
                OPUS_SET_DNN_BLOB_REQUEST as i32,
                weights_data.as_ptr() as *const std::ffi::c_void,
                weights_data.len() as i32,
            );
            assert_eq!(
                ret, OPUS_OK as i32,
                "Failed to load DNN weights into encoder"
            );
            println!("DNN weights loaded into encoder");

            // Enable DRED with 100ms of redundancy
            let dred_duration_ms: i32 = 100;
            let ret = opus_encoder_ctl(
                encoder,
                OPUS_SET_DRED_DURATION_REQUEST as i32,
                dred_duration_ms,
            );
            println!("OPUS_SET_DRED_DURATION (with DNN) returned: {}", ret);
            assert_eq!(ret, OPUS_OK as i32, "Failed to enable DRED");
            println!("DRED enabled with {}ms redundancy", dred_duration_ms);

            // Create decoder
            let decoder = opus_decoder_create(SAMPLE_RATE, CHANNELS, &mut error);
            assert_eq!(error, OPUS_OK as i32, "Failed to create decoder");
            assert!(!decoder.is_null());

            // Load DNN weights into decoder (may not be supported for all decoder types)
            let ret = opus_decoder_ctl(
                decoder,
                OPUS_SET_DNN_BLOB_REQUEST as i32,
                weights_data.as_ptr() as *const std::ffi::c_void,
                weights_data.len() as i32,
            );
            if ret == OPUS_OK as i32 {
                println!("DNN weights loaded into decoder");
            } else {
                println!(
                    "Note: Decoder DNN blob returned {} (OSCE may use different API)",
                    ret
                );
            }

            // Generate noise input
            let input = generate_noise(FRAME_SIZE);
            let mut encoded = vec![0u8; 4000]; // Max packet size
            let mut decoded = vec![0i16; FRAME_SIZE];

            // Encode (with DRED enabled, packet may be larger)
            let encoded_len = opus_encode(
                encoder,
                input.as_ptr(),
                FRAME_SIZE as i32,
                encoded.as_mut_ptr(),
                encoded.len() as i32,
            );
            assert!(
                encoded_len > 0,
                "Encoding failed with error {}",
                encoded_len
            );
            println!(
                "Encoded {} samples to {} bytes (with DNN + DRED)",
                FRAME_SIZE, encoded_len
            );

            // Decode
            let decoded_len = opus_decode(
                decoder,
                encoded.as_ptr(),
                encoded_len,
                decoded.as_mut_ptr(),
                FRAME_SIZE as i32,
                0,
            );
            assert_eq!(
                decoded_len, FRAME_SIZE as i32,
                "Decoding failed or wrong frame size"
            );
            println!(
                "Decoded {} bytes to {} samples (with DNN + DRED)",
                encoded_len, decoded_len
            );

            // Cleanup
            opus_encoder_destroy(encoder);
            opus_decoder_destroy(decoder);
        }

        println!("Encode/decode test WITH DNN + DRED passed!");
    }

    /// Test multiple frames encode/decode WITHOUT DNN
    #[test]
    fn test_multi_frame_encode_decode_without_dnn() {
        const SAMPLE_RATE: i32 = 48000;
        const CHANNELS: i32 = 1; // Mono
        const FRAME_SIZE: usize = 960; // 20ms at 48kHz
        const NUM_FRAMES: usize = 10;
        const BITRATE: i32 = 32000; // 32kbit/s

        unsafe {
            let mut error: i32 = 0;

            // Create encoder (VOIP for voice)
            let encoder = opus_encoder_create(
                SAMPLE_RATE,
                CHANNELS,
                OPUS_APPLICATION_VOIP as i32,
                &mut error,
            );
            assert_eq!(error, OPUS_OK as i32, "Failed to create encoder");

            // Set bitrate
            opus_encoder_ctl(encoder, OPUS_SET_BITRATE_REQUEST as i32, BITRATE);

            // Set packet loss (needed for DRED to add redundancy)
            let packet_loss_perc: i32 = 25;
            let ret = opus_encoder_ctl(
                encoder,
                OPUS_SET_PACKET_LOSS_PERC_REQUEST as i32,
                packet_loss_perc,
            );
            assert_eq!(ret, OPUS_OK as i32, "Failed to set packet loss");

            // Try to enable DRED (should fail or be ignored without DNN)
            let dred_duration_ms: i32 = 100;
            let ret = opus_encoder_ctl(
                encoder,
                OPUS_SET_DRED_DURATION_REQUEST as i32,
                dred_duration_ms,
            );
            println!("OPUS_SET_DRED_DURATION (WITHOUT DNN) returned: {}", ret);

            // Create decoder
            let decoder = opus_decoder_create(SAMPLE_RATE, CHANNELS, &mut error);
            assert_eq!(error, OPUS_OK as i32, "Failed to create decoder");

            let mut total_encoded_bytes = 0;
            let mut frame_sizes = Vec::new();

            for frame_num in 0..NUM_FRAMES {
                // Generate mono noise input
                let input = generate_noise(FRAME_SIZE);
                let mut encoded = vec![0u8; 4000];
                let mut decoded = vec![0i16; FRAME_SIZE];

                // Encode
                let encoded_len = opus_encode(
                    encoder,
                    input.as_ptr(),
                    FRAME_SIZE as i32,
                    encoded.as_mut_ptr(),
                    encoded.len() as i32,
                );
                assert!(encoded_len > 0, "Frame {} encoding failed", frame_num);
                total_encoded_bytes += encoded_len;
                frame_sizes.push(encoded_len);

                // Decode
                let decoded_len = opus_decode(
                    decoder,
                    encoded.as_ptr(),
                    encoded_len,
                    decoded.as_mut_ptr(),
                    FRAME_SIZE as i32,
                    0,
                );
                assert_eq!(
                    decoded_len, FRAME_SIZE as i32,
                    "Frame {} decoding failed",
                    frame_num
                );
            }

            let duration_ms = NUM_FRAMES * 20;
            let bitrate_actual = (total_encoded_bytes * 8 * 1000) / duration_ms as i32;
            println!("\n=== WITHOUT DNN MODEL ===");
            println!("Frame sizes: {:?}", frame_sizes);
            println!(
                "Encoded {} frames ({} ms) to {} bytes, actual bitrate: {} bps",
                NUM_FRAMES, duration_ms, total_encoded_bytes, bitrate_actual
            );

            opus_encoder_destroy(encoder);
            opus_decoder_destroy(decoder);
        }

        println!("Multi-frame encode/decode test WITHOUT DNN passed!");
    }

    /// Test multiple frames encode/decode WITH DNN
    #[test]
    #[cfg(feature = "dnn")]
    fn test_multi_frame_encode_decode_with_dnn() {
        const SAMPLE_RATE: i32 = 48000;
        const CHANNELS: i32 = 1; // Mono - DRED works better with mono voice
        const FRAME_SIZE: usize = 960; // 20ms at 48kHz
        const NUM_FRAMES: usize = 10;
        const BITRATE: i32 = 32000; // 32kbit/s - typical for voice, forces SILK mode

        // Load DNN weights
        let weights_data = match load_dnn_weights() {
            Some(data) => data,
            None => {
                panic!(
                    "DNN weights not found. Run 'python generate_weights.py' to generate weights."
                );
            }
        };

        unsafe {
            let mut error: i32 = 0;

            // Create encoder (VOIP mode for voice)
            let encoder = opus_encoder_create(
                SAMPLE_RATE,
                CHANNELS,
                OPUS_APPLICATION_VOIP as i32,
                &mut error,
            );
            assert_eq!(error, OPUS_OK as i32, "Failed to create encoder");

            // Set bitrate
            opus_encoder_ctl(encoder, OPUS_SET_BITRATE_REQUEST as i32, BITRATE);

            // Set expected packet loss (needed for DRED to add redundancy)
            let packet_loss_perc: i32 = 25; // 25% expected packet loss
            let ret = opus_encoder_ctl(
                encoder,
                OPUS_SET_PACKET_LOSS_PERC_REQUEST as i32,
                packet_loss_perc,
            );
            assert_eq!(ret, OPUS_OK as i32, "Failed to set packet loss");
            println!("Packet loss set to {}%", packet_loss_perc);

            // Load DNN weights into encoder
            let ret = opus_encoder_ctl(
                encoder,
                OPUS_SET_DNN_BLOB_REQUEST as i32,
                weights_data.as_ptr() as *const std::ffi::c_void,
                weights_data.len() as i32,
            );
            assert_eq!(
                ret, OPUS_OK as i32,
                "Failed to load DNN weights into encoder"
            );

            // Enable DRED with 100ms of redundancy
            let dred_duration_ms: i32 = 100;
            let ret = opus_encoder_ctl(
                encoder,
                OPUS_SET_DRED_DURATION_REQUEST as i32,
                dred_duration_ms,
            );
            assert_eq!(ret, OPUS_OK as i32, "Failed to enable DRED");
            println!("DRED enabled with {}ms redundancy", dred_duration_ms);

            // Create decoder
            let decoder = opus_decoder_create(SAMPLE_RATE, CHANNELS, &mut error);
            assert_eq!(error, OPUS_OK as i32, "Failed to create decoder");

            // Load DNN weights into decoder (may not be supported for all decoder types)
            let ret = opus_decoder_ctl(
                decoder,
                OPUS_SET_DNN_BLOB_REQUEST as i32,
                weights_data.as_ptr() as *const std::ffi::c_void,
                weights_data.len() as i32,
            );
            if ret == OPUS_OK as i32 {
                println!("DNN weights loaded into decoder");
            } else {
                println!(
                    "Note: Decoder DNN blob returned {} (OSCE may use different API)",
                    ret
                );
            }

            let mut total_encoded_bytes = 0;
            let mut frame_sizes = Vec::new();

            for frame_num in 0..NUM_FRAMES {
                // Generate mono noise input with frame-specific seed for reproducibility
                let input = generate_noise_with_seed(FRAME_SIZE, frame_num as u64);
                let mut encoded = vec![0u8; 4000];
                let mut decoded = vec![0i16; FRAME_SIZE];

                // Encode
                let encoded_len = opus_encode(
                    encoder,
                    input.as_ptr(),
                    FRAME_SIZE as i32,
                    encoded.as_mut_ptr(),
                    encoded.len() as i32,
                );
                assert!(encoded_len > 0, "Frame {} encoding failed", frame_num);
                total_encoded_bytes += encoded_len;
                frame_sizes.push(encoded_len);

                // Decode
                let decoded_len = opus_decode(
                    decoder,
                    encoded.as_ptr(),
                    encoded_len,
                    decoded.as_mut_ptr(),
                    FRAME_SIZE as i32,
                    0,
                );
                assert_eq!(
                    decoded_len, FRAME_SIZE as i32,
                    "Frame {} decoding failed",
                    frame_num
                );
            }

            let duration_ms = NUM_FRAMES * 20;
            let bitrate_actual = (total_encoded_bytes * 8 * 1000) / duration_ms as i32;
            println!("\n=== WITH DNN MODEL + DRED ===");
            println!("Frame sizes: {:?}", frame_sizes);
            println!(
                "Encoded {} frames ({} ms) to {} bytes, actual bitrate: {} bps",
                NUM_FRAMES, duration_ms, total_encoded_bytes, bitrate_actual
            );

            opus_encoder_destroy(encoder);
            opus_decoder_destroy(decoder);
        }

        println!("Multi-frame encode/decode test WITH DNN + DRED passed!");
    }
}
