use crate::iching::data::{self, TrigramRecord};

/// Fold accented Latin vowels to their ASCII base equivalents. This handles
/// precomposed Unicode (e.g. á U+00E1) without requiring a unicode-normalization
/// dependency. Covers all diacritics that appear in the 8 trigram pinyin values.
fn fold_accents(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ä' | 'ā' | 'ǎ' | 'ã' => 'a',
            'é' | 'è' | 'ê' | 'ë' | 'ē' | 'ě' => 'e',
            'í' | 'ì' | 'î' | 'ï' | 'ī' | 'ǐ' => 'i',
            'ó' | 'ò' | 'ô' | 'ö' | 'ō' | 'ǒ' => 'o',
            'ú' | 'ù' | 'û' | 'ü' | 'ū' | 'ǔ' => 'u',
            'Á' | 'À' | 'Â' | 'Ä' | 'Ā' | 'Ǎ' | 'Ã' => 'A',
            'É' | 'È' | 'Ê' | 'Ë' | 'Ē' | 'Ě' => 'E',
            'Í' | 'Ì' | 'Î' | 'Ï' | 'Ī' | 'Ǐ' => 'I',
            'Ó' | 'Ò' | 'Ô' | 'Ö' | 'Ō' | 'Ǒ' => 'O',
            'Ú' | 'Ù' | 'Û' | 'Ü' | 'Ū' | 'Ǔ' => 'U',
            other => other,
        })
        .collect()
}

/// Look up a trigram by English name, pinyin, Unicode symbol, binary string, or quality.
/// Case-insensitive. Pinyin is matched both with diacritics and without, so
/// both "qián" and "qian" resolve to the same trigram. An optional leading
/// "the " article (case-insensitive) is stripped before lookup.
pub fn lookup_trigram(query: &str) -> Option<&'static TrigramRecord> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return None;
    }
    // Strip an optional leading "the " article (case-insensitive). If "the" is
    // the entire query, that's not a name — return None below via the find.
    let stripped = trimmed
        .strip_prefix("the ")
        .or_else(|| trimmed.strip_prefix("The "))
        .or_else(|| trimmed.strip_prefix("THE "))
        .unwrap_or(trimmed)
        .trim();
    if stripped.is_empty() {
        return None;
    }
    let q = stripped;
    let q_folded = fold_accents(q).to_lowercase();
    data::trigrams().iter().find(|t| {
        t.binary == q
            || t.symbol == q
            || t.english.eq_ignore_ascii_case(q)
            || t.pinyin.eq_ignore_ascii_case(q)
            || t.quality.eq_ignore_ascii_case(q)
            || fold_accents(&t.pinyin).eq_ignore_ascii_case(&q_folded)
    })
}

/// Hexagram numbers (1-indexed) where this trigram appears as the upper trigram.
pub fn hexagrams_with_upper(binary: &str) -> Vec<u8> {
    data::hexagrams()
        .iter()
        .filter(|h| h.upper_trigram == binary)
        .map(|h| h.number)
        .collect()
}

/// Hexagram numbers (1-indexed) where this trigram appears as the lower trigram.
pub fn hexagrams_with_lower(binary: &str) -> Vec<u8> {
    data::hexagrams()
        .iter()
        .filter(|h| h.lower_trigram == binary)
        .map(|h| h.number)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_by_english() {
        let t = lookup_trigram("Heaven").unwrap();
        assert_eq!(t.binary, "111");
        assert_eq!(t.symbol, "☰");
    }

    #[test]
    fn lookup_by_pinyin() {
        let t = lookup_trigram("qián").unwrap();
        assert_eq!(t.binary, "111");
    }

    #[test]
    fn lookup_by_symbol() {
        let t = lookup_trigram("☰").unwrap();
        assert_eq!(t.english, "Heaven");
    }

    #[test]
    fn lookup_by_binary() {
        let t = lookup_trigram("111").unwrap();
        assert_eq!(t.english, "Heaven");
    }

    #[test]
    fn lookup_by_ascii_pinyin() {
        let t = lookup_trigram("qian").unwrap();
        assert_eq!(t.binary, "111");
    }

    #[test]
    fn lookup_unknown_returns_none() {
        assert!(lookup_trigram("nope").is_none());
    }

    #[test]
    fn lookup_by_quality() {
        let t = lookup_trigram("Creative").unwrap();
        assert_eq!(t.english, "Heaven");
        let t = lookup_trigram("arousing").unwrap();
        assert_eq!(t.english, "Thunder");
        let t = lookup_trigram("abysmal").unwrap();
        assert_eq!(t.english, "Water");
        let t = lookup_trigram("gentle").unwrap();
        assert_eq!(t.english, "Wind");
        let t = lookup_trigram("receptive").unwrap();
        assert_eq!(t.english, "Earth");
        let t = lookup_trigram("joyous").unwrap();
        assert_eq!(t.english, "Lake");
        let t = lookup_trigram("clinging").unwrap();
        assert_eq!(t.english, "Fire");
        let t = lookup_trigram("stillness").unwrap();
        assert_eq!(t.english, "Mountain");
    }

    #[test]
    fn lookup_strips_the_article() {
        let t = lookup_trigram("the creative").unwrap();
        assert_eq!(t.english, "Heaven");
        let t = lookup_trigram("THE Heaven").unwrap();
        assert_eq!(t.english, "Heaven");
        let t = lookup_trigram("the abysmal").unwrap();
        assert_eq!(t.english, "Water");
    }

    #[test]
    fn lookup_the_alone_is_not_a_trigram() {
        assert!(lookup_trigram("the").is_none());
    }

    #[test]
    fn heaven_upper_trigram_contains_hexagram_1() {
        let uppers = hexagrams_with_upper("111");
        assert!(
            uppers.contains(&1),
            "Heaven should be upper trigram of hexagram 1"
        );
    }

    #[test]
    fn each_trigram_appears_8_times_total() {
        // Each trigram appears as upper in 8 hexagrams and lower in 8 hexagrams (8*8=64)
        for t in data::trigrams() {
            let upper_count = hexagrams_with_upper(&t.binary).len();
            let lower_count = hexagrams_with_lower(&t.binary).len();
            assert_eq!(upper_count, 8, "{} upper count wrong", t.english);
            assert_eq!(lower_count, 8, "{} lower count wrong", t.english);
        }
    }
}
