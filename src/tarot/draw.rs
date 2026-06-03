use chrono::{DateTime, Utc};
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use crate::tarot::cards::{TarotCard, TarotSuit};

/// Filter to major or minor arcana only
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArcanaFilter {
    Major,
    Minor,
}

/// A single drawn card in the output
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrawnCard {
    pub card: String,
    pub suit: String,
    pub upright: bool,
}

/// The complete result of a draw invocation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrawResult {
    pub drawn: Vec<DrawnCard>,
    pub count: usize,
    pub seed: String,
    pub arcana_filter: Option<String>,
    pub no_reversals: bool,
}

/// Draw error
#[derive(Debug, Clone)]
pub struct DrawError {
    pub message: String,
}

impl std::fmt::Display for DrawError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Draw error: {}", self.message)
    }
}

impl std::error::Error for DrawError {}

/// Draw N cards from a seeded deck, optionally filtered to major or minor arcana.
/// Same seed always produces identical cards and orientations (deterministic).
/// When `no_reversals` is true, all cards come out upright regardless of seed.
pub fn draw_cards(
    count: usize,
    filter: Option<ArcanaFilter>,
    seed: DateTime<Utc>,
    no_reversals: bool,
) -> Result<DrawResult, DrawError> {
    let deck: Vec<TarotCard> = full_deck()
        .into_iter()
        .filter(|card| match filter {
            None => true,
            Some(ArcanaFilter::Major) => card.is_major_arcana(),
            Some(ArcanaFilter::Minor) => !card.is_major_arcana(),
        })
        .collect();

    if count > deck.len() {
        return Err(DrawError {
            message: format!(
                "--count {count} exceeds filtered deck size {} (arcana filter: {:?})",
                deck.len(),
                filter
            ),
        });
    }

    let seed_u64 = seed
        .timestamp_nanos_opt()
        .unwrap_or_else(|| seed.timestamp() * 1_000_000_000) as u64;
    let mut rng = StdRng::seed_from_u64(seed_u64);

    let mut shuffled = deck;
    shuffled.shuffle(&mut rng);

    let drawn = shuffled
        .into_iter()
        .take(count)
        .map(|card| DrawnCard {
            card: card_display_name(card),
            suit: suit_string(card),
            upright: if no_reversals {
                true
            } else {
                rng.gen_bool(0.5)
            },
        })
        .collect();

    Ok(DrawResult {
        drawn,
        count,
        seed: seed.to_rfc3339(),
        arcana_filter: filter.map(|f| match f {
            ArcanaFilter::Major => "major".to_string(),
            ArcanaFilter::Minor => "minor".to_string(),
        }),
        no_reversals,
    })
}

pub(crate) fn suit_string(card: TarotCard) -> String {
    match card.suit() {
        None => "Major Arcana".to_string(),
        Some(TarotSuit::Wands) => "Wands".to_string(),
        Some(TarotSuit::Cups) => "Cups".to_string(),
        Some(TarotSuit::Swords) => "Swords".to_string(),
        Some(TarotSuit::Pentacles) => "Pentacles".to_string(),
    }
}

/// Get the full 78-card tarot deck in order
pub(crate) fn full_deck() -> Vec<TarotCard> {
    (0..78).filter_map(TarotCard::from_index).collect()
}

/// Get human-readable display name for a tarot card
pub(crate) fn card_display_name(card: TarotCard) -> String {
    match card {
        // Major Arcana
        TarotCard::TheFool => "The Fool".to_string(),
        TarotCard::TheMagician => "The Magician".to_string(),
        TarotCard::TheHighPriestess => "The High Priestess".to_string(),
        TarotCard::TheEmpress => "The Empress".to_string(),
        TarotCard::TheEmperor => "The Emperor".to_string(),
        TarotCard::TheHierophant => "The Hierophant".to_string(),
        TarotCard::TheLovers => "The Lovers".to_string(),
        TarotCard::TheChariot => "The Chariot".to_string(),
        TarotCard::Strength => "Strength".to_string(),
        TarotCard::TheHermit => "The Hermit".to_string(),
        TarotCard::WheelOfFortune => "Wheel of Fortune".to_string(),
        TarotCard::Justice => "Justice".to_string(),
        TarotCard::TheHangedMan => "The Hanged Man".to_string(),
        TarotCard::Death => "Death".to_string(),
        TarotCard::Temperance => "Temperance".to_string(),
        TarotCard::TheDevil => "The Devil".to_string(),
        TarotCard::TheTower => "The Tower".to_string(),
        TarotCard::TheStar => "The Star".to_string(),
        TarotCard::TheMoon => "The Moon".to_string(),
        TarotCard::TheSun => "The Sun".to_string(),
        TarotCard::Judgement => "Judgement".to_string(),
        TarotCard::TheWorld => "The World".to_string(),

        // Wands (22-35)
        TarotCard::AceOfWands => "Ace of Wands".to_string(),
        TarotCard::TwoOfWands => "Two of Wands".to_string(),
        TarotCard::ThreeOfWands => "Three of Wands".to_string(),
        TarotCard::FourOfWands => "Four of Wands".to_string(),
        TarotCard::FiveOfWands => "Five of Wands".to_string(),
        TarotCard::SixOfWands => "Six of Wands".to_string(),
        TarotCard::SevenOfWands => "Seven of Wands".to_string(),
        TarotCard::EightOfWands => "Eight of Wands".to_string(),
        TarotCard::NineOfWands => "Nine of Wands".to_string(),
        TarotCard::TenOfWands => "Ten of Wands".to_string(),
        TarotCard::PageOfWands => "Page of Wands".to_string(),
        TarotCard::KnightOfWands => "Knight of Wands".to_string(),
        TarotCard::QueenOfWands => "Queen of Wands".to_string(),
        TarotCard::KingOfWands => "King of Wands".to_string(),

        // Cups (36-49)
        TarotCard::AceOfCups => "Ace of Cups".to_string(),
        TarotCard::TwoOfCups => "Two of Cups".to_string(),
        TarotCard::ThreeOfCups => "Three of Cups".to_string(),
        TarotCard::FourOfCups => "Four of Cups".to_string(),
        TarotCard::FiveOfCups => "Five of Cups".to_string(),
        TarotCard::SixOfCups => "Six of Cups".to_string(),
        TarotCard::SevenOfCups => "Seven of Cups".to_string(),
        TarotCard::EightOfCups => "Eight of Cups".to_string(),
        TarotCard::NineOfCups => "Nine of Cups".to_string(),
        TarotCard::TenOfCups => "Ten of Cups".to_string(),
        TarotCard::PageOfCups => "Page of Cups".to_string(),
        TarotCard::KnightOfCups => "Knight of Cups".to_string(),
        TarotCard::QueenOfCups => "Queen of Cups".to_string(),
        TarotCard::KingOfCups => "King of Cups".to_string(),

        // Swords (50-63)
        TarotCard::AceOfSwords => "Ace of Swords".to_string(),
        TarotCard::TwoOfSwords => "Two of Swords".to_string(),
        TarotCard::ThreeOfSwords => "Three of Swords".to_string(),
        TarotCard::FourOfSwords => "Four of Swords".to_string(),
        TarotCard::FiveOfSwords => "Five of Swords".to_string(),
        TarotCard::SixOfSwords => "Six of Swords".to_string(),
        TarotCard::SevenOfSwords => "Seven of Swords".to_string(),
        TarotCard::EightOfSwords => "Eight of Swords".to_string(),
        TarotCard::NineOfSwords => "Nine of Swords".to_string(),
        TarotCard::TenOfSwords => "Ten of Swords".to_string(),
        TarotCard::PageOfSwords => "Page of Swords".to_string(),
        TarotCard::KnightOfSwords => "Knight of Swords".to_string(),
        TarotCard::QueenOfSwords => "Queen of Swords".to_string(),
        TarotCard::KingOfSwords => "King of Swords".to_string(),

        // Pentacles (64-77)
        TarotCard::AceOfPentacles => "Ace of Pentacles".to_string(),
        TarotCard::TwoOfPentacles => "Two of Pentacles".to_string(),
        TarotCard::ThreeOfPentacles => "Three of Pentacles".to_string(),
        TarotCard::FourOfPentacles => "Four of Pentacles".to_string(),
        TarotCard::FiveOfPentacles => "Five of Pentacles".to_string(),
        TarotCard::SixOfPentacles => "Six of Pentacles".to_string(),
        TarotCard::SevenOfPentacles => "Seven of Pentacles".to_string(),
        TarotCard::EightOfPentacles => "Eight of Pentacles".to_string(),
        TarotCard::NineOfPentacles => "Nine of Pentacles".to_string(),
        TarotCard::TenOfPentacles => "Ten of Pentacles".to_string(),
        TarotCard::PageOfPentacles => "Page of Pentacles".to_string(),
        TarotCard::KnightOfPentacles => "Knight of Pentacles".to_string(),
        TarotCard::QueenOfPentacles => "Queen of Pentacles".to_string(),
        TarotCard::KingOfPentacles => "King of Pentacles".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_draw_result_fields_exist() {
        let result = DrawResult {
            drawn: vec![],
            count: 0,
            seed: "2026-05-07T12:43:23Z".to_string(),
            arcana_filter: None,
            no_reversals: false,
        };
        assert_eq!(result.count, 0);
        assert!(result.drawn.is_empty());
        assert!(result.arcana_filter.is_none());
        assert!(!result.no_reversals);
    }

    #[test]
    fn test_drawn_card_fields_exist() {
        let card = DrawnCard {
            card: "The Tower".to_string(),
            suit: "Major Arcana".to_string(),
            upright: false,
        };
        assert_eq!(card.card, "The Tower");
        assert_eq!(card.suit, "Major Arcana");
        assert!(!card.upright);
    }

    #[test]
    fn test_suit_string_major() {
        assert_eq!(suit_string(TarotCard::TheTower), "Major Arcana");
        assert_eq!(suit_string(TarotCard::TheFool), "Major Arcana");
        assert_eq!(suit_string(TarotCard::TheWorld), "Major Arcana");
    }

    #[test]
    fn test_suit_string_minor() {
        assert_eq!(suit_string(TarotCard::AceOfWands), "Wands");
        assert_eq!(suit_string(TarotCard::KingOfCups), "Cups");
        assert_eq!(suit_string(TarotCard::TwoOfSwords), "Swords");
        assert_eq!(suit_string(TarotCard::FiveOfPentacles), "Pentacles");
    }

    #[test]
    fn test_draw_single_card_default() {
        let seed = Utc.with_ymd_and_hms(2026, 5, 7, 12, 43, 23).unwrap();
        let result = draw_cards(1, None, seed, false).unwrap();
        assert_eq!(result.count, 1);
        assert_eq!(result.drawn.len(), 1);
        assert!(result.arcana_filter.is_none());
        assert!(!result.no_reversals);
    }

    #[test]
    fn test_draw_determinism() {
        let seed = Utc.with_ymd_and_hms(2026, 5, 7, 12, 43, 23).unwrap();
        let result1 = draw_cards(3, None, seed, false).unwrap();
        let result2 = draw_cards(3, None, seed, false).unwrap();
        assert_eq!(result1.drawn[0].card, result2.drawn[0].card);
        assert_eq!(result1.drawn[1].card, result2.drawn[1].card);
        assert_eq!(result1.drawn[2].card, result2.drawn[2].card);
        assert_eq!(result1.drawn[0].upright, result2.drawn[0].upright);
    }

    #[test]
    fn test_draw_no_duplicates() {
        let seed = Utc.with_ymd_and_hms(2026, 5, 7, 12, 43, 23).unwrap();
        let result = draw_cards(78, None, seed, false).unwrap();
        let mut names: Vec<&str> = result.drawn.iter().map(|c| c.card.as_str()).collect();
        let original_len = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), original_len, "drew duplicate cards");
    }

    #[test]
    fn test_draw_major_filter() {
        let seed = Utc.with_ymd_and_hms(2026, 5, 7, 12, 43, 23).unwrap();
        let result = draw_cards(22, Some(ArcanaFilter::Major), seed, false).unwrap();
        assert_eq!(result.drawn.len(), 22);
        assert!(result.drawn.iter().all(|c| c.suit == "Major Arcana"));
        assert_eq!(result.arcana_filter, Some("major".to_string()));
    }

    #[test]
    fn test_draw_minor_filter() {
        let seed = Utc.with_ymd_and_hms(2026, 5, 7, 12, 43, 23).unwrap();
        let result = draw_cards(5, Some(ArcanaFilter::Minor), seed, false).unwrap();
        assert_eq!(result.drawn.len(), 5);
        assert!(result.drawn.iter().all(|c| c.suit != "Major Arcana"));
        assert_eq!(result.arcana_filter, Some("minor".to_string()));
    }

    #[test]
    fn test_draw_count_exceeds_deck_returns_error() {
        let seed = Utc.with_ymd_and_hms(2026, 5, 7, 12, 43, 23).unwrap();
        let result = draw_cards(79, None, seed, false);
        assert!(result.is_err());
        let result = draw_cards(23, Some(ArcanaFilter::Major), seed, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_draw_seed_echoed_in_output() {
        let seed = Utc.with_ymd_and_hms(2026, 5, 7, 12, 43, 23).unwrap();
        let result = draw_cards(1, None, seed, false).unwrap();
        assert!(!result.seed.is_empty());
        assert!(result.seed.contains("2026-05-07"));
    }

    #[test]
    fn test_draw_no_reversals_all_upright() {
        let seed = Utc.with_ymd_and_hms(2026, 5, 7, 12, 43, 23).unwrap();
        let result = draw_cards(10, None, seed, true).unwrap();
        assert!(
            result.drawn.iter().all(|c| c.upright),
            "all cards should be upright with no_reversals"
        );
        assert!(result.no_reversals);
    }

    #[test]
    fn test_draw_no_reversals_same_cards_as_default() {
        let seed = Utc.with_ymd_and_hms(2026, 5, 7, 12, 43, 23).unwrap();
        let with_reversals = draw_cards(10, None, seed, false).unwrap();
        let without_reversals = draw_cards(10, None, seed, true).unwrap();
        let cards_with: Vec<&str> = with_reversals
            .drawn
            .iter()
            .map(|c| c.card.as_str())
            .collect();
        let cards_without: Vec<&str> = without_reversals
            .drawn
            .iter()
            .map(|c| c.card.as_str())
            .collect();
        assert_eq!(
            cards_with, cards_without,
            "same seed should produce same cards regardless of no_reversals"
        );
    }
}
