# Stateful-Deck Tarot Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Redesign jianyi's tarot from a stateless one-shot draw into a deck a caller holds, shuffles, and draws from over time — while jianyi itself stays stateless and pure.

**Architecture:** jianyi gains a library target; the binary becomes a thin CLI over it. The deck lives as an ordered list of `(card, reversed)`; the caller threads a compact, versioned **deck code** between calls. Pure functions only: `new` / `shuffle` / `draw` / `inspect`. Shuffle is a physical riffle + cut-and-flip (reversals are a consequence of cuts, baked in at shuffle time, revealed at draw). jianyi models the draw pile only — the table/layout is the caller's.

**Tech Stack:** Rust 2021, `clap` (derive), `serde`/`serde_json`, `rand` (`StdRng`), `chrono`, `base64`. Spec: `docs/specs/2026-06-12-stateful-deck-tarot-design.md`.

---

## File Structure

```
repos/jianyi/
├── Cargo.toml              # MODIFY: add [lib]; add base64 dep
├── src/
│   ├── lib.rs              # CREATE: pub mod error/output/iching/tarot
│   ├── main.rs             # MODIFY: thin bin over the jianyi lib crate
│   └── tarot/
│       ├── mod.rs          # MODIFY: drop `draw`, add `deck` `code` `ops`
│       ├── cards.rs        # UNCHANGED: TarotCard enum + helpers
│       ├── deck.rs         # CREATE: Deck, DeckCard, ArcanaFilter, naming helpers
│       ├── code.rs         # CREATE: deck-code encode/decode (v1.<base64>)
│       ├── ops.rs          # CREATE: shuffle / draw / Select / DrawnCard
│       ├── cli.rs          # REWRITE: new/shuffle/draw/inspect subcommands
│       └── draw.rs         # DELETE (superseded by ops.rs)
└── tests/
    └── cli.rs              # MODIFY: extend with deck-lifecycle integration tests
```

Responsibilities are split by concern: `deck.rs` owns the deck value and naming, `code.rs` owns serialization, `ops.rs` owns the deterministic operations, `cli.rs` owns argument parsing and rendering. Each is small and testable in isolation.

---

## Task 1: Introduce the library target (pure refactor, no behavior change)

**Files:**
- Create: `src/lib.rs`
- Modify: `src/main.rs`
- Modify: `Cargo.toml`

- [ ] **Step 1: Add the `[lib]` target to `Cargo.toml`**

Insert this block immediately **before** the existing `[[bin]]` block for `jianyi`:

```toml
[lib]
name = "jianyi"
path = "src/lib.rs"
```

- [ ] **Step 2: Create `src/lib.rs`**

```rust
//! jianyi — a portable, deterministic, offline I Ching and tarot calculator.
//!
//! This library exposes the calculation engine; the `jianyi` binary is a thin
//! CLI over it. The same operations are callable from a host (e.g. over FFI).

pub mod error;
pub mod iching;
pub mod output;
pub mod tarot;
```

- [ ] **Step 3: Rewrite `src/main.rs` as a thin binary over the lib**

```rust
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
```

- [ ] **Step 4: Build and run the existing tests to confirm the refactor is green**

Run: `cargo build && cargo test`
Expected: PASS — all existing iching and tarot tests still pass (this task changes wiring only, no behavior). `src/bin/gen-data.rs` is self-contained and unaffected.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml src/lib.rs src/main.rs
git commit -m "refactor: add jianyi library target; binary becomes thin CLI over it"
```

---

## Task 2: Deck value type and naming helpers (`deck.rs`)

**Files:**
- Create: `src/tarot/deck.rs`
- Modify: `src/tarot/mod.rs`
- Test: inline `#[cfg(test)]` in `src/tarot/deck.rs`

- [ ] **Step 1: Register the module in `src/tarot/mod.rs`**

Replace the whole file with:

```rust
pub mod cards;
pub mod cli;
pub mod code;
pub mod deck;
pub mod ops;
```

(This references `code` and `ops`, created in Tasks 3–4. The crate will not compile until those exist; that is expected — this task ends by building only after its own file is written together with the stubs noted below. To keep Task 2 self-contained and green, create empty placeholder files `src/tarot/code.rs` and `src/tarot/ops.rs` each containing only `// filled in a later task` for now.)

- [ ] **Step 2: Create the placeholder files so the crate compiles**

```bash
printf '// filled in Task 3\n' > src/tarot/code.rs
printf '// filled in Task 4\n' > src/tarot/ops.rs
```

- [ ] **Step 3: Write the failing test in `src/tarot/deck.rs`**

```rust
use serde::{Deserialize, Serialize};

use crate::tarot::cards::{TarotCard, TarotSuit};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeckCard {
    pub card: TarotCard,
    pub reversed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deck {
    /// Top of the pile is `cards[0]`.
    pub cards: Vec<DeckCard>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArcanaFilter {
    Major,
    Minor,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_full_deck_is_78_canonical_upright() {
        let deck = Deck::new(None);
        assert_eq!(deck.len(), 78);
        assert!(deck.cards.iter().all(|c| !c.reversed));
        assert_eq!(deck.cards[0].card, TarotCard::TheFool);
        assert_eq!(deck.cards[77].card, TarotCard::KingOfPentacles);
    }

    #[test]
    fn new_major_filter_is_22() {
        let deck = Deck::new(Some(ArcanaFilter::Major));
        assert_eq!(deck.len(), 22);
        assert!(deck.cards.iter().all(|c| c.card.is_major_arcana()));
    }

    #[test]
    fn new_minor_filter_is_56() {
        let deck = Deck::new(Some(ArcanaFilter::Minor));
        assert_eq!(deck.len(), 56);
        assert!(deck.cards.iter().all(|c| !c.card.is_major_arcana()));
    }

    #[test]
    fn suit_and_name_helpers() {
        assert_eq!(suit_string(TarotCard::TheTower), "Major Arcana");
        assert_eq!(suit_string(TarotCard::AceOfWands), "Wands");
        assert_eq!(card_display_name(TarotCard::TheHangedMan), "The Hanged Man");
    }
}
```

- [ ] **Step 4: Run the test to verify it fails**

Run: `cargo test -p jianyi tarot::deck`
Expected: FAIL — `Deck::new`, `suit_string`, `card_display_name` not defined.

- [ ] **Step 5: Implement the impls and helpers**

Add this **above** the `#[cfg(test)]` block in `src/tarot/deck.rs`:

```rust
impl Deck {
    /// A fresh deck in canonical order (indices 0..=77), all upright, unshuffled.
    pub fn new(filter: Option<ArcanaFilter>) -> Self {
        let cards = (0..TarotCard::COUNT)
            .filter_map(TarotCard::from_index)
            .filter(|c| match filter {
                None => true,
                Some(ArcanaFilter::Major) => c.is_major_arcana(),
                Some(ArcanaFilter::Minor) => !c.is_major_arcana(),
            })
            .map(|card| DeckCard {
                card,
                reversed: false,
            })
            .collect();
        Deck { cards }
    }

    pub fn len(&self) -> usize {
        self.cards.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

/// Human-readable suit label ("Major Arcana" for the trumps).
pub fn suit_string(card: TarotCard) -> String {
    match card.suit() {
        None => "Major Arcana".to_string(),
        Some(TarotSuit::Wands) => "Wands".to_string(),
        Some(TarotSuit::Cups) => "Cups".to_string(),
        Some(TarotSuit::Swords) => "Swords".to_string(),
        Some(TarotSuit::Pentacles) => "Pentacles".to_string(),
    }
}
```

Then **move `card_display_name` verbatim** from `src/tarot/draw.rs` (the `pub(crate) fn card_display_name(card: TarotCard) -> String { … }` at lines 126–216, the full 78-arm match) into `src/tarot/deck.rs`, changing its visibility from `pub(crate)` to `pub`. Do not alter the match arms.

- [ ] **Step 6: Run the test to verify it passes**

Run: `cargo test -p jianyi tarot::deck`
Expected: PASS (4 tests).

- [ ] **Step 7: Commit**

```bash
git add src/tarot/mod.rs src/tarot/deck.rs src/tarot/code.rs src/tarot/ops.rs
git commit -m "feat(tarot): Deck/DeckCard value type, ArcanaFilter, naming helpers"
```

---

## Task 3: Deck-code codec (`code.rs`)

**Files:**
- Modify: `Cargo.toml` (add `base64`)
- Modify: `src/tarot/code.rs` (replace placeholder)
- Test: inline `#[cfg(test)]` in `src/tarot/code.rs`

- [ ] **Step 1: Add the `base64` dependency to `Cargo.toml`**

Add to the `[dependencies]` table:

```toml
base64 = "0.22"
```

(Decision: the spec allows a hand-rolled codec *or* a tiny crate; we use the well-tested `base64` crate to avoid subtle padding/alphabet bugs in a correctness-critical path.)

- [ ] **Step 2: Write the failing tests in `src/tarot/code.rs`** (replace the placeholder line)

```rust
use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine;

use crate::tarot::cards::TarotCard;
use crate::tarot::deck::{ArcanaFilter, Deck, DeckCard};

/// Deck-code version tag. Bump when the byte layout changes.
pub const VERSION_PREFIX: &str = "v1.";

#[derive(Debug, PartialEq, Eq)]
pub enum CodeError {
    BadVersion,
    BadBase64,
    Truncated,
    BadCardByte(u8),
}

impl std::fmt::Display for CodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodeError::BadVersion => {
                write!(f, "unrecognized deck-code version (expected `{VERSION_PREFIX}`)")
            }
            CodeError::BadBase64 => write!(f, "deck code is not valid base64"),
            CodeError::Truncated => write!(f, "deck code is truncated"),
            CodeError::BadCardByte(b) => {
                write!(f, "deck code contains an invalid card byte: {b}")
            }
        }
    }
}

impl std::error::Error for CodeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_full_deck() {
        let deck = Deck::new(None);
        let code = encode(&deck);
        assert!(code.starts_with(VERSION_PREFIX));
        assert_eq!(decode(&code).unwrap(), deck);
    }

    #[test]
    fn roundtrip_partial_and_reversed() {
        let deck = Deck {
            cards: vec![
                DeckCard { card: TarotCard::TheTower, reversed: true },
                DeckCard { card: TarotCard::AceOfCups, reversed: false },
                DeckCard { card: TarotCard::KingOfPentacles, reversed: true },
            ],
        };
        assert_eq!(decode(&encode(&deck)).unwrap(), deck);
    }

    #[test]
    fn roundtrip_empty_deck() {
        let deck = Deck { cards: vec![] };
        assert_eq!(decode(&encode(&deck)).unwrap(), deck);
    }

    #[test]
    fn roundtrip_minor_filter() {
        let deck = Deck::new(Some(ArcanaFilter::Minor));
        assert_eq!(decode(&encode(&deck)).unwrap(), deck);
    }

    #[test]
    fn bad_version_rejected() {
        assert_eq!(decode("v2.AAAA"), Err(CodeError::BadVersion));
        assert_eq!(decode("AAAA"), Err(CodeError::BadVersion));
    }

    #[test]
    fn bad_base64_rejected() {
        assert_eq!(decode("v1.!!!!"), Err(CodeError::BadBase64));
    }

    #[test]
    fn truncated_rejected() {
        // count byte says 3 cards but no card bytes follow
        let code = format!("{VERSION_PREFIX}{}", STANDARD_NO_PAD.encode([3u8]));
        assert_eq!(decode(&code), Err(CodeError::Truncated));
    }
}
```

- [ ] **Step 3: Run the tests to verify they fail**

Run: `cargo test -p jianyi tarot::code`
Expected: FAIL — `encode` / `decode` not defined.

- [ ] **Step 4: Implement `encode` / `decode`**

Add **above** the `#[cfg(test)]` block:

```rust
/// Serialize a deck to `v1.<base64>`. Payload: `[count: u8][card_byte × count]`,
/// where each card byte packs the 7-bit card index plus a 1-bit reversed flag
/// as `(index << 1) | reversed`.
pub fn encode(deck: &Deck) -> String {
    let mut bytes = Vec::with_capacity(1 + deck.cards.len());
    bytes.push(deck.cards.len() as u8);
    for dc in &deck.cards {
        let index = dc.card as u8; // discriminants are 0..=77, fit in u8
        bytes.push((index << 1) | dc.reversed as u8);
    }
    format!("{VERSION_PREFIX}{}", STANDARD_NO_PAD.encode(&bytes))
}

/// Parse a `v1.<base64>` deck code back into a `Deck`.
pub fn decode(code: &str) -> Result<Deck, CodeError> {
    let body = code.strip_prefix(VERSION_PREFIX).ok_or(CodeError::BadVersion)?;
    let bytes = STANDARD_NO_PAD
        .decode(body)
        .map_err(|_| CodeError::BadBase64)?;
    let (&count, rest) = bytes.split_first().ok_or(CodeError::Truncated)?;
    if rest.len() != count as usize {
        return Err(CodeError::Truncated);
    }
    let mut cards = Vec::with_capacity(count as usize);
    for &b in rest {
        let index = (b >> 1) as usize;
        let reversed = (b & 1) == 1;
        let card = TarotCard::from_index(index).ok_or(CodeError::BadCardByte(b))?;
        cards.push(DeckCard { card, reversed });
    }
    Ok(Deck { cards })
}
```

- [ ] **Step 5: Run the tests to verify they pass**

Run: `cargo test -p jianyi tarot::code`
Expected: PASS (7 tests).

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock src/tarot/code.rs
git commit -m "feat(tarot): versioned deck-code codec (v1.<base64>)"
```

---

## Task 4: Shuffle (`ops.rs`, part 1 — physical riffle + cut-and-flip)

**Files:**
- Modify: `src/tarot/ops.rs` (replace placeholder)
- Test: inline `#[cfg(test)]` in `src/tarot/ops.rs`

- [ ] **Step 1: Write the failing tests in `src/tarot/ops.rs`** (replace the placeholder line)

```rust
use chrono::{DateTime, Utc};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::tarot::deck::{Deck, DeckCard};

/// Riffle passes per shuffle. ~7 is the accepted threshold to mix a deck.
const RIFFLE_PASSES: usize = 7;
/// Cut-and-flip passes per shuffle (where reversals enter).
const CUT_FLIPS: usize = 3;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn seed() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 2, 11, 0, 0).unwrap()
    }

    #[test]
    fn shuffle_is_deterministic() {
        let deck = Deck::new(None);
        assert_eq!(shuffle(&deck, seed(), false), shuffle(&deck, seed(), false));
    }

    #[test]
    fn shuffle_preserves_the_card_set() {
        let deck = Deck::new(None);
        let shuffled = shuffle(&deck, seed(), false);
        assert_eq!(shuffled.len(), 78);
        let mut idx: Vec<u8> = shuffled.cards.iter().map(|c| c.card as u8).collect();
        idx.sort_unstable();
        idx.dedup();
        assert_eq!(idx.len(), 78, "shuffle lost or duplicated cards");
    }

    #[test]
    fn shuffle_actually_mixes_positions() {
        let deck = Deck::new(None);
        let shuffled = shuffle(&deck, seed(), false);
        let fixed = deck
            .cards
            .iter()
            .zip(shuffled.cards.iter())
            .filter(|(a, b)| a.card == b.card)
            .count();
        assert!(fixed < 8, "too many cards kept their original position: {fixed}");
    }

    #[test]
    fn shuffle_with_reversals_produces_both_orientations() {
        let deck = Deck::new(None);
        let shuffled = shuffle(&deck, seed(), false);
        assert!(shuffled.cards.iter().any(|c| c.reversed), "expected some reversed");
        assert!(shuffled.cards.iter().any(|c| !c.reversed), "expected some upright");
    }

    #[test]
    fn no_reversals_yields_all_upright() {
        let deck = Deck::new(None);
        let shuffled = shuffle(&deck, seed(), true);
        assert!(shuffled.cards.iter().all(|c| !c.reversed));
        assert_eq!(shuffled.len(), 78);
    }
}
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p jianyi tarot::ops`
Expected: FAIL — `shuffle` not defined.

- [ ] **Step 3: Implement `shuffle` and its helpers**

Add **above** the `#[cfg(test)]` block:

```rust
fn seed_to_u64(seed: DateTime<Utc>) -> u64 {
    seed.timestamp_nanos_opt()
        .unwrap_or_else(|| seed.timestamp() * 1_000_000_000) as u64
}

/// One Gilbert–Shannon–Reeds riffle: split near the middle (with jitter),
/// then interleave, taking from each half with probability proportional to its
/// remaining size. Orientation is untouched here.
fn riffle(cards: &[DeckCard], rng: &mut StdRng) -> Vec<DeckCard> {
    let n = cards.len();
    if n < 2 {
        return cards.to_vec();
    }
    let cut = (n / 2 + rng.gen_range(0..=2)).clamp(1, n - 1);
    let (left, right) = cards.split_at(cut);
    let (mut li, mut ri) = (0usize, 0usize);
    let mut out = Vec::with_capacity(n);
    while li < left.len() && ri < right.len() {
        let rl = (left.len() - li) as u32;
        let rr = (right.len() - ri) as u32;
        if rng.gen_ratio(rl, rl + rr) {
            out.push(left[li]);
            li += 1;
        } else {
            out.push(right[ri]);
            ri += 1;
        }
    }
    out.extend_from_slice(&left[li..]);
    out.extend_from_slice(&right[ri..]);
    out
}

/// One cut: lift the bottom segment, rotate it 180° (its order reverses and each
/// card's orientation flips), and set it on top. When `no_reversals` is set, the
/// segment still moves but stays upright.
fn cut_and_flip(cards: &[DeckCard], rng: &mut StdRng, no_reversals: bool) -> Vec<DeckCard> {
    let n = cards.len();
    if n < 2 {
        return cards.to_vec();
    }
    let c = rng.gen_range(1..n);
    let mut out: Vec<DeckCard> = cards[c..]
        .iter()
        .rev()
        .map(|dc| DeckCard {
            card: dc.card,
            reversed: if no_reversals { false } else { !dc.reversed },
        })
        .collect();
    out.extend_from_slice(&cards[..c]);
    out
}

/// Shuffle a deck deterministically from `(deck, seed)`. Riffle for position,
/// then cut-and-flip for orientation. `no_reversals` keeps every card upright.
pub fn shuffle(deck: &Deck, seed: DateTime<Utc>, no_reversals: bool) -> Deck {
    let mut rng = StdRng::seed_from_u64(seed_to_u64(seed));
    let mut cards = deck.cards.clone();
    for _ in 0..RIFFLE_PASSES {
        cards = riffle(&cards, &mut rng);
    }
    for _ in 0..CUT_FLIPS {
        cards = cut_and_flip(&cards, &mut rng, no_reversals);
    }
    Deck { cards }
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p jianyi tarot::ops`
Expected: PASS (5 tests). If `shuffle_actually_mixes_positions` ever fails, that is a real mixing regression — do not relax the threshold without re-deriving it.

- [ ] **Step 5: Commit**

```bash
git add src/tarot/ops.rs
git commit -m "feat(tarot): deterministic physical shuffle (riffle + cut-and-flip)"
```

---

## Task 5: Draw (`ops.rs`, part 2 — Top / At selection)

**Files:**
- Modify: `src/tarot/ops.rs`
- Test: extend the inline `#[cfg(test)]` in `src/tarot/ops.rs`

- [ ] **Step 1: Add the failing tests to the `tests` module in `src/tarot/ops.rs`**

```rust
    #[test]
    fn draw_top_pops_in_order_and_shrinks() {
        let deck = Deck::new(None);
        let (drawn, rest) = draw(&deck, Select::Top(3)).unwrap();
        assert_eq!(drawn.len(), 3);
        assert_eq!(rest.len(), 75);
        assert_eq!(drawn[0].card, "The Fool"); // top of a fresh deck
        assert_eq!(drawn[0].index, 0);
    }

    #[test]
    fn draw_at_removes_the_card_at_that_position() {
        let deck = Deck::new(None); // index i sits at position i
        let (drawn, rest) = draw(&deck, Select::At(39)).unwrap();
        assert_eq!(drawn.len(), 1);
        assert_eq!(drawn[0].index, 39);
        assert_eq!(rest.len(), 77);
        // the card formerly at 40 has shifted down into position 39
        assert!(!rest.cards.iter().any(|c| c.card as u8 == 39));
    }

    #[test]
    fn draw_top_too_many_errors() {
        let deck = Deck::new(Some(crate::tarot::deck::ArcanaFilter::Major)); // 22 cards
        assert!(matches!(
            draw(&deck, Select::Top(23)),
            Err(DrawError::NotEnough { requested: 23, available: 22 })
        ));
    }

    #[test]
    fn draw_at_out_of_range_errors() {
        let deck = Deck::new(None);
        assert!(matches!(
            draw(&deck, Select::At(78)),
            Err(DrawError::OutOfRange { position: 78, len: 78 })
        ));
    }

    #[test]
    fn card_drawn_at_does_not_reappear_after_shuffle() {
        let deck = shuffle(&Deck::new(None), seed(), false);
        let (drawn, rest) = draw(&deck, Select::At(39)).unwrap();
        let gone = drawn[0].index;
        let reshuffled = shuffle(&rest, seed(), false);
        assert_eq!(reshuffled.len(), 77);
        assert!(
            !reshuffled.cards.iter().any(|c| c.card as u8 == gone),
            "a card drawn before the shuffle reappeared"
        );
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p jianyi tarot::ops`
Expected: FAIL — `Select`, `DrawnCard`, `DrawError`, `draw` not defined.

- [ ] **Step 3: Implement the draw types and function**

First, update the imports at the top of `src/tarot/ops.rs`: add a serde line and the naming helpers (replace the existing `use crate::tarot::deck::{Deck, DeckCard};` line):

```rust
use serde::{Deserialize, Serialize};

use crate::tarot::deck::{card_display_name, suit_string, Deck, DeckCard};
```

Then add **above** the `#[cfg(test)]` block in `src/tarot/ops.rs`:

```rust
/// What to draw: the top `n` cards, or the single card at a pile position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Select {
    Top(usize),
    At(usize),
}

/// A revealed card in draw/inspect output. `index` is the card's 0..=77 identity
/// (distinct from a pile `position`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrawnCard {
    pub card: String,
    pub index: u8,
    pub suit: String,
    pub reversed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrawError {
    NotEnough { requested: usize, available: usize },
    OutOfRange { position: usize, len: usize },
}

impl std::fmt::Display for DrawError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DrawError::NotEnough { requested, available } => write!(
                f,
                "cannot draw {requested} card(s); only {available} remain in the deck"
            ),
            DrawError::OutOfRange { position, len } => write!(
                f,
                "no card at position {position}; deck has {len} card(s) (positions 0..{len})"
            ),
        }
    }
}

impl std::error::Error for DrawError {}

/// Expand a `DeckCard` into a revealed `DrawnCard`.
pub fn to_drawn(dc: &DeckCard) -> DrawnCard {
    DrawnCard {
        card: card_display_name(dc.card),
        index: dc.card as u8,
        suit: suit_string(dc.card),
        reversed: dc.reversed,
    }
}

/// Draw card(s) from the pile, returning the revealed cards and the shrunk deck.
pub fn draw(deck: &Deck, select: Select) -> Result<(Vec<DrawnCard>, Deck), DrawError> {
    match select {
        Select::Top(n) => {
            if n > deck.cards.len() {
                return Err(DrawError::NotEnough {
                    requested: n,
                    available: deck.cards.len(),
                });
            }
            let drawn = deck.cards[..n].iter().map(to_drawn).collect();
            let rest = Deck {
                cards: deck.cards[n..].to_vec(),
            };
            Ok((drawn, rest))
        }
        Select::At(pos) => {
            if pos >= deck.cards.len() {
                return Err(DrawError::OutOfRange {
                    position: pos,
                    len: deck.cards.len(),
                });
            }
            let mut cards = deck.cards.clone();
            let dc = cards.remove(pos);
            Ok((vec![to_drawn(&dc)], Deck { cards }))
        }
    }
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p jianyi tarot::ops`
Expected: PASS (10 tests total in the module).

- [ ] **Step 5: Commit**

```bash
git add src/tarot/ops.rs
git commit -m "feat(tarot): draw with Top(n)/At(position) selection"
```

---

## Task 6: CLI rewrite (`cli.rs`) and retire `draw.rs`

**Files:**
- Rewrite: `src/tarot/cli.rs`
- Delete: `src/tarot/draw.rs`

- [ ] **Step 1: Replace `src/tarot/cli.rs` entirely**

```rust
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
```

- [ ] **Step 2: Delete the superseded `draw.rs`**

```bash
git rm src/tarot/draw.rs
```

- [ ] **Step 3: Build and run the unit tests**

Run: `cargo build && cargo test -p jianyi`
Expected: PASS — library builds, all unit tests green. (The integration test `tests/cli.rs` still references the old behavior but should still pass; it is updated in Task 7.)

- [ ] **Step 4: Smoke-test the deck lifecycle by hand**

Run:
```bash
cargo run -q -- tarot new --json
CODE=$(cargo run -q -- tarot new | sed -n 's/^  Code: //p')
cargo run -q -- tarot shuffle "$CODE" --seed 2026-06-02T11:00:00+00:00
cargo run -q -- tarot draw "$CODE" -n 3
cargo run -q -- tarot draw "$CODE" --at 39
cargo run -q -- tarot inspect "$CODE"
cargo run -q -- tarot draw -n 1            # one-shot, no code
```
Expected: `new` prints a `v1.…` code; `shuffle` prints a new code; `draw` prints cards + a new code + remaining count; `inspect` lists the pile; one-shot `draw` prints a single card and no code.

- [ ] **Step 5: Commit**

```bash
git add src/tarot/cli.rs
git commit -m "feat(tarot): new/shuffle/draw/inspect CLI; retire stateless draw.rs"
```

---

## Task 7: Integration tests (`tests/cli.rs`)

**Files:**
- Modify: `tests/cli.rs`

- [ ] **Step 1: Replace the existing `tarot_draw_seeded_is_deterministic` test and add lifecycle tests**

Replace the `tarot_draw_seeded_is_deterministic` test (lines 36–46) with:

```rust
fn stdout_of(args: &[&str]) -> String {
    let out = bin().args(args).output().unwrap();
    assert!(out.status.success(), "command failed: {args:?}");
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn tarot_oneshot_draw_seeded_is_deterministic() {
    let run = || {
        stdout_of(&[
            "--json", "tarot", "draw", "-n", "5", "--seed", "2026-06-02T11:00:00+00:00",
        ])
    };
    assert_eq!(run(), run());
}

#[test]
fn tarot_new_emits_versioned_code() {
    let v: serde_json::Value =
        serde_json::from_str(&stdout_of(&["--json", "tarot", "new"])).unwrap();
    assert_eq!(v["count"], 78);
    assert!(v["deck_code"].as_str().unwrap().starts_with("v1."));
}

#[test]
fn tarot_deck_lifecycle_roundtrips() {
    // new -> capture code -> shuffle -> draw top -> the returned code drives the next draw
    let new: serde_json::Value =
        serde_json::from_str(&stdout_of(&["--json", "tarot", "new"])).unwrap();
    let code = new["deck_code"].as_str().unwrap().to_string();

    let shuffled: serde_json::Value = serde_json::from_str(&stdout_of(&[
        "--json", "tarot", "shuffle", &code, "--seed", "2026-06-02T11:00:00+00:00",
    ]))
    .unwrap();
    let scode = shuffled["deck_code"].as_str().unwrap().to_string();

    let draw: serde_json::Value =
        serde_json::from_str(&stdout_of(&["--json", "tarot", "draw", &scode, "-n", "1"]))
            .unwrap();
    assert_eq!(draw["remaining"], 77);
    assert!(draw["deck_code"].as_str().unwrap().starts_with("v1."));
    assert_eq!(draw["drawn"].as_array().unwrap().len(), 1);
}

#[test]
fn tarot_draw_at_out_of_range_is_usage_error() {
    let new: serde_json::Value =
        serde_json::from_str(&stdout_of(&["--json", "tarot", "new"])).unwrap();
    let code = new["deck_code"].as_str().unwrap();
    let out = bin().args(["tarot", "draw", code, "--at", "78"]).output().unwrap();
    assert_eq!(out.status.code(), Some(2)); // Error::Usage
}

#[test]
fn tarot_bad_deck_code_is_usage_error() {
    let out = bin().args(["tarot", "inspect", "not-a-code"]).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
}
```

- [ ] **Step 2: Run the integration tests**

Run: `cargo test -p jianyi --test cli`
Expected: PASS (existing iching tests + the new tarot tests).

- [ ] **Step 3: Commit**

```bash
git add tests/cli.rs
git commit -m "test(tarot): deck-lifecycle + error-path integration tests"
```

---

## Task 8: README + final verification

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Replace the tarot section of `README.md`**

Find the existing `### Tarot` / tarot `draw` documentation and replace it with documentation for the new surface:

```markdown
### Tarot

Tarot is modeled as a **deck the caller holds**. jianyi stays stateless: each
command takes a deck code and prints a new one. A *deck code* (`v1.…`) is the
deck's order and orientations, compact enough to store and pass around.

**New deck** — a fresh deck in canonical order, all upright, unshuffled:

```
jianyi tarot new
jianyi tarot new --arcana major     # 22-card major arcana only
jianyi --json tarot new
```

**Shuffle** — riffle + cut; reversals enter via the cut. Deterministic with `--seed`:

```
jianyi tarot shuffle <deck-code>
jianyi tarot shuffle <deck-code> --seed 2026-06-02T11:00:00+00:00
jianyi tarot shuffle <deck-code> --no-reversals
```

**Draw** — the top `n` (`-n`) or a single fan-pick at a position (`--at`).
A stateful draw always returns the new deck code; omit the deck code for a
one-shot fresh draw (add `--emit-deck` to also get the resulting code):

```
jianyi tarot draw <deck-code> -n 3
jianyi tarot draw <deck-code> --at 39
jianyi tarot draw -n 1                       # one-shot
```

**Inspect** — expand a deck code to its ordered pile:

```
jianyi tarot inspect <deck-code>
```

Deck code can be `-` to read from stdin, so commands pipeline:

```
jianyi tarot new | sed -n 's/^  Code: //p' | xargs jianyi tarot shuffle
```
```

- [ ] **Step 2: Final full verification**

Run: `cargo build --release && cargo test`
Expected: PASS — release build clean, full test suite green.

Run: `cargo clippy --all-targets`
Expected: no warnings introduced by the new modules.

- [ ] **Step 3: Commit**

```bash
git add README.md
git commit -m "docs: document the stateful-deck tarot CLI"
```

---

## Notes for the implementer

- **TDD discipline:** every code task is RED → GREEN. If a "failing" test passes before you write the implementation, stop and reconsider the diagnosis — do not relabel it.
- **Determinism is the contract:** all randomness derives from the seed via `StdRng::seed_from_u64`. Never use `thread_rng` in the new ops.
- **`index` vs `position`:** `index` (0..=77) is a card's identity; `position` is where it sits in the pile (`--at`). They are different numbers — keep them distinct in code and messages.
- **Out of scope (do not build):** card meanings/lookup, `cut`/`peek`/`reset`, returning a card to the deck, the table/spread model, and the kairos-ffi wiring (the X round). See the spec's "Out of scope" section.
```
