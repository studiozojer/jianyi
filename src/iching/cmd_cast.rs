use crate::error::Error;
use crate::iching::data::lookup_trigram_by_binary;
use crate::iching::{cast, lookup_hexagram, render_lines};
use crate::output::{emit, OutputFormat};
use chrono::Utc;

#[derive(clap::Args)]
pub struct Args {
    /// Optional question. If omitted, produces an open casting.
    #[arg(long)]
    pub question: Option<String>,

    /// Seed timestamp (RFC 3339) for a reproducible cast; defaults to now.
    #[arg(long, alias = "at")]
    pub seed: Option<String>,
}

pub fn run(args: Args, output: OutputFormat) -> Result<(), Error> {
    let timestamp = match &args.seed {
        Some(s) => chrono::DateTime::parse_from_rfc3339(s)
            .map_err(|_| {
                Error::Usage(format!(
                    "invalid --seed '{s}'. Use RFC 3339, e.g. 2026-06-02T11:00:00+00:00"
                ))
            })?
            .with_timezone(&Utc)
            .to_rfc3339(),
        None => Utc::now().to_rfc3339(),
    };
    let result = cast(args.question.as_deref(), &timestamp);
    emit(&result, output, print_pretty).map_err(Error::from)
}

fn print_pretty(result: &crate::iching::CastResult) -> std::io::Result<()> {
    use std::io::Write;
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    write!(out, "{}", format_cast(result))
}

/// Build the full pretty render of a cast as a string.
///
/// Emits the complete canonical record in one pass — line diagram, then for the
/// primary hexagram the Judgment and Image (Chinese + English), the Chinese +
/// English of each changing line, the nuclear hexagram, and (when lines change)
/// the resulting hexagram's Judgment and Image. A reading off `jianyi iching cast`
/// is meant to stand on its own; nothing should need a second lookup to render.
pub fn format_cast(result: &crate::iching::CastResult) -> String {
    use std::fmt::Write;
    let mut s = String::new();

    let _ = writeln!(s, "I Ching");
    let _ = writeln!(s);
    if let Some(ref q) = result.question {
        let _ = writeln!(s, "  {q}");
        let _ = writeln!(s);
    }

    for line in render_lines(&result.lines).lines() {
        let _ = writeln!(s, "  {line}");
    }
    let _ = writeln!(s);

    let primary = lookup_hexagram(result.primary_number).unwrap();
    let upper = lookup_trigram_by_binary(&primary.upper_trigram)
        .map(|t| t.english.as_str())
        .unwrap_or("?");
    let lower = lookup_trigram_by_binary(&primary.lower_trigram)
        .map(|t| t.english.as_str())
        .unwrap_or("?");
    let _ = writeln!(s, "  {upper} over {lower}");
    let _ = writeln!(s);

    let _ = writeln!(s, "  Judgment");
    let _ = writeln!(s, "  {}", primary.judgment.chinese);
    let _ = writeln!(s, "  {}", primary.judgment.english);
    if let Some(r) = &primary.hexagram_reading {
        let _ = writeln!(s);
        let _ = writeln!(s, "  Reading");
        let _ = writeln!(s, "  {r}");
    }
    let _ = writeln!(s);
    let _ = writeln!(s, "  Image");
    let _ = writeln!(s, "  {}", primary.image.chinese);
    let _ = writeln!(s, "  {}", primary.image.english);

    if !result.changing_lines.is_empty() {
        let _ = writeln!(s);
        let _ = writeln!(s, "  Changing lines:");
        for &pos in &result.changing_lines {
            if let Some(l) = primary.lines.iter().find(|l| l.position == pos) {
                // Chinese on the "Line N ·" row, English aligned beneath it.
                let _ = writeln!(s, "    Line {pos} · {}", l.chinese);
                let _ = writeln!(s, "             {}", l.english);
                if let Some(r) = &l.reading {
                    let _ = writeln!(s, "             {r}");
                }
            }
        }
    }

    // Nuclear hexagram — the hidden interior. Always present; rendered compact.
    let nuclear = lookup_hexagram(result.nuclear_number).unwrap();
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "  Nuclear: {}. {} {} — {}",
        nuclear.number, nuclear.chinese, nuclear.pinyin, nuclear.english
    );
    let _ = writeln!(s, "  {}", nuclear.judgment.chinese);
    let _ = writeln!(s, "  {}", nuclear.judgment.english);

    // Resulting hexagram — only when lines change. Judgment and Image, no lines.
    if let Some(num) = result.resulting_number {
        let resulting = lookup_hexagram(num).unwrap();
        let _ = writeln!(s);
        let _ = writeln!(
            s,
            "  Becomes: {}. {} {} — {}",
            resulting.number, resulting.chinese, resulting.pinyin, resulting.english
        );
        let _ = writeln!(s);
        let _ = writeln!(s, "  Judgment");
        let _ = writeln!(s, "  {}", resulting.judgment.chinese);
        let _ = writeln!(s, "  {}", resulting.judgment.english);
        let _ = writeln!(s);
        let _ = writeln!(s, "  Image");
        let _ = writeln!(s, "  {}", resulting.image.chinese);
        let _ = writeln!(s, "  {}", resulting.image.english);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::format_cast;
    use crate::iching::cast::cast;

    // Open cast on a fixed seed: hexagram 45 (萃 Gathering Together),
    // changing line 5 → 16 (豫 Enthusiasm), nuclear 53 (漸 Development).
    const SEED_45: &str = "2026-06-12T21:51:54+00:00";

    #[test]
    fn pretty_includes_full_canonical_record() {
        let result = cast(None, SEED_45);
        // Sanity: the seed still resolves to the expected figure.
        assert_eq!(result.primary_number, 45);
        assert_eq!(result.resulting_number, Some(16));
        assert_eq!(result.nuclear_number, 53);

        let out = format_cast(&result);
        // Primary Image — Chinese must be present, not just English.
        assert!(out.contains("澤上于地"), "image Chinese missing:\n{out}");
        // Changing line — Chinese text of line 5.
        assert!(
            out.contains("萃有位"),
            "changing-line Chinese missing:\n{out}"
        );
        // Nuclear hexagram must be rendered.
        assert!(out.contains("Nuclear:"), "nuclear block missing:\n{out}");
        assert!(out.contains("漸"), "nuclear hexagram missing:\n{out}");
        // Resulting hexagram Judgment — Chinese present.
        assert!(
            out.contains("利建侯行師"),
            "resulting Judgment Chinese missing:\n{out}"
        );
    }
}
