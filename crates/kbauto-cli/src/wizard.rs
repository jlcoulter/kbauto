//! Guided TUI wizard — the primary entry point for `kbauto`.
//!
//! When the user runs `kbauto` with no subcommand, this module launches an
//! interactive wizard that detects the current phase by inspecting the
//! selected directory's contents:
//!
//! - **Scaffold phase**: No `details.md`/`discovery.md` present → create
//!   client directory structure with auto-generated skeleton files.
//! - **Generate phase**: Skeleton files exist but `kb/` is empty → generate
//!   the KB from the edited files.
//! - **Rebase or update**: `kb/` has generated content → offer rebase or
//!   incremental update.
//!
//! The wizard is fully stateless — no session files, no marker files. The
//! `.template-path` file is a configuration reference, not a session marker.

use kbauto_config::AppConfig;
use kbauto_placeholder::build_schema;
use kbauto_template::{
    generate_playbook, read_template_path, scaffold_client_dir, write_template_path,
};
use std::path::{Path, PathBuf};

/// Which phase the wizard should enter, based on directory contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WizardPhase {
    /// No client files exist yet — scaffold the directory.
    Scaffold,
    /// Skeleton files exist but no KB output — generate the KB.
    Generate,
    /// KB output exists — offer rebase or incremental update.
    RebaseOrUpdate,
}

/// Detect the wizard phase by inspecting the client directory's contents.
///
/// - No `details.md` or `discovery.md` → `Scaffold`
/// - Both skeleton files exist but `kb/` is empty → `Generate`
/// - `kb/` has generated content → `RebaseOrUpdate`
pub fn detect_phase(client_dir: &Path) -> WizardPhase {
    let has_details = client_dir.join("details.md").exists();
    let has_discovery = client_dir.join("discovery.md").exists();
    let kb_dir = client_dir.join("kb");

    if !has_details && !has_discovery {
        return WizardPhase::Scaffold;
    }

    if has_details && has_discovery {
        // Check if kb/ has any content
        if kb_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&kb_dir) {
                if entries.filter_map(|e| e.ok()).any(|_| true) {
                    return WizardPhase::RebaseOrUpdate;
                }
            }
        }
        // kb/ doesn't exist or is empty
        return WizardPhase::Generate;
    }

    // Only one of details/discovery exists — treat as Generate if kb/ is empty,
    // otherwise RebaseOrUpdate
    if kb_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&kb_dir) {
            if entries.filter_map(|e| e.ok()).any(|_| true) {
                return WizardPhase::RebaseOrUpdate;
            }
        }
    }
    WizardPhase::Generate
}

/// Run the guided wizard.
///
/// This is the main entry point called when `kbauto` is invoked with no
/// subcommand. It prompts for a client directory, detects the phase, and
/// dispatches to the appropriate handler.
pub async fn run_wizard(config: AppConfig) -> anyhow::Result<()> {
    println!("kbauto — Knowledge Base Playbook Automation");
    println!();

    // Prompt for client directory
    let client_dir = prompt_for_path(
        "Enter the path to the client directory (or a new directory to scaffold): ",
    )?;
    let client_dir = PathBuf::from(client_dir.trim());

    if !client_dir.exists() {
        // Directory doesn't exist — go straight to scaffold
        return run_scaffold_phase(&client_dir, &config).await;
    }

    let phase = detect_phase(&client_dir);

    match phase {
        WizardPhase::Scaffold => run_scaffold_phase(&client_dir, &config).await,
        WizardPhase::Generate => run_generate_phase(&client_dir, &config).await,
        WizardPhase::RebaseOrUpdate => run_rebase_or_update_phase(&client_dir, &config).await,
    }
}

/// Run the scaffold phase (Phase 1).
///
/// Prompts for a template directory, validates it, loads the schema, and
/// scaffolds the client directory with auto-generated skeleton files.
async fn run_scaffold_phase(client_dir: &Path, config: &AppConfig) -> anyhow::Result<()> {
    println!();
    println!("Phase 1: Scaffold New Client");
    println!("---------------------------");
    println!();

    // Prompt for template directory
    let template_dir = prompt_for_path("Enter the path to the playbook template directory: ")?;
    let template_dir = PathBuf::from(template_dir.trim());

    // Validate template directory (FR-020)
    if !template_dir.exists() {
        eprintln!(
            "Error: Template directory not found: {}",
            template_dir.display()
        );
        std::process::exit(1);
    }

    let docs_dir = template_dir.join("docs");
    if !docs_dir.exists() {
        eprintln!(
            "Error: Missing docs/ subdirectory in: {}",
            template_dir.display()
        );
        std::process::exit(1);
    }

    let defaults_path = template_dir.join("defaults.json");
    if !defaults_path.exists() {
        eprintln!(
            "Error: Missing defaults.json in: {}",
            template_dir.display()
        );
        std::process::exit(1);
    }

    // Load template and extract schema
    let template = kbauto_template::load_template(&template_dir)
        .map_err(|e| anyhow::anyhow!("Failed to load template: {e}"))?;

    let defaults_content = std::fs::read_to_string(&defaults_path)?;
    let defaults = kbauto_placeholder::DefaultsFile::from_json(&defaults_content)?;

    // Extract placeholders from all pages and build schema
    let mut all_placeholders = Vec::new();
    for page in &template.pages {
        let phs = kbauto_placeholder::extract_placeholders(&page.content, &page.filename);
        all_placeholders.extend(phs);
    }
    let schema = build_schema(all_placeholders, &template.version);

    // Scaffold the client directory
    scaffold_client_dir(client_dir, &template_dir, &schema, &defaults)?;

    println!();
    println!("Client directory scaffolded: {}", client_dir.display());
    println!();
    println!("Files created:");
    println!(
        "  {}/details.md   — Edit this with client-specific values",
        client_dir.display()
    );
    println!(
        "  {}/discovery.md — Edit this with client voice/tone answers",
        client_dir.display()
    );
    println!(
        "  {}/kb/          — Output directory (empty, will be populated on generation)",
        client_dir.display()
    );
    println!(
        "  {}/.template-path — Template directory reference",
        client_dir.display()
    );
    println!();
    println!("Next steps:");
    println!("  1. Edit details.md and discovery.md with the client's information");
    println!("  2. Run 'kbauto' again pointing at this directory to generate the KB");
    println!();

    Ok(())
}

/// Run the generate phase (Phase 2).
///
/// Reads `.template-path` to locate the template, loads the edited details and
/// discovery files, and generates the KB.
async fn run_generate_phase(client_dir: &Path, config: &AppConfig) -> anyhow::Result<()> {
    println!();
    println!("Phase 2: Generate Knowledge Base");
    println!("----------------------------------");
    println!();

    // Read .template-path to find the template directory
    let template_dir = match read_template_path(client_dir) {
        Ok(path) => path,
        Err(kbauto_template::TemplatePathError::Missing(_)) => {
            println!("No .template-path file found. Let's locate the template directory.");
            let new_path = prompt_for_path("Enter the path to the playbook template directory: ")?;
            let new_path = PathBuf::from(new_path.trim());

            // Validate the new path
            if !new_path.exists() || !new_path.join("docs").exists() {
                eprintln!("Error: Invalid template directory: {}", new_path.display());
                std::process::exit(1);
            }

            // Update .template-path
            write_template_path(client_dir, &new_path)?;
            new_path
        }
        Err(kbauto_template::TemplatePathError::Invalid(old_path)) => {
            println!(
                "The template directory recorded in .template-path no longer exists: {}",
                old_path
            );
            let new_path =
                prompt_for_path("Enter the correct path to the playbook template directory: ")?;
            let new_path = PathBuf::from(new_path.trim());

            // Validate the new path
            if !new_path.exists() || !new_path.join("docs").exists() {
                eprintln!("Error: Invalid template directory: {}", new_path.display());
                std::process::exit(1);
            }

            // Update .template-path
            write_template_path(client_dir, &new_path)?;
            println!("Updated .template-path to: {}", new_path.display());
            new_path
        }
        Err(e) => {
            return Err(anyhow::anyhow!("Error reading .template-path: {e}"));
        }
    };

    println!("Using template: {}", template_dir.display());
    println!();

    // Load details and discovery from the client directory
    let details_path = client_dir.join("details.md");
    let discovery_path = client_dir.join("discovery.md");

    let output_dir = client_dir.join("kb");

    println!("Generating knowledge base...");
    println!();

    let start = std::time::Instant::now();

    let result = generate_playbook(
        &template_dir,
        Some(&details_path),
        Some(&discovery_path),
        &output_dir,
    )
    .await?;

    let elapsed = start.elapsed();

    println!("Generation complete!");
    println!();
    println!("  Pages generated:     {}", result.pages_generated);
    println!("  Placeholders resolved: {}", result.placeholders_resolved);
    println!("  Output directory:    {}", output_dir.display());
    println!("  Time:                {:.2}s", elapsed.as_secs_f64());
    println!();

    // Check for missing values
    if !result.missing_values.is_empty() {
        println!(
            "Warning: {} missing value(s) were filled with defaults.",
            result.missing_values.len()
        );
        for mv in &result.missing_values {
            println!("  - {} (no value in details or defaults)", mv.key);
        }
        println!();
    }

    println!("Next steps:");
    println!(
        "  1. Preview: cd {} && npx docusaurus build",
        output_dir.display()
    );
    println!("  2. Publish when ready");
    println!();

    Ok(())
}

/// Run the rebase or update guidance phase.
///
/// Detects whether the template has been updated (offering rebase) or the
/// client details have changed (offering incremental update).
async fn run_rebase_or_update_phase(client_dir: &Path, config: &AppConfig) -> anyhow::Result<()> {
    println!();
    println!("Existing Client Knowledge Base");
    println!("-------------------------------");
    println!();

    let template_dir = match read_template_path(client_dir) {
        Ok(path) => path,
        Err(kbauto_template::TemplatePathError::Missing(_)) => {
            println!("No .template-path file found. Let's locate the template directory.");
            let new_path = prompt_for_path("Enter the path to the playbook template directory: ")?;
            let new_path = PathBuf::from(new_path.trim());
            if !new_path.exists() || !new_path.join("docs").exists() {
                eprintln!("Error: Invalid template directory: {}", new_path.display());
                std::process::exit(1);
            }
            write_template_path(client_dir, &new_path)?;
            new_path
        }
        Err(kbauto_template::TemplatePathError::Invalid(old_path)) => {
            println!(
                "The template directory recorded in .template-path no longer exists: {}",
                old_path
            );
            let new_path =
                prompt_for_path("Enter the correct path to the playbook template directory: ")?;
            let new_path = PathBuf::from(new_path.trim());
            if !new_path.exists() || !new_path.join("docs").exists() {
                eprintln!("Error: Invalid template directory: {}", new_path.display());
                std::process::exit(1);
            }
            write_template_path(client_dir, &new_path)?;
            println!("Updated .template-path to: {}", new_path.display());
            new_path
        }
        Err(e) => {
            return Err(anyhow::anyhow!("Error reading .template-path: {e}"));
        }
    };

    println!("Using template: {}", template_dir.display());
    println!();
    println!("Options:");
    println!("  1. Regenerate KB from scratch (full generation)");
    println!("  2. Rebase onto new template version (if template updated)");
    println!("  3. Incremental update (if details/discovery changed)");
    println!();

    let choice = prompt_for_path("Choose an option (1/2/3): ")?;
    let choice = choice.trim();

    match choice {
        "1" => {
            // Full regeneration
            let details_path = client_dir.join("details.md");
            let discovery_path = client_dir.join("discovery.md");
            let output_dir = client_dir.join("kb");

            // Clear existing kb/ contents
            if output_dir.exists() {
                std::fs::remove_dir_all(&output_dir)?;
            }
            std::fs::create_dir_all(&output_dir)?;

            println!("Regenerating knowledge base...");
            let start = std::time::Instant::now();
            let result = generate_playbook(
                &template_dir,
                Some(&details_path),
                Some(&discovery_path),
                &output_dir,
            )
            .await?;
            let elapsed = start.elapsed();

            println!();
            println!("Generation complete!");
            println!("  Pages generated:     {}", result.pages_generated);
            println!("  Placeholders resolved: {}", result.placeholders_resolved);
            println!("  Time:                {:.2}s", elapsed.as_secs_f64());
            println!();
        }
        "2" => {
            // Rebase — use the rebase subcommand logic
            println!();
            println!("Rebase: This will update template-origin text to the latest version");
            println!("while preserving your client-customised content.");
            println!();

            let new_version = prompt_for_path(
                "Enter the new template version to rebase onto (or press Enter to use current): ",
            )?;

            // Construct rebase args and call run
            let rebase_args = crate::rebase::RebaseArgs {
                client_kb_dir: client_dir.to_path_buf(),
                old_version: String::new(), // will be detected from frontmatter
                new_version: if new_version.trim().is_empty() {
                    String::new()
                } else {
                    new_version.trim().to_string()
                },
                template_dir,
                output_format: None,
                ollama_url: None,
                ollama_model: None,
                retry_count: None,
            };

            crate::rebase::run(&rebase_args, config)?;
        }
        "3" => {
            // Incremental update
            println!();
            println!("Incremental update: This will re-process only pages affected");
            println!("by changes to your details or discovery files.");
            println!();

            let old_details = client_dir.join("details.md");
            let new_details_path = prompt_for_path(
                "Enter path to updated details file (or press Enter to use current): ",
            )?;
            let new_details = if new_details_path.trim().is_empty() {
                old_details.clone()
            } else {
                PathBuf::from(new_details_path.trim())
            };

            let update_args = crate::update::UpdateArgs {
                template_dir,
                old_details,
                new_details,
                old_discovery: None,
                new_discovery: None,
                output: client_dir.join("kb"),
                ollama_url: None,
                ollama_model: None,
                retry_count: None,
                output_format: None,
            };

            crate::update::run(update_args, config)?;
        }
        _ => {
            println!("Invalid option. Exiting.");
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Prompt the user for input and return the response.
fn prompt_for_path(prompt: &str) -> anyhow::Result<String> {
    use std::io::Write;
    print!("{prompt}");
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input.trim_end().to_string())
}
