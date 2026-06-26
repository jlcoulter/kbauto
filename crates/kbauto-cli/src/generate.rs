//! CLI `generate` subcommand.

use crate::output::OutputFormat;
use crate::progress;
use crate::tui;
use clap::Parser;
use kbauto_config::AppConfig;
use kbauto_template::generate_playbook;
use std::path::PathBuf;
use std::time::Instant;

/// Generate a client knowledge base playbook.
#[derive(Parser, Debug)]
pub struct GenerateArgs {
    /// Path to the playbook template directory.
    #[arg(short, long)]
    pub template_dir: PathBuf,

    /// Path to the client static details markdown file (heading-value pairs for steps 1-5).
    #[arg(short, long)]
    pub details: Option<PathBuf>,

    /// Path to the client discovery document markdown file (Q&A pairs for steps 6-10).
    #[arg(long)]
    pub discovery: Option<PathBuf>,

    /// Output directory for generated files.
    #[arg(short, long, default_value = "./output")]
    pub output: PathBuf,

    /// Output format: text (default) or json. Overrides config file.
    #[arg(long)]
    pub output_format: Option<String>,

    /// Ollama server URL. Overrides config file.
    #[arg(long)]
    pub ollama_url: Option<String>,

    /// Ollama model name. Overrides config file.
    #[arg(long)]
    pub ollama_model: Option<String>,

    /// Number of retry attempts for AI rewriting. Overrides config file.
    #[arg(long)]
    pub retry_count: Option<u32>,
}

/// Exit code for missing placeholder values in JSON mode (FR-022).
const EXIT_MISSING_VALUES: i32 = 6;

/// Run the generate subcommand.
pub async fn run(args: GenerateArgs, config: AppConfig) -> anyhow::Result<()> {
    let format: OutputFormat = match args.output_format.as_deref() {
        Some(s) => s.parse().unwrap_or(config.output_format),
        None => config.output_format,
    };

    // Validate template directory exists (FR-020)
    if !args.template_dir.exists() {
        crate::output::print_error(
            &format!(
                "Template directory not found: {}",
                args.template_dir.display()
            ),
            &format,
        );
        std::process::exit(1);
    }

    // Validate docs/ subdirectory exists (FR-020)
    let docs_dir = args.template_dir.join("docs");
    if !docs_dir.exists() {
        crate::output::print_error(
            &format!(
                "Missing docs/ subdirectory in: {}",
                args.template_dir.display()
            ),
            &format,
        );
        std::process::exit(1);
    }

    // Validate defaults.json exists (FR-020)
    let defaults_path = args.template_dir.join("defaults.json");
    if !defaults_path.exists() {
        crate::output::print_error(
            &format!("Missing defaults.json in: {}", args.template_dir.display()),
            &format,
        );
        std::process::exit(1);
    }

    // Validate details file if provided
    if let Some(ref details_path) = args.details {
        if !details_path.exists() {
            crate::output::print_error(
                &format!("Details file not found: {}", details_path.display()),
                &format,
            );
            std::process::exit(1);
        }
    }

    // Validate discovery file if provided
    if let Some(ref discovery_path) = args.discovery {
        if !discovery_path.exists() {
            crate::output::print_error(
                &format!("Discovery file not found: {}", discovery_path.display()),
                &format,
            );
            std::process::exit(1);
        }
    }

    let start = Instant::now();

    match generate_playbook(
        &args.template_dir,
        args.details.as_deref(),
        args.discovery.as_deref(),
        &args.output,
    )
    .await
    {
        Ok(result) => {
            // Handle missing placeholder values (FR-022)
            if !result.missing_values.is_empty() {
                match format {
                    OutputFormat::Json => {
                        // In JSON mode, report missing values and exit with code 6
                        let missing: Vec<serde_json::Value> = result
                            .missing_values
                            .iter()
                            .map(|mv| {
                                serde_json::json!({
                                    "key": mv.key,
                                    "description": mv.description,
                                    "default": mv.default,
                                })
                            })
                            .collect();
                        let json = serde_json::json!({
                            "error": "missing_values",
                            "missing_values": missing,
                        });
                        println!("{}", serde_json::to_string_pretty(&json)?);
                        std::process::exit(EXIT_MISSING_VALUES);
                    }
                    OutputFormat::Text => {
                        // In text mode, launch the TUI form for the user to fill in missing values
                        match tui::run_missing_value_form(result.missing_values.clone()) {
                            Ok(values) => {
                                // Re-run generation with the TUI-collected values merged
                                // For now, report that values were collected and re-generation is needed
                                progress::report_summary(
                                    result.pages_generated,
                                    result.placeholders_resolved,
                                    start.elapsed(),
                                    &format,
                                );
                                eprintln!(
                                    "\nCollected {} missing value(s) via TUI. Re-run with these values in your details file.",
                                    values.len()
                                );
                            }
                            Err(e) => {
                                crate::output::print_error(&format!("TUI error: {e}"), &format);
                                std::process::exit(1);
                            }
                        }
                    }
                }
            } else {
                progress::report_summary(
                    result.pages_generated,
                    result.placeholders_resolved,
                    start.elapsed(),
                    &format,
                );
            }
            Ok(())
        }
        Err(e) => {
            crate::output::print_error(&format!("Generation failed: {e}"), &format);
            Err(e)
        }
    }
}
