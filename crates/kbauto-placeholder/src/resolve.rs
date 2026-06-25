//! Placeholder resolution from briefs and defaults.

use crate::defaults::DefaultsFile;
use crate::extract::{canonical_key, extract_placeholders};
use crate::types::PlaceholderFormat;

/// A resolved placeholder with its value and provenance.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedPlaceholder {
    /// Canonical UPPERCASE key.
    pub key: String,
    /// The resolved value.
    pub value: String,
    /// Format to use when writing back.
    pub format: PlaceholderFormat,
    /// Where the value came from: `"brief"` or `"default"`.
    pub source: String,
}

/// Result of resolving all placeholders in content.
#[derive(Debug)]
pub struct ResolutionResult {
    /// The content with placeholders resolved.
    pub content: String,
    /// Placeholders that were successfully resolved.
    pub resolved: Vec<ResolvedPlaceholder>,
    /// Keys that could not be resolved.
    pub unresolved_keys: Vec<String>,
}

/// Resolve placeholders in content using values from a brief, with defaults as fallback.
///
/// - Keys from the brief are matched case-insensitively.
/// - Missing brief values fall back to defaults.
/// - Format is preserved (Bare stays Bare, AngleBracket stays AngleBracket).
///
/// # Arguments
///
/// * `content` - Template content with placeholders
/// * `brief` - Key-value pairs from the client brief (keys are lowercased)
/// * `defaults` - Default values from defaults.json
#[must_use = "the resolution result should be used"]
pub fn resolve_placeholders(
    content: &str,
    brief: &std::collections::HashMap<String, String>,
    defaults: &DefaultsFile,
) -> ResolutionResult {
    // Build a lowercase-keyed map from the brief for case-insensitive matching
    let brief_lower: std::collections::HashMap<String, String> = brief
        .iter()
        .map(|(k, v)| (k.to_uppercase(), v.clone()))
        .collect();

    let placeholders = extract_placeholders(content, "");

    let mut result_content = content.to_string();
    let mut resolved = Vec::new();
    let mut unresolved_keys = Vec::new();

    // Process each placeholder, replacing longest matches first to avoid partial replacements
    // We need to be careful about order - process mustache first (longest delimiters),
    // then angle bracket, then bare
    let mut replacements: Vec<(String, String, PlaceholderFormat, String)> = Vec::new();

    for p in &placeholders {
        let key_upper = canonical_key(&p.key);

        // Try brief first (case-insensitive), then defaults
        let (value, source) = if let Some(val) = brief_lower.get(&key_upper) {
            (val.clone(), "brief".to_string())
        } else if let Some(val) = defaults.get_default(&key_upper) {
            (val.to_string(), "default".to_string())
        } else {
            // Unresolved
            if !unresolved_keys.contains(&key_upper) {
                unresolved_keys.push(key_upper.clone());
            }
            continue;
        };

        // Format the placeholder in its original format
        let original_placeholder = format_original_placeholder(&p.key, &p.format);
        let replacement = format_value(&value, &p.format);

        replacements.push((
            original_placeholder,
            replacement,
            p.format.clone(),
            source.clone(),
        ));
        resolved.push(ResolvedPlaceholder {
            key: key_upper,
            value,
            format: p.format.clone(),
            source,
        });
    }

    // Apply replacements - do them in a specific order to avoid conflicts
    // Replace mustache first, then angle bracket, then bare
    replacements.sort_by(|a, b| {
        let ord = |f: &PlaceholderFormat| match f {
            PlaceholderFormat::Mustache => 0,
            PlaceholderFormat::AngleBracket => 1,
            PlaceholderFormat::Bare => 2,
        };
        ord(&a.2).cmp(&ord(&b.2))
    });

    for (original, replacement, _, _) in replacements {
        result_content = result_content.replace(&original, &replacement);
    }

    ResolutionResult {
        content: result_content,
        resolved,
        unresolved_keys,
    }
}

/// Format a placeholder key back into its original form for matching/replacement.
fn format_original_placeholder(key: &str, format: &PlaceholderFormat) -> String {
    match format {
        PlaceholderFormat::Bare => key.to_string(),
        PlaceholderFormat::AngleBracket => format!("<<{}>>", key.to_lowercase()),
        PlaceholderFormat::Mustache => format!("{{{{{}}}}}", key),
    }
}

/// Format a resolved value for insertion into template text.
///
/// The value is returned as-is — placeholder delimiters are only used
/// for finding and matching the original placeholder, not for wrapping
/// the resolved value.
#[must_use]
pub fn format_value(value: &str, format: &PlaceholderFormat) -> String {
    let _ = format; // format is unused; value is always plain text
    value.to_string()
}
