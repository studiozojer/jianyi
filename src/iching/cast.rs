use crate::iching::data;
use serde::Serialize;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum LineValue {
    OldYin,    // 6 — changing yin  → yang in resulting
    YoungYang, // 7 — stable yang
    YoungYin,  // 8 — stable yin
    OldYang,   // 9 — changing yang → yin in resulting
}

impl LineValue {
    pub fn is_yang(&self) -> bool {
        matches!(self, LineValue::YoungYang | LineValue::OldYang)
    }

    pub fn is_changing(&self) -> bool {
        matches!(self, LineValue::OldYin | LineValue::OldYang)
    }

    pub fn to_bin(&self) -> char {
        if self.is_yang() {
            '1'
        } else {
            '0'
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CastResult {
    pub question: Option<String>,
    pub timestamp: String,
    pub lines: [LineValue; 6], // index 0 = line 1 (bottom)
    pub primary_number: u8,
    pub resulting_number: Option<u8>,
    pub nuclear_number: u8,
    pub changing_lines: Vec<u8>, // 1-indexed positions of OldYin/OldYang
}

fn binary_from_lines(lines: &[LineValue; 6]) -> String {
    lines.iter().map(|l| l.to_bin()).collect()
}

/// Hash-based yarrow stalk simulation. Same inputs always produce the same cast.
/// question: None is treated as empty string. timestamp: RFC3339 string.
pub fn cast(question: Option<&str>, timestamp: &str) -> CastResult {
    let q = question.unwrap_or("");
    let input = format!("{}|{}", q, timestamp);
    let hash = Sha256::digest(input.as_bytes());

    let lines: [LineValue; 6] = std::array::from_fn(|i| {
        let val = hash[i] % 16;
        match val {
            0 => LineValue::OldYin,
            1..=5 => LineValue::YoungYang,
            6..=12 => LineValue::YoungYin,
            _ => LineValue::OldYang,
        }
    });

    let primary_binary = binary_from_lines(&lines);
    let primary_number = data::lookup_hexagram_by_binary(&primary_binary)
        .expect("cast produced unknown hexagram binary — logic error")
        .number;

    let changing: Vec<u8> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.is_changing())
        .map(|(i, _)| i as u8 + 1)
        .collect();

    let resulting_number = if changing.is_empty() {
        None
    } else {
        let result_lines: [LineValue; 6] = std::array::from_fn(|i| match lines[i] {
            LineValue::OldYin => LineValue::YoungYang,
            LineValue::OldYang => LineValue::YoungYin,
            other => other,
        });
        let result_binary = binary_from_lines(&result_lines);
        Some(
            data::lookup_hexagram_by_binary(&result_binary)
                .expect("resulting hexagram binary is unknown — logic error")
                .number,
        )
    };

    // Nuclear: lower = lines 2-4 (indices 1,2,3), upper = lines 3-5 (indices 2,3,4)
    let nuc_binary: String = [1usize, 2, 3, 2, 3, 4]
        .iter()
        .map(|&i| lines[i].to_bin())
        .collect();
    let nuclear_number = data::lookup_hexagram_by_binary(&nuc_binary)
        .expect("nuclear hexagram binary is unknown — logic error")
        .number;

    CastResult {
        question: question.map(|s| s.to_string()),
        timestamp: timestamp.to_string(),
        lines,
        primary_number,
        resulting_number,
        nuclear_number,
        changing_lines: changing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TS: &str = "2026-05-07T12:43:23+00:00";

    #[test]
    fn cast_is_deterministic() {
        let a = cast(Some("will this work?"), TS);
        let b = cast(Some("will this work?"), TS);
        assert_eq!(a.lines, b.lines);
        assert_eq!(a.primary_number, b.primary_number);
    }

    #[test]
    fn cast_no_question_uses_empty_string() {
        let a = cast(None, TS);
        let b = cast(Some(""), TS);
        assert_eq!(
            a.lines, b.lines,
            "None and empty string should be equivalent"
        );
    }

    #[test]
    fn primary_number_in_range() {
        let r = cast(Some("test"), TS);
        assert!((1..=64).contains(&r.primary_number));
    }

    #[test]
    fn nuclear_number_in_range() {
        let r = cast(Some("test"), TS);
        assert!((1..=64).contains(&r.nuclear_number));
    }

    #[test]
    fn changing_lines_are_1_indexed() {
        // Generate many casts and verify changing line positions are in 1..=6
        for i in 0..50 {
            let ts = format!("2026-05-07T12:{:02}:00+00:00", i);
            let r = cast(Some("q"), &ts);
            for &pos in &r.changing_lines {
                assert!(
                    (1..=6).contains(&pos),
                    "changing line position {pos} out of range"
                );
            }
        }
    }

    #[test]
    fn resulting_none_when_no_changing_lines() {
        // Find a cast with no changing lines by brute force
        for i in 0..200 {
            let ts = format!("2026-05-{:02}T12:00:00+00:00", (i % 30) + 1);
            let r = cast(Some(&format!("q{i}")), &ts);
            if r.changing_lines.is_empty() {
                assert!(r.resulting_number.is_none());
                return;
            }
        }
        // If we didn't find one, that's fine — changing lines appear ~75% of casts
    }

    #[test]
    fn resulting_some_when_changing_lines_exist() {
        for i in 0..200 {
            let ts = format!("2026-05-{:02}T12:00:00+00:00", (i % 30) + 1);
            let r = cast(Some(&format!("q{i}")), &ts);
            if !r.changing_lines.is_empty() {
                assert!(r.resulting_number.is_some());
                return;
            }
        }
    }

    #[test]
    fn yarrow_distribution_approximately_correct() {
        // Over 10,000 casts, check bucket ratios are within 20% of expected
        let mut counts = [0u32; 4]; // [OldYin, YoungYang, YoungYin, OldYang]
        let total_lines = 10_000 * 6;

        for i in 0..10_000u32 {
            let ts = format!(
                "2026-05-01T{:02}:{:02}:{:02}+00:00",
                i / 3600,
                (i / 60) % 60,
                i % 60
            );
            let r = cast(Some("distribution test"), &ts);
            for line in &r.lines {
                match line {
                    LineValue::OldYin => counts[0] += 1,
                    LineValue::YoungYang => counts[1] += 1,
                    LineValue::YoungYin => counts[2] += 1,
                    LineValue::OldYang => counts[3] += 1,
                }
            }
        }

        // Expected: OldYin=1/16, YoungYang=5/16, YoungYin=7/16, OldYang=3/16
        let expected = [
            total_lines as f64 / 16.0,
            total_lines as f64 * 5.0 / 16.0,
            total_lines as f64 * 7.0 / 16.0,
            total_lines as f64 * 3.0 / 16.0,
        ];
        let names = ["OldYin", "YoungYang", "YoungYin", "OldYang"];
        for (i, (&count, &exp)) in counts.iter().zip(expected.iter()).enumerate() {
            let ratio = count as f64 / exp;
            assert!(
                (0.8..=1.2).contains(&ratio),
                "{} ratio {:.2} outside 20% tolerance (got {}, expected ~{:.0})",
                names[i],
                ratio,
                count,
                exp
            );
        }
    }
}
