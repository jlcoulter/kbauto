//! Per-page progress output during generation.

/// Phase of page generation.
#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum ProgressPhase {
    /// Substituting placeholders (steps 1-5).
    Substituting,
    /// AI rewriting sections (steps 6-10).
    Rewriting,
}

/// Report progress for a single page.
#[allow(dead_code)]
pub fn report_page_progress(
    filename: &str,
    phase: &ProgressPhase,
    format: &crate::output::OutputFormat,
) {
    let phase_str = match phase {
        ProgressPhase::Substituting => "substituting",
        ProgressPhase::Rewriting => "rewriting",
    };
    match format {
        crate::output::OutputFormat::Text => {
            println!("{filename}: {phase_str}...");
        }
        crate::output::OutputFormat::Json => {
            println!(r#"{{"page": "{}", "phase": "{}"}}"#, filename, phase_str);
        }
    }
}

/// Report page completion.
#[allow(dead_code)]
pub fn report_page_complete(filename: &str, format: &crate::output::OutputFormat) {
    match format {
        crate::output::OutputFormat::Text => println!("{filename}: done"),
        crate::output::OutputFormat::Json => {
            println!(r#"{{"page": "{}", "status": "complete"}}"#, filename);
        }
    }
}

/// Report final generation summary.
pub fn report_summary(
    pages_generated: usize,
    placeholders_resolved: usize,
    elapsed: std::time::Duration,
    format: &crate::output::OutputFormat,
) {
    let elapsed_secs = elapsed.as_secs_f64();
    match format {
        crate::output::OutputFormat::Text => {
            println!(
                "Generated {pages_generated} pages, {placeholders_resolved} placeholders resolved in {elapsed_secs:.1}s"
            );
        }
        crate::output::OutputFormat::Json => {
            println!(
                r#"{{"pages_generated": {pages_generated}, "placeholders_resolved": {placeholders_resolved}, "elapsed_seconds": {elapsed_secs}}}"#
            );
        }
    }
}
