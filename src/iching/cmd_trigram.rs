use crate::error::Error;
use crate::iching::{hexagrams_with_lower, hexagrams_with_upper, lookup_trigram, TrigramRecord};
use crate::output::{emit, OutputFormat};
use std::io::{self, Write};

#[derive(clap::Args)]
pub struct Args {
    /// Trigram identifier: English name, pinyin, Unicode symbol, or binary (e.g. heaven, qián, ☰, 111)
    pub name: String,
}

pub fn run(args: Args, output: OutputFormat) -> Result<(), Error> {
    let trigram = lookup_trigram(&args.name).ok_or_else(|| {
        Error::Usage(format!(
            "trigram '{}' not found — try: heaven, qián, ☰, or 111",
            args.name
        ))
    })?;

    emit(trigram, output, |t| {
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        render_trigram(&mut out, t)
    })
    .map_err(Error::from)
}

/// Render a trigram in the established pretty format. Shared with the
/// compose subcommand.
pub(super) fn render_trigram(out: &mut impl Write, t: &TrigramRecord) -> io::Result<()> {
    writeln!(
        out,
        "{}  {} — {} ({})",
        t.symbol, t.pinyin, t.english, t.quality
    )?;
    writeln!(out, "   Family: {}", t.family)?;
    writeln!(out, "   Binary: {}", t.binary)?;
    writeln!(out)?;

    let upper = hexagrams_with_upper(&t.binary);
    let lower = hexagrams_with_lower(&t.binary);

    let fmt_list = |v: &[u8]| -> String {
        v.iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    };

    writeln!(out, "   Upper trigram in: {}", fmt_list(&upper))?;
    writeln!(out, "   Lower trigram in: {}", fmt_list(&lower))?;

    Ok(())
}
