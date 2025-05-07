use anyhow::Result;
use std::{env, path::PathBuf};
use tokio::process::Command;

#[tokio::main]
async fn main() -> Result<()> {
    // Rerun conditions - whole directory
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=swift-library");
    println!("cargo:rerun-if-changed=build.rs"); // Rerun if build script changes

    // 1. Use `swift-bridge-build` to generate Swift/C FFI glue.
    //    You can also use the `swift-bridge` CLI.
    swift_bridge_build::parse_bridges(["src/lib.rs"])
        .write_all_concatenated(swift_bridge_out_dir()?, "biometric");

    // 2. Compile Swift library
    compile_swift().await?;

    // 3. Link to Swift library
    println!("cargo:rustc-link-lib=static=swift-library");
    println!(
        "cargo:rustc-link-search={}",
        &swift_library_static_lib_dir()?.display()
    );

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
        let xcode_path = if let Ok(output) = std::process::Command::new("xcode-select")
            .arg("--print-path")
            .output()
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

    // This fix is for Linux only
    #[cfg(target_os = "linux")]
    {
        // We need to tell cargo which additional libraries to link to!
        //
        // This is required because swift build -Xswiftc -static-stdlib works for executables,
        // but not for libraries (yet). Thus, when trying to link against the produced .a file,
        // not all symbols can be resolved. The undefined symbols can easily be found by running
        // `nm -u .build/debug/libswift-library.a`, usually things such as `swift_retain`
        // and `swift_release` will be missing.
        // Cargo will give you an error message like this if symbols are missing:
        // note: /usr/bin/ld: .build/debug/libswift-library.a(swift_library.swift.o): in function `$ss27_finalizeUninitializedArrayySayxGABnlF':
        //       <compiler-generated>:(.text+0x17): undefined reference to `$sSaMa'
        // or:                                      undefined reference to `swift_release'
        //
        // Thus, we need to explicitly link against the Swift libraries which are required.
        // Unfortunately, the required linker flags depend on the Swift version and the used modules,
        // so they might be different for your project.
        let swift_lib_path = std::env::var("SWIFT_LIBRARY_PATH")
            .unwrap_or_else(|_| "/usr/lib/swift/linux".to_string());

        if !std::path::Path::new(&swift_lib_path).exists() {
            panic!(
                "Swift library path not found at /usr/lib/swift/linux and SWIFT_LIBRARY_PATH environment variable not set"
            );
        }

        println!("cargo:rustc-link-search={}", swift_lib_path);

        // These swift libraries are needed to get all the missing symbols to properly
        // link the Swift library. This is required for `cargo run` as well as `cargo test`.
        println!("cargo:rustc-link-lib=swiftCore");
        println!("cargo:rustc-link-lib=stdc++");
        println!("cargo:rustc-link-lib=swiftSwiftOnoneSupport");
    }

    Ok(())
}

async fn compile_swift() -> Result<()> {
    let swift_package_dir = manifest_dir()?.join("swift-library");

    let mut cmd = Command::new("swift");

    cmd.current_dir(swift_package_dir).arg("build").args([
        "-Xswiftc",
        "-import-objc-header",
        "-Xswiftc",
        &swift_source_dir()?
            .join("bridging-header.h")
            .display()
            .to_string(), // Use display().to_string() to avoid unwrap on Option
    ]);

    if is_release_build() {
        cmd.args(["-c", "release"]); // Removed needless borrow
    }

    let child = cmd.spawn()?;

    let exit_status = child.wait_with_output().await?;

    assert!(
        exit_status.status.success(),
        r"
Stderr: {}
Stdout: {}
",
        String::from_utf8_lossy(&exit_status.stderr).into_owned(), // Use lossy conversion to avoid unwrap on Result
        String::from_utf8_lossy(&exit_status.stdout).into_owned(), // Use lossy conversion to avoid unwrap on Result
    );

    Ok(())
}

fn swift_bridge_out_dir() -> Result<PathBuf> {
    generated_code_dir()
}

fn manifest_dir() -> Result<PathBuf> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    Ok(PathBuf::from(manifest_dir))
}

fn is_release_build() -> bool {
    std::env::var("PROFILE").unwrap_or_default() == "release"
}

fn swift_source_dir() -> Result<PathBuf> {
    Ok(manifest_dir()?.join("swift-library/Sources/swift-library"))
}

fn generated_code_dir() -> Result<PathBuf> {
    Ok(swift_source_dir()?.join("generated"))
}

fn swift_library_static_lib_dir() -> Result<PathBuf> {
    let debug_or_release = if is_release_build() {
        "release"
    } else {
        "debug"
    };

    Ok(manifest_dir()?.join(format!("swift-library/.build/{debug_or_release}")))
}
