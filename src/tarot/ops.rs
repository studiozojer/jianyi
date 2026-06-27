use chrono::{DateTime, Utc};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use serde::{Deserialize, Serialize};

use crate::tarot::cards::TarotCard;
use crate::tarot::deck::{card_display_name, suit_string, Deck, DeckCard};

/// Riffle passes per shuffle. ~7 is the accepted threshold to mix a deck.
const RIFFLE_PASSES: usize = 7;
/// Cut-and-flip passes per shuffle (where reversals enter).
const CUT_FLIPS: usize = 3;

fn seed_to_u64(seed: DateTime<Utc>) -> u64 {
    seed.timestamp_nanos_opt()
        .unwrap_or_else(|| seed.timestamp() * 1_000_000_000) as u64
}

/// Proportional GSR interleave of two ordered piles: drop from each side with
/// probability proportional to its remaining size; within-pile order preserved.
/// Orientation is untouched.
fn interleave(left: &[DeckCard], right: &[DeckCard], rng: &mut StdRng) -> Vec<DeckCard> {
    let (mut li, mut ri) = (0usize, 0usize);
    let mut out = Vec::with_capacity(left.len() + right.len());
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

/// One Gilbert–Shannon–Reeds riffle: split near the middle (with jitter), then
/// interleave proportionally. Orientation is untouched here.
fn riffle(cards: &[DeckCard], rng: &mut StdRng) -> Vec<DeckCard> {
    let n = cards.len();
    if n < 2 {
        return cards.to_vec();
    }
    let cut = (n / 2 + rng.gen_range(0..=2)).clamp(1, n - 1);
    let (left, right) = cards.split_at(cut);
    interleave(left, right, rng)
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
    if no_reversals {
        for c in &mut cards {
            c.reversed = false;
        }
    }
    Deck { cards }
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReturnError {
    BadIndex(u8),
    Duplicate(u8),
}

impl std::fmt::Display for ReturnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReturnError::BadIndex(i) => {
                write!(f, "no card with index {i}; valid indices are 0..78")
            }
            ReturnError::Duplicate(i) => {
                write!(f, "card index {i} is already in the deck; cannot return it twice")
            }
        }
    }
}

impl std::error::Error for ReturnError {}

/// Place a card on the **bottom** of the pile — the far end from the draw point
/// (`cards[0]` is the top, so the bottom is the end of the vec). This is the
/// inverse of a top draw: a card drawn out can be set back under the deck, and
/// the deck code keeps tracking the full order. `reversed` is the orientation
/// the card lands with. Errors if the index isn't a real card, or the card is
/// already in the deck (a deck can't hold the same card twice).
pub fn return_to_bottom(deck: &Deck, index: u8, reversed: bool) -> Result<Deck, ReturnError> {
    let card = TarotCard::from_index(index as usize).ok_or(ReturnError::BadIndex(index))?;
    if deck.cards.iter().any(|dc| dc.card == card) {
        return Err(ReturnError::Duplicate(index));
    }
    let mut cards = deck.cards.clone();
    cards.push(DeckCard { card, reversed });
    Ok(Deck { cards })
}

/// Cut the pile: keep the top `n` cards as the upper half, the remainder as the
/// lower. Returns `(top, bottom)`. `n` is clamped to `0..=len`. Pure slicing —
/// `merge(&top, &bottom)` restores the original order.
pub fn split(deck: &Deck, keep_top: usize) -> (Deck, Deck) {
    let n = keep_top.min(deck.cards.len());
    (
        Deck { cards: deck.cards[..n].to_vec() },
        Deck { cards: deck.cards[n..].to_vec() },
    )
}

/// Set the `top` pile back above the `bottom` pile (`top.cards` then
/// `bottom.cards`). The inverse of `split`.
pub fn merge(top: &Deck, bottom: &Deck) -> Deck {
    let mut cards = Vec::with_capacity(top.cards.len() + bottom.cards.len());
    cards.extend_from_slice(&top.cards);
    cards.extend_from_slice(&bottom.cards);
    Deck { cards }
}

/// Place a card on the **top** of the pile (`cards[0]`) — the inverse of a top
/// draw, mirroring [`return_to_bottom`]. Errors if the index isn't a real card,
/// or the card is already in the deck.
pub fn return_to_top(deck: &Deck, index: u8, reversed: bool) -> Result<Deck, ReturnError> {
    let card = TarotCard::from_index(index as usize).ok_or(ReturnError::BadIndex(index))?;
    if deck.cards.iter().any(|dc| dc.card == card) {
        return Err(ReturnError::Duplicate(index));
    }
    let mut cards = Vec::with_capacity(deck.cards.len() + 1);
    cards.push(DeckCard { card, reversed });
    cards.extend_from_slice(&deck.cards);
    Ok(Deck { cards })
}

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

    #[test]
    fn no_reversals_clears_prior_reversals_on_reshuffle() {
        // A deck that already carries reversals from a prior shuffle...
        let with_rev = shuffle(&Deck::new(None), seed(), false);
        assert!(
            with_rev.cards.iter().any(|c| c.reversed),
            "precondition: the with-reversals shuffle should produce some reversed cards"
        );
        // ...must come out entirely upright when reshuffled with no_reversals.
        // Use a distinct seed (offset +13s) that is known to leave reversed cards
        // in the top segment of cut_and_flip when the bug is present.
        let reshuffle_seed = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 13).unwrap();
        let reshuffled = shuffle(&with_rev, reshuffle_seed, true);
        assert!(reshuffled.cards.iter().all(|c| !c.reversed));
        assert_eq!(reshuffled.len(), 78);
    }

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
        // the card formerly at 39 is gone from the pile
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

    #[test]
    fn return_appends_to_the_bottom() {
        // Draw the top card off a fresh deck, then return it: it should land at
        // the very bottom (last position), not back on top.
        let deck = Deck::new(None);
        let (drawn, rest) = draw(&deck, Select::Top(1)).unwrap();
        assert_eq!(drawn[0].index, 0); // The Fool was on top
        let returned = return_to_bottom(&rest, drawn[0].index, drawn[0].reversed).unwrap();
        assert_eq!(returned.len(), 78);
        assert_eq!(returned.cards.last().unwrap().card as u8, 0);
        assert_ne!(returned.cards[0].card as u8, 0, "returned card must not be on top");
    }

    #[test]
    fn return_honors_orientation() {
        let deck = Deck::new(Some(crate::tarot::deck::ArcanaFilter::Major));
        let (drawn, rest) = draw(&deck, Select::Top(1)).unwrap();
        let returned = return_to_bottom(&rest, drawn[0].index, true).unwrap();
        let bottom = returned.cards.last().unwrap();
        assert_eq!(bottom.card as u8, drawn[0].index);
        assert!(bottom.reversed, "returned card should carry the given orientation");
    }

    #[test]
    fn return_round_trip_preserves_the_card_set() {
        // Draw three off the top, return all three: the deck is whole again.
        let deck = shuffle(&Deck::new(None), seed(), false);
        let (drawn, mut rest) = draw(&deck, Select::Top(3)).unwrap();
        for d in &drawn {
            rest = return_to_bottom(&rest, d.index, d.reversed).unwrap();
        }
        assert_eq!(rest.len(), 78);
        let mut idx: Vec<u8> = rest.cards.iter().map(|c| c.card as u8).collect();
        idx.sort_unstable();
        idx.dedup();
        assert_eq!(idx.len(), 78, "round-trip lost or duplicated cards");
    }

    #[test]
    fn return_duplicate_is_rejected() {
        // The Fool (index 0) is still on top of a fresh deck.
        let deck = Deck::new(None);
        assert_eq!(return_to_bottom(&deck, 0, false), Err(ReturnError::Duplicate(0)));
    }

    #[test]
    fn return_bad_index_is_rejected() {
        let deck = Deck::new(None);
        assert_eq!(return_to_bottom(&deck, 78, false), Err(ReturnError::BadIndex(78)));
    }

    #[test]
    fn split_then_merge_restores_original_order() {
        let deck = shuffle(&Deck::new(None), seed(), false);
        let (top, bottom) = split(&deck, 39);
        assert_eq!(top.len(), 39);
        assert_eq!(bottom.len(), 39);
        // top's first card is the deck's top card (the one you'd be holding)
        assert_eq!(top.cards[0], deck.cards[0]);
        // merge(top, bottom) puts the held top half back above the bottom half
        assert_eq!(merge(&top, &bottom), deck);
    }

    #[test]
    fn split_keep_top_is_clamped_and_handles_odd() {
        let deck = Deck::new(None); // 78
        let (t, b) = split(&deck, 40);
        assert_eq!((t.len(), b.len()), (40, 38));
        // clamp: keep_top beyond len yields the whole deck on top, empty bottom
        let (t2, b2) = split(&deck, 999);
        assert_eq!((t2.len(), b2.len()), (78, 0));
        // keep_top 0 → empty top, whole bottom
        let (t3, b3) = split(&deck, 0);
        assert_eq!((t3.len(), b3.len()), (0, 78));
    }

    #[test]
    fn return_to_top_places_card_on_top_and_is_inverse_of_top_draw() {
        let deck = Deck::new(None);
        let (drawn, rest) = draw(&deck, Select::Top(1)).unwrap(); // The Fool off the top
        assert_eq!(drawn[0].index, 0);
        let restored = return_to_top(&rest, drawn[0].index, drawn[0].reversed).unwrap();
        assert_eq!(restored.len(), 78);
        assert_eq!(restored.cards[0].card as u8, 0, "returned card must land on top");
        assert_eq!(restored, deck, "return_to_top inverts a top draw exactly");
    }

    #[test]
    fn return_to_top_rejects_duplicate_and_bad_index() {
        let deck = Deck::new(None); // The Fool (0) already on top
        assert_eq!(return_to_top(&deck, 0, false), Err(ReturnError::Duplicate(0)));
        assert_eq!(return_to_top(&deck, 78, false), Err(ReturnError::BadIndex(78)));
    }
}
