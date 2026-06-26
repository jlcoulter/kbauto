//! TUI missing-value data model.
//!
//! This module provides the data structures for collecting missing placeholder
//! values from the user via a TUI form. The actual ratatui rendering is in
//! the US1 (generate) implementation; this is the data model only.
//!
//! When `generate_playbook()` encounters placeholders that have no value in the
//! static details and no default, the CLI collects them into a `MissingValueForm`,
//! presents a TUI for the user to fill them in, and then merges the results
//! back into the placeholder resolution map.

use std::collections::HashMap;

/// A single missing placeholder value that needs user input.
#[derive(Debug, Clone)]
pub struct MissingValue {
    /// The placeholder key (e.g. "TEAM_NAME").
    pub key: String,
    /// A human-readable description of what this value represents.
    pub description: String,
    /// The default value from `defaults.json`, if any.
    pub default: Option<String>,
}

/// A form for collecting missing placeholder values from the user.
///
/// Contains the list of missing values and a map of user-entered values.
/// Call `resolve()` to produce a merged `HashMap<String, String>` that
/// includes user values, falling back to defaults where no user value was
/// provided.
#[derive(Debug, Clone)]
pub struct MissingValueForm {
    /// The list of missing values that need to be filled in.
    pub missing: Vec<MissingValue>,
    /// User-entered values, keyed by placeholder key.
    pub values: HashMap<String, String>,
}

impl MissingValueForm {
    /// Create a new form with the given missing values and an empty values map.
    pub fn new(missing: Vec<MissingValue>) -> Self {
        Self {
            missing,
            values: HashMap::new(),
        }
    }

    /// Set a user-entered value for a placeholder key.
    pub fn set_value(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    /// Resolve all missing values into a single map.
    ///
    /// For each missing value:
    /// - If the user provided a value, use it.
    /// - Otherwise, if a default exists, use the default.
    /// - Otherwise, the key is omitted (still missing).
    pub fn resolve(&self) -> HashMap<String, String> {
        let mut result = HashMap::new();
        for mv in &self.missing {
            if let Some(user_value) = self.values.get(&mv.key) {
                result.insert(mv.key.clone(), user_value.clone());
            } else if let Some(ref default) = mv.default {
                result.insert(mv.key.clone(), default.clone());
            }
            // If neither user value nor default, the key remains missing
        }
        result
    }
}
