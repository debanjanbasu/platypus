// swift-tools-version: 6.1
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "swift-library",
    platforms: [
        .iOS(.v12),
        .macOS(.v10_13),
        .watchOS(.v7),
        .macCatalyst(.v13),
        .visionOS(.v1),
    ],
    products: [
        .library(name: "cxxLibrary", targets: ["cxxLibrary"]),
        // Products define the executables and libraries a package produces, making them visible to other packages.
        .library(
            name: "swift-library",
            type: .static,
            targets: ["swift-library"]),
    ],
    targets: [
        .target(
            name: "cxxLibrary",
            cxxSettings: [
                .unsafeFlags([
                    "-I", "../target/",
                ])
            ]
        ),
        // Targets are the basic building blocks of a package, defining a module or a test suite.
        // Targets can depend on other targets in this package and products from dependencies.
        .target(
            name: "swift-library",
            dependencies: ["cxxLibrary"],
            swiftSettings: [
                .unsafeFlags([
                    "-module-name", "SwiftLibrary",
                    "-cxx-interoperability-mode=default",
                    "-emit-clang-header-path", "swift-library.h",
                    "-Xcc", "-std=c++23",
                    // For better performance - we really don't need for Swift to check for runtime exclusivity
                    "-enforce-exclusivity=none",
                ]),
                .interoperabilityMode(.Cxx),
            ],
            linkerSettings: [
                .unsafeFlags([
                    "-Xlinker", "-sectcreate",
                    "-Xlinker", "__TEXT",
                    "-Xlinker", "__info_plist",
                    "-Xlinker", "Resources/Info.plist",
                ])
            ]),
    ],
)
