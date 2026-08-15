# iOS app

Boundary reserved for a SwiftUI app. Not started yet.

Will consume the `ffi` crate (`../../ffi`) as a compiled XCFramework plus
generated Swift bindings (via [uniffi](https://mozilla.github.io/uniffi-rs/)),
staged into this directory by a future `scripts/build-ios.sh`, the same way
`scripts/stage-package.sh` stages the web component's npm package today. This
directory holds the Xcode project and Swift UI code only - no Rust.
