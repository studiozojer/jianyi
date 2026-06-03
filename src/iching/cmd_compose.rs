use crate::error::Error;
use crate::iching::data::lookup_trigram_by_binary;
use crate::iching::{
    lines_from_binary, moving_lines, parse_expr, render_lines, Expr, HexagramRecord, LineValue,
    TrigramRecord,
};
use crate::output::OutputFormat;
use std::io::{self, Write};

#[derive(serde::Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum ComposeOutput<'a> {
    Trigram {
        trigram: &'a TrigramRecord,
    },
    Hexagram {
        hexagram: &'a HexagramRecord,
    },
    Transition {
        source: &'a HexagramRecord,
        destination: &'a HexagramRecord,
        moving_lines: Vec<u8>,
        moving_line_texts: Vec<MovingLineText>,
    },
}

#[derive(serde::Serialize)]
struct MovingLineText {
    position: u8,
    text: String,
}

#[derive(clap::Args)]
pub struct Args {
    /// Expression: trigram, hexagram number, composition ("heaven over thunder"),
    /// or transition ("44 -> 26"). Quote it — `>` and `->` are shell-active.
    pub expr: String,

    /// Show all six line texts on hexagram-shaped output / both hexagrams' full
    /// judgment + image + line texts on transition output.
    #[arg(long)]
    pub full: bool,

    /// Show only the ASCII glyph render (no text). For transitions, both
    /// glyphs side-by-side.
    #[arg(long, conflicts_with = "full")]
    pub glyph: bool,
}

pub fn run(args: Args, output: OutputFormat) -> Result<(), Error> {
    let expr = parse_expr(&args.expr).map_err(|e| Error::Usage(e.to_string()))?;

    if output == OutputFormat::Json {
        let payload = match &expr {
            Expr::Trigram(t) => ComposeOutput::Trigram { trigram: t },
            Expr::Hexagram(h) => ComposeOutput::Hexagram { hexagram: h },
            Expr::Transition {
                source,
                destination,
            } => {
                let movers = moving_lines(source, destination);
                let texts =
                    movers
                        .iter()
                        .filter_map(|pos| {
                            source.lines.iter().find(|l| l.position == *pos).map(|l| {
                                MovingLineText {
                                    position: l.position,
                                    text: l.english.clone(),
                                }
                            })
                        })
                        .collect();
                ComposeOutput::Transition {
                    source,
                    destination,
                    moving_lines: movers,
                    moving_line_texts: texts,
                }
            }
        };
        crate::output::emit(&payload, output, |_| Ok(()))?;
        return Ok(());
    }

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    match expr {
        Expr::Trigram(t) => {
            // --full and --glyph are silently ignored for trigram output.
            crate::iching::cmd_trigram::render_trigram(&mut out, t)?;
        }
        Expr::Hexagram(h) => {
            if args.glyph {
                let lines = lines_from_binary(&h.binary);
                let render = render_lines(&lines);
                write!(&mut out, "{}", render)?;
                return Ok(());
            }
            crate::iching::cmd_hexagram::render_hexagram(&mut out, h, args.full)?;
        }
        Expr::Transition {
            source,
            destination,
        } => {
            render_transition(&mut out, source, destination, args.full, args.glyph)?;
        }
    }

    Ok(())
}

fn render_transition(
    out: &mut impl Write,
    source: &HexagramRecord,
    destination: &HexagramRecord,
    full: bool,
    glyph_only: bool,
) -> io::Result<()> {
    let movers = moving_lines(source, destination);

    // Build source line values with OldYang/OldYin at moving positions so
    // render_lines produces the proper dual-column primary→resulting glyph
    // with changing-line markers (○ for old yang, × for old yin).
    let src_bytes = source.binary.as_bytes();
    let lines: [LineValue; 6] = std::array::from_fn(|i| {
        let line_pos = (i + 1) as u8;
        let is_changing = movers.contains(&line_pos);
        let is_yang = src_bytes[i] == b'1';
        match (is_yang, is_changing) {
            (true, true) => LineValue::OldYang,
            (true, false) => LineValue::YoungYang,
            (false, true) => LineValue::OldYin,
            (false, false) => LineValue::YoungYin,
        }
    });

    let glyph = render_lines(&lines);

    if glyph_only {
        write!(out, "{}", glyph)?;
        return Ok(());
    }

    // The glyph from render_lines already includes both hexagrams' labels.
    write!(out, "{}", glyph)?;
    writeln!(out)?;

    let trigram_phrase = |binary: &str| -> String {
        lookup_trigram_by_binary(binary)
            .map(|t| format!("{} ({})", t.english, t.quality))
            .unwrap_or_default()
    };

    // Source: composition + judgment + image
    writeln!(out, "── Source ──")?;
    writeln!(
        out,
        "{} over {}",
        trigram_phrase(&source.upper_trigram),
        trigram_phrase(&source.lower_trigram)
    )?;
    writeln!(out)?;
    writeln!(out, "Judgment")?;
    writeln!(out, "{}", source.judgment.english)?;
    writeln!(out)?;
    writeln!(out, "Image")?;
    writeln!(out, "{}", source.image.english)?;
    writeln!(out)?;

    // Moving lines (source's interpretation of the transition)
    let movers_str = movers
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    writeln!(
        out,
        "Moving lines: {}",
        if movers.is_empty() {
            "(none)".to_string()
        } else {
            movers_str
        }
    )?;
    writeln!(out)?;

    for pos in &movers {
        if let Some(l) = source.lines.iter().find(|l| l.position == *pos) {
            writeln!(out, "Line {}: {}", l.position, l.english)?;
        }
    }

    if full {
        writeln!(out)?;
        writeln!(out, "All source lines")?;
        for line in &source.lines {
            writeln!(out, "  {}. {}", line.position, line.english)?;
        }
    }

    writeln!(out)?;

    // Destination: composition + judgment + image
    writeln!(out, "── Destination ──")?;
    writeln!(
        out,
        "{} over {}",
        trigram_phrase(&destination.upper_trigram),
        trigram_phrase(&destination.lower_trigram)
    )?;
    writeln!(out)?;
    writeln!(out, "Judgment")?;
    writeln!(out, "{}", destination.judgment.english)?;
    writeln!(out)?;
    writeln!(out, "Image")?;
    writeln!(out, "{}", destination.image.english)?;

    if full {
        writeln!(out)?;
        writeln!(out, "All destination lines")?;
        for line in &destination.lines {
            writeln!(out, "  {}. {}", line.position, line.english)?;
        }
    }

    Ok(())
}
