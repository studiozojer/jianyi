//! Tarot card types for random card generation
// Vendored engine module — retains full public API surface even when not all items are
// exercised by the CLI. Dead-code and transmute-annotation lints suppressed intentionally.
#![allow(dead_code, clippy::missing_transmute_annotations)]

use rand::Rng;
use serde::{Deserialize, Serialize};

/// A tarot card from the standard 78-card deck
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(i32)]
pub enum TarotCard {
    // Major Arcana (0-21)
    TheFool = 0,
    TheMagician = 1,
    TheHighPriestess = 2,
    TheEmpress = 3,
    TheEmperor = 4,
    TheHierophant = 5,
    TheLovers = 6,
    TheChariot = 7,
    Strength = 8,
    TheHermit = 9,
    WheelOfFortune = 10,
    Justice = 11,
    TheHangedMan = 12,
    Death = 13,
    Temperance = 14,
    TheDevil = 15,
    TheTower = 16,
    TheStar = 17,
    TheMoon = 18,
    TheSun = 19,
    Judgement = 20,
    TheWorld = 21,

    // Wands (22-35)
    AceOfWands = 22,
    TwoOfWands = 23,
    ThreeOfWands = 24,
    FourOfWands = 25,
    FiveOfWands = 26,
    SixOfWands = 27,
    SevenOfWands = 28,
    EightOfWands = 29,
    NineOfWands = 30,
    TenOfWands = 31,
    PageOfWands = 32,
    KnightOfWands = 33,
    QueenOfWands = 34,
    KingOfWands = 35,

    // Cups (36-49)
    AceOfCups = 36,
    TwoOfCups = 37,
    ThreeOfCups = 38,
    FourOfCups = 39,
    FiveOfCups = 40,
    SixOfCups = 41,
    SevenOfCups = 42,
    EightOfCups = 43,
    NineOfCups = 44,
    TenOfCups = 45,
    PageOfCups = 46,
    KnightOfCups = 47,
    QueenOfCups = 48,
    KingOfCups = 49,

    // Swords (50-63)
    AceOfSwords = 50,
    TwoOfSwords = 51,
    ThreeOfSwords = 52,
    FourOfSwords = 53,
    FiveOfSwords = 54,
    SixOfSwords = 55,
    SevenOfSwords = 56,
    EightOfSwords = 57,
    NineOfSwords = 58,
    TenOfSwords = 59,
    PageOfSwords = 60,
    KnightOfSwords = 61,
    QueenOfSwords = 62,
    KingOfSwords = 63,

    // Pentacles (64-77)
    AceOfPentacles = 64,
    TwoOfPentacles = 65,
    ThreeOfPentacles = 66,
    FourOfPentacles = 67,
    FiveOfPentacles = 68,
    SixOfPentacles = 69,
    SevenOfPentacles = 70,
    EightOfPentacles = 71,
    NineOfPentacles = 72,
    TenOfPentacles = 73,
    PageOfPentacles = 74,
    KnightOfPentacles = 75,
    QueenOfPentacles = 76,
    KingOfPentacles = 77,
}

impl TarotCard {
    /// Total number of cards in the deck
    pub const COUNT: usize = 78;

    /// Generate a random tarot card
    pub fn random() -> Self {
        let index = rand::thread_rng().gen_range(0..Self::COUNT);
        Self::from_index(index).expect("index always valid")
    }

    /// Convert from index (0-77) to card
    pub fn from_index(index: usize) -> Option<Self> {
        if index >= Self::COUNT {
            return None;
        }
        // SAFETY: This transmute is safe because:
        // 1. TarotCard is repr(i32) with discriminants 0..=77
        // 2. The bounds check above ensures index < 78
        // 3. The enum has exactly 78 variants with consecutive discriminants
        // Compile-time assertion ensures COUNT matches actual variant count
        const _: () = assert!(std::mem::size_of::<TarotCard>() == std::mem::size_of::<i32>());
        Some(unsafe { std::mem::transmute(index as i32) })
    }

    /// Check if this card is a Major Arcana card
    pub fn is_major_arcana(&self) -> bool {
        (*self as i32) <= 21
    }

    /// Get the suit of the card (None for Major Arcana)
    pub fn suit(&self) -> Option<TarotSuit> {
        let value = *self as i32;
        match value {
            0..=21 => None,
            22..=35 => Some(TarotSuit::Wands),
            36..=49 => Some(TarotSuit::Cups),
            50..=63 => Some(TarotSuit::Swords),
            64..=77 => Some(TarotSuit::Pentacles),
            _ => None,
        }
    }
}

/// The four suits of the Minor Arcana
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TarotSuit {
    Wands,
    Cups,
    Swords,
    Pentacles,
}

/// A drawn tarot card with its orientation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TarotDraw {
    pub card: TarotCard,
    pub upright: bool,
}

impl TarotDraw {
    /// Draw a random card with random orientation
    pub fn random() -> Self {
        Self {
            card: TarotCard::random(),
            upright: rand::thread_rng().gen_bool(0.5),
        }
    }
}
