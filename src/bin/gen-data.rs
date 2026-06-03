//! Dev-only generator: merges data/structure.json with the prose renderings in
//! tycho's i-ching graph into data/hexagrams.json. Not part of the shipped CLI.

use std::path::Path;

/// Prose parsed from one hexagram graph node.
#[derive(Debug, PartialEq)]
struct Prose {
    number: u8,
    judgment_zh: String,
    judgment_en: String,
    image_zh: String,
    image_en: String,
    lines: Vec<(String, String)>, // (chinese, english), 6 entries, position = index+1
    hexagram_reading: Option<String>, // "## Our reading" body, None if empty
}

/// Parse a single hexagram node's markdown into Prose.
fn parse_node(md: &str) -> Result<Prose, String> {
    // --- frontmatter: hexagram number ---
    let number = md
        .lines()
        .find_map(|l| l.strip_prefix("hexagram:"))
        .ok_or("missing `hexagram:` in frontmatter")?
        .trim()
        .parse::<u8>()
        .map_err(|e| format!("bad hexagram number: {e}"))?;

    // Split into "## " sections (drop everything before the first one).
    let mut judgment_zh = String::new();
    let mut judgment_en = String::new();
    let mut image_zh = String::new();
    let mut image_en = String::new();
    let mut lines: Vec<(String, String)> = Vec::new();
    let mut hexagram_reading: Option<String> = None;

    for section in md.split("\n## ").skip(1) {
        let (heading, body) = section.split_once('\n').unwrap_or((section, ""));
        let body = strip_fenced_blocks(body);
        let key = heading.split(['—', '-']).next().unwrap_or("").trim();
        match key {
            "Judgment" => {
                let (zh, en) = split_bold_then_prose(&body)?;
                judgment_zh = zh;
                judgment_en = en;
            }
            "Image" => {
                let (zh, en) = split_bold_then_prose(&body)?;
                image_zh = zh;
                image_en = en;
            }
            "Lines" => lines = parse_lines(&body)?,
            "Our reading" => {
                let t = body.trim();
                hexagram_reading = if t.is_empty() {
                    None
                } else {
                    Some(t.to_string())
                };
            }
            _ => {}
        }
    }

    if lines.len() != 6 {
        return Err(format!(
            "hexagram {number}: expected 6 lines, got {}",
            lines.len()
        ));
    }
    Ok(Prose {
        number,
        judgment_zh,
        judgment_en,
        image_zh,
        image_en,
        lines,
        hexagram_reading,
    })
}

/// Drop ```-fenced code blocks (the ASCII glyph art) from a section body.
fn strip_fenced_blocks(body: &str) -> String {
    let mut out = String::new();
    let mut in_fence = false;
    for line in body.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if !in_fence {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// A section that is `**<chinese>**` then a blank line then English prose.
fn split_bold_then_prose(body: &str) -> Result<(String, String), String> {
    let bold = body
        .lines()
        .find(|l| l.trim_start().starts_with("**"))
        .ok_or("missing bold chinese line")?;
    let zh = bold
        .trim()
        .trim_start_matches("**")
        .trim_end_matches("**")
        .trim()
        .to_string();
    // English = first non-empty line after the bold line that is not itself bold.
    let mut after_bold = false;
    let mut en = String::new();
    for line in body.lines() {
        if !after_bold {
            if line.trim() == bold.trim() {
                after_bold = true;
            }
            continue;
        }
        let t = line.trim();
        if t.is_empty() {
            if en.is_empty() {
                continue;
            } else {
                break;
            }
        }
        if t.starts_with("**") {
            break;
        }
        if !en.is_empty() {
            en.push(' ');
        }
        en.push_str(t);
    }
    if en.is_empty() {
        return Err("missing english prose".into());
    }
    Ok((zh, en))
}

/// Parse the Lines section: six `**Line N** · <chinese>` lines, each followed by English.
fn parse_lines(body: &str) -> Result<Vec<(String, String)>, String> {
    let lines: Vec<&str> = body.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let l = lines[i].trim();
        if let Some(rest) = l.strip_prefix("**Line ") {
            // rest looks like "1** · 初九：拔茅茹..."
            let zh = rest
                .split_once('·')
                .map(|(_, z)| z.trim().to_string())
                .ok_or("malformed line header (missing `·`)")?;
            // English: next non-empty line.
            let mut en = String::new();
            let mut j = i + 1;
            while j < lines.len() {
                let t = lines[j].trim();
                if t.is_empty() {
                    j += 1;
                    continue;
                }
                en = t.to_string();
                break;
            }
            if en.is_empty() {
                return Err("missing english for a line".into());
            }
            out.push((zh, en));
            i = j + 1;
        } else {
            i += 1;
        }
    }
    Ok(out)
}

fn main() {
    let graph_dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "../../tycho/graphs/divination/i-ching/hexagrams".to_string());
    let structure: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string("data/structure.json").expect("structure.json"),
    )
    .expect("structure.json parse");
    let skeleton = structure.as_array().expect("structure is array");

    let mut out = Vec::new();
    for entry in skeleton {
        let number = entry["number"].as_u64().unwrap() as u8;
        let node_path = find_node(Path::new(&graph_dir), number)
            .unwrap_or_else(|| panic!("no graph node for hexagram {number}"));
        let md = std::fs::read_to_string(&node_path).unwrap();
        let p = parse_node(&md).unwrap_or_else(|e| panic!("hexagram {number}: {e}"));

        let lines: Vec<serde_json::Value> = p
            .lines
            .iter()
            .enumerate()
            .map(|(idx, (zh, en))| {
                serde_json::json!({
                    "position": (idx + 1) as u8,
                    "chinese": zh,
                    "english": en,
                    "reading": serde_json::Value::Null,
                })
            })
            .collect();

        out.push(serde_json::json!({
            "number": number,
            "binary": entry["binary"],
            "chinese": entry["chinese"],
            "pinyin": entry["pinyin"],
            "english": entry["english"],
            "upper_trigram": entry["upper_trigram"],
            "lower_trigram": entry["lower_trigram"],
            "judgment": { "chinese": p.judgment_zh, "english": p.judgment_en },
            "image": { "chinese": p.image_zh, "english": p.image_en },
            "lines": lines,
            "hexagram_reading": p.hexagram_reading,
        }));
    }

    let json = serde_json::to_string_pretty(&out).unwrap();
    std::fs::write("data/hexagrams.json", json + "\n").expect("write hexagrams.json");
    eprintln!("wrote {} hexagrams", out.len());
}

/// Graph node files are named `<N>-<glyph>.md`; match on the numeric prefix.
fn find_node(dir: &Path, number: u8) -> Option<std::path::PathBuf> {
    let prefix = format!("{number}-");
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .find_map(|e| {
            let name = e.file_name();
            let name = name.to_string_lossy();
            if name.starts_with(&prefix) && name.ends_with(".md") {
                // guard against "1-" matching "11-"
                let after = &name[prefix.len()..];
                if after
                    .chars()
                    .next()
                    .map(|c| !c.is_ascii_digit())
                    .unwrap_or(false)
                {
                    return Some(e.path());
                }
            }
            None
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hexagram_11() {
        let md = include_str!("../../tests/fixtures/hexagram-11.md");
        let p = parse_node(md).expect("parse");
        assert_eq!(p.number, 11);
        assert_eq!(p.judgment_zh, "小往大來，吉亨。");
        assert!(p.judgment_en.starts_with("The small departs"));
        assert!(p.image_zh.contains("天地交泰"));
        assert!(p.image_en.starts_with("Heaven and earth unite"));
        assert_eq!(p.lines.len(), 6);
        assert!(p.lines[0].0.contains("拔茅茹"));
        assert!(p.lines[0].1.starts_with("Pulling up rushes"));
        assert_eq!(p.hexagram_reading, None); // empty in v0
    }
}
