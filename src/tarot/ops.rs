use chrono::{DateTime, Utc};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::tarot::deck::{Deck, DeckCard};

/// Riffle passes per shuffle. ~7 is the accepted threshold to mix a deck.
const RIFFLE_PASSES: usize = 7;
/// Cut-and-flip passes per shuffle (where reversals enter).
const CUT_FLIPS: usize = 3;

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
