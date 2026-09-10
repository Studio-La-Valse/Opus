use clap::{Parser, Subcommand};
use cli::commands::render::{self, RenderCommand};
use cli::commands::validate::{self, ValidateArgs};

/// Engrave and validate MusicXML scores.
#[derive(Parser, Debug)]
#[command(name = "opus", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// The `Render` variant is much the larger of the two, and grows with every
/// render option added. Boxing it -- clippy's suggestion -- is not available
/// here: clap's derive needs a `Subcommand`, which `Box<RenderCommand>` is not.
/// The cost is a few hundred bytes on the stack of one value, parsed once from
/// argv and then consumed, so there is nothing here to save.
#[allow(clippy::large_enum_variant)]
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
