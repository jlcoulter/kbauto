//! CLI output formatting (human and JSON).

pub use kbauto_config::OutputFormat;

/// Print an error message to stderr in the appropriate format.
pub fn print_error(msg: &str, format: &OutputFormat) {
    match format {
        OutputFormat::Text => eprintln!("Error: {msg}"),
        OutputFormat::Json => eprintln!(r#"{{"error": "{}"}}"#, msg.replace('"', "\\\"")),
    }
}