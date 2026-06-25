//! T019: Placeholder resolution contract tests
//!
//! Tests for: resolve_placeholders, format_value, ResolutionResult
//! Covers round-trip format preservation, partial resolution with fallback to defaults,
//! unresolved key reporting, and case-insensitive key matching from briefs.

use kbauto_placeholder::{DefaultsFile, PlaceholderFormat, format_value, resolve_placeholders};
use std::collections::HashMap;

// --- Round-trip format preservation ---

#[test]
fn resolve_bare_placeholder_preserves_format() {
    let content = "Welcome to TEAM_NAME.";
    let mut brief = HashMap::new();
    brief.insert("team_name".to_string(), "Example Corp".to_string());
    let defaults_json = r#"{"version":"1.0.0","defaults":[{"key":"TEAM_NAME","value":"Default Co","type":"text"}]}"#;
    let defaults = DefaultsFile::from_json(defaults_json).expect("should parse defaults");

    let result = resolve_placeholders(content, &brief, &defaults);
    // The resolved content should replace TEAM_NAME with the brief value
    assert!(
        result.content.contains("Example Corp"),
        "should resolve bare placeholder with brief value"
    );
    // Check that the resolution records the Bare format
    let bare_resolved: Vec<_> = result
        .resolved
        .iter()
        .filter(|r| r.format == PlaceholderFormat::Bare)
        .collect();
    assert!(
        !bare_resolved.is_empty(),
        "should preserve Bare format in resolution"
    );
}

#[test]
fn resolve_angle_bracket_placeholder_preserves_format() {
    let content = "Welcome to <<team_name>>.";
    let mut brief = HashMap::new();
    brief.insert("team_name".to_string(), "Example Corp".to_string());
    let defaults_json = r#"{"version":"1.0.0","defaults":[{"key":"TEAM_NAME","value":"Default Co","type":"text"}]}"#;
    let defaults = DefaultsFile::from_json(defaults_json).expect("should parse defaults");

    let result = resolve_placeholders(content, &brief, &defaults);
    assert!(
        result.content.contains("Example Corp"),
        "should resolve angle-bracket placeholder with brief value"
    );
    let angle_resolved: Vec<_> = result
        .resolved
        .iter()
        .filter(|r| r.format == PlaceholderFormat::AngleBracket)
        .collect();
    assert!(
        !angle_resolved.is_empty(),
        "should preserve AngleBracket format in resolution"
    );
}

#[test]
fn resolve_mustache_placeholder_preserves_format() {
    let content = "Welcome to {{TEAM_NAME}}.";
    let mut brief = HashMap::new();
    brief.insert("team_name".to_string(), "Example Corp".to_string());
    let defaults_json = r#"{"version":"1.0.0","defaults":[{"key":"TEAM_NAME","value":"Default Co","type":"text"}]}"#;
    let defaults = DefaultsFile::from_json(defaults_json).expect("should parse defaults");

    let result = resolve_placeholders(content, &brief, &defaults);
    assert!(
        result.content.contains("Example Corp"),
        "should resolve mustache placeholder with brief value"
    );
    let mustache_resolved: Vec<_> = result
        .resolved
        .iter()
        .filter(|r| r.format == PlaceholderFormat::Mustache)
        .collect();
    assert!(
        !mustache_resolved.is_empty(),
        "should preserve Mustache format in resolution"
    );
}

// --- Partial resolution with fallback to defaults ---

#[test]
fn fallback_to_defaults_when_brief_missing() {
    let content = "Welcome to TEAM_NAME at TEAM_EMAIL.";
    let brief = HashMap::new(); // no brief entries
    let defaults_json = r#"{"version":"1.0.0","defaults":[
        {"key":"TEAM_NAME","value":"Default Co","type":"text"},
        {"key":"TEAM_EMAIL","value":"info@default.com","type":"text"}
    ]}"#;
    let defaults = DefaultsFile::from_json(defaults_json).expect("should parse defaults");

    let result = resolve_placeholders(content, &brief, &defaults);
    assert!(
        result.content.contains("Default Co"),
        "should fall back to default for TEAM_NAME"
    );
    assert!(
        result.content.contains("info@default.com"),
        "should fall back to default for TEAM_EMAIL"
    );
}

#[test]
fn brief_overrides_defaults() {
    let content = "Welcome to TEAM_NAME.";
    let mut brief = HashMap::new();
    brief.insert("team_name".to_string(), "Example Corp".to_string());
    let defaults_json = r#"{"version":"1.0.0","defaults":[{"key":"TEAM_NAME","value":"Default Co","type":"text"}]}"#;
    let defaults = DefaultsFile::from_json(defaults_json).expect("should parse defaults");

    let result = resolve_placeholders(content, &brief, &defaults);
    assert!(
        result.content.contains("Example Corp"),
        "brief value should override default"
    );
    assert!(
        !result.content.contains("Default Co"),
        "default should not appear when brief provides value"
    );
}

#[test]
fn partial_brief_with_default_fallback() {
    // Brief provides TEAM_NAME but not TEAM_EMAIL → fallback for EMAIL
    let content = "Welcome to TEAM_NAME. Contact TEAM_EMAIL.";
    let mut brief = HashMap::new();
    brief.insert("team_name".to_string(), "Example Corp".to_string());
    let defaults_json = r#"{"version":"1.0.0","defaults":[
        {"key":"TEAM_NAME","value":"Default Co","type":"text"},
        {"key":"TEAM_EMAIL","value":"info@default.com","type":"text"}
    ]}"#;
    let defaults = DefaultsFile::from_json(defaults_json).expect("should parse defaults");

    let result = resolve_placeholders(content, &brief, &defaults);
    assert!(
        result.content.contains("Example Corp"),
        "brief should provide TEAM_NAME"
    );
    assert!(
        result.content.contains("info@default.com"),
        "default should provide TEAM_EMAIL"
    );
}

// --- Unresolved key reporting ---

#[test]
fn reports_unresolved_keys() {
    let content = "Welcome to TEAM_NAME. Contact TEAM_UNKNOWN.";
    let brief = HashMap::new();
    let defaults_json = r#"{"version":"1.0.0","defaults":[
        {"key":"TEAM_NAME","value":"Default Co","type":"text"}
    ]}"#;
    let defaults = DefaultsFile::from_json(defaults_json).expect("should parse defaults");

    let result = resolve_placeholders(content, &brief, &defaults);
    assert!(
        result.unresolved_keys.contains(&"TEAM_UNKNOWN".to_string()),
        "should report TEAM_UNKNOWN as unresolved"
    );
    assert!(
        !result.unresolved_keys.contains(&"TEAM_NAME".to_string()),
        "should not report TEAM_NAME as unresolved since it has a default"
    );
}

#[test]
fn no_unresolved_when_all_resolved() {
    let content = "Welcome to TEAM_NAME.";
    let mut brief = HashMap::new();
    brief.insert("team_name".to_string(), "Example Corp".to_string());
    let defaults_json = r#"{"version":"1.0.0","defaults":[]}"#;
    let defaults = DefaultsFile::from_json(defaults_json).expect("should parse defaults");

    let result = resolve_placeholders(content, &brief, &defaults);
    assert!(
        result.unresolved_keys.is_empty(),
        "should have no unresolved keys when all are resolved"
    );
}

// --- Case-insensitive key matching from briefs ---

#[test]
fn brief_key_matching_is_case_insensitive() {
    let content = "Welcome to TEAM_NAME.";
    // Brief keys are lowercased, but should match UPPERCASE placeholders
    let mut brief = HashMap::new();
    brief.insert("team_name".to_string(), "Example Corp".to_string());
    let defaults_json = r#"{"version":"1.0.0","defaults":[]}"#;
    let defaults = DefaultsFile::from_json(defaults_json).expect("should parse defaults");

    let result = resolve_placeholders(content, &brief, &defaults);
    assert!(
        result.content.contains("Example Corp"),
        "lowercase brief key should match UPPERCASE placeholder"
    );
}

#[test]
fn brief_key_mixed_case_matches() {
    let content = "Welcome to TEAM_NAME.";
    let mut brief = HashMap::new();
    brief.insert("Team_Name".to_string(), "Example Corp".to_string());
    let defaults_json = r#"{"version":"1.0.0","defaults":[]}"#;
    let defaults = DefaultsFile::from_json(defaults_json).expect("should parse defaults");

    let result = resolve_placeholders(content, &brief, &defaults);
    assert!(
        result.content.contains("Example Corp"),
        "mixed-case brief key should match UPPERCASE placeholder"
    );
}

// --- format_value ---

#[test]
fn format_value_bare_returns_plain() {
    let result = format_value("Example Corp", &PlaceholderFormat::Bare);
    assert_eq!(
        result, "Example Corp",
        "bare format should return plain value"
    );
}

#[test]
fn format_value_angle_bracket_returns_plain() {
    let result = format_value("team_name", &PlaceholderFormat::AngleBracket);
    assert_eq!(
        result, "team_name",
        "resolved values should be plain text regardless of format"
    );
}

#[test]
fn format_value_mustache_returns_plain() {
    let result = format_value("TEAM_NAME", &PlaceholderFormat::Mustache);
    assert_eq!(
        result, "TEAM_NAME",
        "resolved values should be plain text regardless of format"
    );
}
