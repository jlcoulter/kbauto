//! Playbook generation pipeline.

use crate::details::StaticDetails;
use crate::discovery::DiscoveryDocument;
use crate::template::load_template;
use crate::tui_model::MissingValue;
use kbauto_placeholder::{DefaultsFile, resolve_placeholders};
use kbauto_provenance::{FrontmatterData, parse_frontmatter, write_frontmatter};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Result of generating a client playbook.
#[derive(Debug)]
pub struct GenerationResult {
    /// Output directory where generated files were written.
    pub output_dir: std::path::PathBuf,
    /// Number of pages generated.
    pub pages_generated: usize,
    /// Number of placeholders resolved.
    pub placeholders_resolved: usize,
    /// Placeholders that were not resolved (no value in details or defaults).
    /// These should be collected by the caller for TUI input or error reporting.
    pub missing_values: Vec<MissingValue>,
}

/// Generate a client playbook from a template, static details, and discovery document.
///
/// Orchestrates the full generation pipeline:
/// 1. Load template and defaults
/// 2. Extract placeholders
/// 3. Resolve placeholders from static details (steps 1-5)
/// 4. AI-rewrite sections using discovery context (steps 6-10) with retry/fallback
/// 5. Track provenance at paragraph level
/// 6. Write output with Docusaurus frontmatter
///
/// - `details_path`: Path to static details markdown file (heading-value pairs).
///   If `None`, default-only preview is generated.
/// - `discovery_path`: Path to discovery document markdown file (Q&A pairs).
///   If `None`, only substitution is performed (no AI rewriting).
/// - When both are `None`, a default-only preview is generated.
///
/// # Errors
///
/// Returns an error if the template cannot be loaded, the details/discovery
/// files are invalid, or the generation pipeline fails.
pub async fn generate_playbook(
    template_dir: &Path,
    details_path: Option<&Path>,
    discovery_path: Option<&Path>,
    output_dir: &Path,
) -> anyhow::Result<GenerationResult> {
    // 1. Load template
    let template = load_template(template_dir).map_err(|e| anyhow::anyhow!("{e}"))?;

    // 2. Load defaults
    let defaults_content = fs::read_to_string(template_dir.join("defaults.json"))?;
    let defaults = DefaultsFile::from_json(&defaults_content)?;

    // 3. Load static details if provided (heading-value pairs for substitution)
    let details: HashMap<String, String> = if let Some(dp) = details_path {
        let static_details =
            StaticDetails::from_markdown_file(dp).map_err(|e| anyhow::anyhow!("{e}"))?;
        static_details.entries
    } else {
        HashMap::new()
    };

    // 4. Load discovery document if provided (Q&A pairs for AI rewriting context)
    let _discovery: Option<DiscoveryDocument> = if let Some(disc_path) = discovery_path {
        Some(
            DiscoveryDocument::from_markdown_file(disc_path)
                .map_err(|e| anyhow::anyhow!("{e}"))?,
        )
    } else {
        None
    };

    // 5. Create output directory
    fs::create_dir_all(output_dir)?;

    let mut pages_generated = 0;
    let mut placeholders_resolved = 0;
    let mut missing_keys: HashMap<String, Option<String>> = HashMap::new(); // key -> default

    // 6. Process each page
    for page in &template.pages {
        // Try to parse frontmatter to preserve it
        let (mut frontmatter_data, body) =
            if let Ok((data, body)) = parse_frontmatter(&page.content) {
                (data, body.to_string())
            } else {
                (
                    FrontmatterData {
                        provenance: HashMap::new(),
                        playbook_version: Some(template.version.clone()),
                        extra: HashMap::new(),
                    },
                    page.content.clone(),
                )
            };

        // Ensure playbook_version is set
        frontmatter_data.playbook_version = Some(template.version.clone());

        // Resolve placeholders in the body
        let result = resolve_placeholders(&body, &details, &defaults);

        // Collect unresolved placeholder keys
        for key in &result.unresolved_keys {
            missing_keys.entry(key.clone()).or_insert_with(|| {
                defaults.get_default(key).map(|s| s.to_string())
            });
        }

        // Track provenance for each paragraph
        let paragraphs = kbauto_provenance::split_paragraphs(&result.content);
        for (i, para) in paragraphs.iter().enumerate() {
            let has_placeholder = page.placeholders.iter().any(|key| {
                // Check if this placeholder was in this paragraph
                para.text.contains(key)
                    || para.text.contains(&kbauto_placeholder::canonical_key(key))
            });

            let provenance = if !has_placeholder && !para.text.trim().is_empty() {
                // If the paragraph had placeholders resolved, it's substituted
                // Otherwise, check if any resolved placeholder was in this paragraph
                let was_resolved = result.resolved.iter().any(|r| para.text.contains(&r.value));
                if was_resolved {
                    "substituted".to_string()
                } else {
                    "template".to_string()
                }
            } else if has_placeholder {
                "substituted".to_string()
            } else {
                // Check if this paragraph contains resolved values
                let was_resolved = result.resolved.iter().any(|r| para.text.contains(&r.value));
                if was_resolved {
                    "substituted".to_string()
                } else {
                    "template".to_string()
                }
            };

            frontmatter_data.provenance.insert(i, provenance);
        }

        // TODO: When AI rewriting is implemented, use discovery context here
        // to rewrite paragraphs, updating provenance to "ai_rewritten" where applicable.
        // Currently only substitution is performed.

        placeholders_resolved += result.resolved.len();

        // Write the output with frontmatter
        let output_content = write_frontmatter(&frontmatter_data, &result.content)?;

        // Write to file
        let output_path = output_dir.join(&page.filename);
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&output_path, output_content)?;

        pages_generated += 1;
    }

    // Build MissingValue list from collected unresolved keys
    let missing_values: Vec<MissingValue> = missing_keys
        .into_iter()
        .map(|(key, default)| MissingValue {
            key: key.clone(),
            description: format!("Placeholder {{{key}}} was not resolved"),
            default,
        })
        .collect();

    Ok(GenerationResult {
        output_dir: output_dir.to_path_buf(),
        pages_generated,
        placeholders_resolved,
        missing_values,
    })
}