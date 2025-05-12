// swift-tools-version: 6.1
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "swift-library",
    platforms: [
        .iOS(.v12),
        .macOS(.v10_15),  // Had to be upgraded to use Task, and LAContext
        .watchOS(.v7),
        .macCatalyst(.v13),
        .visionOS(.v1),
    ],
    products: [
        // Products define the executables and libraries a package produces, making them visible to other packages.
        .library(
            name: "swift-library",
            type: .static,
            targets: ["swift-library"])
    ],
    targets: [
        // Targets are the basic building blocks of a package, defining a module or a test suite.
        // Targets can depend on other targets in this package and products from dependencies.
        .target(
            name: "swift-library",
            swiftSettings: [
                .swiftLanguageMode(.v6),
                .interoperabilityMode(.C),
                .unsafeFlags([
                    // For better performance - we really don't need for Swift to check for runtime exclusivity
                    "-enforce-exclusivity=none",
                    "-import-objc-header",
                    "Sources/swift-library/bridging-header.h",
                ]),
            ],
            linkerSettings: [
                .unsafeFlags([
                    "-Xlinker", "-sectcreate",
                    "-Xlinker", "__TEXT",
                    "-Xlinker", "__info_plist",
                    "-Xlinker", "Resources/Info.plist",
                ])
            ])
    ],
)
