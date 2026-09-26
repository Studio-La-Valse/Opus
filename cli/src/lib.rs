//! Library face of the `opus` CLI, exposing its command implementations so the
//! workspace `tests` crate can exercise them. The `opus` binary (`src/main.rs`)
//! is a thin `clap` front end over this module.

pub mod commands;
pub mod smufl_fonts;
