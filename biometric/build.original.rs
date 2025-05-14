use anyhow::{Result, anyhow};
use serde::Deserialize;
use std::{env, path::PathBuf};
use tokio::process::Command;

#[tokio::main]
async fn main() -> Result<()> {
    // Rerun conditions
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=swift-library");
    println!("cargo:rerun-if-changed=build.rs"); // Rerun if build script changes

    // This is required, as somehow cargo tests are ignoring this
    println!("cargo:rustc-env=DYLD_FALLBACK_LIBRARY_PATH=/usr/lib/swift");

    // 0. Get Swift target info once, as it's needed by multiple steps.
    let swift_target_info = get_swift_target_info().await?;

    // 1. Use `swift-bridge-build` to generate Swift/C FFI glue.
    swift_bridge_build::parse_bridges(["src/lib.rs"])
        .write_all_concatenated(swift_bridge_out_dir()?, env!("CARGO_PKG_NAME"));

    // 2. Compile Swift library
    link_swift_package("swift-library", "swift-library/", &swift_target_info).await?;

    // 3. Link Swift runtime libraries
    link_swift(&swift_target_info)?;

    // 4. Special linker items for various OS
    // This fix is for macOS only
    #[cfg(target_os = "macos")]
    {
        // Without this we will get warnings about not being able to find dynamic libraries, and then
        // we won't be able to compile since the Swift static libraries depend on them:
        // For example:
        // ld: warning: Could not find or use auto-linked library 'swiftCompatibility51'
        // ld: warning: Could not find or use auto-linked library 'swiftCompatibility50'
        // ld: warning: Could not find or use auto-linked library 'swiftCompatibilityDynamicReplacements'
        // ld: warning: Could not find or use auto-linked library 'swiftCompatibilityConcurrency'
        let xcode_path = if let Ok(output) = Command::new("xcode-select")
            .arg("--print-path")
            .output()
            .await
        {
            String::from_utf8(output.stdout.as_slice().into())?
                .trim()
                .to_string()
        } else {
            "/Applications/Xcode.app/Contents/Developer".to_string()
        };
        println!(
            "cargo:rustc-link-search={}/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx/",
            &xcode_path
        );
        println!("cargo:rustc-link-search=/usr/lib/swift");
    }

    Ok(())
}

fn manifest_dir() -> Result<PathBuf> {
    Ok(PathBuf::from(env::var("CARGO_MANIFEST_DIR")?))
}

fn swift_bridge_out_dir() -> Result<PathBuf> {
    Ok(manifest_dir()?
        .join("swift-library/Sources/swift-library")
        .join("generated"))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SwiftTargetInfo {
    unversioned_triple: String,
    #[serde(rename = "librariesRequireRPath")]
    libraries_require_rpath: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SwiftPaths {
    runtime_library_paths: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SwiftTarget {
    target: SwiftTargetInfo,
    paths: SwiftPaths,
}

/// Retrieves Swift target information by executing `swift -print-target-info`.
///
/// This function spawns a `swift` process with the `-print-target-info` argument,
/// captures its standard output, and then attempts to parse this output as JSON
/// into a `SwiftTarget` struct.
///
/// # Errors
///
/// This function will return an error in the following situations:
/// - If the `swift` command cannot be executed (e.g., it's not found in the system's PATH,
///   or there are permission issues).
/// - If the `swift -print-target-info` command executes but returns a non-zero exit code.
/// - If the standard output of the `swift` command is not valid UTF-8.
/// - If the standard output of the `swift` command is not valid JSON, or if it cannot be
///   deserialized into the `SwiftTarget` struct.
async fn get_swift_target_info() -> Result<SwiftTarget> {
    let output = Command::new("swift")
        .args(["-print-target-info"])
        .output()
        .await?;

    if !output.status.success() {
        return Err(anyhow!(
            "Failed to execute 'swift -print-target-info'. Exit status: {}. Stderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    serde_json::from_slice(&output.stdout).map_err(|e| {
        anyhow!(
            "Failed to parse JSON from 'swift -print-target-info': Raw output: {}\nError: {}",
            String::from_utf8_lossy(&output.stdout),
            e
        )
    })
}

/// Links the Swift runtime libraries.
///
/// This function uses pre-fetched Swift target information and, if the target does not
/// require `RPath` for its libraries, it instructs cargo to search for Swift runtime
/// libraries in the paths specified by `swift -print-target-info`.
///
/// # Arguments
///
/// * `swift_target_info`: Pre-fetched Swift target information.
///
/// # Errors
///
/// This function will return an error if `swift_target_info.target.libraries_require_rpath`
/// is true, indicating that the Swift libraries require `RPath`. This typically means the
/// deployment target (e.g., minimum macOS version) needs to be adjusted.
fn link_swift(swift_target_info: &SwiftTarget) -> Result<()> {
    if swift_target_info.target.libraries_require_rpath {
        return Err(anyhow!(
            "Libraries require RPath! Change minimum MacOS value to fix (e.g., in Package.swift or project settings)."
        ));
    }

    for path in &swift_target_info.paths.runtime_library_paths {
        println!("cargo:rustc-link-search=native={path}");
    }
    Ok(())
}

/// Compiles a Swift package and links it to the Rust project.
///
/// This function performs the following steps:
/// 1. Determines the build profile (e.g., "debug" or "release") from the `PROFILE` environment variable.
/// 2. Executes `swift build` for the specified package and profile.
/// 3. Uses pre-fetched Swift target information to construct the path to the compiled Swift static library.
/// 4. Instructs cargo to link against this static library and search in its directory.
///
/// # Arguments
///
/// * `package_name`: The name of the Swift package (e.g., "`MySwiftLib`"). This is used to form the library name `lib<package_name>.a`.
/// * `package_root`: The path to the root directory of the Swift package, relative to the manifest directory.
/// * `swift_target_info`: Pre-fetched Swift target information.
///
/// # Errors
///
/// This function will return an error in the following situations:
/// - If the `PROFILE` environment variable is not set.
/// - If the `swift build` command fails to execute (e.g., `swift` not found, permission issues).
/// - If the `swift build` command executes but returns a non-zero exit code (compilation failure).
async fn link_swift_package(
    package_name: &str,
    package_root: &str,
    swift_target_info: &SwiftTarget,
) -> Result<()> {
    let profile = env::var("PROFILE")?;

    let output = Command::new("swift")
        .args(["build", "-c", &profile])
        .current_dir(manifest_dir()?.join(package_root)) // Ensure current_dir is relative to manifest
        .output()
        .await?;

    if !output.status.success() {
        return Err(anyhow!(
            "Failed to compile Swift package '{}' in directory '{}'.\n\
             Command: `swift build -c {}` (run in {})\n\
             Status: {}\n\
             Stdout:\n{}\n\
             Stderr:\n{}",
            package_name,
            package_root,
            profile,
            manifest_dir()?.join(package_root).display(),
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Construct the path to the directory containing the compiled Swift static library.
    // This path is relative to the package_root.
    // e.g., <package_root>/.build/<target_triple>/<profile>/
    let lib_search_subdir = PathBuf::from(".build")
        .join(&swift_target_info.target.unversioned_triple)
        .join(&profile);

    // The full path for cargo to search is manifest_dir / package_root / lib_search_subdir
    let full_lib_search_path = manifest_dir()?.join(package_root).join(lib_search_subdir);

    println!(
        "cargo:rustc-link-search=native={}",
        full_lib_search_path.display()
    );
    println!("cargo:rustc-link-lib=static={package_name}");

    Ok(())
}
