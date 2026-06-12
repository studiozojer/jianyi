use std::io::Read;

use chrono::Utc;
use serde::Serialize;

use crate::error::Error;
use crate::output::{emit, OutputFormat};
use crate::tarot::code::{decode, encode};
use crate::tarot::deck::{ArcanaFilter, Deck};
use crate::tarot::ops::{draw, shuffle, to_drawn, DrawnCard, Select};

#[derive(clap::Args)]
pub struct Args {
    #[command(subcommand)]
    pub subcommand: Subcommand,
}

#[derive(clap::Subcommand)]
pub enum Subcommand {
    /// Create a fresh deck in canonical order (all upright, unshuffled).
    New(NewArgs),
    /// Shuffle a deck. Deterministic with --seed. Riffle + cut; reversals via the cut.
    Shuffle(ShuffleArgs),
    /// Draw card(s): top N (-n) or a single fan-pick (--at). One-shot if no deck code.
    Draw(DrawArgs),
    /// Expand a deck code to its ordered pile.
    Inspect(InspectArgs),
}

#[derive(clap::Args)]
pub struct NewArgs {
    /// Restrict to a subset: major | minor (default: full 78-card deck)
    #[arg(long)]
    pub arcana: Option<String>,
}

#[derive(clap::Args)]
pub struct ShuffleArgs {
    /// Deck code to shuffle (or `-` to read from stdin)
    pub deck_code: String,
    /// Seed timestamp (RFC 3339); defaults to now
    #[arg(long)]
    pub seed: Option<String>,
    /// Shuffle without introducing reversals (all cards stay upright)
    #[arg(long)]
    pub no_reversals: bool,
}

#[derive(clap::Args)]
pub struct DrawArgs {
    /// Deck code to draw from (or `-` for stdin). Omit for a one-shot fresh draw.
    pub deck_code: Option<String>,
    /// Number of cards to draw from the top (default 1). Conflicts with --at.
    #[arg(short = 'n', long, conflicts_with = "at")]
    pub count: Option<usize>,
    /// Fan-pick: draw the single card at this pile position (top = 0). Conflicts with -n.
    #[arg(long, conflicts_with = "count")]
    pub at: Option<usize>,
    /// Seed for the one-shot shuffle (RFC 3339); used only when no deck code is given
    #[arg(long)]
    pub seed: Option<String>,
    /// Also emit the resulting deck code (one-shot only; a stateful draw always emits it)
    #[arg(long)]
    pub emit_deck: bool,
}

#[derive(clap::Args)]
pub struct InspectArgs {
    /// Deck code to inspect (or `-` for stdin)
    pub deck_code: String,
}

#[derive(Serialize)]
struct DeckOut {
    deck_code: String,
    count: usize,
    arcana: Option<String>,
}

#[derive(Serialize)]
struct DrawOut {
    drawn: Vec<DrawnCard>,
    deck_code: Option<String>,
    remaining: usize,
}

#[derive(Serialize)]
struct InspectOut {
    count: usize,
    pile: Vec<DrawnCard>,
}

pub fn run(args: Args, output: OutputFormat) -> Result<(), Error> {
    match args.subcommand {
        Subcommand::New(a) => run_new(a, output),
        Subcommand::Shuffle(a) => run_shuffle(a, output),
        Subcommand::Draw(a) => run_draw(a, output),
        Subcommand::Inspect(a) => run_inspect(a, output),
    }
}

fn parse_arcana(s: Option<&str>) -> Result<Option<ArcanaFilter>, Error> {
    match s {
        None => Ok(None),
        Some("major") => Ok(Some(ArcanaFilter::Major)),
        Some("minor") => Ok(Some(ArcanaFilter::Minor)),
        Some(other) => Err(Error::Usage(format!(
            "invalid --arcana value '{other}'. Use 'major' or 'minor'."
        ))),
    }
}

fn read_code(arg: &str) -> Result<String, Error> {
    if arg == "-" {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        Ok(buf.trim().to_string())
    } else {
        Ok(arg.to_string())
    }
}

fn load_deck(arg: &str) -> Result<Deck, Error> {
    let code = read_code(arg)?;
    decode(&code).map_err(|e| Error::Usage(format!("invalid deck code: {e}")))
}

fn parse_seed(seed: &Option<String>) -> Result<chrono::DateTime<Utc>, Error> {
    match seed {
        Some(s) => chrono::DateTime::parse_from_rfc3339(s)
            .map_err(|_| {
                Error::Usage(format!(
                    "invalid seed timestamp '{s}'. Use RFC 3339: 2026-06-02T11:00:00+00:00"
                ))
            })
            .map(|d| d.with_timezone(&Utc)),
        None => Ok(Utc::now()),
    }
}

fn run_new(args: NewArgs, output: OutputFormat) -> Result<(), Error> {
    let filter = parse_arcana(args.arcana.as_deref())?;
    let deck = Deck::new(filter);
    let out = DeckOut {
        deck_code: encode(&deck),
        count: deck.len(),
        arcana: args.arcana,
    };
    emit(&out, output, render_deck).map_err(Error::from)
}

fn run_shuffle(args: ShuffleArgs, output: OutputFormat) -> Result<(), Error> {
    let deck = load_deck(&args.deck_code)?;
    let seed = parse_seed(&args.seed)?;
    let shuffled = shuffle(&deck, seed, args.no_reversals);
    let out = DeckOut {
        deck_code: encode(&shuffled),
        count: shuffled.len(),
        arcana: None,
    };
    emit(&out, output, render_deck).map_err(Error::from)
}

fn run_draw(args: DrawArgs, output: OutputFormat) -> Result<(), Error> {
    let select = match (args.at, args.count) {
        (Some(pos), _) => Select::At(pos),
        (None, c) => Select::Top(c.unwrap_or(1)),
    };
    let (deck, emit_code) = match &args.deck_code {
        Some(code) => (load_deck(code)?, true),
        None => {
            let seed = parse_seed(&args.seed)?;
            (shuffle(&Deck::new(None), seed, false), args.emit_deck)
        }
    };
    let (drawn, remaining) = draw(&deck, select).map_err(|e| Error::Usage(e.to_string()))?;
    let out = DrawOut {
        drawn,
        deck_code: if emit_code {
            Some(encode(&remaining))
        } else {
            None
        },
        remaining: remaining.len(),
    };
    emit(&out, output, render_draw).map_err(Error::from)
}

fn run_inspect(args: InspectArgs, output: OutputFormat) -> Result<(), Error> {
    let deck = load_deck(&args.deck_code)?;
    let pile: Vec<DrawnCard> = deck.cards.iter().map(to_drawn).collect();
    let out = InspectOut {
        count: pile.len(),
        pile,
    };
    emit(&out, output, render_inspect).map_err(Error::from)
}

fn render_deck(out: &DeckOut) -> std::io::Result<()> {
    use std::io::Write;
    let stdout = std::io::stdout();
    let mut w = stdout.lock();
    writeln!(w, "Deck")?;
    writeln!(w, "  Count: {}", out.count)?;
    if let Some(arcana) = &out.arcana {
        writeln!(w, "  Arcana: {arcana}")?;
    }
    writeln!(w, "  Code: {}", out.deck_code)?;
    Ok(())
}

fn render_draw(out: &DrawOut) -> std::io::Result<()> {
    use std::io::Write;
    let stdout = std::io::stdout();
    let mut w = stdout.lock();
    writeln!(w, "Tarot Draw")?;
    writeln!(w)?;
    writeln!(w, "  Cards:")?;
    for (i, card) in out.drawn.iter().enumerate() {
        let orientation = if card.reversed { "reversed" } else { "upright" };
        writeln!(
            w,
            "    {}. {} ({}, {})",
            i + 1,
            card.card,
            card.suit,
            orientation
        )?;
    }
    writeln!(w)?;
    writeln!(w, "  Remaining: {}", out.remaining)?;
    if let Some(code) = &out.deck_code {
        writeln!(w, "  Code: {code}")?;
    }
    Ok(())
}

fn render_inspect(out: &InspectOut) -> std::io::Result<()> {
    use std::io::Write;
    let stdout = std::io::stdout();
    let mut w = stdout.lock();
    writeln!(w, "Deck ({} cards, top first)", out.count)?;
    for (i, card) in out.pile.iter().enumerate() {
        let orientation = if card.reversed { "reversed" } else { "upright" };
        writeln!(w, "  {}. {} ({}, {})", i, card.card, card.suit, orientation)?;
    }
    Ok(())
}
