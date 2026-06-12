use crate::error::Error;
use crate::output::{emit, OutputFormat};
use crate::tarot::code;
use crate::tarot::deck::{ArcanaFilter, Deck};
use crate::tarot::ops::{self, Select};
use chrono::Utc;
use serde::Serialize;

#[derive(clap::Args)]
pub struct Args {
    #[command(subcommand)]
    pub subcommand: Subcommand,
}

#[derive(clap::Subcommand)]
pub enum Subcommand {
    /// Create a new unshuffled deck and emit its deck code.
    New(NewArgs),
    /// Shuffle a deck (by code) and emit the new deck code.
    Shuffle(ShuffleArgs),
    /// Draw card(s). Pass a deck code for stateful draws; omit for a one-shot seeded draw.
    Draw(DrawArgs),
    /// Inspect a deck by code: show its current state without drawing.
    Inspect(InspectArgs),
}

// ---------------------------------------------------------------------------
// new
// ---------------------------------------------------------------------------

#[derive(clap::Args)]
pub struct NewArgs {
    /// Restrict to a subset: major | minor (default: full deck)
    #[arg(long)]
    pub arcana: Option<String>,
}

#[derive(Debug, Serialize)]
struct DeckInfo {
    deck_code: String,
    count: usize,
    arcana: String,
}

fn arcana_label(filter: Option<ArcanaFilter>) -> String {
    match filter {
        None => "full".to_string(),
        Some(ArcanaFilter::Major) => "major".to_string(),
        Some(ArcanaFilter::Minor) => "minor".to_string(),
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

fn run_new(args: NewArgs, output: OutputFormat) -> Result<(), Error> {
    let filter = parse_arcana(args.arcana.as_deref())?;
    let deck = Deck::new(filter);
    let info = DeckInfo {
        count: deck.len(),
        arcana: arcana_label(filter),
        deck_code: code::encode(&deck),
    };
    emit(&info, output, |v| {
        use std::io::Write;
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        writeln!(out, "New deck")?;
        writeln!(out)?;
        writeln!(out, "  Code:   {}", v.deck_code)?;
        writeln!(out, "  Count:  {}", v.count)?;
        writeln!(out, "  Arcana: {}", v.arcana)?;
        Ok(())
    })
    .map_err(Error::from)
}

// ---------------------------------------------------------------------------
// shuffle
// ---------------------------------------------------------------------------

#[derive(clap::Args)]
pub struct ShuffleArgs {
    /// Deck code to shuffle.
    pub deck_code: String,
    /// Seed timestamp (RFC 3339); defaults to now.
    #[arg(long)]
    pub seed: Option<String>,
    /// Draw all cards upright; does not affect which cards are drawn.
    #[arg(long)]
    pub no_reversals: bool,
}

fn run_shuffle(args: ShuffleArgs, output: OutputFormat) -> Result<(), Error> {
    let deck = code::decode(&args.deck_code)
        .map_err(|e| Error::Usage(format!("invalid deck code: {e}")))?;

    let filter = infer_filter(&deck);

    let seed_ts = parse_seed(args.seed.as_deref())?;
    let shuffled = ops::shuffle(&deck, seed_ts, args.no_reversals);
    let info = DeckInfo {
        count: shuffled.len(),
        arcana: arcana_label(filter),
        deck_code: code::encode(&shuffled),
    };
    emit(&info, output, |v| {
        use std::io::Write;
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        writeln!(out, "Shuffled deck")?;
        writeln!(out)?;
        writeln!(out, "  Code:   {}", v.deck_code)?;
        writeln!(out, "  Count:  {}", v.count)?;
        writeln!(out, "  Arcana: {}", v.arcana)?;
        Ok(())
    })
    .map_err(Error::from)
}

// ---------------------------------------------------------------------------
// draw
// ---------------------------------------------------------------------------

#[derive(clap::Args)]
pub struct DrawArgs {
    /// Deck code for stateful draw (omit for one-shot seeded draw).
    pub deck_code: Option<String>,
    /// Number of cards to draw (one-shot mode only; default 1).
    #[arg(short = 'n', long, default_value_t = 1)]
    pub count: usize,
    /// Draw the card at this zero-based pile position instead of the top.
    #[arg(long)]
    pub at: Option<usize>,
    /// Seed timestamp (RFC 3339); used in one-shot mode; defaults to now.
    #[arg(long)]
    pub seed: Option<String>,
    /// Restrict to a subset: major | minor (one-shot mode only).
    #[arg(long)]
    pub arcana: Option<String>,
    /// Draw all cards upright.
    #[arg(long)]
    pub no_reversals: bool,
}

#[derive(Debug, Serialize)]
struct DrawOutput {
    drawn: Vec<DrawnCardOut>,
    deck_code: Option<String>,
    remaining: usize,
}

#[derive(Debug, Serialize)]
struct DrawnCardOut {
    card: String,
    index: u8,
    suit: String,
    reversed: bool,
}

fn run_draw(args: DrawArgs, output: OutputFormat) -> Result<(), Error> {
    match args.deck_code {
        Some(ref dc) => run_draw_stateful(dc, &args, output),
        None => run_draw_oneshot(args, output),
    }
}

fn run_draw_stateful(deck_code: &str, args: &DrawArgs, output: OutputFormat) -> Result<(), Error> {
    let deck = code::decode(deck_code)
        .map_err(|e| Error::Usage(format!("invalid deck code: {e}")))?;

    let select = match args.at {
        Some(pos) => Select::At(pos),
        None => Select::Top(args.count),
    };

    let (drawn_cards, rest) =
        ops::draw(&deck, select).map_err(|e| Error::Usage(format!("{e}")))?;

    let new_code = code::encode(&rest);
    let drawn: Vec<DrawnCardOut> = drawn_cards
        .iter()
        .map(|dc| DrawnCardOut {
            card: dc.card.clone(),
            index: dc.index,
            suit: dc.suit.clone(),
            reversed: dc.reversed,
        })
        .collect();

    let result = DrawOutput {
        remaining: rest.len(),
        deck_code: Some(new_code),
        drawn,
    };

    emit(&result, output, |v| {
        use std::io::Write;
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        writeln!(out, "Draw")?;
        writeln!(out)?;
        for (i, card) in v.drawn.iter().enumerate() {
            let orientation = if card.reversed { "reversed" } else { "upright" };
            writeln!(out, "  {}. {} ({}, {})", i + 1, card.card, card.suit, orientation)?;
        }
        writeln!(out)?;
        writeln!(out, "  Remaining: {}", v.remaining)?;
        if let Some(ref code) = v.deck_code {
            writeln!(out, "  Next deck: {code}")?;
        }
        Ok(())
    })
    .map_err(Error::from)
}

fn run_draw_oneshot(args: DrawArgs, output: OutputFormat) -> Result<(), Error> {
    use crate::tarot::draw::{draw_cards, ArcanaFilter as OldFilter};
    let filter = match args.arcana.as_deref() {
        None => None,
        Some("major") => Some(OldFilter::Major),
        Some("minor") => Some(OldFilter::Minor),
        Some(other) => {
            return Err(Error::Usage(format!(
                "invalid --arcana value '{other}'. Use 'major' or 'minor'."
            )))
        }
    };
    let seed_ts = parse_seed(args.seed.as_deref())?;
    let result = draw_cards(args.count, filter, seed_ts, args.no_reversals)
        .map_err(|e| Error::Generic(format!("draw failed: {e}")))?;

    // Re-emit in the new unified JSON shape for consistency.
    let drawn: Vec<DrawnCardOut> = result
        .drawn
        .iter()
        .enumerate()
        .map(|(i, c)| {
            use crate::tarot::cards::TarotCard;
            // find card index by scanning full deck
            let index = (0..TarotCard::COUNT)
                .filter_map(TarotCard::from_index)
                .find(|tc| crate::tarot::deck::card_display_name(*tc) == c.card)
                .map(|tc| tc as u8)
                .unwrap_or(i as u8);
            DrawnCardOut {
                card: c.card.clone(),
                index,
                suit: c.suit.clone(),
                reversed: !c.upright,
            }
        })
        .collect();

    let out = DrawOutput {
        remaining: 0, // one-shot: no remaining deck
        deck_code: None,
        drawn,
    };

    emit(&out, output, |v| {
        use std::io::Write;
        let stdout = std::io::stdout();
        let mut out_handle = stdout.lock();
        writeln!(out_handle, "Tarot Draw")?;
        writeln!(out_handle)?;
        for (i, card) in v.drawn.iter().enumerate() {
            let orientation = if card.reversed { "reversed" } else { "upright" };
            writeln!(out_handle, "  {}. {} ({}, {})", i + 1, card.card, card.suit, orientation)?;
        }
        Ok(())
    })
    .map_err(Error::from)
}

// ---------------------------------------------------------------------------
// inspect
// ---------------------------------------------------------------------------

#[derive(clap::Args)]
pub struct InspectArgs {
    /// Deck code to inspect.
    pub deck_code: String,
}

fn run_inspect(args: InspectArgs, output: OutputFormat) -> Result<(), Error> {
    let deck = code::decode(&args.deck_code)
        .map_err(|e| Error::Usage(format!("invalid deck code: {e}")))?;
    let filter = infer_filter(&deck);
    let info = DeckInfo {
        count: deck.len(),
        arcana: arcana_label(filter),
        deck_code: args.deck_code,
    };
    emit(&info, output, |v| {
        use std::io::Write;
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        writeln!(out, "Deck")?;
        writeln!(out)?;
        writeln!(out, "  Code:   {}", v.deck_code)?;
        writeln!(out, "  Count:  {}", v.count)?;
        writeln!(out, "  Arcana: {}", v.arcana)?;
        Ok(())
    })
    .map_err(Error::from)
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

fn parse_seed(seed: Option<&str>) -> Result<chrono::DateTime<Utc>, Error> {
    match seed {
        Some(s) => chrono::DateTime::parse_from_rfc3339(s)
            .map_err(|_| {
                Error::Usage(format!(
                    "invalid seed timestamp '{s}'. Use RFC 3339: 2026-06-02T11:00:00+00:00"
                ))
            })
            .map(|dt| dt.with_timezone(&Utc)),
        None => Ok(Utc::now()),
    }
}

/// Heuristic: guess arcana filter from card count (22 → major, 56 → minor, else full).
fn infer_filter(deck: &Deck) -> Option<ArcanaFilter> {
    use crate::tarot::cards::TarotCard;
    let has_major = deck.cards.iter().any(|c| c.card.is_major_arcana());
    let has_minor = deck.cards.iter().any(|c| !c.card.is_major_arcana());
    // Check if all cards in the deck are consistent with a filter.
    let all_major = deck.cards.len() <= TarotCard::COUNT && !has_minor && has_major;
    let all_minor = !has_major && has_minor;
    if all_major {
        Some(ArcanaFilter::Major)
    } else if all_minor {
        Some(ArcanaFilter::Minor)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// dispatch
// ---------------------------------------------------------------------------

pub fn run(args: Args, output: OutputFormat) -> Result<(), Error> {
    match args.subcommand {
        Subcommand::New(a) => run_new(a, output),
        Subcommand::Shuffle(a) => run_shuffle(a, output),
        Subcommand::Draw(a) => run_draw(a, output),
        Subcommand::Inspect(a) => run_inspect(a, output),
    }
}
