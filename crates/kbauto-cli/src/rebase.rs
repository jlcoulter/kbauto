//! CLI `rebase` subcommand.

use crate::output::OutputFormat;
use clap::Parser;
use kbauto_config::AppConfig;
use kbauto_rebase::rebase_client_kb;
use std::path::PathBuf;

/// Rebase a client KB onto a new playbook version.
///
/// Updates template-origin text while preserving substituted
/// and rewritten text. Flags conflicts where both the base
/// and client text have changed.
#[derive(Parser, Debug)]
pub struct RebaseArgs {
    /// Path to the client KB directory.
    #[arg(long)]
    pub client_kb_dir: PathBuf,

    /// Old playbook version (e.g. "1.0.0").
    #[arg(long)]
    pub old_version: String,

    /// New playbook version (e.g. "2.0.0").
    #[arg(long)]
    pub new_version: String,

    /// Path to the template directory containing the new version.
    #[arg(long)]
    pub template_dir: PathBuf,

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

/// Run the rebase subcommand.
pub fn run(args: &RebaseArgs, config: &AppConfig) -> anyhow::Result<()> {
    let format: OutputFormat = match args.output_format.as_deref() {
        Some(s) => s.parse().unwrap_or(config.output_format),
        None => config.output_format,
    };

    // Validate paths
    if !args.client_kb_dir.exists() {
        anyhow::bail!(
            "Client KB directory not found: {}",
            args.client_kb_dir.display()
        );
    }
    if !args.template_dir.exists() {
        anyhow::bail!(
            "Template directory not found: {}",
            args.template_dir.display()
        );
    }
    if !args.template_dir.join("docs").exists() {
        anyhow::bail!(
            "Template directory missing docs/ subdirectory: {}",
            args.template_dir.display()
        );
    }
    if !args.template_dir.join("defaults.json").exists() {
        anyhow::bail!(
            "Template directory missing defaults.json: {}",
            args.template_dir.display()
        );
    }

    match format {
        OutputFormat::Text => {
            eprintln!("Rebasing client KB onto version {}...", args.new_version);
        }
        OutputFormat::Json => {}
    }

    let result = rebase_client_kb(
        &args.client_kb_dir,
        &args.old_version,
        &args.new_version,
        &args.template_dir,
    )?;

    match format {
        OutputFormat::Text => {
            eprintln!("Rebase complete:");
            eprintln!("  Pages updated: {}", result.pages_updated);
            eprintln!("  Conflicts: {}", result.conflicts);
            eprintln!("  Output: {}", result.output_dir.display());
        }
        OutputFormat::Json => {
            let json = serde_json::json!({
                "pages_updated": result.pages_updated,
                "conflicts": result.conflicts,
                "output_dir": result.output_dir.to_string_lossy(),
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
    }

    Ok(())
}