# Release Process

This document describes how to create a new release of `opus-head-sys`.

## Pre-release Checklist

1. **Update version** in `Cargo.toml`
2. **Update CHANGELOG** (if you have one)
3. **Run tests** to ensure everything works:
   ```bash
   cargo test --all-features
   cargo clippy --all-features
   ```
4. **Verify package** builds correctly:
   ```bash
   cargo publish --dry-run
   ```

## Creating a Release

### 1. Commit Version Changes

```bash
git add Cargo.toml
git commit -m "chore: bump version to X.Y.Z"
git push origin main
```

### 2. Create and Push a Git Tag

The release pipeline triggers on version tags matching `v*`:

```bash
# Create annotated tag
git tag -a v0.1.0 -m "Release v0.1.0"

# Push the tag
git push origin v0.1.0
```

### 3. Release Pipeline

Once you push the tag, the GitHub Actions release pipeline (`.github/workflows/release.yml`) will automatically:

1. **Generate DNN weights** - Compiles and runs the official Opus `write_lpcnet_weights.c` program
2. **Create artifacts**:
   - `opus_data-<model_hash>.bin` - DNN weights blob
   - `opus_data-<model_hash>.bin.md5` - MD5 checksum
3. **Create GitHub Release** with:
   - Release notes
   - Attached weight files as downloadable assets

Monitor the workflow at: `https://github.com/xnorpx/rust-opus/actions`

### 4. Publish to crates.io

After the GitHub release is created successfully:

```bash
# Login to crates.io (first time only)
cargo login

# Publish the crate
cargo publish
```

**Note:** The crate excludes the large DNN weight data files (`*_data.c`) to stay under crates.io's 10MB limit. Users need to download weights separately from GitHub releases when using DRED/OSCE features.

## Version Numbering

This project follows [Semantic Versioning](https://semver.org/):

- **MAJOR** (X.0.0) - Incompatible API changes
- **MINOR** (0.X.0) - New features, backwards compatible
- **PATCH** (0.0.X) - Bug fixes, backwards compatible

For pre-releases, use suffixes like `-alpha`, `-beta`, `-rc.1`:
```bash
git tag -a v0.2.0-alpha.1 -m "Pre-release v0.2.0-alpha.1"
```

## Manual Weight Generation

If you need to generate weights locally (e.g., for testing):

```bash
# Requires a C compiler (MSVC, GCC, or Clang)
python generate_weights.py --output ./weights

# Output files:
#   weights/opus_data-<hash>.bin      - DNN weights blob
#   weights/opus_data-<hash>.bin.md5  - MD5 checksum
```

## Troubleshooting

### Publish fails with "crate too large"

Ensure the exclusions in `Cargo.toml` are correct:
```bash
cargo package --list | grep -E "\.c$|data"
```

No `*_data.c` files should appear in the output.

### Release workflow fails

Check the GitHub Actions logs. Common issues:
- Missing C compiler in CI environment
- Opus source not properly checked out (submodules)

### Weights don't match expected hash

The model hash comes from `vendored/opus/autogen.sh`. If Opus updates their models, the hash will change automatically.

## Release History

### v0.1.0
- Initial release
- FFI bindings to Opus codec
- DRED (Deep REDundancy) support
- OSCE (Opus Speech Coding Enhancement) support
- Runtime DNN weight loading via `OPUS_SET_DNN_BLOB()`
