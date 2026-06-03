use crate::iching::cast::LineValue;
use crate::iching::data::{self, HexagramRecord};

const COL_WIDTH: usize = 23;

const GLYPH_YOUNG_YANG: &str = "━━━━━━━━━━━━━━━━━━━━━━━";
const GLYPH_YOUNG_YIN: &str = "━━━━━━━━━━   ━━━━━━━━━━";
const GLYPH_OLD_YANG: &str = "━━━━━━━━━━━○━━━━━━━━━━━";
const GLYPH_OLD_YIN: &str = "━━━━━━━━━━ × ━━━━━━━━━━";

fn glyph(l: LineValue) -> &'static str {
    match l {
        LineValue::YoungYang => GLYPH_YOUNG_YANG,
        LineValue::YoungYin => GLYPH_YOUNG_YIN,
        LineValue::OldYang => GLYPH_OLD_YANG,
        LineValue::OldYin => GLYPH_OLD_YIN,
    }
}

/// Display width — CJK and fullwidth characters count as 2 columns.
fn display_width(s: &str) -> usize {
    s.chars()
        .map(|c| {
            let cp = c as u32;
            if (0x1100..=0x115F).contains(&cp)
                || (0x2E80..=0x303F).contains(&cp)
                || (0x3040..=0x33FF).contains(&cp)
                || (0x3400..=0x4DBF).contains(&cp)
                || (0x4E00..=0x9FFF).contains(&cp)
                || (0xF900..=0xFAFF).contains(&cp)
                || (0xFE10..=0xFE1F).contains(&cp)
                || (0xFE30..=0xFE4F).contains(&cp)
                || (0xFF01..=0xFF60).contains(&cp)
                || (0xFFE0..=0xFFE6).contains(&cp)
            {
                2
            } else {
                1
            }
        })
        .sum()
}

/// Center `s` within `width` display columns. Extra padding goes right.
fn dcenter(s: &str, width: usize) -> String {
    let dw = display_width(s);
    if dw >= width {
        return s.to_string();
    }
    let pad = width - dw;
    let left = pad / 2;
    let right = pad - left;
    format!("{}{}{}", " ".repeat(left), s, " ".repeat(right))
}

/// Label block: ["N. 漢字 Pinyin", "English name", "wrapped if long"]
fn wrap_label(hex: &HexagramRecord) -> Vec<String> {
    let header = format!("{}. {} {}", hex.number, hex.chinese, hex.pinyin);
    let mut lines = vec![header];
    let mut current = String::new();
    for word in hex.english.split_whitespace() {
        if current.is_empty() {
            current = word.to_string();
        } else if display_width(&current) + 1 + display_width(word) <= COL_WIDTH {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(current);
            current = word.to_string();
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// Construct stable (young) lines from a hexagram binary string.
/// Used for hexagram-lookup display where there are no changing lines.
pub fn lines_from_binary(binary: &str) -> [LineValue; 6] {
    let chars: Vec<char> = binary.chars().collect();
    std::array::from_fn(|i| {
        if chars[i] == '1' {
            LineValue::YoungYang
        } else {
            LineValue::YoungYin
        }
    })
}

/// ASCII render for a set of lines. Returns a multi-line string.
/// Single-column when no changing lines; paired primary→resulting columns otherwise.
pub fn render_lines(lines: &[LineValue; 6]) -> String {
    let changing: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.is_changing())
        .map(|(i, _)| i)
        .collect();

    // Glyphs run top-to-bottom (line 6 = index 5 first, line 1 = index 0 last)
    let primary_glyphs: Vec<&str> = (0..6).rev().map(|i| glyph(lines[i])).collect();

    let primary_binary: String = lines.iter().map(|l| l.to_bin()).collect();
    let primary_hex = data::lookup_hexagram_by_binary(&primary_binary)
        .expect("render_lines called with invalid binary");
    let primary_label: Vec<String> = wrap_label(primary_hex)
        .iter()
        .map(|ln| dcenter(ln, COL_WIDTH))
        .collect();

    if changing.is_empty() {
        let mut out = primary_glyphs.join("\n");
        out.push('\n');
        out.push_str(&primary_label.join("\n"));
        out.push('\n');
        return out;
    }

    let result_lines: [LineValue; 6] = std::array::from_fn(|i| match lines[i] {
        LineValue::OldYin => LineValue::YoungYang,
        LineValue::OldYang => LineValue::YoungYin,
        other => other,
    });
    let result_glyphs: Vec<&str> = (0..6).rev().map(|i| glyph(result_lines[i])).collect();

    let result_binary: String = result_lines.iter().map(|l| l.to_bin()).collect();
    let result_hex = data::lookup_hexagram_by_binary(&result_binary)
        .expect("resulting hexagram binary is invalid — logic error");
    let result_label: Vec<String> = wrap_label(result_hex)
        .iter()
        .map(|ln| dcenter(ln, COL_WIDTH))
        .collect();

    // Glyph rows: "→" at row index 2 (the trigram seam), spaces elsewhere
    let glyph_rows: Vec<String> = (0..6)
        .map(|i| {
            let sep = if i == 2 { "   →   " } else { "       " };
            format!("{}{}{}", primary_glyphs[i], sep, result_glyphs[i])
        })
        .collect();

    // Bottom-align label blocks
    let blank = " ".repeat(COL_WIDTH);
    let max_h = primary_label.len().max(result_label.len());
    let pad_p: Vec<String> = std::iter::repeat_n(blank.clone(), max_h - primary_label.len())
        .chain(primary_label.iter().cloned())
        .collect();
    let pad_r: Vec<String> = std::iter::repeat_n(blank.clone(), max_h - result_label.len())
        .chain(result_label.iter().cloned())
        .collect();
    let label_rows: Vec<String> = pad_p
        .iter()
        .zip(pad_r.iter())
        .map(|(p, r)| format!("{}       {}", p, r))
        .collect();

    let mut out = glyph_rows.join("\n");
    out.push('\n');
    out.push_str(&label_rows.join("\n"));
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_all_yang_no_arrow() {
        // Hexagram 1 — all yang, no changing lines
        let lines: [LineValue; 6] = [LineValue::YoungYang; 6];
        let out = render_lines(&lines);
        assert!(
            !out.contains('→'),
            "single-column render should not contain →"
        );
        assert!(out.contains(GLYPH_YOUNG_YANG));
    }

    #[test]
    fn render_all_yin_no_arrow() {
        let lines: [LineValue; 6] = [LineValue::YoungYin; 6];
        let out = render_lines(&lines);
        assert!(!out.contains('→'));
        assert!(out.contains(GLYPH_YOUNG_YIN));
    }

    #[test]
    fn render_with_changing_lines_has_arrow() {
        let mut lines: [LineValue; 6] = [LineValue::YoungYang; 6];
        lines[0] = LineValue::OldYin;
        let out = render_lines(&lines);
        assert!(out.contains('→'));
        assert!(out.contains(GLYPH_OLD_YIN));
    }

    #[test]
    fn lines_from_binary_all_yang() {
        let lines = lines_from_binary("111111");
        for l in &lines {
            assert_eq!(*l, LineValue::YoungYang);
        }
    }

    #[test]
    fn lines_from_binary_all_yin() {
        let lines = lines_from_binary("000000");
        for l in &lines {
            assert_eq!(*l, LineValue::YoungYin);
        }
    }

    #[test]
    fn lines_from_binary_hexagram_28() {
        // 011110 — bottom-to-top
        let lines = lines_from_binary("011110");
        assert_eq!(lines[0], LineValue::YoungYin); // line 1
        assert_eq!(lines[5], LineValue::YoungYin); // line 6
        assert_eq!(lines[1], LineValue::YoungYang); // line 2
    }

    #[test]
    fn render_contains_hexagram_name() {
        let lines = lines_from_binary("111111");
        let out = render_lines(&lines);
        assert!(
            out.contains("Creative"),
            "render should include hexagram name"
        );
    }

    #[test]
    fn display_width_cjk_counts_double() {
        assert_eq!(display_width("乾"), 2);
        assert_eq!(display_width("abc"), 3);
        assert_eq!(display_width("1. 乾 Qián"), 10); // "1. " (3) + "乾" (2) + " Qián" (5)
    }
}
