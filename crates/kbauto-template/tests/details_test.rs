//! Tests for StaticDetails parser.

use kbauto_template::StaticDetails;
use std::path::Path;

fn fixture_dir() -> &'static Path {
    Path::new("tests/fixtures")
}

#[test]
fn parse_details_with_heading_value_pairs() {
    let md = r#"## Team Name
We are Example Corp

## Team Email
hello@example.com

## About the Firm
We specialize in small business accounting.
"#;
    let details = StaticDetails::from_markdown(md, "test").unwrap();
    assert_eq!(details.get("TEAM_NAME"), Some("We are Example Corp"));
    assert_eq!(details.get("TEAM_EMAIL"), Some("hello@example.com"));
    assert_eq!(
        details.get("ABOUT_THE_FIRM"),
        Some("We specialize in small business accounting.")
    );
}

#[test]
fn h3_headings_parsed() {
    let md = "### Contact Info\ncall us\n";
    let details = StaticDetails::from_markdown(md, "test").unwrap();
    assert_eq!(details.get("CONTACT_INFO"), Some("call us"));
}

#[test]
fn heading_normalisation_spaces_and_hyphens() {
    let md = "## Team-Name Value\ncontent\n";
    let details = StaticDetails::from_markdown(md, "test").unwrap();
    // Both spaces and hyphens should become underscores, then uppercased
    assert!(details.contains_key("TEAM_NAME_VALUE"));
}

#[test]
fn canonical_key_normalisation() {
    let md = "## Client Phone Number\n0400 123 456\n";
    let details = StaticDetails::from_markdown(md, "test").unwrap();
    assert_eq!(details.get("CLIENT_PHONE_NUMBER"), Some("0400 123 456"));
}

#[test]
fn empty_section_has_empty_value() {
    let md = "## Empty Section\n## Next Section\nhas content\n";
    let details = StaticDetails::from_markdown(md, "test").unwrap();
    assert_eq!(details.get("EMPTY_SECTION"), Some(""));
    assert_eq!(details.get("NEXT_SECTION"), Some("has content"));
}

#[test]
fn empty_details_returns_error() {
    let details = StaticDetails::from_markdown("no headings here", "test");
    assert!(details.is_err());
}

#[test]
fn from_file_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("details.md");
    std::fs::write(
        &path,
        "## Team Name\nExample Corp\n\n## Email\ninfo@example.com\n",
    )
    .unwrap();

    let details = StaticDetails::from_markdown_file(&path).unwrap();
    assert_eq!(details.get("TEAM_NAME"), Some("Example Corp"));
    assert_eq!(details.get("EMAIL"), Some("info@example.com"));
}

#[test]
fn missing_file_returns_error() {
    let result = StaticDetails::from_markdown_file(Path::new("/nonexistent/details.md"));
    assert!(result.is_err());
}

#[test]
fn len_and_is_empty() {
    let md = "## A\nval1\n\n## B\nval2\n";
    let details = StaticDetails::from_markdown(md, "test").unwrap();
    assert_eq!(details.len(), 2);
    assert!(!details.is_empty());

    let empty = StaticDetails::from_markdown("no headings", "test");
    assert!(empty.is_err());
}

#[test]
fn case_insensitive_get() {
    let md = "## Team Name\nExample\n";
    let details = StaticDetails::from_markdown(md, "test").unwrap();
    // Canonical keys are UPPERCASE, but get() should normalise the lookup
    assert_eq!(details.get("team_name"), Some("Example"));
    assert_eq!(details.get("TEAM_NAME"), Some("Example"));
    assert_eq!(details.get("Team_Name"), Some("Example"));
}
