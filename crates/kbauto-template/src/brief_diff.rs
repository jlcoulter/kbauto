//! Brief diff detection.
//!
//! Compares two brief section maps (old vs new) and identifies which
//! keys changed, were added, or were removed. Keys are compared
//! case-insensitively using the canonical UPPERCASE form.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result of comparing two brief section maps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefDiff {
    /// Keys whose values differ between old and new briefs.
    pub changed_keys: Vec<String>,
    /// Keys present in the new brief but absent from the old brief.
    pub added_keys: Vec<String>,
    /// Keys present in the old brief but absent from the new brief.
    pub removed_keys: Vec<String>,
}

/// Compare two brief section maps and identify differences.
///
/// Keys are normalised to canonical UPPERCASE form before comparison,
/// matching the placeholder key convention used throughout kbauto.
#[must_use]
pub fn brief_diff(
    old_brief: &HashMap<String, String>,
    new_brief: &HashMap<String, String>,
) -> BriefDiff {
    // Normalise both maps to uppercase keys
    let old_normalised: HashMap<String, String> = old_brief
        .iter()
        .map(|(k, v)| (kbauto_placeholder::canonical_key(k), v.clone()))
        .collect();

    let new_normalised: HashMap<String, String> = new_brief
        .iter()
        .map(|(k, v)| (kbauto_placeholder::canonical_key(k), v.clone()))
        .collect();

    let mut changed_keys = Vec::new();
    let mut added_keys = Vec::new();
    let mut removed_keys = Vec::new();

    // Collect all keys from both maps
    let all_keys: std::collections::BTreeSet<String> = old_normalised
        .keys()
        .chain(new_normalised.keys())
        .cloned()
        .collect();

    for key in all_keys {
        match (old_normalised.get(&key), new_normalised.get(&key)) {
            (Some(old_val), Some(new_val)) => {
                if old_val != new_val {
                    changed_keys.push(key);
                }
            }
            (None, Some(_)) => {
                added_keys.push(key);
            }
            (Some(_), None) => {
                removed_keys.push(key);
            }
            (None, None) => {
                // Unreachable: keys come from at least one map
            }
        }
    }

    BriefDiff {
        changed_keys,
        added_keys,
        removed_keys,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_detect_changed() {
        let old = HashMap::from([
            ("team_name".to_string(), "Example".to_string()),
            ("team_email".to_string(), "old@example.com".to_string()),
        ]);
        let new = HashMap::from([
            ("team_name".to_string(), "Example".to_string()),
            ("team_email".to_string(), "new@example.com".to_string()),
        ]);
        let diff = brief_diff(&old, &new);
        assert_eq!(diff.changed_keys, vec!["TEAM_EMAIL"]);
        assert!(diff.added_keys.is_empty());
        assert!(diff.removed_keys.is_empty());
    }

    #[test]
    fn diff_detect_added() {
        let old = HashMap::from([("team_name".to_string(), "Example".to_string())]);
        let new = HashMap::from([
            ("team_name".to_string(), "Example".to_string()),
            ("team_phone".to_string(), "555-1234".to_string()),
        ]);
        let diff = brief_diff(&old, &new);
        assert!(diff.changed_keys.is_empty());
        assert_eq!(diff.added_keys, vec!["TEAM_PHONE"]);
        assert!(diff.removed_keys.is_empty());
    }

    #[test]
    fn diff_detect_removed() {
        let old = HashMap::from([
            ("team_name".to_string(), "Example".to_string()),
            ("team_email".to_string(), "hi@example.com".to_string()),
        ]);
        let new = HashMap::from([("team_name".to_string(), "Example".to_string())]);
        let diff = brief_diff(&old, &new);
        assert!(diff.changed_keys.is_empty());
        assert!(diff.added_keys.is_empty());
        assert_eq!(diff.removed_keys, vec!["TEAM_EMAIL"]);
    }

    #[test]
    fn diff_case_insensitive() {
        let old = HashMap::from([("TEAM_NAME".to_string(), "Example".to_string())]);
        let new = HashMap::from([("team_name".to_string(), "Beta".to_string())]);
        let diff = brief_diff(&old, &new);
        assert_eq!(diff.changed_keys, vec!["TEAM_NAME"]);
    }

    #[test]
    fn diff_empty_maps() {
        let old = HashMap::new();
        let new = HashMap::new();
        let diff = brief_diff(&old, &new);
        assert!(diff.changed_keys.is_empty());
        assert!(diff.added_keys.is_empty());
        assert!(diff.removed_keys.is_empty());
    }
}
