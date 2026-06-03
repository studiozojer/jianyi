use crate::error::Error;
use crate::output::{emit, OutputFormat};
use crate::tarot::draw::{draw_cards, ArcanaFilter, DrawResult};
use chrono::Utc;

#[derive(clap::Args)]
pub struct Args {
    #[command(subcommand)]
    pub subcommand: Subcommand,
}

#[derive(clap::Subcommand)]
pub enum Subcommand {
    /// Draw N cards from the tarot deck. Deterministic with --seed.
    Draw(DrawArgs),
}

#[derive(clap::Args)]
pub struct DrawArgs {
    /// Number of cards to draw
    #[arg(long, default_value_t = 1)]
    pub count: usize,
    /// Seed timestamp (RFC 3339); defaults to now
    #[arg(long, alias = "at")]
    pub seed: Option<String>,
    /// Restrict to a subset: major | minor (default: full deck)
    #[arg(long)]
    pub arcana: Option<String>,
    /// Draw all cards upright; does not affect which cards are drawn
    #[arg(long)]
    pub no_reversals: bool,
}

pub fn run(args: Args, output: OutputFormat) -> Result<(), Error> {
    match args.subcommand {
        Subcommand::Draw(d) => run_draw(d, output),
    }
}

fn run_draw(args: DrawArgs, output: OutputFormat) -> Result<(), Error> {
    let seed_ts = match &args.seed {
        Some(s) => chrono::DateTime::parse_from_rfc3339(s)
            .map_err(|_| {
                Error::Usage(format!(
                    "invalid seed timestamp '{s}'. Use RFC 3339: 2026-06-02T11:00:00+00:00"
                ))
            })?
            .with_timezone(&Utc),
        None => Utc::now(),
    };

    let filter = match args.arcana.as_deref() {
        None => None,
        Some("major") => Some(ArcanaFilter::Major),
        Some("minor") => Some(ArcanaFilter::Minor),
        Some(other) => {
            return Err(Error::Usage(format!(
                "invalid --arcana value '{other}'. Use 'major' or 'minor'."
            )))
        }
    };

    let result = draw_cards(args.count, filter, seed_ts, args.no_reversals)
        .map_err(|e| Error::Generic(format!("draw failed: {e}")))?;
    emit(&result, output, render_pretty).map_err(Error::from)
}

fn render_pretty(result: &DrawResult) -> std::io::Result<()> {
    use std::io::Write;
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    writeln!(out, "Tarot Draw")?;
    writeln!(out)?;
    writeln!(out, "  Seed: {}", result.seed)?;
    writeln!(out, "  Count: {}", result.count)?;
    if let Some(filter) = &result.arcana_filter {
        writeln!(out, "  Arcana filter: {filter}")?;
    }
    if result.no_reversals {
        writeln!(out, "  Reversals: disabled")?;
    }
    writeln!(out)?;
    writeln!(out, "  Cards:")?;
    for (i, card) in result.drawn.iter().enumerate() {
        if result.no_reversals {
            writeln!(out, "    {}. {} ({})", i + 1, card.card, card.suit)?;
        } else {
            let orientation = if card.upright { "upright" } else { "reversed" };
            writeln!(out, "    {}. {} ({}, {})", i + 1, card.card, card.suit, orientation)?;
        }
    }
    Ok(())
}
