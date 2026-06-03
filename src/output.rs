#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OutputFormat {
    Pretty,
    Json,
}

/// Render a value in the requested output format. JSON goes straight to stdout as the
/// serialized form of `value`. Pretty dispatches to a caller-supplied closure so each
/// command controls its own human-readable layout.
pub fn emit<T, F>(value: &T, output: OutputFormat, pretty: F) -> std::io::Result<()>
where
    T: serde::Serialize,
    F: FnOnce(&T) -> std::io::Result<()>,
{
    match output {
        OutputFormat::Json => {
            let stdout = std::io::stdout();
            let mut lock = stdout.lock();
            serde_json::to_writer(&mut lock, value)?;
            use std::io::Write;
            writeln!(&mut lock)?;
            Ok(())
        }
        OutputFormat::Pretty => pretty(value),
    }
}
