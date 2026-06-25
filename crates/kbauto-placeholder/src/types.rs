//! Placeholder types and formats.

use serde::{Deserialize, Serialize};

/// The format a placeholder appears in within template text.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaceholderFormat {
    /// Uppercase bare: `TEAM_NAME`.
    Bare,
    /// Angle-bracket delimited: `<<team_name>>`.
    AngleBracket,
    /// Mustache-style: `{{TEAM_NAME}}`.
    Mustache,
}

/// The type of value a placeholder expects.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaceholderType {
    /// Single-line text value.
    Text,
    /// Multi-line text value.
    LongText,
    /// Numeric value.
    Number,
    /// Boolean yes/no.
    Boolean,
    /// List of values.
    List,
    /// Date value.
    Date,
}

/// A single placeholder extracted from a template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Placeholder {
    /// Canonical uppercase key: e.g. `TEAM_NAME`.
    pub key: String,
    /// The format this placeholder was found in.
    pub format: PlaceholderFormat,
    /// The type of value expected.
    pub placeholder_type: PlaceholderType,
    /// Source file where this placeholder was found.
    pub source_file: String,
    /// Line number in the source file (1-based).
    pub line_number: usize,
    /// Default value from defaults.json, if any.
    pub default_value: Option<String>,
    /// Human-readable description of the placeholder.
    pub description: Option<String>,
}

/// Schema of all placeholders across template files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceholderSchema {
    /// Version of the playbook these placeholders belong to.
    pub version: String,
    /// All unique placeholders, keyed by canonical key.
    pub placeholders: std::collections::HashMap<String, Placeholder>,
}
