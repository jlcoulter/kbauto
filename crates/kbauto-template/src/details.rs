//! Static details parsing.
//!
//! Parses a static details markdown file into a structured key-value mapping.
//! The details file uses markdown headings (## or ###) as keys and the body
//! text between headings as values. Headings are normalised to canonical
//! UPPERCASE keys (spaces and hyphens become underscores).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// A parsed static details document with canonical key-to-value mapping.
///
/// Keys are normalised: trimmed, spaces and hyphens replaced with underscores,
/// then uppercased. This matches the placeholder key convention used
/// throughout kbauto.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StaticDetails {
    /// Details entries keyed by canonical UPPERCASE name.
    pub entries: HashMap<String, String>,
}

/// Errors that can occur when parsing a static details document.
#[derive(Debug, thiserror::Error)]
pub enum DetailsError {
    /// IO error reading the details file.
    #[error("IO error reading details: {0}")]
    Io(#[from] std::io::Error),
    /// The details file contains no parseable sections.
    #[error("details file is empty: {0}")]
    EmptyFile(String),
}

impl StaticDetails {
    /// Parse a static details document from a markdown file.
    ///
    /// Sections are delimited by `## ` or `### ` headings.
    /// Section names are normalised to canonical UPPERCASE keys.
    /// Content between headings is trimmed and stored as values.
    ///
    /// # Errors
    ///
    /// Returns `DetailsError` if the file cannot be read or has no sections.
    pub fn from_markdown_file(path: &Path) -> Result<Self, DetailsError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_markdown(&content, &path.display().to_string())
    }

    /// Parse a static details document from a markdown string.
    ///
    /// Sections are delimited by `## ` or `### ` headings.
    /// Section names are normalised to canonical keys: spaces and hyphens
    /// become underscores, then uppercased — matching the placeholder
    /// convention used throughout kbauto.
    ///
    /// # Errors
    ///
    /// Returns `DetailsError::EmptyFile` if no sections are found.
    #[must_use = "parsing details must be checked for errors"]
    pub fn from_markdown(content: &str, source: &str) -> Result<Self, DetailsError> {
        let mut entries = HashMap::new();
        let mut current_key = String::new();
        let mut current_value = String::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("### ") || trimmed.starts_with("## ") {
                // Save the previous section
                if !current_key.is_empty() {
                    let value_trimmed = current_value.trim().to_string();
                    entries.insert(current_key.clone(), value_trimmed);
                }
                // Start new section — normalise heading to canonical key
                let heading = if trimmed.starts_with("### ") {
                    &trimmed[4..]
                } else {
                    &trimmed[3..]
                };
                current_key = canonicalise_key(heading);
                current_value.clear();
            } else {
                // Accumulate content for the current section
                if !current_key.is_empty() {
                    current_value.push_str(line);
                    current_value.push('\n');
                }
            }
        }

        // Save the last section
        if !current_key.is_empty() {
            let value_trimmed = current_value.trim().to_string();
            entries.insert(current_key, value_trimmed);
        }

        if entries.is_empty() {
            return Err(DetailsError::EmptyFile(source.to_string()));
        }

        Ok(Self { entries })
    }

    /// Look up a value by key (case-insensitive via canonical normalisation).
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&str> {
        let canonical = canonicalise_key(key);
        self.entries.get(&canonical).map(|s| s.as_str())
    }

    /// Check if the details contain a section (case-insensitive).
    #[must_use]
    pub fn contains_key(&self, key: &str) -> bool {
        let canonical = canonicalise_key(key);
        self.entries.contains_key(&canonical)
    }

    /// Return the number of entries in the details.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Return true if the details have no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Normalise a key to canonical form: trim, replace spaces/hyphens with
/// underscores, uppercase.
fn canonicalise_key(key: &str) -> String {
    key.trim()
        .replace(' ', "_")
        .replace('-', "_")
        .to_uppercase()
}