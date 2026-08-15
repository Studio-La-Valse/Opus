//! Shared native binding boundary for the iOS and Android apps under
//! `platforms/`, mirroring what `wasm/` is for `web/`: a thin shell around
//! `lib` that does no rendering itself, just exposes `lib`'s flat-buffer
//! render output across the FFI boundary.
//!
//! Not wired up yet - no uniffi dependency or generated bindings live here
//! until the mobile app work actually starts. This crate exists so the
//! workspace boundary is in place ahead of that.
