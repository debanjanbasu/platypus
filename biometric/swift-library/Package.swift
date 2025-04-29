// swift-tools-version: 6.1
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "swift-library",
    products: [
        // Products define the executables and libraries a package produces, making them visible to other packages.
        .library(
            name: "swift-library",
            type: .static,
            targets: ["swift-library"])
    ],
    //dependencies: [
    //    // Dependencies declare other packages that this package depends on.
    //    .package(
    //        name: "swift-bridge-generated",
    //        path: "./Sources/swift-library/generated/SwiftBridgeCore.swift")
    //],
    targets: [
        // Targets are the basic building blocks of a package, defining a module or a test suite.
        // Targets can depend on other targets in this package and products from dependencies.
        .target(
            name: "swift-library"),
        .testTarget(
            name: "swift-libraryTests",
            dependencies: ["swift-library"]
        ),
    ],
)
