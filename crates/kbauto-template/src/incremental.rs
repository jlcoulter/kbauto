//! Incremental update pipeline.
//!
//! When a client's static details or discovery document changes, only pages
//! whose placeholders are affected need to be regenerated. Unchanged pages
//! are copied byte-for-byte from the existing output, preserving their content
//! exactly.

use crate::affect::find_affected_pages;
use crate::brief_diff::brief_diff;
use crate::template::load_template;
use kbauto_placeholder::{DefaultsFile, resolve_placeholders};
use kbauto_provenance::{FrontmatterData, parse_frontmatter, write_frontmatter};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Result of an incremental update operation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IncrementalResult {
    /// Number of pages that were regenerated.
    pub pages_updated: usize,
    /// Number of pages left unchanged (byte-identical copy).
    pub pages_unchanged: usize,
    /// Total number of placeholders whose values changed.
    pub placeholders_updated: usize,
    /// Number of provenance markers updated (paragraphs re-classified).
    pub provenance_updates: usize,
}

/// Perform an incremental update when static details or discovery context changes.
///
/// 1. Compute the diff between old and new details.
/// 2. Find which template pages contain the changed placeholder keys.
/// 3. Regenerate only those pages, copying all others byte-for-byte.
/// 4. Update provenance metadata on regenerated pages only.
///
/// # Arguments
///
/// * `template_dir`  – Path to the playbook template directory.
/// * `output_dir`    – Path to the existing output directory (will be updated in-place).
/// * `old_details`   – The previous details entry map (from `StaticDetails::entries`).
/// * `new_details`   – The new details entry map (from `StaticDetails::entries`).
/// * `defaults`      – Defaults file for placeholder resolution fallback.
///
/// # Errors
///
/// Returns an error if the template cannot be loaded, files cannot be
/// read or written, or provenance parsing fails on existing output files.
pub fn incremental_update(
    template_dir: &Path,
    output_dir: &Path,
    old_details: &HashMap<String, String>,
    new_details: &HashMap<String, String>,
    defaults: &DefaultsFile,
) -> anyhow::Result<IncrementalResult> {
    // 1. Compute details diff
    let diff = brief_diff(old_details, new_details);

    // All keys that changed value, were added, or were removed
    let all_changed_keys: Vec<String> = diff
        .changed_keys
        .iter()
        .chain(diff.added_keys.iter())
        .chain(diff.removed_keys.iter())
        .cloned()
        .collect();

    let placeholders_updated = all_changed_keys.len();

    // 2. Load template
    let template = load_template(template_dir).map_err(|e| anyhow::anyhow!("{e}"))?;

    // 3. Find affected pages
    let affected_filenames = find_affected_pages(&template, &all_changed_keys);
    let affected_set: std::collections::HashSet<&str> =
        affected_filenames.iter().map(|s| s.as_str()).collect();

    let mut pages_updated = 0;
    let mut pages_unchanged = 0;
    let mut provenance_updates = 0;

    // 4. Process each page
    for page in &template.pages {
        if affected_set.contains(page.filename.as_str()) {
            // Regenerate this page with the new details
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
            // Resolve placeholders with the new details
            let result = resolve_placeholders(&body, new_details, defaults);

            // Track provenance for each paragraph
            let paragraphs = kbauto_provenance::split_paragraphs(&result.content);
            frontmatter_data.provenance.clear();
            let mut page_prov_updates = 0;
            for (i, para) in paragraphs.iter().enumerate() {
                let has_placeholder = page.placeholders.iter().any(|key| {
                    para.text.contains(key)
                        || para.text.contains(&kbauto_placeholder::canonical_key(key))
                });

                let provenance = if has_placeholder {
                    "substituted".to_string()
                } else {
                    let was_resolved = result.resolved.iter().any(|r| para.text.contains(&r.value));
                    if was_resolved {
                        "substituted".to_string()
                    } else {
                        "template".to_string()
                    }
                };
                page_prov_updates += 1;
                frontmatter_data.provenance.insert(i, provenance);
            }
            provenance_updates += page_prov_updates;

            let output_content = write_frontmatter(&frontmatter_data, &result.content)?;

            // Write to file
            let output_path = output_dir.join(&page.filename);
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&output_path, output_content)?;

            pages_updated += 1;
        } else {
            // Copy unchanged page byte-for-byte from existing output
            // If the output file already exists, leave it untouched.
            // If it doesn't exist (first-time incremental on a fresh dir), generate it.
            let output_path = output_dir.join(&page.filename);
            if output_path.exists() {
                // Byte-identical – no changes needed
                pages_unchanged += 1;
            } else {
                // Need to generate this page for the first time with the new brief
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

                frontmatter_data.playbook_version = Some(template.version.clone());

                let result = resolve_placeholders(&body, new_details, defaults);

                let paragraphs = kbauto_provenance::split_paragraphs(&result.content);
                frontmatter_data.provenance.clear();
                for (i, para) in paragraphs.iter().enumerate() {
                    let has_placeholder = page.placeholders.iter().any(|key| {
                        para.text.contains(key)
                            || para.text.contains(&kbauto_placeholder::canonical_key(key))
                    });
                    let provenance = if has_placeholder {
                        "substituted".to_string()
                    } else {
                        let was_resolved =
                            result.resolved.iter().any(|r| para.text.contains(&r.value));
                        if was_resolved {
                            "substituted".to_string()
                        } else {
                            "template".to_string()
                        }
                    };
                    frontmatter_data.provenance.insert(i, provenance);
                }

                let output_content = write_frontmatter(&frontmatter_data, &result.content)?;

                if let Some(parent) = output_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&output_path, output_content)?;

                pages_unchanged += 1;
            }
        }
    }

    Ok(IncrementalResult {
        pages_updated,
        pages_unchanged,
        placeholders_updated,
        provenance_updates,
    })
}
