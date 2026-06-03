use crate::error::Error;
use crate::output::OutputFormat;

#[derive(clap::Args)]
pub struct Args {
    #[command(subcommand)]
    pub subcommand: Subcommand,
}

#[derive(clap::Subcommand)]
pub enum Subcommand {
    /// Cast the I Ching. Optionally provide a question; omit for an open casting.
    Cast(super::cmd_cast::Args),
    /// Resolve a compose expression to a trigram, hexagram, or transition.
    Compose(super::cmd_compose::Args),
    /// Look up a hexagram by number (1–64).
    Hexagram(super::cmd_hexagram::Args),
    /// Look up a trigram by name, pinyin, symbol, or binary (e.g. heaven, qián, ☰, 111).
    Trigram(super::cmd_trigram::Args),
}

pub fn run(args: Args, output: OutputFormat) -> Result<(), Error> {
    match args.subcommand {
        Subcommand::Cast(a) => super::cmd_cast::run(a, output),
        Subcommand::Compose(a) => super::cmd_compose::run(a, output),
        Subcommand::Hexagram(a) => super::cmd_hexagram::run(a, output),
        Subcommand::Trigram(a) => super::cmd_trigram::run(a, output),
    }
}
