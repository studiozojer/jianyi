pub use crate::iching::data::{lookup_hexagram, HexagramRecord};

/// Positions (1-indexed, bottom-up per I Ching convention) where the source
/// and destination hexagrams' binary strings differ. Returns the empty vec
/// for identical hexagrams.
pub fn moving_lines(source: &HexagramRecord, destination: &HexagramRecord) -> Vec<u8> {
    let src_bytes = source.binary.as_bytes();
    let dst_bytes = destination.binary.as_bytes();
    debug_assert_eq!(src_bytes.len(), 6);
    debug_assert_eq!(dst_bytes.len(), 6);
    let mut positions = Vec::new();
    // Binary string is stored bottom-to-top: char[0] = line 1, char[5] = line 6.
    for line_pos in 1u8..=6 {
        let char_idx = (line_pos - 1) as usize;
        if src_bytes[char_idx] != dst_bytes[char_idx] {
            positions.push(line_pos);
        }
    }
    positions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_hexagram_28_preponderance() {
        let h = lookup_hexagram(28).unwrap();
        assert_eq!(h.pinyin, "Dà Guò");
        assert_eq!(h.upper_trigram, "110");
        assert_eq!(h.lower_trigram, "011");
        assert_eq!(h.lines.len(), 6);
        assert!(!h.judgment.english.is_empty());
        assert!(!h.image.english.is_empty());
    }

    #[test]
    fn moving_lines_identical_hexagrams_empty() {
        let h = lookup_hexagram(1).unwrap();
        assert!(moving_lines(h, h).is_empty());
    }

    #[test]
    fn moving_lines_hex1_to_hex2_all_six_move() {
        // Hex 1: 111111 (all yang). Hex 2: 000000 (all yin). Every line moves.
        let h1 = lookup_hexagram(1).unwrap();
        let h2 = lookup_hexagram(2).unwrap();
        assert_eq!(moving_lines(h1, h2), vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn moving_lines_position_indexing_bottom_up() {
        // Hex 1 (Heaven/Heaven) has binary "111111".
        // Hex 11 T'ai (Earth over Heaven) has binary = lower(Heaven=111) + upper(Earth=000) = "111000".
        // Comparing: chars 0-2 same (both 111), chars 3-5 differ (111 vs 000).
        // With char_idx = line_pos - 1, the upper three positions 4, 5, 6 are the moving lines.
        let h1 = lookup_hexagram(1).unwrap();
        let h11 = lookup_hexagram(11).unwrap();
        assert_eq!(moving_lines(h1, h11), vec![4, 5, 6]);
    }
}
