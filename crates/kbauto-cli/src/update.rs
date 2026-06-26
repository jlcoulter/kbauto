//! CLI `update` subcommand for incremental details/discovery changes.

use crate::output::OutputFormat;
use clap::Parser;
use kbauto_config::AppConfig;
use kbauto_template::{StaticDetails, brief_diff, incremental_update};
use std::path::PathBuf;

/// Incrementally update a client KB when details or discovery context has changed.
///
/// Computes the diff between old and new details, identifies affected
/// pages, and regenerates only those pages. Unchanged pages are left
/// byte-for-byte identical in the output directory.
#[derive(Parser, Debug)]
pub struct UpdateArgs {
    /// Path to the playbook template directory.
    #[arg(short, long)]
    pub template_dir: PathBuf,

    /// Path to the old (previous) client static details markdown file.
    #[arg(long)]
    pub old_details: PathBuf,

    /// Path to the new (updated) client static details markdown file.
    #[arg(long)]
    pub new_details: PathBuf,

    /// Path to the old (previous) client discovery document (optional).
    #[arg(long)]
    pub old_discovery: Option<PathBuf>,

    /// Path to the new (updated) client discovery document (optional).
    #[arg(long)]
    pub new_discovery: Option<PathBuf>,

    /// Output directory for generated files (existing output to update).
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

/// Run the update subcommand.
pub fn run(args: UpdateArgs, config: &AppConfig) -> anyhow::Result<()> {
    let format: OutputFormat = match args.output_format.as_deref() {
        Some(s) => s.parse().unwrap_or(config.output_format),
        None => config.output_format,
    };

    // Validate template directory
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

    // Validate old details file
    if !args.old_details.exists() {
        crate::output::print_error(
            &format!("Old details file not found: {}", args.old_details.display()),
            &format,
        );
        std::process::exit(1);
    }

    // Validate new details file
    if !args.new_details.exists() {
        crate::output::print_error(
            &format!("New details file not found: {}", args.new_details.display()),
            &format,
        );
        std::process::exit(1);
    }

    // Validate old discovery file if provided
    if let Some(ref path) = args.old_discovery {
        if !path.exists() {
            crate::output::print_error(
                &format!("Old discovery file not found: {}", path.display()),
                &format,
            );
            std::process::exit(1);
        }
    }

    // Validate new discovery file if provided
    if let Some(ref path) = args.new_discovery {
        if !path.exists() {
            crate::output::print_error(
                &format!("New discovery file not found: {}", path.display()),
                &format,
            );
            std::process::exit(1);
        }
    }

    // Validate defaults.json
    let defaults_path = args.template_dir.join("defaults.json");
    if !defaults_path.exists() {
        crate::output::print_error(
            &format!("Missing defaults.json in: {}", args.template_dir.display()),
            &format,
        );
        std::process::exit(1);
    }

    // Load old and new details
    let old_details = StaticDetails::from_markdown_file(&args.old_details)
        .map_err(|e| anyhow::anyhow!("Failed to parse old details: {e}"))?;
    let new_details = StaticDetails::from_markdown_file(&args.new_details)
        .map_err(|e| anyhow::anyhow!("Failed to parse new details: {e}"))?;

    // TODO: When AI rewriting is integrated, load discovery documents and
    // pass them through. Discovery changes trigger re-rewriting of affected
    // paragraphs (provenance = "ai_rewritten").
    let _ = (args.old_discovery, args.new_discovery);

    // Compute diff for reporting
    let diff = brief_diff(&old_details.entries, &new_details.entries);

    // Load defaults
    let defaults_content = std::fs::read_to_string(&defaults_path)?;
    let defaults = kbauto_placeholder::DefaultsFile::from_json(&defaults_content)?;

    // Run incremental update
    let result = incremental_update(
        &args.template_dir,
        &args.output,
        &old_details.entries,
        &new_details.entries,
        &defaults,
    )?;

    // Output summary
    match format {
        OutputFormat::Text => {
            println!("Incremental update complete.");
            println!("  Pages updated:    {}", result.pages_updated);
            println!("  Pages unchanged:  {}", result.pages_unchanged);
            println!("  Placeholders changed: {}", result.placeholders_updated);
            if !diff.changed_keys.is_empty() {
                println!("  Changed keys:    {}", diff.changed_keys.join(", "));
            }
            if !diff.added_keys.is_empty() {
                println!("  Added keys:      {}", diff.added_keys.join(", "));
            }
            if !diff.removed_keys.is_empty() {
                println!("  Removed keys:    {}", diff.removed_keys.join(", "));
            }
        }
        OutputFormat::Json => {
            let summary = serde_json::json!({
                "pages_updated": result.pages_updated,
                "pages_unchanged": result.pages_unchanged,
                "placeholders_updated": result.placeholders_updated,
                "changed_keys": diff.changed_keys,
                "added_keys": diff.added_keys,
                "removed_keys": diff.removed_keys,
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&summary).unwrap_or_else(|_| "{}".to_string())
            );
        }
    }

    Ok(())
}
