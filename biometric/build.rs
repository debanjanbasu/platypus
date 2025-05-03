use anyhow::{Result, anyhow};
use autocxx_build::Builder;
use serde::Deserialize;
use std::env;
use std::path::PathBuf;
use std::process::Command;

#[tokio::main]
async fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=swift-library/swift-library.h");
    println!("cargo:rerun-if-changed=swift-library/Sources/swift-library/swift_library.swift");

    // Compile the Swift package
    // link_swift_package("swift-library","./swift-library/")?;
    
    // Get this from swift
    // let target = get_swift_target_info();
    
    let include_path_swift = PathBuf::from("swift-library");
    let include_path_rust = PathBuf::from("src");
    
    // This assumes all your C++ bindings are in lib.rs
    let mut b = Builder::new("src/lib.rs", [&include_path_rust, &include_path_swift]).build()?;
    b.flag_if_supported("-std=c++23").compile("biometric"); // arbitrary library name, pick anything
    
    // Add instructions to link to any C++ libraries you need.
    // link_swift()?;
    println!("cargo:rustc-link-lib=static=swift-library");
    println!("cargo:rustc-link-search=swift-library/.build");

    Ok(())
}

fn link_swift() -> Result<()> {
    let swift_target_info = get_swift_target_info();

    swift_target_info?
        .paths
        .runtime_library_paths
        .iter()
        .for_each(|path| {
            println!("cargo:rustc-link-search=native={path}");
        });
    Ok(())
}

fn link_swift_package(package_name: &str, package_root: &str) -> Result<()> {
    let profile = env::var("PROFILE")?;

    if !Command::new("swift")
        .args(["build", "-c", &profile])
        .current_dir(package_root)
        .status()?
        .success()
    {
        return Err(anyhow!("Failed to compile swift package {package_name}"));
    }

    let swift_target_info = get_swift_target_info();

    println!(
        "cargo:rustc-link-search=native={}.build/{}/{}",
        package_root, swift_target_info?.target.unversioned_triple, profile
    );
    println!("cargo:rustc-link-lib=static={package_name}");
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwiftTargetInfo {
    pub unversioned_triple: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwiftPaths {
    pub runtime_library_paths: Vec<String>,
    pub runtime_resource_path: String,
}

#[derive(Debug, Deserialize)]
pub struct SwiftTarget {
    pub target: SwiftTargetInfo,
    pub paths: SwiftPaths,
}

fn get_swift_target_info() -> Result<SwiftTarget> {
    let swift_target_info_str = Command::new("swift")
        .args(["-print-target-info"])
        .output()?
        .stdout;
    Ok(serde_json::from_slice(&swift_target_info_str)?)
}
