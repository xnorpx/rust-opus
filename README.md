# opus-head-sys

Rust FFI bindings to the [Opus audio codec](https://opus-codec.org/) with vendored source code, including AI/DNN models.

## Overview

This crate provides low-level bindings to libopus, built from vendored source code. The goal is to track Opus master as closely as possible with frequent updates.

### Features

- **Vendored source** - Opus is compiled from source, no system dependencies required
- **AI models included** - Ships with Opus 1.5+ DNN models for DRED (Deep Redundancy) and other ML features
- **Full API coverage** - Core, Multistream (surround), and Projection (ambisonics) APIs
- **Static linking** - Single binary with no runtime dependencies on libopus

### What this crate is NOT

This crate focuses on simplicity and staying close to upstream Opus. The following are **non-goals**:

- Dynamic linking to system libopus
- pkg-config or system library detection

If you need these features, consider other opus-sys crates like [`audiopus_sys`](https://crates.io/crates/audiopus_sys) or [`opus-sys`](https://crates.io/crates/opus-sys).

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
