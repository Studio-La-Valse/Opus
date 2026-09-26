//! `opus font`: managing the SMuFL fonts `render` can engrave in.
//!
//! Kept apart from every other command. What it installs is only ever read
//! back through [`smufl_fonts`](crate::smufl_fonts), so it can move -- into an
//! installer, say -- without `render` noticing.

use clap::Subcommand;

pub mod install;
pub mod list;

#[derive(Subcommand, Debug)]
pub enum FontCommand {
    /// Install a SMuFL font from a folder on disk: its metadata where the SMuFL
    /// specification says to look for it, and its OpenType files as per-user
    /// fonts.
    Install(install::InstallArgs),
    /// List the SMuFL fonts whose metadata is installed, and where.
    List,
}

pub fn run(command: FontCommand) {
    match command {
        FontCommand::Install(args) => install::run(args),
        FontCommand::List => list::run(),
    }
}
