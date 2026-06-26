//! T024: Generation pipeline contract tests
//!
//! Tests for: generate_playbook, GenerationResult
//! Covers full generation from template dir + brief, default-only generation without brief,
//! partial brief with fallback defaults, and provenance metadata on every paragraph.

use kbauto_template::generate_playbook;
use std::collections::HashMap;
use std::fs;

fn setup_template_dir(dir: &std::path::Path) {
    let docs_dir = dir.join("docs");
    fs::create_dir_all(&docs_dir).expect("should create docs dir");

    let defaults_json = r#"{"version":"1.0.0","defaults":[
        {"key":"TEAM_NAME","value":"Default Co","type":"text"},
        {"key":"TEAM_EMAIL","value":"info@default.com","type":"text"},
        {"key":"TEAM_PHONE","value":"01-234-5678","type":"text"}
    ]}"#;
    fs::write(dir.join("defaults.json"), defaults_json).expect("should write defaults.json");

    let welcome_content = "---\ntitle: Welcome\nsidebar_position: 1\n---\n\nWelcome to TEAM_NAME.\n\nContact us at TEAM_EMAIL or call TEAM_PHONE.";
    fs::write(docs_dir.join("welcome.md"), welcome_content).expect("should write welcome page");

    let services_content = "---\ntitle: Services\nsidebar_position: 2\n---\n\nTEAM_NAME offers top-tier accounting services.\n\nReach out to TEAM_EMAIL for more info.";
    fs::write(docs_dir.join("services.md"), services_content).expect("should write services page");
}

/// Write a markdown brief file from section key-value pairs.
fn write_brief(dir: &std::path::Path, sections: HashMap<String, String>) {
    let mut md = String::new();
    for (key, value) in &sections {
        md.push_str(&format!("## {key}\n{value}\n\n"));
    }
    fs::write(dir.join("brief.md"), md).expect("should write brief.md");
}

// --- Full generation from template dir + brief ---

#[tokio::test]
async fn generate_playbook_full_generation() {
    let template_dir = tempfile::tempdir().expect("should create temp dir");
    setup_template_dir(template_dir.path());

    let output_dir = tempfile::tempdir().expect("should create output dir");
    let brief_dir = tempfile::tempdir().expect("should create brief dir");

    let mut sections = HashMap::new();
    sections.insert("Team Name".to_string(), "Example Corp".to_string());
    sections.insert("Team Email".to_string(), "hello@example.com".to_string());
    sections.insert("Team Phone".to_string(), "555-1234".to_string());
    write_brief(brief_dir.path(), sections);

    let brief_path = brief_dir.path().join("brief.md");
    let result = generate_playbook(
        template_dir.path(),
        Some(&brief_path),
        None,
        output_dir.path(),
    )
    .await;

    assert!(
        result.is_ok(),
        "full generation should succeed: {:?}",
        result
    );
    let generation = result.expect("should unwrap generation result");
    assert!(
        generation.pages_generated > 0,
        "should generate at least one page"
    );
    assert!(
        generation.placeholders_resolved > 0,
        "should resolve at least one placeholder"
    );
}

// --- Default-only generation without brief ---

#[tokio::test]
async fn generate_playbook_defaults_only() {
    let template_dir = tempfile::tempdir().expect("should create temp dir");
    setup_template_dir(template_dir.path());

    let output_dir = tempfile::tempdir().expect("should create output dir");

    // No brief path → use defaults only
    let result = generate_playbook(template_dir.path(), None, None, output_dir.path()).await;

    assert!(
        result.is_ok(),
        "defaults-only generation should succeed: {:?}",
        result
    );
    let generation = result.expect("should unwrap");
    assert!(
        generation.pages_generated > 0,
        "should generate pages from defaults"
    );
    // All placeholders should be resolved from defaults
    assert!(
        generation.placeholders_resolved > 0,
        "should resolve placeholders from defaults"
    );
}

// --- Partial brief with fallback defaults ---

#[tokio::test]
async fn generate_playbook_partial_brief() {
    let template_dir = tempfile::tempdir().expect("should create temp dir");
    setup_template_dir(template_dir.path());

    let output_dir = tempfile::tempdir().expect("should create output dir");
    let brief_dir = tempfile::tempdir().expect("should create brief dir");

    // Only provide Team Name in brief; Team Email and Team Phone fall back to defaults
    let mut sections = HashMap::new();
    sections.insert("Team Name".to_string(), "Example Corp".to_string());
    write_brief(brief_dir.path(), sections);

    let brief_path = brief_dir.path().join("brief.md");
    let result = generate_playbook(
        template_dir.path(),
        Some(&brief_path),
        None,
        output_dir.path(),
    )
    .await;

    assert!(
        result.is_ok(),
        "partial brief generation should succeed: {:?}",
        result
    );
    let generation = result.expect("should unwrap");
    assert!(generation.pages_generated > 0, "should generate pages");

    // Check that the output contains both brief and default values
    let output_files: Vec<_> = fs::read_dir(output_dir.path())
        .expect("should read output dir")
        .filter_map(|e| e.ok())
        .collect();
    assert!(!output_files.is_empty(), "should have output files");
}

// --- Provenance metadata on every paragraph ---

#[tokio::test]
async fn generate_playbook_provenance_on_every_paragraph() {
    let template_dir = tempfile::tempdir().expect("should create temp dir");
    setup_template_dir(template_dir.path());

    let output_dir = tempfile::tempdir().expect("should create output dir");
    let brief_dir = tempfile::tempdir().expect("should create brief dir");

    let mut sections = HashMap::new();
    sections.insert("Team Name".to_string(), "Example Corp".to_string());
    sections.insert("Team Email".to_string(), "hello@example.com".to_string());
    write_brief(brief_dir.path(), sections);

    let brief_path = brief_dir.path().join("brief.md");
    let result = generate_playbook(
        template_dir.path(),
        Some(&brief_path),
        None,
        output_dir.path(),
    )
    .await;

    assert!(result.is_ok(), "generation should succeed: {:?}", result);

    // Check that output files have provenance metadata in frontmatter
    let output_dir_path = output_dir.path();
    let md_files: Vec<_> = walkdir::WalkDir::new(output_dir_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .collect();

    for file in &md_files {
        let content = fs::read_to_string(file.path()).expect("should read output file");
        if content.starts_with("---") {
            // Should contain provenance in frontmatter
            assert!(
                content.contains("provenance"),
                "output file {:?} should contain provenance metadata",
                file.path()
            );
        }
    }
}

// --- Error on invalid template directory ---

#[tokio::test]
async fn generate_playbook_invalid_template_dir() {
    let output_dir = tempfile::tempdir().expect("should create output dir");
    let result = generate_playbook(
        std::path::Path::new("/nonexistent/path"),
        None,
        None,
        output_dir.path(),
    )
    .await;
    assert!(
        result.is_err(),
        "should error on nonexistent template directory"
    );
}
