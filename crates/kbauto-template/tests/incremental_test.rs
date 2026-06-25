//! T053: Incremental update contract tests
//!
//! Tests for brief diff detection, affected page identification,
//! partial regeneration, provenance update on changed pages only,
//! and byte-identical unchanged pages after incremental update.

use kbauto_template::{
    StaticDetails, brief_diff, find_affected_pages, incremental_update, load_template,
};
use std::collections::HashMap;
use std::fs;

// ---------- Helpers ----------

fn setup_template_dir(dir: &std::path::Path) {
    let docs_dir = dir.join("docs");
    fs::create_dir_all(&docs_dir).expect("should create docs dir");

    let defaults_json = r#"{"version":"1.0.0","defaults":[
        {"key":"TEAM_NAME","value":"Default Co","type":"text"},
        {"key":"TEAM_EMAIL","value":"info@default.com","type":"text"},
        {"key":"TEAM_PHONE","value":"01-234-5678","type":"text"},
        {"key":"FIRM_SPECIALTY","value":"General Services","type":"text"}
    ]}"#;
    fs::write(dir.join("defaults.json"), defaults_json).expect("should write defaults.json");

    // welcome.md uses TEAM_NAME and TEAM_EMAIL
    let welcome = "---\ntitle: Welcome\nsidebar_position: 1\n---\n\nWelcome to TEAM_NAME.\n\nContact us at TEAM_EMAIL.";
    fs::write(docs_dir.join("welcome.md"), welcome).expect("should write welcome.md");

    // services.md uses TEAM_NAME and TEAM_PHONE
    let services = "---\ntitle: Services\nsidebar_position: 2\n---\n\nTEAM_NAME offers great services.\n\nCall TEAM_PHONE for details.";
    fs::write(docs_dir.join("services.md"), services).expect("should write services.md");

    // about.md uses FIRM_SPECIALTY only
    let about = "---\ntitle: About\nsidebar_position: 3\n---\n\nWe specialise in FIRM_SPECIALTY.";
    fs::write(docs_dir.join("about.md"), about).expect("should write about.md");
}

/// Write a details file with section names matching placeholder keys (UPPERCASE_SNAKE_CASE).
/// The StaticDetails parser canonicalises heading text to UPPERCASE, so using
/// "TEAM_NAME" as heading produces key "TEAM_NAME" directly.
fn write_brief(dir: &std::path::Path, sections: &HashMap<String, String>) -> std::path::PathBuf {
    let mut md = String::new();
    for (key, value) in sections {
        md.push_str(&format!("## {key}\n{value}\n\n"));
    }
    let path = dir.join("brief.md");
    fs::write(&path, md).expect("should write brief.md");
    path
}

fn brief_sections(path: &std::path::Path) -> HashMap<String, String> {
    let details = StaticDetails::from_markdown_file(path).expect("should parse details");
    details.entries
}

// ---------- T053: Brief diff detection ----------

#[test]
fn brief_diff_detects_changed_value() {
    let old = HashMap::from([
        ("TEAM_NAME".to_string(), "Example".to_string()),
        ("TEAM_EMAIL".to_string(), "old@example.com".to_string()),
    ]);
    let new = HashMap::from([
        ("TEAM_NAME".to_string(), "Example".to_string()),
        ("TEAM_EMAIL".to_string(), "new@example.com".to_string()),
    ]);
    let diff = brief_diff(&old, &new);
    assert!(diff.changed_keys.contains(&"TEAM_EMAIL".to_string()));
    assert!(!diff.changed_keys.contains(&"TEAM_NAME".to_string()));
}

#[test]
fn brief_diff_detects_added_key() {
    let old = HashMap::from([("TEAM_NAME".to_string(), "Example".to_string())]);
    let new = HashMap::from([
        ("TEAM_NAME".to_string(), "Example".to_string()),
        ("TEAM_PHONE".to_string(), "555-1234".to_string()),
    ]);
    let diff = brief_diff(&old, &new);
    assert!(diff.added_keys.contains(&"TEAM_PHONE".to_string()));
    assert!(diff.changed_keys.is_empty());
}

#[test]
fn brief_diff_detects_removed_key() {
    let old = HashMap::from([
        ("TEAM_NAME".to_string(), "Example".to_string()),
        ("TEAM_EMAIL".to_string(), "hi@example.com".to_string()),
    ]);
    let new = HashMap::from([("TEAM_NAME".to_string(), "Example".to_string())]);
    let diff = brief_diff(&old, &new);
    assert!(diff.removed_keys.contains(&"TEAM_EMAIL".to_string()));
}

#[test]
fn brief_diff_case_insensitive() {
    let old = HashMap::from([("TEAM_NAME".to_string(), "Example".to_string())]);
    let new = HashMap::from([("team_name".to_string(), "Beta".to_string())]);
    let diff = brief_diff(&old, &new);
    assert_eq!(diff.changed_keys, vec!["TEAM_NAME"]);
}

#[test]
fn brief_diff_no_changes() {
    let old = HashMap::from([("TEAM_NAME".to_string(), "Example".to_string())]);
    let new = HashMap::from([("TEAM_NAME".to_string(), "Example".to_string())]);
    let diff = brief_diff(&old, &new);
    assert!(diff.changed_keys.is_empty());
    assert!(diff.added_keys.is_empty());
    assert!(diff.removed_keys.is_empty());
}

// ---------- T053: Affected page identification ----------

#[test]
fn affected_pages_identifies_correct_pages() {
    let template_dir = tempfile::tempdir().expect("should create temp dir");
    setup_template_dir(template_dir.path());
    let template = load_template(template_dir.path()).expect("should load template");

    // Changing TEAM_EMAIL should affect welcome.md but not services.md or about.md
    let changed = vec!["TEAM_EMAIL".to_string()];
    let affected = find_affected_pages(&template, &changed);
    assert!(affected.contains(&"welcome.md".to_string()));
    assert!(!affected.contains(&"services.md".to_string()));
    assert!(!affected.contains(&"about.md".to_string()));
}

#[test]
fn affected_pages_multiple_keys() {
    let template_dir = tempfile::tempdir().expect("should create temp dir");
    setup_template_dir(template_dir.path());
    let template = load_template(template_dir.path()).expect("should load template");

    // Changing both TEAM_NAME and FIRM_SPECIALTY should affect all three pages
    let changed = vec!["TEAM_NAME".to_string(), "FIRM_SPECIALTY".to_string()];
    let affected = find_affected_pages(&template, &changed);
    assert!(affected.contains(&"welcome.md".to_string()));
    assert!(affected.contains(&"services.md".to_string()));
    assert!(affected.contains(&"about.md".to_string()));
}

#[test]
fn affected_pages_no_keys() {
    let template_dir = tempfile::tempdir().expect("should create temp dir");
    setup_template_dir(template_dir.path());
    let template = load_template(template_dir.path()).expect("should load template");

    let affected = find_affected_pages(&template, &[]);
    assert!(affected.is_empty());
}

// ---------- T053: Partial regeneration (only affected pages updated) ----------

#[test]
fn incremental_update_only_regenerates_affected_pages() {
    let template_dir = tempfile::tempdir().expect("should create temp dir");
    setup_template_dir(template_dir.path());
    let output_dir = tempfile::tempdir().expect("should create output dir");

    // Initial generation with old brief
    let old_sections = HashMap::from([
        ("TEAM_NAME".to_string(), "Example Corp".to_string()),
        ("TEAM_EMAIL".to_string(), "old@example.com".to_string()),
        ("TEAM_PHONE".to_string(), "555-0000".to_string()),
        ("FIRM_SPECIALTY".to_string(), "Tax Prep".to_string()),
    ]);
    let brief_dir = tempfile::tempdir().expect("should create brief dir");
    let brief_path = write_brief(brief_dir.path(), &old_sections);

    // Do initial generation
    let rt = tokio::runtime::Runtime::new().expect("should create runtime");
    rt.block_on(async {
        let r = kbauto_template::generate_playbook(
            template_dir.path(),
            Some(&brief_path),
            None,
            output_dir.path(),
        )
        .await;
        assert!(r.is_ok(), "initial generation should succeed: {:?}", r);
    });

    // Now do incremental update changing only TEAM_EMAIL
    let new_sections = HashMap::from([
        ("TEAM_NAME".to_string(), "Example Corp".to_string()),
        ("TEAM_EMAIL".to_string(), "new@example.com".to_string()),
        ("TEAM_PHONE".to_string(), "555-0000".to_string()),
        ("FIRM_SPECIALTY".to_string(), "Tax Prep".to_string()),
    ]);

    let defaults_content = fs::read_to_string(template_dir.path().join("defaults.json"))
        .expect("should read defaults");
    let defaults = kbauto_placeholder::DefaultsFile::from_json(&defaults_content).unwrap();

    let old_brief = brief_sections(&brief_path);
    let brief_dir2 = tempfile::tempdir().expect("should create brief dir 2");
    let new_brief_path = write_brief(brief_dir2.path(), &new_sections);
    let new_brief_map = brief_sections(&new_brief_path);

    let result = incremental_update(
        template_dir.path(),
        output_dir.path(),
        &old_brief,
        &new_brief_map,
        &defaults,
    );
    assert!(
        result.is_ok(),
        "incremental update should succeed: {:?}",
        result
    );
    let inc = result.unwrap();

    // At least one page should be updated (welcome.md contains TEAM_EMAIL)
    assert!(
        inc.pages_updated >= 1,
        "at least one page should be updated, got {}",
        inc.pages_updated
    );
}

// ---------- T053: Provenance update only on changed pages ----------

#[test]
fn incremental_update_provenance_only_on_changed_pages() {
    let template_dir = tempfile::tempdir().expect("should create temp dir");
    setup_template_dir(template_dir.path());
    let output_dir = tempfile::tempdir().expect("should create output dir");

    let old_sections = HashMap::from([
        ("TEAM_NAME".to_string(), "Example".to_string()),
        ("TEAM_EMAIL".to_string(), "old@example.com".to_string()),
        ("TEAM_PHONE".to_string(), "555-0000".to_string()),
        ("FIRM_SPECIALTY".to_string(), "Tax".to_string()),
    ]);
    let brief_dir = tempfile::tempdir().expect("should create brief dir");
    let brief_path = write_brief(brief_dir.path(), &old_sections);

    // Initial generation
    let rt = tokio::runtime::Runtime::new().expect("should create runtime");
    rt.block_on(async {
        let r = kbauto_template::generate_playbook(
            template_dir.path(),
            Some(&brief_path),
            None,
            output_dir.path(),
        )
        .await;
        assert!(r.is_ok(), "initial generation should succeed");
    });

    // Record about.md content before incremental update
    let about_before =
        fs::read_to_string(output_dir.path().join("about.md")).expect("should read about.md");

    // Change only TEAM_EMAIL (in welcome.md, not in about.md)
    let new_sections = HashMap::from([
        ("TEAM_NAME".to_string(), "Example".to_string()),
        ("TEAM_EMAIL".to_string(), "new@example.com".to_string()),
        ("TEAM_PHONE".to_string(), "555-0000".to_string()),
        ("FIRM_SPECIALTY".to_string(), "Tax".to_string()),
    ]);

    let defaults_content = fs::read_to_string(template_dir.path().join("defaults.json"))
        .expect("should read defaults");
    let defaults = kbauto_placeholder::DefaultsFile::from_json(&defaults_content).unwrap();

    let old_brief = brief_sections(&brief_path);
    let brief_dir2 = tempfile::tempdir().expect("should create brief dir 2");
    let new_brief_path = write_brief(brief_dir2.path(), &new_sections);
    let new_brief_map = brief_sections(&new_brief_path);

    let result = incremental_update(
        template_dir.path(),
        output_dir.path(),
        &old_brief,
        &new_brief_map,
        &defaults,
    );
    assert!(result.is_ok(), "incremental update should succeed");

    // about.md should be byte-identical because it doesn't contain TEAM_EMAIL
    let about_after = fs::read_to_string(output_dir.path().join("about.md"))
        .expect("should read about.md after update");
    assert_eq!(about_before, about_after, "about.md should be unchanged");
}

// ---------- T053: Byte-identical unchanged pages ----------

#[test]
fn incremental_update_byte_identical_unchanged_pages() {
    let template_dir = tempfile::tempdir().expect("should create temp dir");
    setup_template_dir(template_dir.path());
    let output_dir = tempfile::tempdir().expect("should create output dir");

    let old_sections = HashMap::from([
        ("TEAM_NAME".to_string(), "Example".to_string()),
        ("TEAM_EMAIL".to_string(), "old@example.com".to_string()),
        ("TEAM_PHONE".to_string(), "555-0000".to_string()),
        ("FIRM_SPECIALTY".to_string(), "Tax Prep".to_string()),
    ]);
    let brief_dir = tempfile::tempdir().expect("should create brief dir");
    let brief_path = write_brief(brief_dir.path(), &old_sections);

    // Initial generation
    let rt = tokio::runtime::Runtime::new().expect("should create runtime");
    rt.block_on(async {
        let r = kbauto_template::generate_playbook(
            template_dir.path(),
            Some(&brief_path),
            None,
            output_dir.path(),
        )
        .await;
        assert!(r.is_ok(), "initial generation should succeed");
    });

    // Read about.md bytes before
    let about_bytes_before =
        fs::read(output_dir.path().join("about.md")).expect("should read about.md");

    // Change TEAM_EMAIL only
    let new_sections = HashMap::from([
        ("TEAM_NAME".to_string(), "Example".to_string()),
        ("TEAM_EMAIL".to_string(), "new@example.com".to_string()),
        ("TEAM_PHONE".to_string(), "555-0000".to_string()),
        ("FIRM_SPECIALTY".to_string(), "Tax Prep".to_string()),
    ]);

    let defaults_content = fs::read_to_string(template_dir.path().join("defaults.json"))
        .expect("should read defaults");
    let defaults = kbauto_placeholder::DefaultsFile::from_json(&defaults_content).unwrap();

    let old_brief = brief_sections(&brief_path);
    let brief_dir2 = tempfile::tempdir().expect("should create brief dir 2");
    let new_brief_path = write_brief(brief_dir2.path(), &new_sections);
    let new_brief_map = brief_sections(&new_brief_path);

    let result = incremental_update(
        template_dir.path(),
        output_dir.path(),
        &old_brief,
        &new_brief_map,
        &defaults,
    );
    assert!(result.is_ok());

    // about.md should be byte-for-byte identical
    let about_bytes_after =
        fs::read(output_dir.path().join("about.md")).expect("should read about.md after");
    assert_eq!(
        about_bytes_before, about_bytes_after,
        "about.md should be byte-identical after incremental update"
    );
}

// ---------- T053: IncrementalResult struct verification ----------

#[test]
fn incremental_result_counts() {
    let template_dir = tempfile::tempdir().expect("should create temp dir");
    setup_template_dir(template_dir.path());
    let output_dir = tempfile::tempdir().expect("should create output dir");

    let old_sections = HashMap::from([
        ("TEAM_NAME".to_string(), "Example".to_string()),
        ("TEAM_EMAIL".to_string(), "old@example.com".to_string()),
        ("TEAM_PHONE".to_string(), "555-0000".to_string()),
        ("FIRM_SPECIALTY".to_string(), "Tax Prep".to_string()),
    ]);
    let brief_dir = tempfile::tempdir().expect("should create brief dir");
    let brief_path = write_brief(brief_dir.path(), &old_sections);

    // Initial generation
    let rt = tokio::runtime::Runtime::new().expect("should create runtime");
    rt.block_on(async {
        let r = kbauto_template::generate_playbook(
            template_dir.path(),
            Some(&brief_path),
            None,
            output_dir.path(),
        )
        .await;
        assert!(r.is_ok());
    });

    // Change TEAM_NAME and TEAM_EMAIL
    let new_sections = HashMap::from([
        ("TEAM_NAME".to_string(), "Beta LLC".to_string()),
        ("TEAM_EMAIL".to_string(), "new@example.com".to_string()),
        ("TEAM_PHONE".to_string(), "555-0000".to_string()),
        ("FIRM_SPECIALTY".to_string(), "Tax Prep".to_string()),
    ]);

    let defaults_content = fs::read_to_string(template_dir.path().join("defaults.json"))
        .expect("should read defaults");
    let defaults = kbauto_placeholder::DefaultsFile::from_json(&defaults_content).unwrap();

    let old_brief = brief_sections(&brief_path);
    let brief_dir2 = tempfile::tempdir().expect("should create brief dir 2");
    let new_brief_path = write_brief(brief_dir2.path(), &new_sections);
    let new_brief_map = brief_sections(&new_brief_path);

    let result = incremental_update(
        template_dir.path(),
        output_dir.path(),
        &old_brief,
        &new_brief_map,
        &defaults,
    );
    assert!(result.is_ok());
    let inc = result.unwrap();

    // TEAM_NAME + TEAM_EMAIL changed = 2 placeholder keys
    assert_eq!(inc.placeholders_updated, 2);

    // welcome.md (TEAM_NAME, TEAM_EMAIL) and services.md (TEAM_NAME) are affected
    // about.md is not affected (only FIRM_SPECIALTY, unchanged)
    assert!(
        inc.pages_updated >= 2,
        "at least 2 pages should be updated, got {}",
        inc.pages_updated
    );
    assert!(
        inc.pages_unchanged >= 1,
        "at least 1 page should be unchanged, got {}",
        inc.pages_unchanged
    );

    // Total should equal number of pages
    let total = inc.pages_updated + inc.pages_unchanged;
    assert_eq!(total, 3, "total pages should be 3");
}