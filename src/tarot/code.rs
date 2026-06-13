use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;

use crate::tarot::cards::TarotCard;
use crate::tarot::deck::{Deck, DeckCard};

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
    format!("{VERSION_PREFIX}{}", URL_SAFE_NO_PAD.encode(&bytes))
}

/// Parse a `v1.<base64>` deck code back into a `Deck`.
pub fn decode(code: &str) -> Result<Deck, CodeError> {
    let body = code.strip_prefix(VERSION_PREFIX).ok_or(CodeError::BadVersion)?;
    let bytes = URL_SAFE_NO_PAD
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tarot::deck::ArcanaFilter;

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
        let code = format!("{VERSION_PREFIX}{}", URL_SAFE_NO_PAD.encode([3u8]));
        assert_eq!(decode(&code), Err(CodeError::Truncated));
    }
}
