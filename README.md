# opus-head-sys

Rust FFI bindings to the [Opus audio codec](https://opus-codec.org/) with vendored source code, including support for AI/DNN features (DRED, OSCE).

## Overview

This crate provides low-level bindings to libopus, built from vendored source code. The goal is to track Opus master as closely as possible with frequent updates.

### Features

- **Vendored source** - Opus is compiled from source, no system dependencies required
- **AI/DNN support** - DRED (Deep Redundancy) and OSCE (Opus Speech Coding Enhancement) via runtime weight loading
- **Full API coverage** - Core, Multistream (surround), and Projection (ambisonics) APIs
- **Static linking** - Single binary with no runtime dependencies on libopus
- **Small crate size** - Under 5MB (DNN weights loaded at runtime, not embedded)

### What this crate is NOT

This crate focuses on simplicity and staying close to upstream Opus. The following are **non-goals**:

- Dynamic linking to system libopus
- pkg-config or system library detection

If you need these features, consider other opus-sys crates like [`audiopus_sys`](https://crates.io/crates/audiopus_sys) or [`opus-sys`](https://crates.io/crates/opus-sys).

## Using AI Features (DRED/OSCE)

The AI features require loading DNN weights at runtime. This keeps the crate small while still supporting the full Opus AI capabilities.

### 1. Enable the features

```toml
[dependencies]
opus-head-sys = { version = "0.1", features = ["dred", "osce"] }
```

### 2. Download the weights file

Download `opus_data-<hash>.bin` (~14MB) from the [releases page](https://github.com/xnorpx/rust-opus/releases).

Place it in your project or a known location to load at runtime.

### 3. Load weights at runtime

```rust
use opus_head_sys::*;

// Read the weights file
let weights = std::fs::read("opus_data-<hash>.bin").expect("Failed to read weights");

unsafe {
    // Create encoder
    let mut error = 0;
    let encoder = opus_encoder_create(48000, 1, OPUS_APPLICATION_VOIP as i32, &mut error);
    
    // Load DNN weights into encoder
    opus_encoder_ctl(
        encoder,
        OPUS_SET_DNN_BLOB_REQUEST as i32,
        weights.as_ptr() as *const std::ffi::c_void,
        weights.len() as i32,
    );
    
    // Now DRED encoding is available
    // Enable DRED:
    opus_encoder_ctl(encoder, OPUS_SET_DRED_DURATION_REQUEST as i32, 100); // 100ms of redundancy
    
    // ... encode audio ...
    
    opus_encoder_destroy(encoder);
}
```

The same applies to the decoder for OSCE (speech enhancement) features.

### Why runtime loading?

The DNN weights are ~14MB, which would exceed crates.io's 10MB limit if embedded. Runtime loading also allows:
- Updating weights without recompiling
- Sharing weights across multiple encoder/decoder instances
- Optional AI features (don't load weights if you don't need them)

## Vendored Version

The vendored Opus source is tracked in `vendored/OPUS_VERSION`. Run the update script to sync with upstream:

```bash
python vendor_opus.py
```

## License

The Rust bindings in this crate are licensed under MIT OR Apache-2.0.

The vendored Opus codec is licensed under the BSD 3-Clause License. See [NOTICE](NOTICE) for full copyright information and patent licenses.

## Links

- [Opus Codec](https://opus-codec.org/)
- [Opus GitHub](https://github.com/xiph/opus)
- [RFC 6716](https://tools.ietf.org/html/rfc6716) - Opus specification
