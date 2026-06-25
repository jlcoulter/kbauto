//! T022: Template loading contract tests
//!
//! Tests for: load_template, list_page_files, PlaybookTemplate, TemplateError
//! Covers loading from TEMPLATE_DIR/docs/*.md convention, parsing defaults.json,
//! version validation against PlaybookVersion, missing directory/file errors, and listing page files.

use kbauto_template::{TemplateError, list_page_files, load_template};
use std::path::PathBuf;

// Helper to create test fixtures
use std::fs;

fn setup_template_dir(dir: &std::path::Path) {
    let docs_dir = dir.join("docs");
    fs::create_dir_all(&docs_dir).expect("should create docs dir");

    let defaults_json = r#"{"version":"1.0.0","defaults":[{"key":"TEAM_NAME","value":"Default Co","type":"text"}]}"#;
    fs::write(dir.join("defaults.json"), defaults_json).expect("should write defaults.json");

    let page_content =
        "---\ntitle: Getting Started\nsidebar_position: 1\n---\n\nWelcome to TEAM_NAME.";
    fs::write(docs_dir.join("getting-started.md"), page_content).expect("should write page");
}

fn setup_template_dir_no_defaults(dir: &std::path::Path) {
    let docs_dir = dir.join("docs");
    fs::create_dir_all(&docs_dir).expect("should create docs dir");
    // No defaults.json
    let page_content = "---\ntitle: Test\n---\n\nContent.";
    fs::write(docs_dir.join("test.md"), page_content).expect("should write page");
}

fn setup_template_dir_no_docs(dir: &std::path::Path) {
    fs::create_dir_all(dir).expect("should create dir");
    let defaults_json = r#"{"version":"1.0.0","defaults":[]}"#;
    fs::write(dir.join("defaults.json"), defaults_json).expect("should write defaults.json");
}

// --- Load from TEMPLATE_DIR/docs/*.md ---

#[test]
fn load_template_from_valid_dir() {
    let dir = tempfile::tempdir().expect("should create temp dir");
    setup_template_dir(dir.path());
    let result = load_template(dir.path());
    assert!(result.is_ok(), "should load valid template directory");
    let template = result.expect("should unwrap");
    assert_eq!(
        template.version, "1.0.0",
        "version should match defaults.json"
    );
    assert!(
        !template.pages.is_empty(),
        "should have loaded at least one page"
    );
}

#[test]
fn load_template_loads_page_content() {
    let dir = tempfile::tempdir().expect("should create temp dir");
    setup_template_dir(dir.path());
    let result = load_template(dir.path());
    let template = result.expect("should load");
    let getting_started = template
        .pages
        .iter()
        .find(|p| p.filename.contains("getting-started"));
    assert!(
        getting_started.is_some(),
        "should find getting-started.md page"
    );
    let page = getting_started.unwrap();
    assert!(
        page.content.contains("TEAM_NAME"),
        "page should contain placeholder"
    );
}

// --- Parse defaults.json ---

#[test]
fn load_template_parses_defaults() {
    let dir = tempfile::tempdir().expect("should create temp dir");
    setup_template_dir(dir.path());
    let result = load_template(dir.path());
    let template = result.expect("should load");
    assert_eq!(
        template.version, "1.0.0",
        "should parse version from defaults.json"
    );
}

#[test]
fn load_template_with_multiple_pages() {
    let dir = tempfile::tempdir().expect("should create temp dir");
    let docs_dir = dir.path().join("docs");
    fs::create_dir_all(&docs_dir).expect("should create docs dir");

    let defaults_json = r#"{"version":"1.0.0","defaults":[]}"#;
    fs::write(dir.path().join("defaults.json"), defaults_json).expect("should write defaults.json");

    fs::write(docs_dir.join("page1.md"), "Content 1 with TEAM_NAME").expect("should write page1");
    fs::write(docs_dir.join("page2.md"), "Content 2 with TEAM_EMAIL").expect("should write page2");
    fs::write(docs_dir.join("page3.md"), "Content 3").expect("should write page3");

    let template = load_template(dir.path()).expect("should load");
    assert_eq!(template.pages.len(), 3, "should load all 3 pages");
}

// --- Version validation against PlaybookVersion ---

#[test]
fn load_template_validates_version() {
    let dir = tempfile::tempdir().expect("should create temp dir");
    setup_template_dir(dir.path());
    let result = load_template(dir.path());
    let template = result.expect("should load");
    assert_eq!(
        template.parsed_version.major, 1,
        "version major should be 1"
    );
    assert_eq!(
        template.parsed_version.minor, 0,
        "version minor should be 0"
    );
    assert_eq!(
        template.parsed_version.patch, 0,
        "version patch should be 0"
    );
}

#[test]
fn load_template_rejects_invalid_version() {
    let dir = tempfile::tempdir().expect("should create temp dir");
    let docs_dir = dir.path().join("docs");
    fs::create_dir_all(&docs_dir).expect("should create docs dir");
    let defaults_json = r#"{"version":"not-a-version","defaults":[]}"#;
    fs::write(dir.path().join("defaults.json"), defaults_json).expect("should write defaults.json");
    let result = load_template(dir.path());
    // Should fail because "not-a-version" is not valid semver
    assert!(
        result.is_err(),
        "should reject invalid version in defaults.json"
    );
}

// --- Missing directory/file errors ---

#[test]
fn load_template_nonexistent_dir() {
    let result = load_template(PathBuf::from("/nonexistent/path/that/does/not/exist").as_path());
    assert!(result.is_err(), "should error on nonexistent directory");
    match result {
        Err(TemplateError::NotFound(_)) => {} // expected
        Err(e) => panic!("expected NotFound error, got: {e}"),
        Ok(_) => panic!("should not succeed with nonexistent dir"),
    }
}

#[test]
fn load_template_missing_docs_dir() {
    let dir = tempfile::tempdir().expect("should create temp dir");
    setup_template_dir_no_docs(dir.path());
    let result = load_template(dir.path());
    assert!(
        result.is_err(),
        "should error when docs/ subdirectory is missing"
    );
    match result {
        Err(TemplateError::MissingDocsDir(_)) => {} // expected
        Err(e) => panic!("expected MissingDocsDir error, got: {e}"),
        Ok(_) => panic!("should not succeed without docs/ dir"),
    }
}

#[test]
fn load_template_missing_defaults_json() {
    let dir = tempfile::tempdir().expect("should create temp dir");
    setup_template_dir_no_defaults(dir.path());
    let result = load_template(dir.path());
    assert!(
        result.is_err(),
        "should error when defaults.json is missing"
    );
    match result {
        Err(TemplateError::MissingDefaults(_)) => {} // expected
        Err(e) => panic!("expected MissingDefaults error, got: {e}"),
        Ok(_) => panic!("should not succeed without defaults.json"),
    }
}

// --- List page files ---

#[test]
fn list_page_files_returns_markdown_files() {
    let dir = tempfile::tempdir().expect("should create temp dir");
    let docs_dir = dir.path().join("docs");
    fs::create_dir_all(&docs_dir).expect("should create docs dir");

    fs::write(docs_dir.join("page1.md"), "content 1").expect("should write page1");
    fs::write(docs_dir.join("page2.md"), "content 2").expect("should write page2");
    fs::write(docs_dir.join("not-a-page.txt"), "text file").expect("should write txt");

    let result = list_page_files(dir.path());
    assert!(result.is_ok(), "should list page files");
    let files = result.expect("should unwrap");
    assert_eq!(files.len(), 2, "should only find .md files");
    assert!(
        files
            .iter()
            .all(|f| f.extension().is_some_and(|e| e == "md")),
        "all files should have .md extension"
    );
}

#[test]
fn list_page_files_empty_docs_dir() {
    let dir = tempfile::tempdir().expect("should create temp dir");
    let docs_dir = dir.path().join("docs");
    fs::create_dir_all(&docs_dir).expect("should create docs dir");

    let result = list_page_files(dir.path());
    assert!(result.is_ok(), "should succeed with empty docs dir");
    let files = result.expect("should unwrap");
    assert_eq!(files.len(), 0, "should find no .md files in empty dir");
}
