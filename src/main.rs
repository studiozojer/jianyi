use std::process::ExitCode;

use clap::{Parser, Subcommand};

use jianyi::output::OutputFormat;
use jianyi::{iching, tarot};

#[derive(Parser)]
#[command(
    name = "jianyi",
    version,
    about = "A portable, deterministic, offline I Ching and tarot calculator."
)]
struct Cli {
    /// Emit JSON instead of pretty output. Stable schema; suitable for scripts.
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// I Ching — cast, hexagram lookup, trigram lookup, compose.
    Iching(iching::cli::Args),
    /// Tarot — new deck, shuffle, draw, inspect.
    Tarot(tarot::cli::Args),
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let output = if cli.json {
        OutputFormat::Json
    } else {
        OutputFormat::Pretty
    };

    let result = match cli.command {
        Command::Iching(args) => iching::cli::run(args, output),
        Command::Tarot(args) => tarot::cli::run(args, output),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error ({}): {e}", e.kind());
            ExitCode::from(e.exit_code())
        }
    }
}
