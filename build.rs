//! Build script for opus-head-sys
//!
//! Compiles the vendored Opus library using CMake with cross-platform support.

use cmake::Config;
use std::{env, path::PathBuf};

macro_rules! warn {
    ($($arg:tt)*) => {
        println!("cargo:warning={}", format!($($arg)*));
    };
}

fn main() {
    if let Err(e) = build_opus() {
        panic!("Failed to build Opus: {}", e);
    }
}

fn build_opus() -> Result<(), Box<dyn std::error::Error>> {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    let target_triple = env::var("TARGET")?;

    // Skip build for WASM targets
    if target_arch.starts_with("wasm") {
        warn!(
            "WASM target detected ({}), skipping Opus build",
            target_arch
        );
        return Ok(());
    }

    warn!("Building Opus for {} ({})", target_triple, target_arch);

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let opus_dir = manifest_dir.join("vendored").join("opus");

    if !opus_dir.is_dir() {
        return Err(format!(
            "Missing Opus source directory: {}. Run 'python vendor_opus.py' to download.",
            opus_dir.display()
        )
        .into());
    }

    println!("cargo:rerun-if-changed=vendored/opus");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");

    let mut config = Config::new(&opus_dir);

    // Use RelWithDebInfo for debug builds (gives debug symbols but links release CRT)
    // On Windows, pure Debug builds use MSVCRTD which conflicts with Rust's CRT
    let profile = if env::var("PROFILE").unwrap_or_default() == "release" {
        "Release"
    } else {
        "RelWithDebInfo"
    };

    config
        .profile(profile)
        .define("OPUS_BUILD_SHARED_LIBRARY", "OFF")
        .define("OPUS_BUILD_TESTING", "OFF")
        .define("OPUS_BUILD_PROGRAMS", "OFF")
        .define("OPUS_INSTALL_PKG_CONFIG_MODULE", "OFF")
        .define("OPUS_INSTALL_CMAKE_CONFIG_MODULE", "OFF");

    // Platform-specific configuration
    configure_for_platform(&mut config, &target_os, &target_arch, &target_triple);

    // CPU feature detection for x86_64
    if target_arch == "x86_64" {
        configure_x86_features(&mut config);
    }

    // Configure Cargo feature flags
    configure_features(&mut config, &target_os, &target_arch);

    // Windows-specific: enable Control Flow Guard
    let dst = if target_os == "windows" {
        config.cflag("/guard:cf").build()
    } else {
        config.build()
    };

    // Tell cargo where to find the library
    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-search=native={}/lib64", dst.display());
    println!("cargo:rustc-link-lib=static=opus");

    warn!("Opus build complete");
    Ok(())
}

fn configure_for_platform(
    config: &mut Config,
    target_os: &str,
    target_arch: &str,
    target_triple: &str,
) {
    // Pass ANDROID_ABI if set in environment (for Android cross-compilation)
    if let Ok(abi) = env::var("ANDROID_ABI") {
        config.define("ANDROID_ABI", abi);
    }

    let host_arch = env::var("CARGO_CFG_TARGET_ARCH")
        .map(|_| env::consts::ARCH)
        .unwrap_or(env::consts::ARCH);

    match target_os {
        "ios" => configure_ios(config, target_arch, target_triple),
        "macos" => configure_macos(config, target_arch),
        // Only apply cross-compilation settings when host is x86/x64 and target is ARM64
        "windows" if target_arch == "aarch64" => configure_windows_arm64(config, host_arch),
        _ => {}
    }
}

fn configure_ios(config: &mut Config, target_arch: &str, target_triple: &str) {
    let deployment_target =
        env::var("IPHONEOS_DEPLOYMENT_TARGET").unwrap_or_else(|_| "14.0".to_string());

    let is_simulator = target_triple.contains("sim");
    let (sdk_name, arch) = match (target_arch, is_simulator) {
        ("aarch64", true) => ("iphonesimulator", "arm64"),
        ("aarch64", false) => ("iphoneos", "arm64"),
        ("x86_64", _) => ("iphonesimulator", "x86_64"),
        _ => {
            warn!("Unsupported iOS architecture: {}", target_arch);
            return;
        }
    };

    // Get SDK path - prefer SDKROOT env var, fall back to xcrun
    let sdk_path = env::var("SDKROOT").unwrap_or_else(|_| {
        std::process::Command::new("xcrun")
            .args(["--sdk", sdk_name, "--show-sdk-path"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    });

    let cflags =
        env::var("CFLAGS").unwrap_or_else(|_| format!("-isysroot {} -arch {}", sdk_path, arch));

    warn!("iOS SDK: {}, CFLAGS: {}", sdk_path, cflags);

    config
        .define("CMAKE_SYSTEM_NAME", "iOS")
        .define("CMAKE_OSX_SYSROOT", &sdk_path)
        .define("CMAKE_OSX_ARCHITECTURES", arch)
        .define("CMAKE_OSX_DEPLOYMENT_TARGET", &deployment_target);

    for flag in cflags.split_whitespace() {
        config.cflag(flag).cxxflag(flag);
    }
}

fn configure_macos(config: &mut Config, target_arch: &str) {
    let (arch, cmake_processor) = match target_arch {
        "aarch64" => ("arm64", "aarch64"),
        "x86_64" => ("x86_64", "x86_64"),
        _ => {
            warn!("Unsupported macOS architecture: {}", target_arch);
            return;
        }
    };

    config
        .define("CMAKE_OSX_ARCHITECTURES", arch)
        .define("CMAKE_SYSTEM_PROCESSOR", cmake_processor);
}

fn configure_windows_arm64(config: &mut Config, host_arch: &str) {
    // Always set ARM64 processor to ensure Opus CMake properly detects ARM architecture
    config.define("CMAKE_SYSTEM_PROCESSOR", "ARM64");

    // Apply cross-compilation settings only when host is x86/x64
    if host_arch == "x86_64" || host_arch == "x86" {
        warn!("Windows ARM64: configuring for ARM64 cross-compilation");
        config.define("CMAKE_SYSTEM_NAME", "Windows");
    } else {
        warn!("Windows ARM64: native ARM64 build");
    }

    // TODO(xnorpx): Revisit this and make pr for Opus
    // MSVC doesn't define __ARM_NEON like GCC/Clang, but Opus's NEON source
    // files check for it. Define it manually since NEON is always available on ARM64.
    config.cflag("/D__ARM_NEON=1");

    // There's a bug in Opus where dnn_arm.h declares DNN_COMPUTE_LINEAR_IMPL extern
    // when OPUS_HAVE_RTCD && OPUS_ARM_MAY_HAVE_NEON, but arm_dnn_map.c only defines it
    // when OPUS_ARM_MAY_HAVE_DOTPROD. This causes unresolved symbol errors.
    //
    // Workaround: Disable RTCD and force the direct NEON function call path by:
    // 1. Disabling MAY_HAVE_NEON (prevents RTCD dispatch table declaration)
    // 2. Manually defining the macros needed for dnn_arm.h to use direct calls
    config
        .define("OPUS_MAY_HAVE_NEON", "OFF")
        .cflag("/DOPUS_ARM_MAY_HAVE_NEON_INTR=1")
        .cflag("/DOPUS_ARM_PRESUME_NEON_INTR=1");
}

fn configure_x86_features(config: &mut Config) {
    if let Ok(target_features) = env::var("CARGO_CFG_TARGET_FEATURE") {
        let features: Vec<&str> = target_features.split(',').map(|s| s.trim()).collect();

        let has_sse41 = features.contains(&"sse4.1");
        let has_avx2 = features.contains(&"avx2");
        let has_fma = features.contains(&"fma");

        if has_sse41 {
            warn!("SSE4.1 detected, enabling OPUS_X86_PRESUME_SSE4_1");
            config.define("OPUS_X86_PRESUME_SSE4_1", "ON");
        }

        if has_avx2 && has_fma {
            warn!("AVX2+FMA detected, enabling OPUS_X86_PRESUME_AVX2");
            config.define("OPUS_X86_PRESUME_AVX2", "ON");
        }
    }
}

fn configure_features(config: &mut Config, target_os: &str, target_arch: &str) {
    // Check Cargo feature flags
    let dnn_enabled = env::var("CARGO_FEATURE_DNN").is_ok();
    let fast_math_enabled = env::var("CARGO_FEATURE_FAST_MATH").is_ok();

    // DRED/OSCE don't work on some platforms
    let is_android_armv7 = target_os == "android" && target_arch == "arm";

    if is_android_armv7 {
        warn!(
            "AI features (DRED/OSCE) not supported on {}-{}",
            target_os, target_arch
        );
        config.define("OPUS_DRED", "OFF").define("OPUS_OSCE", "OFF");
    } else if dnn_enabled {
        warn!("DNN features enabled (DRED + OSCE)");
        config.define("OPUS_DRED", "ON");
        config.define("OPUS_OSCE", "ON");
        // Use runtime weight loading - weight data files are too large to embed (~83MB)
        // Users must call OPUS_SET_DNN_BLOB() to provide weights at runtime
        warn!("DNN weights will be loaded at runtime via OPUS_SET_DNN_BLOB()");
        config.define("OPUS_RUNTIME_WEIGHTS", "ON");
    } else {
        config.define("OPUS_DRED", "OFF");
        config.define("OPUS_OSCE", "OFF");
    }

    // Performance optimizations
    if fast_math_enabled {
        warn!("FAST_MATH feature enabled");
        config
            .define("OPUS_FLOAT_APPROX", "ON")
            .define("OPUS_FAST_MATH", "ON");
    }
}
