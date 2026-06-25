//! Tests for DefaultsFile parsing and validation.

use kbauto_placeholder::DefaultsFile;

#[test]
fn parse_valid_defaults_json() {
    let json = r#"{
        "version": "1.0.0",
        "defaults": [
            {"key": "TEAM_NAME", "value": "Structure Your Story", "type": "text"},
            {"key": "TEAM_EMAIL", "value": "hello@structureyourstory.com", "type": "text"}
        ]
    }"#;
    let df = DefaultsFile::from_json(json).expect("should parse valid defaults");
    assert_eq!(df.version, "1.0.0");
    assert_eq!(df.defaults.len(), 2);
}

#[test]
fn get_default_by_key() {
    let json = r#"{
        "version": "1.0.0",
        "defaults": [
            {"key": "TEAM_NAME", "value": "Structure Your Story", "type": "text"},
            {"key": "TEAM_EMAIL", "value": "hello@structureyourstory.com", "type": "text"}
        ]
    }"#;
    let df = DefaultsFile::from_json(json).expect("should parse");
    assert_eq!(df.get_default("TEAM_NAME"), Some("Structure Your Story"));
    assert_eq!(
        df.get_default("TEAM_EMAIL"),
        Some("hello@structureyourstory.com")
    );
    assert_eq!(df.get_default("NONEXISTENT"), None);
}

#[test]
fn reject_invalid_json() {
    let result = DefaultsFile::from_json("not json");
    assert!(result.is_err(), "should reject invalid JSON");
}

#[test]
fn reject_missing_version() {
    let json = r#"{"defaults": []}"#;
    let result = DefaultsFile::from_json(json);
    // Missing "version" field should fail if version is required
    // Depending on serde config, this might succeed with empty version.
    // Our struct requires version field, so this should fail.
    assert!(result.is_err(), "should reject missing version field");
}

#[test]
fn round_trip_json_serialization() {
    let json = r#"{
        "version": "1.0.0",
        "defaults": [
            {"key": "TEAM_NAME", "value": "Structure Your Story", "type": "text", "description": "The team name"}
        ]
    }"#;
    let df = DefaultsFile::from_json(json).expect("should parse");
    let round_tripped = df.to_json().expect("should serialize");
    let df2 = DefaultsFile::from_json(&round_tripped).expect("should re-parse");
    assert_eq!(df, df2, "round-trip should preserve all data");
}

#[test]
fn version_mismatch_detection() {
    // This tests that version mismatch can be detected.
    // The actual comparison logic is in the template crate,
    // but we verify the version field is accessible.
    let json = r#"{"version": "2.0.0", "defaults": []}"#;
    let df = DefaultsFile::from_json(json).expect("should parse");
    assert_eq!(df.version, "2.0.0");
}
