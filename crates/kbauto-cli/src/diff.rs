//! CLI `diff` subcommand.

use crate::output::OutputFormat;
use clap::Parser;
use kbauto_config::AppConfig;
use kbauto_rebase::diff_playbooks;
use std::path::PathBuf;

/// Display a diff report between two playbook versions.
///
/// Compares two version directories and lists added, removed,
/// and modified pages with change summaries.
#[derive(Parser, Debug)]
pub struct DiffArgs {
    /// Path to the old version template directory.
    #[arg(long)]
    pub old_version_dir: PathBuf,

    /// Path to the new version template directory.
    #[arg(long)]
    pub new_version_dir: PathBuf,

    /// Output format: text (default) or json. Overrides config file.
    #[arg(long)]
    pub output_format: Option<String>,
}

/// Run the diff subcommand.
pub fn run(args: &DiffArgs, config: &AppConfig) -> anyhow::Result<()> {
    let format: OutputFormat = match args.output_format.as_deref() {
        Some(s) => s.parse().unwrap_or(config.output_format),
        None => config.output_format,
    };

    if !args.old_version_dir.exists() {
        anyhow::bail!(
            "Old version directory not found: {}",
            args.old_version_dir.display()
        );
    }
    if !args.new_version_dir.exists() {
        anyhow::bail!(
            "New version directory not found: {}",
            args.new_version_dir.display()
        );
    }

    let report = diff_playbooks(&args.old_version_dir, &args.new_version_dir)?;

    match format {
        OutputFormat::Text => {
            println!("{report}");
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
    }

    Ok(())
}