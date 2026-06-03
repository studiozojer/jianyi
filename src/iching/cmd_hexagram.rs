use crate::error::Error;
use crate::iching::{lines_from_binary, lookup_hexagram, render_lines, HexagramRecord};
use crate::output::{emit, OutputFormat};
use std::io::{self, Write};

#[derive(clap::Args)]
pub struct Args {
    /// Hexagram number (1–64)
    pub number: u8,
    /// Show all six line texts in addition to judgment and image
    #[arg(long)]
    pub full: bool,
    /// Show only the ASCII glyph render and name
    #[arg(long, conflicts_with = "full")]
    pub glyph: bool,
}

pub fn run(args: Args, output: OutputFormat) -> Result<(), Error> {
    let hex = lookup_hexagram(args.number).ok_or_else(|| {
        Error::Usage(format!(
            "hexagram {} not found — valid range is 1–64",
            args.number
        ))
    })?;

    if args.glyph {
        if output == OutputFormat::Json {
            return Err(Error::Usage(
                "--glyph is not compatible with --json".to_string(),
            ));
        }
        let render = render_lines(&lines_from_binary(&hex.binary));
        let mut stdout = std::io::stdout().lock();
        write!(stdout, "{render}")?;
        return Ok(());
    }

    emit(hex, output, |h| {
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        render_hexagram(&mut out, h, args.full)
    })
    .map_err(Error::from)
}

pub(super) fn render_hexagram(
    out: &mut impl Write,
    h: &HexagramRecord,
    full: bool,
) -> io::Result<()> {
    let upper = crate::iching::data::lookup_trigram_by_binary(&h.upper_trigram)
        .map(|t| format!("{} ({})", t.english, t.quality))
        .unwrap_or_default();
    let lower = crate::iching::data::lookup_trigram_by_binary(&h.lower_trigram)
        .map(|t| format!("{} ({})", t.english, t.quality))
        .unwrap_or_default();

    writeln!(
        out,
        "{}. {} {} — {}",
        h.number, h.chinese, h.pinyin, h.english
    )?;
    writeln!(out, "{upper} over {lower}")?;
    writeln!(out)?;
    write!(out, "{}", render_lines(&lines_from_binary(&h.binary)))?;
    writeln!(out)?;

    writeln!(out, "Judgment")?;
    writeln!(out, "{}", h.judgment.chinese)?;
    writeln!(out, "{}", h.judgment.english)?;
    if let Some(r) = &h.hexagram_reading {
        writeln!(out)?;
        writeln!(out, "Reading")?;
        writeln!(out, "{r}")?;
    }
    writeln!(out)?;
    writeln!(out, "Image")?;
    writeln!(out, "{}", h.image.english)?;

    if full {
        writeln!(out)?;
        writeln!(out, "Lines")?;
        for line in &h.lines {
            writeln!(out, "  {}. {}", line.position, line.english)?;
            if let Some(r) = &line.reading {
                writeln!(out, "     {r}")?;
            }
        }
    }
    Ok(())
}
