// swift-tools-version:4.0
import PackageDescription

let package = Package(
    name: "SpecModel",
    products: [
        .library(name: "SpecModel", targets: ["SpecModel"]),
    ],
    dependencies: [
    ],
    targets: [
        .target(
            name: "SpecModel",
            dependencies: []),
        .testTarget(
            name: "SpecModelTests",
            dependencies: ["SpecModel"]),
    ]
)
