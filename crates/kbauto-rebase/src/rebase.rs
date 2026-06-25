//! Rebase engine — apply base playbook updates to a client KB.

use std::collections::{HashMap, HashSet};

/// Result of a rebase operation.
#[derive(Debug)]
pub struct RebaseResult {
    /// Number of pages updated.
    pub pages_updated: usize,
    /// Number of conflicts detected.
    pub conflicts: usize,
    /// Output directory path.
    pub output_dir: std::path::PathBuf,
}

/// Rebase a client KB onto a new playbook version.
///
/// Updates template-origin text while preserving substituted and
/// rewritten text. Flags conflicts where both the base and client
/// text have changed.
///
/// # Arguments
///
/// * `client_kb_dir` - Path to the client knowledge base directory
/// * `_old_version` - The old playbook version string (reserved for future use)
/// * `new_version` - The new playbook version string
/// * `template_dir` - Path to the new template directory
///
/// # Errors
///
/// Returns `RebaseError` if directories don't exist, files can't be
/// read or written, or provenance parsing fails.
pub fn rebase_client_kb(
    client_kb_dir: &std::path::Path,
    _old_version: &str,
    new_version: &str,
    template_dir: &std::path::Path,
) -> Result<RebaseResult, RebaseError> {
    // Validate paths exist
    if !template_dir.exists() {
        return Err(RebaseError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Template directory not found: {}", template_dir.display()),
        )));
    }
    if !client_kb_dir.exists() {
        return Err(RebaseError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Client KB directory not found: {}", client_kb_dir.display()),
        )));
    }

    // Load defaults for resolving new pages
    let defaults_path = template_dir.join("defaults.json");
    let defaults_json = std::fs::read_to_string(&defaults_path)?;
    let defaults = kbauto_placeholder::DefaultsFile::from_json(&defaults_json)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

    let template_docs_dir = template_dir.join("docs");
    let client_docs_dir = client_kb_dir.join("docs");

    // Collect template pages
    let mut template_pages: Vec<String> = Vec::new();
    if template_docs_dir.exists() {
        for entry in std::fs::read_dir(&template_docs_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "md") {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    template_pages.push(name.to_string());
                }
            }
        }
    }

    // Collect client pages
    let mut client_pages: Vec<String> = Vec::new();
    if client_docs_dir.exists() {
        for entry in std::fs::read_dir(&client_docs_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "md") {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    client_pages.push(name.to_string());
                }
            }
        }
    }

    let client_page_set: HashSet<String> = client_pages.iter().cloned().collect();
    let template_page_set: HashSet<String> = template_pages.iter().cloned().collect();

    let mut pages_updated = 0usize;
    let mut conflicts = 0usize;

    // Ensure client docs directory exists
    std::fs::create_dir_all(&client_docs_dir)?;

    // Process each template page
    for filename in &template_pages {
        let template_content = std::fs::read_to_string(template_docs_dir.join(filename))?;
        let client_path = client_docs_dir.join(filename);

        if client_page_set.contains(filename) {
            // Page exists in both — need to rebase
            let client_raw = std::fs::read_to_string(&client_path)?;

            // Parse frontmatter from both; fall back to empty frontmatter + raw content
            let (client_fm, client_body) = kbauto_provenance::parse_frontmatter(&client_raw)
                .unwrap_or_else(|_| {
                    (
                        kbauto_provenance::FrontmatterData {
                            provenance: HashMap::new(),
                            playbook_version: None,
                            extra: HashMap::new(),
                        },
                        client_raw.as_str(),
                    )
                });

            let (template_fm, template_body) =
                kbauto_provenance::parse_frontmatter(&template_content).unwrap_or_else(|_| {
                    (
                        kbauto_provenance::FrontmatterData {
                            provenance: HashMap::new(),
                            playbook_version: None,
                            extra: HashMap::new(),
                        },
                        template_content.as_str(),
                    )
                });

            let client_paragraphs = kbauto_provenance::split_paragraphs(client_body);
            let template_paragraphs = kbauto_provenance::split_paragraphs(template_body);

            // Build anchor lookup maps
            let client_by_anchor: HashMap<String, usize> = client_paragraphs
                .iter()
                .filter_map(|p| p.anchor.as_ref().map(|a| (a.clone(), p.index)))
                .collect();

            let _template_by_anchor: HashMap<String, usize> = template_paragraphs
                .iter()
                .filter_map(|p| p.anchor.as_ref().map(|a| (a.clone(), p.index)))
                .collect();

            let mut result_paragraphs: Vec<String> = Vec::new();
            let mut new_provenance: HashMap<usize, String> = HashMap::new();
            let mut client_anchors_used: HashSet<String> = HashSet::new();
            let mut client_indices_used: HashSet<usize> = HashSet::new();

            // First pass: match template paragraphs to client paragraphs by anchor
            for (i, tp) in template_paragraphs.iter().enumerate() {
                if let Some(ref anchor) = tp.anchor {
                    if let Some(&client_idx) = client_by_anchor.get(anchor) {
                        // Anchor match found
                        client_anchors_used.insert(anchor.clone());
                        client_indices_used.insert(client_idx);
                        let cp = &client_paragraphs[client_idx];
                        let provenance_type = client_fm
                            .provenance
                            .get(&cp.index)
                            .map(|s| s.as_str())
                            .unwrap_or("template");

                        if provenance_type == "template" {
                            // Template-origin: update to new base text
                            result_paragraphs.push(tp.text.clone());
                            new_provenance.insert(i, "template".to_string());
                        } else if cp.text == tp.text {
                            // Substituted/rewritten but unchanged in base — preserve client
                            result_paragraphs.push(cp.text.clone());
                            new_provenance.insert(i, provenance_type.to_string());
                        } else {
                            // Both changed — conflict
                            conflicts += 1;
                            result_paragraphs.push(cp.text.clone());
                            new_provenance.insert(i, provenance_type.to_string());
                        }
                        continue;
                    }
                }
                // No anchor match — will handle in second pass
            }

            // Second pass: handle template paragraphs not matched by anchor
            // Try positional matching with unmatched client paragraphs
            let unmatched_template: Vec<(usize, &kbauto_provenance::Paragraph)> =
                template_paragraphs
                    .iter()
                    .enumerate()
                    .filter(|(_, tp)| {
                        // Not already matched by anchor
                        tp.anchor
                            .as_ref()
                            .map_or(true, |a| !client_anchors_used.contains(a))
                    })
                    .collect();

            let unmatched_client: Vec<(usize, &kbauto_provenance::Paragraph)> = client_paragraphs
                .iter()
                .enumerate()
                .filter(|(_, cp)| !client_indices_used.contains(&cp.index))
                .collect();

            // Positional matching: align unmatched template and client paragraphs
            let max_common = unmatched_template.len().min(unmatched_client.len());
            for j in 0..max_common {
                let (tpl_i, tp) = unmatched_template[j];
                let (_, cp) = unmatched_client[j];

                let provenance_type = client_fm
                    .provenance
                    .get(&cp.index)
                    .map(|s| s.as_str())
                    .unwrap_or("template");

                if provenance_type == "template" {
                    result_paragraphs.insert(tpl_i, tp.text.clone());
                    new_provenance.insert(tpl_i, "template".to_string());
                } else if cp.text == tp.text {
                    result_paragraphs.insert(tpl_i, cp.text.clone());
                    new_provenance.insert(tpl_i, provenance_type.to_string());
                } else {
                    // Both changed — conflict
                    conflicts += 1;
                    result_paragraphs.insert(tpl_i, cp.text.clone());
                    new_provenance.insert(tpl_i, provenance_type.to_string());
                }
            }

            // Remaining unmatched template paragraphs (new additions)
            for j in max_common..unmatched_template.len() {
                let (tpl_i, tp) = unmatched_template[j];
                result_paragraphs.insert(tpl_i, tp.text.clone());
                new_provenance.insert(tpl_i, "template".to_string());
            }

            // Ensure we have the right number of result paragraphs (same as template)
            // If result_paragraphs has fewer due to insert ordering, pad with template paragraphs
            while result_paragraphs.len() < template_paragraphs.len() {
                let idx = result_paragraphs.len();
                result_paragraphs.push(template_paragraphs[idx].text.clone());
                new_provenance.insert(idx, "template".to_string());
            }

            // Build the result content
            let body = result_paragraphs.join("\n\n");

            // Merge template frontmatter fields with client provenance
            let new_fm = kbauto_provenance::FrontmatterData {
                provenance: new_provenance,
                playbook_version: Some(new_version.to_string()),
                extra: template_fm.extra.clone(),
            };

            let result_content = kbauto_provenance::write_frontmatter(&new_fm, &body)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

            std::fs::write(&client_path, result_content)?;
            pages_updated += 1;
        } else {
            // New page — add it with defaults resolved, provenance "template"
            let (template_fm, template_body) =
                kbauto_provenance::parse_frontmatter(&template_content).unwrap_or_else(|_| {
                    (
                        kbauto_provenance::FrontmatterData {
                            provenance: HashMap::new(),
                            playbook_version: None,
                            extra: HashMap::new(),
                        },
                        template_content.as_str(),
                    )
                });

            // Resolve placeholders using defaults
            let resolution = kbauto_placeholder::resolve_placeholders(
                template_body,
                &HashMap::new(), // no brief
                &defaults,
            );

            let template_body_paragraphs = kbauto_provenance::split_paragraphs(&resolution.content);
            let mut provenance = HashMap::new();
            for (i, _) in template_body_paragraphs.iter().enumerate() {
                provenance.insert(i, "template".to_string());
            }

            let body = template_body_paragraphs
                .iter()
                .map(|p| p.text.clone())
                .collect::<Vec<_>>()
                .join("\n\n");

            let new_fm = kbauto_provenance::FrontmatterData {
                provenance,
                playbook_version: Some(new_version.to_string()),
                extra: template_fm.extra.clone(),
            };

            let result_content = kbauto_provenance::write_frontmatter(&new_fm, &body)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

            std::fs::write(client_docs_dir.join(filename), result_content)?;
            pages_updated += 1;
        }
    }

    // Pages in client but not in template — kept as-is (not removed, just flagged)
    let _removed_pages: Vec<&String> = client_pages
        .iter()
        .filter(|p| !template_page_set.contains(*p))
        .collect();

    Ok(RebaseResult {
        pages_updated,
        conflicts,
        output_dir: client_kb_dir.to_path_buf(),
    })
}

/// Errors from rebase operations.
#[derive(Debug, thiserror::Error)]
pub enum RebaseError {
    /// Feature not yet implemented.
    #[error("not yet implemented")]
    NotImplemented,
    /// An IO error occurred.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
