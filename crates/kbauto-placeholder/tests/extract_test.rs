//! T018: Placeholder extraction contract tests
//!
//! Tests for: extract_placeholders, build_schema, canonical_key, detect_format
//! Covers all three placeholder formats, canonical key normalization,
//! code block skipping, and multi-file schema building.

use kbauto_placeholder::{
    PlaceholderFormat, build_schema, canonical_key, detect_format, extract_placeholders,
};

// --- Bare format extraction ---

#[test]
fn extract_bare_placeholder() {
    let content = "Welcome to TEAM_NAME, your trusted advisor.";
    let found = extract_placeholders(content, "page.md");
    assert_eq!(found.len(), 1, "should find one bare placeholder");
    assert_eq!(found[0].key, "TEAM_NAME");
    assert_eq!(found[0].format, PlaceholderFormat::Bare);
    assert_eq!(found[0].source_file, "page.md");
}

#[test]
fn extract_multiple_bare_placeholders() {
    let content = "TEAM_NAME can be reached at TEAM_EMAIL or TEAM_PHONE.";
    let found = extract_placeholders(content, "page.md");
    assert_eq!(found.len(), 3, "should find three bare placeholders");
    let keys: Vec<&str> = found.iter().map(|p| p.key.as_str()).collect();
    assert!(keys.contains(&"TEAM_NAME"), "should find TEAM_NAME");
    assert!(keys.contains(&"TEAM_EMAIL"), "should find TEAM_EMAIL");
    assert!(keys.contains(&"TEAM_PHONE"), "should find TEAM_PHONE");
}

#[test]
fn bare_placeholder_minimum_length() {
    // Bare placeholders must be at least 3 chars: [A-Z][A-Z0-9_]{2,}
    let content = "AB is too short but ABC is valid.";
    let found = extract_placeholders(content, "page.md");
    assert!(
        found.iter().all(|p| p.key.len() >= 3),
        "all bare placeholders should be >= 3 chars"
    );
}

// --- AngleBracket format extraction ---

#[test]
fn extract_angle_bracket_placeholder() {
    let content = "Welcome to <<team_name>>, your trusted advisor.";
    let found = extract_placeholders(content, "page.md");
    assert_eq!(found.len(), 1, "should find one angle-bracket placeholder");
    assert_eq!(
        found[0].key, "TEAM_NAME",
        "angle-bracket key should be normalized to UPPERCASE"
    );
    assert_eq!(found[0].format, PlaceholderFormat::AngleBracket);
}

#[test]
fn extract_multiple_angle_bracket_placeholders() {
    let content = "<<team_name>> can be reached at <<team_email>>.";
    let found = extract_placeholders(content, "page.md");
    assert_eq!(found.len(), 2, "should find two angle-bracket placeholders");
}

// --- Mustache format extraction ---

#[test]
fn extract_mustache_placeholder() {
    let content = "Welcome to {{TEAM_NAME}}, your trusted advisor.";
    let found = extract_placeholders(content, "page.md");
    assert_eq!(found.len(), 1, "should find one mustache placeholder");
    assert_eq!(found[0].key, "TEAM_NAME");
    assert_eq!(found[0].format, PlaceholderFormat::Mustache);
}

#[test]
fn extract_multiple_mustache_placeholders() {
    let content = "{{TEAM_NAME}} can be reached at {{TEAM_EMAIL}} or {{TEAM_PHONE}}.";
    let found = extract_placeholders(content, "page.md");
    assert_eq!(found.len(), 3, "should find three mustache placeholders");
}

// --- Canonical key normalization ---

#[test]
fn canonical_key_normalizes_to_uppercase() {
    assert_eq!(canonical_key("team_name"), "TEAM_NAME");
    assert_eq!(canonical_key("TEAM_NAME"), "TEAM_NAME");
    assert_eq!(canonical_key("Team_Name"), "TEAM_NAME");
}

#[test]
fn canonical_key_from_angle_bracket() {
    // <<team_name>> → TEAM_NAME
    assert_eq!(canonical_key("team_name"), "TEAM_NAME");
}

#[test]
fn canonical_key_from_mustache() {
    // {{TEAM_NAME}} → TEAM_NAME (already uppercase inside delimiters)
    assert_eq!(canonical_key("TEAM_NAME"), "TEAM_NAME");
}

// --- Code block skipping ---

#[test]
fn no_extraction_inside_code_blocks() {
    let content = "Some text\n```\nTEAM_NAME should not be extracted\n```\nAfter code";
    let found = extract_placeholders(content, "page.md");
    assert!(
        found
            .iter()
            .all(|p| p.key != "TEAM_NAME" || p.line_number < 2 || p.line_number > 4),
        "should not extract placeholders inside fenced code blocks"
    );
}

#[test]
fn no_extraction_inside_inline_code() {
    let content = "Use `TEAM_NAME` as a variable name.";
    let found = extract_placeholders(content, "page.md");
    // Inline code should also be skipped
    assert!(
        !found.iter().any(|p| p.key == "TEAM_NAME"),
        "should not extract placeholders inside inline code"
    );
}

#[test]
fn extraction_after_code_block() {
    let content = "```\nTEAM_NAME\n```\nTEAM_NAME is extracted here";
    let found = extract_placeholders(content, "page.md");
    assert!(
        found.iter().any(|p| p.key == "TEAM_NAME"),
        "should extract placeholders after code block"
    );
}

// --- Multi-file schema building ---

#[test]
fn build_schema_deduplicates_by_canonical_key() {
    // Same key from two different files should be deduplicated
    let p1 = kbauto_placeholder::Placeholder {
        key: "TEAM_NAME".to_string(),
        format: PlaceholderFormat::Bare,
        placeholder_type: kbauto_placeholder::PlaceholderType::Text,
        source_file: "page1.md".to_string(),
        line_number: 1,
        default_value: None,
        description: None,
    };
    let p2 = kbauto_placeholder::Placeholder {
        key: "TEAM_NAME".to_string(),
        format: PlaceholderFormat::AngleBracket,
        placeholder_type: kbauto_placeholder::PlaceholderType::Text,
        source_file: "page2.md".to_string(),
        line_number: 5,
        default_value: None,
        description: None,
    };
    let schema = build_schema(vec![p1, p2], "1.0.0");
    assert_eq!(schema.version, "1.0.0");
    assert_eq!(schema.placeholders.len(), 1, "should deduplicate same key");
    assert!(schema.placeholders.contains_key("TEAM_NAME"));
}

#[test]
fn build_schema_preserves_different_keys() {
    let p1 = kbauto_placeholder::Placeholder {
        key: "TEAM_NAME".to_string(),
        format: PlaceholderFormat::Bare,
        placeholder_type: kbauto_placeholder::PlaceholderType::Text,
        source_file: "page1.md".to_string(),
        line_number: 1,
        default_value: None,
        description: None,
    };
    let p2 = kbauto_placeholder::Placeholder {
        key: "TEAM_EMAIL".to_string(),
        format: PlaceholderFormat::Mustache,
        placeholder_type: kbauto_placeholder::PlaceholderType::Text,
        source_file: "page1.md".to_string(),
        line_number: 3,
        default_value: None,
        description: None,
    };
    let schema = build_schema(vec![p1, p2], "2.0.0");
    assert_eq!(schema.placeholders.len(), 2, "should keep different keys");
    assert!(schema.placeholders.contains_key("TEAM_NAME"));
    assert!(schema.placeholders.contains_key("TEAM_EMAIL"));
}

// --- detect_format ---

#[test]
fn detect_bare_format() {
    assert_eq!(detect_format("TEAM_NAME"), Some(PlaceholderFormat::Bare));
}

#[test]
fn detect_angle_bracket_format() {
    assert_eq!(
        detect_format("<<team_name>>"),
        Some(PlaceholderFormat::AngleBracket)
    );
}

#[test]
fn detect_mustache_format() {
    assert_eq!(
        detect_format("{{TEAM_NAME}}"),
        Some(PlaceholderFormat::Mustache)
    );
}

#[test]
fn detect_format_returns_none_for_plain_text() {
    assert_eq!(detect_format("hello world"), None);
}
