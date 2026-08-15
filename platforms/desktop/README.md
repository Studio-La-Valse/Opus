# Desktop app

Boundary reserved for a native desktop app. Not started yet - the GUI
framework choice is still open, leaning towards
[iced](https://iced.rs/) (a native, non-webview toolkit) over Tauri
(webview-backed, would reuse `web/`+`wasm/` almost as-is).

Once decided, this becomes a normal Cargo workspace member (add
`platforms/desktop` to the root `Cargo.toml`'s `members`) depending on `lib`
directly - no `ffi` crate needed, since desktop stays in Rust end to end.
