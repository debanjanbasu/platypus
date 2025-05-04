use anyhow::{Result, anyhow};
use autocxx_build::Builder;
use serde::Deserialize;
use std::env;
use std::path::PathBuf;
use tokio::process::Command;

#[tokio::main]
async fn main() -> Result<()> {
    // Rerun conditions
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=swift-library/swift-library.h");
    println!("cargo:rerun-if-changed=swift-library/Sources/swift-library/swift_library.swift");
    println!("cargo:rerun-if-changed=build.rs"); // Rerun if build script changes

    // --- Autocxx build for C++/Rust bindings ---
    let include_path_swift = PathBuf::from("swift-library");
    let include_path_rust = PathBuf::from("src");

    Builder::new("src/lib.rs", [&include_path_rust, &include_path_swift])
        .build()?
        .flag_if_supported("-std=c++23")
        .compile("biometric"); // Arbitrary library name

    // --- Swift Integration ---
    // Add Swift runtime library search paths
    link_swift_runtime_libs().await?;

    // Compile the Swift package and link the static library
    build_and_link_swift_package("swift-library", "swift-library").await?;

    Ok(())
}

/// Adds the necessary Swift runtime library paths to the linker search paths.
async fn link_swift_runtime_libs() -> Result<()> {
    let swift_target_info = get_swift_target_info().await?;
    for path in swift_target_info.paths.runtime_library_paths {
        println!("cargo:rustc-link-search=native={path}");
    }
    Ok(())
}

/// Compiles the specified Swift package and links the resulting static library.
async fn build_and_link_swift_package(package_name: &str, package_root: &str) -> Result<()> {
    let profile = env::var("PROFILE").unwrap_or_else(|_| "release".to_string());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    // Calculate the expected location of the built Swift static library
    // Note: Assumes the standard SwiftPM build directory structure.
    let swift_build_dir = manifest_dir
        .join(package_root)
        .join(".build")
        .join(&profile);

    // Compile the Swift library using Swift Package Manager
    Command::new("swift")
        .args(["build", "--product", package_name, "-c", &profile])
        .current_dir(package_root)
        .status()
        .await?;

    // Add the directory containing the compiled Swift static library to the linker search path
    println!("cargo:rustc-link-search={}", swift_build_dir.display());
    // Link the static library (lib<package_name>.a)
    println!("cargo:rustc-link-lib=static={package_name}");

    Ok(())
}

// --- Swift Target Information Structs (for JSON parsing) ---

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SwiftPaths {
    runtime_library_paths: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SwiftTarget {
    paths: SwiftPaths,
}

/// Executes `swift -print-target-info` and parses the JSON output.
async fn get_swift_target_info() -> Result<SwiftTarget> {
    let output = Command::new("swift")
        .args(["-print-target-info"])
        .output()
        .await?;

    serde_json::from_slice(&output.stdout).map_err(|e| {
        anyhow!(
            "Failed to parse swift target info JSON: {e}\nOutput:\n{}",
            String::from_utf8_lossy(&output.stdout)
        )
    })
}
