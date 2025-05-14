use anyhow::{Result, anyhow};
use serde::Deserialize;
use std::{env, path::PathBuf};
use tokio::process::Command;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SwiftPaths {
    runtime_library_paths: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SwiftTargetInfo {
    paths: SwiftPaths,
}

/// Helper function to run a command from an optional directory and get its stdout as a trimmed String.
/// Returns an `anyhow::Result` indicating success or failure.
async fn run_command(
    program: &str,
    args: &[&str],
    current_dir: Option<&PathBuf>,
) -> Result<String> {
    let mut command = Command::new(program);
    command.args(args);

    if let Some(dir) = current_dir {
        command.current_dir(dir);
    }

    let output = command.output().await?;

    if !output.status.success() {
        return Err(anyhow!(
            "Command failed: {} with args {:?}\nStatus: {}\nStdout: {}\nStderr: {}",
            program,
            args,
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout.trim().to_owned())
}

/// Builds the Swift library and emits cargo instructions.
/// This function is called when the target vendor is "apple".
async fn build_swift_library() -> Result<()> {
    let out_dir = env::var("OUT_DIR")?;
    let profile = env::var("PROFILE")?;
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    let rust_target = env::var("TARGET")?; // Read the full target triple

    // Determine Swift target architecture, SDK name for xcrun,
    // the OS suffix for the swift build triple, and the OS segment
    // for the output path, based on the full Rust target triple.
    let (swift_target_arch, swift_sdk_name, swift_build_triple_os_suffix) =
        match rust_target.as_str() {
            "x86_64-apple-darwin" => ("x86_64", "macosx", "macosx"),
            "aarch64-apple-darwin" => ("arm64", "macosx", "macosx"),
            "aarch64-apple-ios" => ("arm64", "iphoneos", "ios"),
            "aarch64-apple-ios-sim" => ("arm64", "iphonesimulator", "ios-simulator"),
            "x86_64-apple-ios-sim" => ("x86_64", "iphonesimulator", "ios-simulator"),
            "aarch64-apple-watchos" => ("arm64", "watchos", "watchos"),
            "aarch64-apple-watchos-sim" => ("arm64", "watchos", "watchos-simulator"),
            "aarch64-apple-maccatalyst" => ("arm64", "maccatalyst", "maccatalyst"),
            "aarch64-apple-visionos" => ("arm64", "xros", "xros"),
            "aarch64-apple-visionos-sim" => ("arm64", "xros", "xros-simulator"),
            // Add more targets as needed
            _ => return Err(anyhow!("Unsupported target: {}", rust_target)), // Handle unknown targets
        };

    // CARGO_CFG_TARGET_VENDOR should be "apple" here

    // Query xcrun to get the version of the target SDK.
    let target_os_version = run_command(
        "xcrun",
        &["--sdk", swift_sdk_name, "--show-sdk-version"],
        None,
    )
    .await?;

    // Construct the target triple for the 'swift build --triple' command.
    let swift_build_triple = format!(
        "{}-{}-{}",
        swift_target_arch, "apple", swift_build_triple_os_suffix
    );

    // Construct the target triple for the 'swift -target ... -print-target-info' command.
    // This typically includes the OS version.
    let swift_runtime_triple = format!("{swift_build_triple}{target_os_version}");

    // Find Swift runtime libraries using the runtime triple
    let raw_target_info = run_command(
        "swift",
        &["-target", &swift_runtime_triple, "-print-target-info"],
        None,
    )
    .await?;

    let target_info: SwiftTargetInfo = serde_json::from_slice(raw_target_info.as_bytes())?;

    for path in target_info.paths.runtime_library_paths {
        println!("cargo:rustc-link-search=native={path}");
    }

    // Get SDK path for the Swift build command
    let sdk_path =
        run_command("xcrun", &["--sdk", swift_sdk_name, "--show-sdk-path"], None).await?;

    // Attempt to set DYLD_ROOT_PATH for the runtime environment
    println!("cargo:rustc-env=DYLD_ROOT_PATH={sdk_path}");

    // Build the Swift package from the swift-library directory
    // Fix: Use &manifest_dir to avoid moving the String
    let swift_library_dir = PathBuf::from(&manifest_dir).join("swift-library");

    // Start compiling the swift code
    run_command(
        "xcrun",
        &[
            "--sdk",
            swift_sdk_name, // Use the appropriate target SDK (e.g., iphonesimulator)
            "env",
            "-i", // Fairly new bug: https://forums.swift.org/t/swiftpm-bogus-invalid-manifest-error-xcode/78906
            "swift",
            "build",
            "--sdk",
            &sdk_path,
            "--triple",
            &swift_build_triple, // Use the build triple including simulator suffix if applicable
            "--build-path",
            &out_dir,
            "-c",
            &profile,
        ],
        Some(&swift_library_dir), // Pass the working directory
    )
    .await?;

    // The build output path uses a triple format, but for simulators, the directory
    // segment often doesn't include the "-simulator" suffix. Use the determined segment.
    let out_path_triple_segment = format!(
        "{}-{}-{}",
        swift_target_arch,
        "apple",
        swift_build_triple_os_suffix // Changed to use swift_build_triple_os_suffix
    );
    println!("cargo:rustc-link-search=native={out_dir}/{out_path_triple_segment}/{profile}");
    // Link the generated Swift library.
    // The library name is derived from the Swift package product name.
    println!("cargo:rustc-link-lib=static=swift-library");
    // manifest_dir is now available because it wasn't moved
    println!("cargo:rerun-if-changed={manifest_dir}/Sources/**/*.swift");

    Ok(())
}

fn swift_bridge_out_dir() -> Result<PathBuf> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    Ok(PathBuf::from(manifest_dir)
        .join("swift-library/Sources/swift-library")
        .join("generated"))
}

#[tokio::main]
async fn main() -> Result<()> {
    let target_vendor = env::var("CARGO_CFG_TARGET_VENDOR")?;

    // This is required, as somehow cargo is ignoring this
    println!("cargo:rustc-env=DYLD_FALLBACK_LIBRARY_PATH=/usr/lib/swift");

    if target_vendor == "apple" {
        // 1. Use `swift-bridge-build` to generate Swift/C FFI glue.
        swift_bridge_build::parse_bridges(["src/lib.rs"])
            .write_all_concatenated(swift_bridge_out_dir()?, env!("CARGO_PKG_NAME"));
        // 2. Build the swift library
        build_swift_library().await?;
    }

    Ok(())
}
