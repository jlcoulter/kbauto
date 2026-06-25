//! T068: End-to-end integration test.
//!
//! Validates the complete pipeline: placeholder extraction → generation →
//! provenance tracking → rebase → diff report → incremental update.
//!
//! Also tests the two-document input model (--details + --discovery),
//! missing value detection, and config integration.

use std::collections::HashMap;
use std::fs;

/// Helper: create a template directory with defaults and pages.
fn setup_template_dir(dir: &std::path::Path, version: &str) {
    let docs = dir.join("docs");
    fs::create_dir_all(&docs).unwrap();
    fs::write(
        dir.join("defaults.json"),
        format!(
            r#"{{"version":"{version}","defaults":[{{"key":"TEAM_NAME","value":"Default Team","type":"text"}},{{"key":"INDUSTRY","value":"Accounting","type":"text"}}]}}"#
        ),
    )
    .unwrap();
    fs::write(
        docs.join("welcome.md"),
        "# Welcome\n\nWelcome {{TEAM_NAME}} to our practice.\n\nWe serve the {{INDUSTRY}} industry.\n",
    )
    .unwrap();
    fs::write(
        docs.join("services.md"),
        "# Services\n\nOur {{INDUSTRY}} services are top-notch.\n\nContact {{TEAM_NAME}} for details.\n",
    )
    .unwrap();
}

#[test]
fn e2e_placeholder_extraction_and_resolution() {
    let content = "Hello {{TEAM_NAME}}, welcome to {{INDUSTRY}}.";
    let placeholders = kbauto_placeholder::extract_placeholders(content, "test.md");
    assert!(!placeholders.is_empty(), "should extract placeholders");

    let mut brief = HashMap::new();
    brief.insert("TEAM_NAME".to_string(), "Example".to_string());
    brief.insert("INDUSTRY".to_string(), "Accounting".to_string());
    let defaults = kbauto_placeholder::DefaultsFile {
        version: "1.0.0".to_string(),
        defaults: vec![],
    };
    let result = kbauto_placeholder::resolve_placeholders(content, &brief, &defaults);
    assert!(result.content.contains("Example"), "should resolve TEAM_NAME");
    assert!(
        result.content.contains("Accounting"),
        "should resolve INDUSTRY"
    );
}

#[tokio::test]
async fn e2e_generation_pipeline() {
    let template_dir = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    let brief_dir = tempfile::tempdir().unwrap();

    setup_template_dir(template_dir.path(), "1.0.0");
    fs::write(
        brief_dir.path().join("brief.md"),
        "## Team Name\nExample Co\n\n## Industry\nTax\n",
    )
    .unwrap();

    let result = kbauto_template::generate_playbook(
        template_dir.path(),
        Some(&brief_dir.path().join("brief.md")),
        None,
        output_dir.path(),
    )
    .await;

    assert!(result.is_ok(), "generation should succeed");
    let generation = result.unwrap();
    assert!(
        generation.pages_generated >= 2,
        "should generate at least 2 pages"
    );
    assert!(output_dir.path().join("welcome.md").exists());
    assert!(output_dir.path().join("services.md").exists());
}

#[test]
fn e2e_provenance_tracking() {
    let content = "---\nprovenance:\n  0: template\n  1: substituted\n---\n\n# Title\n\nTemplate text.\n\nSubstituted text.\n";
    let (fm, _rest) = kbauto_provenance::parse_frontmatter(content).unwrap();
    assert!(fm.provenance.contains_key(&0), "should have para 0");
    assert_eq!(fm.provenance.get(&0).unwrap(), "template");
    assert_eq!(fm.provenance.get(&1).unwrap(), "substituted");
}

#[test]
fn e2e_diff_report_between_versions() {
    let old_dir = tempfile::tempdir().unwrap();
    let new_dir = tempfile::tempdir().unwrap();

    setup_template_dir(old_dir.path(), "1.0.0");
    // New version with different text
    let new_docs = new_dir.path().join("docs");
    fs::create_dir_all(&new_docs).unwrap();
    fs::write(
        new_dir.path().join("defaults.json"),
        r#"{"version":"2.0.0","defaults":[]}"#,
    )
    .unwrap();
    fs::write(
        new_docs.join("welcome.md"),
        "# Welcome\n\nWelcome to our updated practice.\n",
    )
    .unwrap();
    fs::write(
        new_docs.join("services.md"),
        "# Services\n\nOur services are top-notch.\n",
    )
    .unwrap();

    let report = kbauto_rebase::diff_playbooks(old_dir.path(), new_dir.path()).unwrap();
    assert!(!report.changes.is_empty(), "should detect changes");
    assert_eq!(&report.old_version, "1.0.0");
    assert_eq!(&report.new_version, "2.0.0");
}

#[test]
fn e2e_incremental_update() {
    let template_dir = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();

    setup_template_dir(template_dir.path(), "1.0.0");

    let mut old_brief = HashMap::new();
    old_brief.insert("TEAM_NAME".to_string(), "Example".to_string());
    old_brief.insert("INDUSTRY".to_string(), "Accounting".to_string());

    let mut new_brief = HashMap::new();
    new_brief.insert("TEAM_NAME".to_string(), "Example Plus".to_string());
    new_brief.insert("INDUSTRY".to_string(), "Accounting".to_string());

    let defaults = kbauto_placeholder::DefaultsFile {
        version: "1.0.0".to_string(),
        defaults: vec![],
    };

    let result = kbauto_template::incremental_update(
        template_dir.path(),
        output_dir.path(),
        &old_brief,
        &new_brief,
        &defaults,
    );
    assert!(result.is_ok(), "incremental update should succeed");
    let update = result.unwrap();
    assert!(
        update.placeholders_updated > 0,
        "should update placeholders"
    );
}

// --- New tests for two-document model ---

#[tokio::test]
async fn e2e_generation_with_details_only() {
    let template_dir = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    let details_dir = tempfile::tempdir().unwrap();

    setup_template_dir(template_dir.path(), "1.0.0");
    fs::write(
        details_dir.path().join("details.md"),
        "## Team Name\nExample Co\n\n## Industry\nTax\n",
    )
    .unwrap();

    let result = kbauto_template::generate_playbook(
        template_dir.path(),
        Some(&details_dir.path().join("details.md")),
        None,
        output_dir.path(),
    )
    .await;

    assert!(result.is_ok(), "generation with details only should succeed");
    let generation = result.unwrap();
    assert!(generation.pages_generated >= 2, "should generate at least 2 pages");
    // With details provided, all placeholders should be resolved
    assert!(
        generation.missing_values.is_empty(),
        "should have no missing values when details cover all placeholders"
    );
}

#[tokio::test]
async fn e2e_generation_with_details_and_discovery() {
    let template_dir = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();
    let input_dir = tempfile::tempdir().unwrap();

    setup_template_dir(template_dir.path(), "1.0.0");
    fs::write(
        input_dir.path().join("details.md"),
        "## Team Name\nExample Co\n\n## Industry\nTax\n",
    )
    .unwrap();
    fs::write(
        input_dir.path().join("discovery.md"),
        "### Q: What services do you offer?\nA: Tax planning and advisory.\n\n### Q: Who is your target market?\nA: Small businesses.\n",
    )
    .unwrap();

    let result = kbauto_template::generate_playbook(
        template_dir.path(),
        Some(&input_dir.path().join("details.md")),
        Some(&input_dir.path().join("discovery.md")),
        output_dir.path(),
    )
    .await;

    assert!(
        result.is_ok(),
        "generation with details + discovery should succeed"
    );
    let generation = result.unwrap();
    assert!(generation.pages_generated >= 2);
    assert!(
        generation.missing_values.is_empty(),
        "should have no missing values when details cover all placeholders"
    );
}

#[tokio::test]
async fn e2e_generation_detects_missing_values() {
    let template_dir = tempfile::tempdir().unwrap();
    let output_dir = tempfile::tempdir().unwrap();

    // Template with placeholders that have defaults and ones that don't
    let docs = template_dir.path().join("docs");
    fs::create_dir_all(&docs).unwrap();
    fs::write(
        template_dir.path().join("defaults.json"),
        r#"{"version":"1.0.0","defaults":[{"key":"TEAM_NAME","value":"Default Team","type":"text"}]}"#,
    )
    .unwrap();
    // A page with a placeholder that has no default (CONTACT_EMAIL)
    fs::write(
        docs.join("contact.md"),
        "# Contact\n\nEmail us at {{CONTACT_EMAIL}}.\n\nTeam: {{TEAM_NAME}}.\n",
    )
    .unwrap();

    // Generate without providing details — should have missing values
    let result = kbauto_template::generate_playbook(
        template_dir.path(),
        None,
        None,
        output_dir.path(),
    )
    .await;

    assert!(result.is_ok(), "generation should succeed even with missing values");
    let generation = result.unwrap();
    assert!(
        !generation.missing_values.is_empty(),
        "should detect missing placeholder values"
    );
    // CONTACT_EMAIL should be missing (no default)
    let missing_keys: Vec<&str> = generation.missing_values.iter().map(|mv| mv.key.as_str()).collect();
    assert!(
        missing_keys.contains(&"CONTACT_EMAIL"),
        "CONTACT_EMAIL should be in missing values, got: {missing_keys:?}"
    );
    // TEAM_NAME should NOT be missing (has a default)
    assert!(
        !missing_keys.contains(&"TEAM_NAME"),
        "TEAM_NAME has a default and should not be missing"
    );
}

#[test]
fn e2e_static_details_parsing() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("details.md"),
        "## Team Name\nExample Co\n\n## Industry\nTax\n\n## Location\nMelbourne\n",
    )
    .unwrap();

    let details = kbauto_template::StaticDetails::from_markdown_file(
        &dir.path().join("details.md"),
    )
    .unwrap();

    assert_eq!(details.entries.get("TEAM_NAME").unwrap(), "Example Co");
    assert_eq!(details.entries.get("INDUSTRY").unwrap(), "Tax");
    assert_eq!(details.entries.get("LOCATION").unwrap(), "Melbourne");
}

#[test]
fn e2e_discovery_document_parsing() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("discovery.md"),
        "### What services do you offer?\nTax planning and advisory.\n\n### Who is your target market?\nSmall businesses.\n",
    )
    .unwrap();

    let discovery = kbauto_template::DiscoveryDocument::from_markdown_file(
        &dir.path().join("discovery.md"),
    )
    .unwrap();

    assert_eq!(discovery.questions.len(), 2);
    assert_eq!(
        discovery.questions[0].question,
        "What services do you offer?"
    );
    assert_eq!(discovery.questions[0].answer, "Tax planning and advisory.");
}