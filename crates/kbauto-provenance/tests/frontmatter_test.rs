//! T020: Frontmatter contract tests
//!
//! Tests for: parse_frontmatter, write_frontmatter, FrontmatterData
//! Covers parsing Docusaurus frontmatter (--- delimiters), creating new frontmatter,
//! preserving Docusaurus fields (title, sidebar_position, etc.), and round-trip.

use kbauto_provenance::{FrontmatterData, parse_frontmatter, write_frontmatter};
use std::collections::HashMap;

// --- Parse existing Docusaurus frontmatter ---

#[test]
fn parse_frontmatter_with_delimiters() {
    let content =
        "---\ntitle: Getting Started\nsidebar_position: 1\n---\n\nThis is the body content.";
    let (data, body) = parse_frontmatter(content).expect("should parse valid frontmatter");
    assert_eq!(
        data.extra.get("title").and_then(|v| v.as_str()),
        Some("Getting Started"),
        "should parse title from frontmatter"
    );
    assert_eq!(
        data.extra.get("sidebar_position").and_then(|v| v.as_i64()),
        Some(1),
        "should parse sidebar_position from frontmatter"
    );
    assert!(
        body.trim().starts_with("This is the body content"),
        "body should start with content after frontmatter"
    );
}

#[test]
fn parse_frontmatter_without_delimiters() {
    let content = "No frontmatter here, just plain text.";
    let result = parse_frontmatter(content);
    assert!(result.is_err(), "should fail without --- delimiters");
}

#[test]
fn parse_frontmatter_empty_body() {
    let content = "---\ntitle: Test\n---\n";
    let (data, body) =
        parse_frontmatter(content).expect("should parse frontmatter with empty body");
    assert_eq!(
        data.extra.get("title").and_then(|v| v.as_str()),
        Some("Test"),
        "should parse title"
    );
    assert!(body.trim().is_empty(), "body should be empty");
}

#[test]
fn parse_frontmatter_with_provenance() {
    let content = "---\ntitle: Getting Started\nprovenance:\n  0: template\n  1: substituted\n---\n\nBody text.";
    let (data, _body) =
        parse_frontmatter(content).expect("should parse frontmatter with provenance");
    assert_eq!(
        data.provenance.get(&0),
        Some(&"template".to_string()),
        "paragraph 0 should be template"
    );
    assert_eq!(
        data.provenance.get(&1),
        Some(&"substituted".to_string()),
        "paragraph 1 should be substituted"
    );
}

// --- Create new frontmatter ---

#[test]
fn create_frontmatter_from_scratch() {
    let mut data = FrontmatterData {
        provenance: HashMap::new(),
        playbook_version: Some("1.0.0".to_string()),
        extra: HashMap::new(),
    };
    data.extra.insert(
        "title".to_string(),
        serde_yaml::Value::String("My Page".to_string()),
    );
    data.extra.insert(
        "sidebar_position".to_string(),
        serde_yaml::Value::Number(2.into()),
    );
    data.provenance.insert(0, "template".to_string());

    let result = write_frontmatter(&data, "Body content.").expect("should write frontmatter");
    assert!(
        result.starts_with("---\n"),
        "should start with --- delimiter"
    );
    assert!(
        result.contains("---\n\nBody content."),
        "should contain body after closing ---"
    );
    assert!(result.contains("title: My Page"), "should contain title");
}

#[test]
fn write_frontmatter_preserves_provenance() {
    let mut data = FrontmatterData {
        provenance: HashMap::new(),
        playbook_version: Some("2.1.0".to_string()),
        extra: HashMap::new(),
    };
    data.provenance.insert(0, "template".to_string());
    data.provenance.insert(1, "substituted".to_string());
    data.provenance.insert(2, "rewritten".to_string());

    let result = write_frontmatter(&data, "Body").expect("should write frontmatter");
    assert!(
        result.contains("provenance"),
        "should include provenance in output"
    );
    assert!(
        result.contains("template"),
        "should include template provenance"
    );
    assert!(
        result.contains("substituted"),
        "should include substituted provenance"
    );
    assert!(
        result.contains("rewritten"),
        "should include rewritten provenance"
    );
}

// --- Preserve Docusaurus fields ---

#[test]
fn preserve_docusaurus_title_field() {
    let content = "---\ntitle: About Our Firm\n---\n\nSome body.";
    let (data, _) = parse_frontmatter(content).expect("should parse");
    assert_eq!(
        data.extra.get("title").and_then(|v| v.as_str()),
        Some("About Our Firm"),
        "should preserve title field"
    );
}

#[test]
fn preserve_docusaurus_sidebar_position() {
    let content = "---\ntitle: Test\nsidebar_position: 5\n---\n\nBody.";
    let (data, _) = parse_frontmatter(content).expect("should parse");
    assert_eq!(
        data.extra.get("sidebar_position").and_then(|v| v.as_i64()),
        Some(5),
        "should preserve sidebar_position field"
    );
}

#[test]
fn preserve_docusaurus_multiple_fields() {
    let content = "---\ntitle: Services\nsidebar_position: 3\ndescription: Our services\nhide_title: false\n---\n\nBody.";
    let (data, _) = parse_frontmatter(content).expect("should parse");
    assert_eq!(
        data.extra.get("title").and_then(|v| v.as_str()),
        Some("Services")
    );
    assert_eq!(
        data.extra.get("sidebar_position").and_then(|v| v.as_i64()),
        Some(3)
    );
    assert_eq!(
        data.extra.get("description").and_then(|v| v.as_str()),
        Some("Our services")
    );
    assert_eq!(
        data.extra.get("hide_title").and_then(|v| v.as_bool()),
        Some(false)
    );
}

#[test]
fn preserve_playbook_version() {
    let content = "---\ntitle: Test\nplaybook_version: \"1.2.3\"\n---\n\nBody.";
    let (data, _) = parse_frontmatter(content).expect("should parse");
    assert_eq!(
        data.playbook_version,
        Some("1.2.3".to_string()),
        "should preserve playbook_version"
    );
}

// --- Serialize/deserialize round-trip ---

#[test]
fn round_trip_frontmatter() {
    let mut data = FrontmatterData {
        provenance: HashMap::new(),
        playbook_version: Some("1.0.0".to_string()),
        extra: HashMap::new(),
    };
    data.extra.insert(
        "title".to_string(),
        serde_yaml::Value::String("Round Trip".to_string()),
    );
    data.extra.insert(
        "sidebar_position".to_string(),
        serde_yaml::Value::Number(1.into()),
    );
    data.provenance.insert(0, "template".to_string());
    data.provenance.insert(1, "substituted".to_string());

    let written = write_frontmatter(&data, "Body text.").expect("should write");
    let (parsed_data, parsed_body) = parse_frontmatter(&written).expect("should re-parse");

    assert_eq!(
        parsed_data.playbook_version, data.playbook_version,
        "playbook_version should round-trip"
    );
    assert_eq!(
        parsed_data.provenance, data.provenance,
        "provenance should round-trip"
    );
    assert_eq!(parsed_body.trim(), "Body text.", "body should round-trip");
    assert_eq!(
        parsed_data.extra.get("title").and_then(|v| v.as_str()),
        Some("Round Trip"),
        "title should round-trip"
    );
}

#[test]
fn round_trip_preserves_all_docusaurus_fields() {
    let content = "---\ntitle: Services\nsidebar_position: 3\ndescription: Our services\nhide_title: false\n---\n\nService descriptions here.";
    let (data, body) = parse_frontmatter(content).expect("should parse");
    let rewritten = write_frontmatter(&data, body).expect("should write");
    let (data2, body2) = parse_frontmatter(&rewritten).expect("should re-parse");

    assert_eq!(
        data2.extra.get("title").and_then(|v| v.as_str()),
        data.extra.get("title").and_then(|v| v.as_str()),
        "title should survive round-trip"
    );
    assert_eq!(
        data2.extra.get("sidebar_position"),
        data.extra.get("sidebar_position"),
        "sidebar_position should survive round-trip"
    );
    assert_eq!(body2.trim(), body.trim(), "body should survive round-trip");
}
