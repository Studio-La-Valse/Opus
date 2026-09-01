mod commands;

use clap::{Parser, Subcommand};
use commands::render::{self, RenderCommand};
use commands::validate::{self, ValidateArgs};

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Engrave a MusicXML file and write it to disk.
    Render {
        #[command(subcommand)]
        command: RenderCommand,
    },
    Validate(ValidateArgs),
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Render { command } => render::run(command),
        Command::Validate(args) => validate::run(args),
    }
}
