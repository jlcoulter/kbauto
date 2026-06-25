//! CLI `schema` subcommand.

use crate::output::OutputFormat;
use clap::Parser;
use kbauto_config::AppConfig;
use kbauto_placeholder::extract_placeholders;
use kbauto_template::list_page_files;
use std::collections::BTreeMap;

/// Display placeholder schema for a template directory.
#[derive(Parser, Debug)]
pub struct SchemaArgs {
    /// Path to the playbook template directory.
    #[arg(short, long)]
    pub template_dir: std::path::PathBuf,

    /// Output format: text (default) or json. Overrides config file.
    #[arg(long)]
    pub output_format: Option<String>,
}

/// A placeholder entry in the schema output.
#[derive(serde::Serialize)]
struct SchemaEntry {
    key: String,
    files: Vec<String>,
}

/// Run the schema subcommand.
pub fn run(args: SchemaArgs, config: &AppConfig) -> anyhow::Result<()> {
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

    // Load all page files and extract placeholders
    let page_files = list_page_files(&args.template_dir).map_err(|e| anyhow::anyhow!("{e}"))?;

    let mut schema: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for page_path in &page_files {
        let content = std::fs::read_to_string(page_path)?;
        let filename = page_path
            .strip_prefix(&docs_dir)
            .unwrap_or(page_path)
            .to_string_lossy()
            .to_string();

        let placeholders = extract_placeholders(&content, &filename);
        for p in placeholders {
            let canonical = kbauto_placeholder::canonical_key(&p.key);
            schema.entry(canonical).or_default().push(filename.clone());
        }
    }

    // Deduplicate file lists
    for entries in schema.values_mut() {
        entries.sort();
        entries.dedup();
    }

    match format {
        OutputFormat::Text => {
            println!("Placeholder Schema for {}", args.template_dir.display());
            println!("{}", "-".repeat(50));
            for (key, files) in &schema {
                println!("{key}");
                println!("  files: {}", files.join(", "));
            }
            println!("\nTotal placeholders: {}", schema.len());
        }
        OutputFormat::Json => {
            let entries: Vec<SchemaEntry> = schema
                .into_iter()
                .map(|(key, files)| SchemaEntry { key, files })
                .collect();
            let json = serde_json::to_string_pretty(&entries)?;
            println!("{json}");
        }
    }

    Ok(())
}
