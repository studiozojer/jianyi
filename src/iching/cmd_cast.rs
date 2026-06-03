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

    writeln!(out, "I Ching")?;
    writeln!(out)?;
    if let Some(ref q) = result.question {
        writeln!(out, "  {q}")?;
        writeln!(out)?;
    }

    for line in render_lines(&result.lines).lines() {
        writeln!(out, "  {line}")?;
    }
    writeln!(out)?;

    let primary = lookup_hexagram(result.primary_number).unwrap();
    let upper = lookup_trigram_by_binary(&primary.upper_trigram).map(|t| t.english.as_str()).unwrap_or("?");
    let lower = lookup_trigram_by_binary(&primary.lower_trigram).map(|t| t.english.as_str()).unwrap_or("?");
    writeln!(out, "  {upper} over {lower}")?;
    writeln!(out)?;

    writeln!(out, "  Judgment")?;
    writeln!(out, "  {}", primary.judgment.chinese)?;
    writeln!(out, "  {}", primary.judgment.english)?;
    if let Some(r) = &primary.hexagram_reading {
        writeln!(out)?;
        writeln!(out, "  Reading")?;
        writeln!(out, "  {r}")?;
    }
    writeln!(out)?;
    writeln!(out, "  Image")?;
    writeln!(out, "  {}", primary.image.english)?;

    if !result.changing_lines.is_empty() {
        writeln!(out)?;
        writeln!(out, "  Changing lines:")?;
        for &pos in &result.changing_lines {
            if let Some(l) = primary.lines.iter().find(|l| l.position == pos) {
                writeln!(out, "    Line {pos} · {}", l.english)?;
                if let Some(r) = &l.reading {
                    writeln!(out, "      {r}")?;
                }
            }
        }
        if let Some(num) = result.resulting_number {
            let resulting = lookup_hexagram(num).unwrap();
            writeln!(out)?;
            writeln!(
                out,
                "  Becomes: {}. {} {} — {}",
                resulting.number, resulting.chinese, resulting.pinyin, resulting.english
            )?;
            writeln!(out)?;
            writeln!(out, "  Judgment")?;
            writeln!(out, "  {}", resulting.judgment.english)?;
        }
    }
    Ok(())
}
