//! Docusaurus frontmatter parsing and writing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Data extracted from Docusaurus frontmatter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrontmatterData {
    /// Provenance classifications keyed by paragraph index.
    pub provenance: HashMap<usize, String>,
    /// Playbook version this page was generated from.
    pub playbook_version: Option<String>,
    /// Any additional Docusaurus frontmatter fields to preserve.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

/// Parse frontmatter from a markdown string.
///
/// Expects `---` delimiters at the top. Returns the parsed frontmatter
/// and the remaining body content.
///
/// # Errors
///
/// Returns an error if the YAML is invalid or delimiters are missing.
#[must_use = "parsing frontmatter must be checked for errors"]
pub fn parse_frontmatter(content: &str) -> Result<(FrontmatterData, &str), FrontmatterError> {
    // Check that content starts with ---
    if !content.starts_with("---") {
        return Err(FrontmatterError::MissingDelimiters);
    }

    // Find the closing ---
    // Skip the first --- line, then find the next ---
    let after_first = &content[3..];
    let rest = after_first.trim_start_matches(['\r', '\n']);

    // Find the closing ---
    let _closing_pos = rest.find("\n---").or_else(|| {
        // Handle case where closing --- is at end without trailing newline
        if rest.contains("---") {
            // Find position of --- after newline or start
            let pos = rest.find("---")?;
            Some(pos.checked_sub(0)?)
        } else {
            None
        }
    });

    // Actually, let's be more careful. We need to find the second ---
    // that starts at the beginning of a line.
    let yaml_content;
    let body;

    // Strategy: split on "\n---" and take the first part as YAML
    // The closing --- must be at the start of a line
    let lines: Vec<&str> = content.lines().collect();
    let mut closing_line = None;

    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            closing_line = Some(i);
            break;
        }
    }

    match closing_line {
        None => Err(FrontmatterError::MissingDelimiters),
        Some(end_idx) => {
            // YAML content is between line 0 (opening ---) and end_idx (closing ---)
            let yaml_lines: Vec<&str> = lines[1..end_idx].to_vec();
            yaml_content = yaml_lines.join("\n");

            // Body is everything after the closing ---
            let body_start: usize =
                lines[0..=end_idx].iter().map(|l| l.len()).sum::<usize>() + end_idx; // +end_idx for newlines
            body = if content.len() > body_start {
                content[body_start..].trim_start_matches(['\r', '\n'])
            } else {
                ""
            };

            // Parse the YAML
            let yaml_value: serde_yaml::Value =
                serde_yaml::from_str(&yaml_content).map_err(FrontmatterError::YamlError)?;

            // Extract known fields
            let mut provenance = HashMap::new();
            let mut playbook_version = None;
            let mut extra = HashMap::new();

            if let serde_yaml::Value::Mapping(mapping) = yaml_value {
                for (key, value) in mapping {
                    if let serde_yaml::Value::String(key_str) = key {
                        match key_str.as_str() {
                            "provenance" => {
                                // provenance is a mapping of paragraph_index -> classification
                                if let serde_yaml::Value::Mapping(prop_map) = value {
                                    for (pk, pv) in prop_map {
                                        if let (
                                            serde_yaml::Value::Number(idx),
                                            serde_yaml::Value::String(cls),
                                        ) = (pk, pv)
                                        {
                                            if let Some(idx_u64) = idx.as_u64() {
                                                provenance.insert(idx_u64 as usize, cls.clone());
                                            }
                                        }
                                    }
                                }
                            }
                            "playbook_version" => {
                                if let serde_yaml::Value::String(v) = &value {
                                    playbook_version = Some(v.clone());
                                } else if let serde_yaml::Value::Number(n) = &value {
                                    // Handle numeric versions like "1.0.0" that might parse as string
                                    playbook_version = Some(n.to_string());
                                }
                                // Also keep it in extra
                                extra.insert(key_str.clone(), value);
                            }
                            _ => {
                                extra.insert(key_str.clone(), value);
                            }
                        }
                    }
                }
            }

            let data = FrontmatterData {
                provenance,
                playbook_version,
                extra,
            };

            Ok((data, body))
        }
    }
}

/// Write frontmatter back to a markdown string with `---` delimiters.
///
/// # Errors
///
/// Returns an error if YAML serialization fails.
#[must_use = "serialization result should be used"]
pub fn write_frontmatter(data: &FrontmatterData, body: &str) -> Result<String, FrontmatterError> {
    // Build the YAML mapping
    let mut mapping = serde_yaml::Mapping::new();

    // Insert extra fields first (title, sidebar_position, etc.)
    for (key, value) in &data.extra {
        mapping.insert(serde_yaml::Value::String(key.clone()), value.clone());
    }

    // Insert playbook_version if present
    if let Some(ref version) = data.playbook_version {
        mapping.insert(
            serde_yaml::Value::String("playbook_version".to_string()),
            serde_yaml::Value::String(version.clone()),
        );
    }

    // Insert provenance if non-empty
    if !data.provenance.is_empty() {
        let mut prop_map = serde_yaml::Mapping::new();
        // Sort by index for deterministic output
        let mut entries: Vec<_> = data.provenance.iter().collect();
        entries.sort_by_key(|(k, _)| *k);
        for (idx, cls) in entries {
            prop_map.insert(
                serde_yaml::Value::Number((*idx).into()),
                serde_yaml::Value::String(cls.clone()),
            );
        }
        mapping.insert(
            serde_yaml::Value::String("provenance".to_string()),
            serde_yaml::Value::Mapping(prop_map),
        );
    }

    let yaml_value = serde_yaml::Value::Mapping(mapping);
    let yaml_str = serde_yaml::to_string(&yaml_value).map_err(FrontmatterError::YamlError)?;

    // Remove leading "---\n" that serde_yaml might add
    let yaml_str = yaml_str
        .trim_start_matches("---\n")
        .trim_start_matches("---\r\n");

    Ok(format!("---\n{}---\n\n{}", yaml_str, body))
}

/// Errors that can occur when working with frontmatter.
#[derive(Debug, thiserror::Error)]
pub enum FrontmatterError {
    /// The markdown content is missing `---` frontmatter delimiters.
    #[error("missing frontmatter delimiters")]
    MissingDelimiters,
    /// YAML parsing or serialization error.
    #[error("YAML parse error: {0}")]
    YamlError(#[from] serde_yaml::Error),
}
