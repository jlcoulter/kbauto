//! Defaults file parsing and validation.

use serde::{Deserialize, Serialize};

/// A default value entry from defaults.json.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DefaultValue {
    /// The placeholder key this default applies to.
    pub key: String,
    /// The default value.
    pub value: String,
    /// Type of the value (e.g. `"text"`, `"number"`).
    #[serde(rename = "type", default = "default_type")]
    pub value_type: String,
    /// Optional human-readable description of this default.
    #[serde(default)]
    pub description: Option<String>,
}

fn default_type() -> String {
    "text".to_string()
}

/// The full defaults.json file structure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DefaultsFile {
    /// Playbook version this defaults file corresponds to.
    pub version: String,
    /// List of default values for placeholders.
    pub defaults: Vec<DefaultValue>,
}

impl DefaultsFile {
    /// Parse a defaults.json string into a `DefaultsFile`.
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON is invalid or missing required fields.
    #[must_use = "parsing defaults must be checked for errors"]
    pub fn from_json(json: &str) -> Result<Self, DefaultsError> {
        serde_json::from_str(json).map_err(DefaultsError::JsonError)
    }

    /// Serialize this `DefaultsFile` to a pretty-printed JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails.
    #[must_use = "serialization result should be used"]
    pub fn to_json(&self) -> Result<String, DefaultsError> {
        serde_json::to_string_pretty(self).map_err(DefaultsError::JsonError)
    }

    /// Look up a default value by key (case-sensitive).
    ///
    /// Returns `None` if no default exists for the given key.
    #[must_use]
    pub fn get_default(&self, key: &str) -> Option<&str> {
        self.defaults
            .iter()
            .find(|d| d.key == key)
            .map(|d| d.value.as_str())
    }
}

/// Errors that can occur when parsing defaults.
#[derive(Debug, thiserror::Error)]
pub enum DefaultsError {
    /// JSON parsing or serialization error.
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    /// The version in the defaults file does not match the expected version.
    #[error("version mismatch: expected {expected}, found {found}")]
    VersionMismatch {
        /// The expected playbook version.
        expected: String,
        /// The version found in the defaults file.
        found: String,
    },
}
