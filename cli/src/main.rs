mod commands;

use clap::{Parser, Subcommand};
use commands::render::{self, RenderCommand};
use commands::validate::{self, ValidateArgs};

/// Engrave and validate MusicXML scores.
#[derive(Parser, Debug)]
#[command(name = "opus", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Engrave a MusicXML file and write it to disk.
    Render {
        #[command(subcommand)]
        format: RenderCommand,
    },
    /// Parse a MusicXML file and report validation issues.
    Validate(ValidateArgs),
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Render { format } => render::run(format),
        Command::Validate(args) => validate::run(args),
    }
}
