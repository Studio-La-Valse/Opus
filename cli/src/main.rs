mod commands;

use clap::{Parser, Subcommand};
use commands::render::{self, RenderArgs};
use commands::validate::{self, ValidateArgs};

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Render(RenderArgs),
    Validate(ValidateArgs),
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Render(args) => render::run(args),
        Command::Validate(args) => validate::run(args),
    }
}
