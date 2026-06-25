//! kbauto: Knowledge Base Playbook Automation CLI
//!
//! Generate, rebase, and diff client-specific knowledge base playbooks
//! from versioned Docusaurus templates.
//!
//! Running `kbauto` with no subcommand launches a guided TUI wizard that
//! walks the user through the full client lifecycle (scaffold → generate →
//! rebase/update). All subcommands remain available as an expert mode for
//! scripting and automation.

use clap::Parser;
use kbauto_config::AppConfig;

mod diff;
mod generate;
mod output;
mod progress;
mod rebase;
mod schema;
mod tui;
mod update;
mod wizard;

/// Knowledge Base Playbook Automation
#[derive(Parser, Debug)]
#[command(
    name = "kbauto",
    version,
    about = "Generate, rebase, and diff client KB playbooks"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

/// Available CLI subcommands (expert mode for scripting/automation).
#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Generate a client KB from a template and details/discovery
    Generate(generate::GenerateArgs),
    /// Display placeholder schema for a template directory
    Schema(schema::SchemaArgs),
    /// Rebase a client KB onto a new playbook version
    Rebase(rebase::RebaseArgs),
    /// Incrementally update a client KB when only the details/discovery have changed
    Update(update::UpdateArgs),
    /// Display a diff report between two playbook versions
    Diff(diff::DiffArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Auto-create config file with defaults if it doesn't exist (FR-023 update).
    if let Err(e) = AppConfig::ensure_config_exists() {
        // Non-fatal: if we can't auto-create, proceed with in-memory defaults
        eprintln!("Warning: could not auto-create config file: {e}");
    }

    // Load config file (XDG path), gracefully falling back to defaults
    // when the file is missing, but erroring on malformed TOML (FR-023).
    let config = match AppConfig::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

    match cli.command {
        // No subcommand → launch the guided wizard (FR-014)
        None => wizard::run_wizard(config).await?,
        // Expert mode: subcommands behave identically to wizard steps but
        // driven by CLI flags and arguments.
        Some(Commands::Generate(args)) => generate::run(args, config).await?,
        Some(Commands::Schema(args)) => schema::run(args, &config)?,
        Some(Commands::Rebase(args)) => rebase::run(&args, &config)?,
        Some(Commands::Update(args)) => update::run(args, &config)?,
        Some(Commands::Diff(args)) => diff::run(&args, &config)?,
    }
    Ok(())
}