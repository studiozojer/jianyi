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

/// Get human-readable display name for a tarot card
pub fn card_display_name(card: TarotCard) -> String {
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
