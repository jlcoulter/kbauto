//! Page-level and paragraph-level diffing.

use serde::{Deserialize, Serialize};

/// A change to a single page between two playbook versions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PageChange {
    /// New page added in the new version.
    Added {
        /// Filename of the added page.
        filename: String,
    },
    /// Page removed in the new version.
    Removed {
        /// Filename of the removed page.
        filename: String,
    },
    /// Page modified between versions.
    Modified {
        /// Filename of the modified page.
        filename: String,
        /// Paragraph-level changes within the page.
        paragraph_changes: Vec<ParagraphChange>,
    },
}

/// A change to a single paragraph within a page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParagraphChange {
    /// Paragraph index in the document.
    pub index: usize,
    /// Anchor for matching.
    pub anchor: Option<String>,
    /// Old paragraph text.
    pub old_text: String,
    /// New paragraph text.
    pub new_text: String,
}

/// A diff report between two playbook versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffReport {
    /// Old version string.
    pub old_version: String,
    /// New version string.
    pub new_version: String,
    /// Page-level changes.
    pub changes: Vec<PageChange>,
}

/// Check if a paragraph is a heading (starts with #).
fn is_heading(text: &str) -> bool {
    text.trim().starts_with('#')
}

/// Diff two pages at the paragraph level.
///
/// Returns a list of paragraph-level changes between old and new content.
/// Headings (lines starting with #) are skipped.
/// Matching is done by anchor first, then by positional index for unmatched paragraphs.
#[must_use = "diff results should be used"]
pub fn diff_pages(old_content: &str, new_content: &str) -> Vec<ParagraphChange> {
    let old_all = kbauto_provenance::split_paragraphs(old_content);
    let new_all = kbauto_provenance::split_paragraphs(new_content);

    // Filter out headings and re-index content paragraphs starting from 0
    let old_paragraphs: Vec<kbauto_provenance::Paragraph> = old_all
        .iter()
        .filter(|p| !is_heading(&p.text))
        .enumerate()
        .map(|(i, p)| kbauto_provenance::Paragraph {
            index: i,
            text: p.text.clone(),
            anchor: p.anchor.clone(),
        })
        .collect();

    let new_paragraphs: Vec<kbauto_provenance::Paragraph> = new_all
        .iter()
        .filter(|p| !is_heading(&p.text))
        .enumerate()
        .map(|(i, p)| kbauto_provenance::Paragraph {
            index: i,
            text: p.text.clone(),
            anchor: p.anchor.clone(),
        })
        .collect();

    // Build a map from anchor -> old paragraph for anchor-based matching
    let old_by_anchor: std::collections::HashMap<String, usize> = old_paragraphs
        .iter()
        .filter_map(|p| p.anchor.as_ref().map(|a| (a.clone(), p.index)))
        .collect();

    let mut changes = Vec::new();
    let mut matched_old_indices: std::collections::HashSet<usize> =
        std::collections::HashSet::new();

    // First pass: match new paragraphs to old paragraphs by anchor
    for new_p in &new_paragraphs {
        if let Some(ref anchor) = new_p.anchor {
            if let Some(&old_idx) = old_by_anchor.get(anchor) {
                // Anchor match found
                matched_old_indices.insert(old_idx);
                let old_p = &old_paragraphs[old_idx];
                if old_p.text != new_p.text {
                    changes.push(ParagraphChange {
                        index: new_p.index,
                        anchor: Some(anchor.clone()),
                        old_text: old_p.text.clone(),
                        new_text: new_p.text.clone(),
                    });
                }
            }
        }
    }

    // Collect unmatched new and old paragraphs for positional matching
    let unmatched_new: Vec<&kbauto_provenance::Paragraph> = new_paragraphs
        .iter()
        .filter(|p| {
            // Not matched by anchor (either no anchor, or anchor not found in old)
            p.anchor
                .as_ref()
                .map_or(true, |a| !old_by_anchor.contains_key(a))
        })
        .collect();

    let unmatched_old: Vec<&kbauto_provenance::Paragraph> = old_paragraphs
        .iter()
        .filter(|p| !matched_old_indices.contains(&p.index))
        .collect();

    // Match unmatched paragraphs by position (align sequentially)
    let max_common = unmatched_new.len().min(unmatched_old.len());
    for i in 0..max_common {
        let new_p = unmatched_new[i];
        let old_p = unmatched_old[i];
        if old_p.text != new_p.text {
            changes.push(ParagraphChange {
                index: new_p.index,
                anchor: new_p.anchor.clone(),
                old_text: old_p.text.clone(),
                new_text: new_p.text.clone(),
            });
        }
    }

    // Remaining unmatched new paragraphs are additions
    for i in max_common..unmatched_new.len() {
        let new_p = unmatched_new[i];
        changes.push(ParagraphChange {
            index: new_p.index,
            anchor: new_p.anchor.clone(),
            old_text: String::new(),
            new_text: new_p.text.clone(),
        });
    }

    // Remaining unmatched old paragraphs are removals
    for i in max_common..unmatched_old.len() {
        let old_p = unmatched_old[i];
        changes.push(ParagraphChange {
            index: old_p.index,
            anchor: old_p.anchor.clone(),
            old_text: old_p.text.clone(),
            new_text: String::new(),
        });
    }

    // Sort by index
    changes.sort_by_key(|c| c.index);
    changes
}

/// Diff two playbook versions at the page level.
///
/// Compares directories and categorises pages as added, removed, or modified.
///
/// # Errors
///
/// Returns an error if the directories cannot be read or defaults.json
/// files cannot be parsed.
pub fn diff_playbooks(
    old_dir: &std::path::Path,
    new_dir: &std::path::Path,
) -> Result<DiffReport, DiffError> {
    // Read versions from defaults.json
    let old_defaults_path = old_dir.join("defaults.json");
    let new_defaults_path = new_dir.join("defaults.json");

    let old_defaults_json = std::fs::read_to_string(&old_defaults_path)?;
    let new_defaults_json = std::fs::read_to_string(&new_defaults_path)?;

    let old_defaults = kbauto_placeholder::DefaultsFile::from_json(&old_defaults_json)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    let new_defaults = kbauto_placeholder::DefaultsFile::from_json(&new_defaults_json)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

    // List .md files from both docs/ subdirectories
    let old_docs_dir = old_dir.join("docs");
    let new_docs_dir = new_dir.join("docs");

    let mut old_files: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut new_files: std::collections::HashSet<String> = std::collections::HashSet::new();

    if old_docs_dir.exists() {
        for entry in std::fs::read_dir(&old_docs_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "md") {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    old_files.insert(name.to_string());
                }
            }
        }
    }

    if new_docs_dir.exists() {
        for entry in std::fs::read_dir(&new_docs_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "md") {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    new_files.insert(name.to_string());
                }
            }
        }
    }

    let mut changes = Vec::new();

    // Added files (in new but not in old)
    for filename in &new_files {
        if !old_files.contains(filename) {
            changes.push(PageChange::Added {
                filename: filename.clone(),
            });
        }
    }

    // Removed files (in old but not in new)
    for filename in &old_files {
        if !new_files.contains(filename) {
            changes.push(PageChange::Removed {
                filename: filename.clone(),
            });
        }
    }

    // Modified files (in both)
    for filename in &old_files {
        if new_files.contains(filename) {
            let old_content = std::fs::read_to_string(old_docs_dir.join(filename))?;
            let new_content = std::fs::read_to_string(new_docs_dir.join(filename))?;

            let paragraph_changes = diff_pages(&old_content, &new_content);
            if !paragraph_changes.is_empty() {
                changes.push(PageChange::Modified {
                    filename: filename.clone(),
                    paragraph_changes,
                });
            }
        }
    }

    Ok(DiffReport {
        old_version: old_defaults.version,
        new_version: new_defaults.version,
        changes,
    })
}

impl std::fmt::Display for DiffReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let added: Vec<_> = self
            .changes
            .iter()
            .filter_map(|c| match c {
                PageChange::Added { filename } => Some(filename.clone()),
                _ => None,
            })
            .collect();
        let removed: Vec<_> = self
            .changes
            .iter()
            .filter_map(|c| match c {
                PageChange::Removed { filename } => Some(filename.clone()),
                _ => None,
            })
            .collect();
        let modified: Vec<_> = self
            .changes
            .iter()
            .filter_map(|c| match c {
                PageChange::Modified { filename, .. } => Some(filename.clone()),
                _ => None,
            })
            .collect();

        writeln!(f, "Diff: {} → {}", self.old_version, self.new_version)?;
        if self.changes.is_empty() {
            writeln!(f, "No changes detected.")?;
        } else {
            if !added.is_empty() {
                writeln!(f, "Added ({} pages):", added.len())?;
                for f_name in &added {
                    writeln!(f, "  + {f_name}")?;
                }
            }
            if !removed.is_empty() {
                writeln!(f, "Removed ({} pages):", removed.len())?;
                for f_name in &removed {
                    writeln!(f, "  - {f_name}")?;
                }
            }
            if !modified.is_empty() {
                writeln!(f, "Modified ({} pages):", modified.len())?;
                for f_name in &modified {
                    writeln!(f, "  ~ {f_name}")?;
                }
            }
        }
        Ok(())
    }
}

/// Errors from diff operations.
#[derive(Debug, thiserror::Error)]
pub enum DiffError {
    /// Feature not yet implemented.
    #[error("not yet implemented")]
    NotImplemented,
    /// An IO error occurred.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
