# Android app

Boundary reserved for the Android app (Kotlin vs. Java still open). Not
started yet.

Will consume the `ffi` crate (`../../ffi`) as compiled `.so` files per ABI
plus generated Kotlin bindings (via
[uniffi](https://mozilla.github.io/uniffi-rs/)), staged into this directory
by a future `scripts/build-android.sh`, the same way `scripts/stage-package.sh`
stages the web component's npm package today. This directory holds the
Gradle project and UI code only - no Rust.
