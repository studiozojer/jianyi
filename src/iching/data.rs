use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

#[derive(Debug, Deserialize, Serialize)]
pub struct Rendering {
    pub chinese: String,
    pub english: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LineRecord {
    pub position: u8,
    pub chinese: String,
    pub english: String,
    /// Original interpretive reading; populated by the corpus over time.
    pub reading: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HexagramRecord {
    pub number: u8,
    pub binary: String,
    pub chinese: String,
    pub pinyin: String,
    pub english: String,
    pub upper_trigram: String,
    pub lower_trigram: String,
    pub judgment: Rendering,
    pub image: Rendering,
    pub lines: Vec<LineRecord>,
    /// Hexagram-level interpretive reading; populated by the corpus over time.
    pub hexagram_reading: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TrigramRecord {
    pub binary: String,
    pub symbol: String,
    pub pinyin: String,
    pub english: String,
    pub quality: String,
    pub family: String,
}

static HEXAGRAMS_DATA: LazyLock<Vec<HexagramRecord>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../data/hexagrams.json"))
        .expect("hexagrams.json failed to parse — data file is malformed")
});

static TRIGRAMS_DATA: LazyLock<Vec<TrigramRecord>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../data/trigrams.json"))
        .expect("trigrams.json failed to parse — data file is malformed")
});

pub fn hexagrams() -> &'static [HexagramRecord] {
    &HEXAGRAMS_DATA
}

pub fn trigrams() -> &'static [TrigramRecord] {
    &TRIGRAMS_DATA
}

pub fn lookup_hexagram(number: u8) -> Option<&'static HexagramRecord> {
    hexagrams().iter().find(|h| h.number == number)
}

pub fn lookup_hexagram_by_binary(binary: &str) -> Option<&'static HexagramRecord> {
    hexagrams().iter().find(|h| h.binary == binary)
}

pub fn lookup_trigram_by_binary(binary: &str) -> Option<&'static TrigramRecord> {
    trigrams().iter().find(|t| t.binary == binary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hexagram_count_is_64() {
        assert_eq!(hexagrams().len(), 64);
    }

    #[test]
    fn trigram_count_is_8() {
        assert_eq!(trigrams().len(), 8);
    }

    #[test]
    fn all_hexagrams_have_6_lines() {
        for h in hexagrams() {
            assert_eq!(
                h.lines.len(),
                6,
                "hexagram {} has wrong line count",
                h.number
            );
        }
    }

    #[test]
    fn renderings_are_nonempty() {
        for h in hexagrams() {
            assert!(
                !h.judgment.english.is_empty(),
                "hexagram {} judgment",
                h.number
            );
            assert!(
                !h.judgment.chinese.is_empty(),
                "hexagram {} judgment zh",
                h.number
            );
            assert!(!h.image.english.is_empty(), "hexagram {} image", h.number);
        }
    }

    #[test]
    fn reading_slots_present_and_null_in_v0() {
        for h in hexagrams() {
            assert!(h.hexagram_reading.is_none());
            assert!(h.lines.iter().all(|l| l.reading.is_none()));
        }
    }

    #[test]
    fn lookup_by_binary_roundtrip() {
        let h = lookup_hexagram(28).unwrap();
        let by_bin = lookup_hexagram_by_binary(&h.binary).unwrap();
        assert_eq!(by_bin.number, 28);
    }

    #[test]
    fn lookup_out_of_range_returns_none() {
        assert!(lookup_hexagram(0).is_none());
        assert!(lookup_hexagram(65).is_none());
    }
}
