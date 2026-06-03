//! Expression parser for `kairos iching compose`.
//!
//! Grammar:
//!   expression := hex (CHANGE hex)?
//!   hex        := number | trigram OVER trigram | trigram UNDER trigram
//!   trigram    := "the"? name
//!   OVER       := "over"
//!   UNDER      := "under"
//!   CHANGE     := "->" | "→" | ">" | "changing to"

use crate::iching::data::{HexagramRecord, TrigramRecord};
use crate::iching::hexagram_data::lookup_hexagram;
use crate::iching::trigram::lookup_trigram;

/// Result of parsing a compose expression.
#[derive(Debug)]
pub enum Expr {
    Trigram(&'static TrigramRecord),
    Hexagram(&'static HexagramRecord),
    Transition {
        source: &'static HexagramRecord,
        destination: &'static HexagramRecord,
    },
}

/// User-facing parse errors. Each variant carries enough context for a
/// helpful message at the CLI boundary.
#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
    Empty,
    UnknownName(String),
    HexagramOutOfRange(u32),
    SingletonInTransition(String),
    TooManyTrigrams,
    StackOnNonTrigrams,
    UnexpectedToken(String),
    TrailingTokens,
    MissingOperand,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Empty => write!(f, "expression cannot be empty"),
            ParseError::UnknownName(n) => write!(
                f,
                "unknown trigram or hexagram name: '{}' — try: heaven, qián, ☰, 111, or 'the creative'",
                n
            ),
            ParseError::HexagramOutOfRange(n) => {
                write!(f, "hexagram number must be 1–64, got {}", n)
            }
            ParseError::SingletonInTransition(n) => write!(
                f,
                "transition operands must be hexagrams; '{}' is a trigram — write '{} over {}' or use a number",
                n, n, n
            ),
            ParseError::TooManyTrigrams => write!(
                f,
                "too many trigrams; a hexagram is exactly two trigrams"
            ),
            ParseError::StackOnNonTrigrams => {
                write!(f, "stacking ('over'/'under') operates on trigrams, not hexagrams")
            }
            ParseError::UnexpectedToken(t) => write!(f, "unexpected token: '{}'", t),
            ParseError::TrailingTokens => write!(f, "trailing tokens after expression"),
            ParseError::MissingOperand => write!(f, "expected another operand"),
        }
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, PartialEq, Eq)]
enum Token {
    Number(u32),
    Name(String),
    Over,
    Under,
    Change,
}

fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
    let normalized = input.trim();
    if normalized.is_empty() {
        return Err(ParseError::Empty);
    }
    let words: Vec<&str> = normalized.split_whitespace().collect();
    let mut tokens = Vec::with_capacity(words.len());
    let mut i = 0;
    while i < words.len() {
        let w = words[i];
        let lower = w.to_lowercase();
        match lower.as_str() {
            "the" => {
                // Article: skip; next word forms a Name if present.
                i += 1;
                continue;
            }
            "over" => tokens.push(Token::Over),
            "under" => tokens.push(Token::Under),
            "->" | "→" | ">" => tokens.push(Token::Change),
            "changing" => {
                if i + 1 < words.len() && words[i + 1].eq_ignore_ascii_case("to") {
                    tokens.push(Token::Change);
                    i += 2;
                    continue;
                }
                return Err(ParseError::UnexpectedToken(w.to_string()));
            }
            _ => {
                if let Ok(n) = w.parse::<u32>() {
                    tokens.push(Token::Number(n));
                } else {
                    tokens.push(Token::Name(w.to_string()));
                }
            }
        }
        i += 1;
    }
    if tokens.is_empty() {
        return Err(ParseError::Empty);
    }
    Ok(tokens)
}

/// Internal intermediate: a single "side" of an optional transition.
enum Unit {
    Number(u32),
    SingletonTrigram(String),
    Composition { upper: String, lower: String },
}

fn parse_unit(tokens: &[Token], cursor: &mut usize) -> Result<Unit, ParseError> {
    if *cursor >= tokens.len() {
        return Err(ParseError::MissingOperand);
    }
    match &tokens[*cursor] {
        Token::Number(n) => {
            *cursor += 1;
            if matches!(tokens.get(*cursor), Some(Token::Over) | Some(Token::Under)) {
                return Err(ParseError::StackOnNonTrigrams);
            }
            Ok(Unit::Number(*n))
        }
        Token::Name(name) => {
            let first = name.clone();
            *cursor += 1;
            // Peek for OVER/UNDER.
            match tokens.get(*cursor) {
                Some(Token::Over) => {
                    *cursor += 1;
                    let second = expect_name(tokens, cursor)?;
                    if matches!(tokens.get(*cursor), Some(Token::Over) | Some(Token::Under)) {
                        return Err(ParseError::TooManyTrigrams);
                    }
                    Ok(Unit::Composition {
                        upper: first,
                        lower: second,
                    })
                }
                Some(Token::Under) => {
                    *cursor += 1;
                    let second = expect_name(tokens, cursor)?;
                    if matches!(tokens.get(*cursor), Some(Token::Over) | Some(Token::Under)) {
                        return Err(ParseError::TooManyTrigrams);
                    }
                    Ok(Unit::Composition {
                        upper: second,
                        lower: first,
                    })
                }
                Some(Token::Number(_)) => Err(ParseError::UnexpectedToken(format!(
                    "{:?}",
                    tokens[*cursor]
                ))),
                _ => Ok(Unit::SingletonTrigram(first)),
            }
        }
        Token::Over | Token::Under => Err(ParseError::UnexpectedToken("over/under".into())),
        Token::Change => Err(ParseError::MissingOperand),
    }
}

fn expect_name(tokens: &[Token], cursor: &mut usize) -> Result<String, ParseError> {
    match tokens.get(*cursor) {
        Some(Token::Name(n)) => {
            let s = n.clone();
            *cursor += 1;
            Ok(s)
        }
        Some(Token::Number(_)) => Err(ParseError::StackOnNonTrigrams),
        _ => Err(ParseError::MissingOperand),
    }
}

fn unit_to_hex(unit: Unit) -> Result<&'static HexagramRecord, ParseError> {
    match unit {
        Unit::Number(n) => {
            if !(1..=64).contains(&n) {
                return Err(ParseError::HexagramOutOfRange(n));
            }
            lookup_hexagram(n as u8).ok_or(ParseError::HexagramOutOfRange(n))
        }
        Unit::SingletonTrigram(name) => Err(ParseError::SingletonInTransition(name)),
        Unit::Composition { upper, lower } => {
            let u = lookup_trigram(&upper).ok_or_else(|| ParseError::UnknownName(upper.clone()))?;
            let l = lookup_trigram(&lower).ok_or_else(|| ParseError::UnknownName(lower.clone()))?;
            // Hexagram binary is stored bottom-to-top: lower trigram first.
            let binary = format!("{}{}", l.binary, u.binary);
            crate::iching::data::lookup_hexagram_by_binary(&binary)
                .ok_or(ParseError::UnknownName(format!("{} / {}", upper, lower)))
        }
    }
}

/// Parse a compose expression into an `Expr`.
pub fn parse(input: &str) -> Result<Expr, ParseError> {
    let tokens = tokenize(input)?;
    let mut cursor = 0usize;
    let first = parse_unit(&tokens, &mut cursor)?;

    if cursor < tokens.len() {
        // Expect a CHANGE token next.
        match tokens.get(cursor) {
            Some(Token::Change) => cursor += 1,
            Some(t) => return Err(ParseError::UnexpectedToken(format!("{:?}", t))),
            None => unreachable!(),
        }
        let second = parse_unit(&tokens, &mut cursor)?;
        if cursor < tokens.len() {
            return Err(ParseError::TrailingTokens);
        }
        let source = unit_to_hex(first)?;
        let destination = unit_to_hex(second)?;
        return Ok(Expr::Transition {
            source,
            destination,
        });
    }

    // Singleton expression (no transition).
    match first {
        Unit::Number(n) => {
            if !(1..=64).contains(&n) {
                return Err(ParseError::HexagramOutOfRange(n));
            }
            let h = lookup_hexagram(n as u8).ok_or(ParseError::HexagramOutOfRange(n))?;
            Ok(Expr::Hexagram(h))
        }
        Unit::SingletonTrigram(name) => {
            let t = lookup_trigram(&name).ok_or(ParseError::UnknownName(name))?;
            Ok(Expr::Trigram(t))
        }
        Unit::Composition { upper, lower } => {
            let u = lookup_trigram(&upper).ok_or_else(|| ParseError::UnknownName(upper.clone()))?;
            let l = lookup_trigram(&lower).ok_or_else(|| ParseError::UnknownName(lower.clone()))?;
            // Hexagram binary is stored bottom-to-top: lower trigram first.
            let binary = format!("{}{}", l.binary, u.binary);
            let h = crate::iching::data::lookup_hexagram_by_binary(&binary)
                .ok_or(ParseError::UnknownName(format!("{} / {}", upper, lower)))?;
            Ok(Expr::Hexagram(h))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn singleton_number() {
        match parse("44").unwrap() {
            Expr::Hexagram(h) => assert_eq!(h.number, 44),
            _ => panic!("expected Hexagram"),
        }
    }

    #[test]
    fn singleton_trigram_english() {
        match parse("heaven").unwrap() {
            Expr::Trigram(t) => assert_eq!(t.english, "Heaven"),
            _ => panic!("expected Trigram"),
        }
    }

    #[test]
    fn singleton_trigram_with_article() {
        match parse("the creative").unwrap() {
            Expr::Trigram(t) => assert_eq!(t.english, "Heaven"),
            _ => panic!("expected Trigram"),
        }
    }

    #[test]
    fn empty_input_errors() {
        assert!(matches!(parse(""), Err(ParseError::Empty)));
        assert!(matches!(parse("   "), Err(ParseError::Empty)));
    }

    #[test]
    fn composition_over() {
        // Heaven (111) over Thunder (100) = bottom-to-top 100+111 = hex 25 (Innocence).
        match parse("heaven over thunder").unwrap() {
            Expr::Hexagram(h) => assert_eq!(h.number, 25),
            _ => panic!("expected Hexagram"),
        }
    }

    #[test]
    fn composition_under_swaps_position() {
        // "Thunder under Lake" = Thunder is lower, Lake is upper.
        // bottom-to-top: Thunder(100) + Lake(110) = hex 17 (Following).
        match parse("the arousing under lake").unwrap() {
            Expr::Hexagram(h) => assert_eq!(h.number, 17),
            _ => panic!("expected Hexagram"),
        }
    }

    #[test]
    fn composition_by_symbol() {
        // ☰ (Heaven) over ☵ (Water) = bottom-to-top 010+111 = hex 6 (Conflict).
        match parse("☰ over ☵").unwrap() {
            Expr::Hexagram(h) => assert_eq!(h.number, 6),
            _ => panic!("expected Hexagram"),
        }
    }

    #[test]
    fn composition_by_quality_with_article() {
        // "the creative over the abysmal" = Heaven over Water = hex 6.
        match parse("the creative over the abysmal").unwrap() {
            Expr::Hexagram(h) => assert_eq!(h.number, 6),
            _ => panic!("expected Hexagram"),
        }
    }

    #[test]
    fn transition_number_to_number_arrow() {
        match parse("44 -> 26").unwrap() {
            Expr::Transition {
                source,
                destination,
            } => {
                assert_eq!(source.number, 44);
                assert_eq!(destination.number, 26);
            }
            _ => panic!("expected Transition"),
        }
    }

    #[test]
    fn transition_accepts_unicode_arrow() {
        match parse("44 → 26").unwrap() {
            Expr::Transition {
                source,
                destination,
            } => {
                assert_eq!(source.number, 44);
                assert_eq!(destination.number, 26);
            }
            _ => panic!("expected Transition"),
        }
    }

    #[test]
    fn transition_accepts_gt_as_arrow() {
        match parse("44 > 26").unwrap() {
            Expr::Transition {
                source,
                destination,
            } => {
                assert_eq!(source.number, 44);
                assert_eq!(destination.number, 26);
            }
            _ => panic!("expected Transition"),
        }
    }

    #[test]
    fn transition_accepts_changing_to() {
        match parse("44 changing to 26").unwrap() {
            Expr::Transition {
                source,
                destination,
            } => {
                assert_eq!(source.number, 44);
                assert_eq!(destination.number, 26);
            }
            _ => panic!("expected Transition"),
        }
    }

    #[test]
    fn transition_number_to_composition() {
        // 44 changing to thunder over mountain.
        // Thunder over Mountain: upper=Thunder(100), lower=Mountain(001).
        // Binary bottom-to-top: 001 + 100 = "001100" = hex 62 (Preponderance of the Small).
        match parse("44 changing to thunder over mountain").unwrap() {
            Expr::Transition {
                source,
                destination,
            } => {
                assert_eq!(source.number, 44);
                assert_eq!(destination.number, 62);
            }
            _ => panic!("expected Transition"),
        }
    }

    #[test]
    fn transition_composition_to_composition() {
        // "the creative over the abysmal" = Heaven over Water:
        //   upper=Heaven(111), lower=Water(010), binary = 010+111 = "010111" = hex 6.
        // "the gentle under the mountain" = Wind under Mountain (Wind lower, Mountain upper):
        //   upper=Mountain(001), lower=Wind(011), binary = 011+001 = "011001" = hex 18.
        match parse("the creative over the abysmal changing to the gentle under the mountain")
            .unwrap()
        {
            Expr::Transition {
                source,
                destination,
            } => {
                assert_eq!(source.number, 6);
                assert_eq!(destination.number, 18);
            }
            _ => panic!("expected Transition"),
        }
    }

    #[test]
    fn err_unknown_name() {
        match parse("foo") {
            Err(ParseError::UnknownName(n)) => assert_eq!(n, "foo"),
            other => panic!("expected UnknownName, got {:?}", other),
        }
    }

    #[test]
    fn err_unknown_name_in_composition() {
        match parse("foo over thunder") {
            Err(ParseError::UnknownName(n)) => assert_eq!(n, "foo"),
            other => panic!("expected UnknownName, got {:?}", other),
        }
    }

    #[test]
    fn err_singleton_in_transition() {
        // "heaven -> earth" — both sides are bare trigrams, not hex operands.
        match parse("heaven -> earth") {
            Err(ParseError::SingletonInTransition(n)) => assert_eq!(n, "heaven"),
            other => panic!("expected SingletonInTransition, got {:?}", other),
        }
    }

    #[test]
    fn err_hexagram_out_of_range() {
        match parse("88") {
            Err(ParseError::HexagramOutOfRange(88)) => {}
            other => panic!("expected HexagramOutOfRange(88), got {:?}", other),
        }
        match parse("0") {
            Err(ParseError::HexagramOutOfRange(0)) => {}
            other => panic!("expected HexagramOutOfRange(0), got {:?}", other),
        }
    }

    #[test]
    fn err_stack_on_numbers() {
        // "1 over 26" — stacking is for trigrams; numbers are hexagrams.
        match parse("1 over 26") {
            Err(ParseError::StackOnNonTrigrams) => {}
            other => panic!("expected StackOnNonTrigrams, got {:?}", other),
        }
    }

    #[test]
    fn err_too_many_trigrams() {
        match parse("heaven over thunder over lake") {
            Err(ParseError::TooManyTrigrams) => {}
            other => panic!("expected TooManyTrigrams, got {:?}", other),
        }
    }

    #[test]
    fn err_the_alone_is_empty() {
        match parse("the") {
            Err(ParseError::Empty) => {}
            other => panic!("expected Empty, got {:?}", other),
        }
    }

    #[test]
    fn err_trailing_tokens() {
        // After a complete expression, no more tokens permitted.
        match parse("44 -> 26 over thunder") {
            Err(_) => {} // accept any error — could be TrailingTokens or StackOnNonTrigrams
            Ok(e) => panic!("expected error, got {:?}", e),
        }
    }

    #[test]
    fn err_missing_operand_after_change() {
        match parse("44 ->") {
            Err(ParseError::MissingOperand) => {}
            other => panic!("expected MissingOperand, got {:?}", other),
        }
    }

    #[test]
    fn err_missing_operand_after_over() {
        match parse("heaven over") {
            Err(ParseError::MissingOperand) => {}
            other => panic!("expected MissingOperand, got {:?}", other),
        }
    }
}
