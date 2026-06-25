//! Template page loading and representation.

use serde::{Deserialize, Serialize};

/// A single page from a playbook template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplatePage {
    /// Filename relative to the docs/ directory.
    pub filename: String,
    /// Raw markdown content with frontmatter.
    pub content: String,
    /// Placeholder keys found in this page.
    pub placeholders: Vec<String>,
}
