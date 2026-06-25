//! End-to-end tests for the wizard scaffold→generate flow.
//!
//! These tests exercise the library-level functions that the wizard calls,
//! simulating the two-session workflow:
//! 1. Scaffold phase: create client directory with skeleton files
//! 2. User edits skeleton files
//! 3. Generate phase: read edited files and generate KB

use kbauto_placeholder::{DefaultsFile, build_schema, extract_placeholders};
use kbauto_template::{
    generate_playbook, read_template_path, scaffold_client_dir, write_template_path,
};
use std::fs;
use tempfile::tempdir;

/// Create a test template directory with docs/ and defaults.json.
fn create_test_template(dir: &std::path::Path, version: &str) {
    fs::create_dir_all(dir.join("docs")).unwrap();

    // Create a welcome page with placeholders
    fs::write(
        dir.join("docs/welcome.md"),
        "---\nslug: welcome\ntitle: Welcome\n---\n\n# Welcome to {{FIRM_NAME}}\n\nAt **{{FIRM_NAME}}**, our mission is simple: {{FIRM_TAGLINE}}.\n\n## Contact\n\n- Email: {{CONTACT_EMAIL}}\n- Phone: {{PHONE}}\n",
    )
    .unwrap();

    // Create a services page
    fs::write(
        dir.join("docs/services.md"),
        "---\nslug: services\ntitle: Services\n---\n\n# Our Services\n\n{{FIRM_NAME}} provides tailored services for {{CLIENT_INDUSTRY}}.\n",
    )
    .unwrap();

    // Create defaults.json
    let defaults_json = format!(
        r#"{{"version": "{version}", "defaults": [{{"key": "FIRM_NAME", "value": "Your Firm Name", "type": "text"}}, {{"key": "FIRM_TAGLINE", "value": "Trusted accounting", "type": "text"}}, {{"key": "CONTACT_EMAIL", "value": "hello@example.com", "type": "text"}}, {{"key": "PHONE", "value": "(555) 000-0000", "type": "text"}}, {{"key": "CLIENT_INDUSTRY", "value": "small business", "type": "text"}}]}}"#
    );
    fs::write(dir.join("defaults.json"), defaults_json).unwrap();
}

/// Simulate phase 1: scaffold a new client directory.
fn simulate_scaffold(
    client_dir: &std::path::Path,
    template_dir: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load template
    let template = kbauto_template::load_template(template_dir)?;
    let defaults_content = fs::read_to_string(template_dir.join("defaults.json"))?;
    let defaults = DefaultsFile::from_json(&defaults_content)?;

    // Extract placeholders and build schema
    let mut all_placeholders = Vec::new();
    for page in &template.pages {
        let phs = extract_placeholders(&page.content, &page.filename);
        all_placeholders.extend(phs);
    }
    let schema = build_schema(all_placeholders, &template.version);

    // Scaffold
    scaffold_client_dir(client_dir, template_dir, &schema, &defaults)?;
    Ok(())
}

#[test]
fn two_session_scaffold_then_generate() {
    let dir = tempdir().unwrap();
    let template_dir = dir.path().join("template");
    let client_dir = dir.path().join("example");

    // Create template
    create_test_template(&template_dir, "1.0.0");

    // Phase 1: Scaffold
    simulate_scaffold(&client_dir, &template_dir).unwrap();

    // Verify scaffold created all expected files
    assert!(client_dir.join("details.md").exists());
    assert!(client_dir.join("discovery.md").exists());
    assert!(client_dir.join("kb").exists());
    assert!(client_dir.join(".template-path").exists());

    // Verify skeleton details has placeholder keys
    let details = fs::read_to_string(client_dir.join("details.md")).unwrap();
    assert!(details.contains("## FIRM_NAME"));
    assert!(details.contains("## CONTACT_EMAIL"));

    // Simulate user editing the details file
    fs::write(
        client_dir.join("details.md"),
        "## FIRM_NAME\nExample Corp\n\n## FIRM_TAGLINE\nYour finances, our passion\n\n## CONTACT_EMAIL\nhello@example.com\n\n## PHONE\n(555) 123-4567\n\n## CLIENT_INDUSTRY\nhospitality\n",
    )
    .unwrap();

    // Phase 2: Generate
    // Read .template-path to find the template
    let template_path = read_template_path(&client_dir).unwrap();
    assert_eq!(template_path, template_dir.canonicalize().unwrap());

    // Generate the KB
    let output_dir = client_dir.join("kb");
    let result = {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(generate_playbook(
            &template_path,
            Some(&client_dir.join("details.md")),
            Some(&client_dir.join("discovery.md")),
            &output_dir,
        ))
        .unwrap()
    };

    // Verify generation succeeded
    assert!(result.pages_generated > 0);
    assert!(result.placeholders_resolved > 0);

    // Verify output files exist
    assert!(output_dir.join("welcome.md").exists());
    assert!(output_dir.join("services.md").exists());

    // Verify placeholders were resolved in the output
    let welcome = fs::read_to_string(output_dir.join("welcome.md")).unwrap();
    assert!(welcome.contains("Example Corp"));
    assert!(welcome.contains("hello@example.com"));
    assert!(!welcome.contains("{{FIRM_NAME}}"));
}

#[test]
fn scaffold_then_generate_with_defaults_only() {
    let dir = tempdir().unwrap();
    let template_dir = dir.path().join("template");
    let client_dir = dir.path().join("preview");

    create_test_template(&template_dir, "1.0.0");

    // Scaffold
    simulate_scaffold(&client_dir, &template_dir).unwrap();

    // Don't edit the skeleton files — use defaults as-is
    let template_path = read_template_path(&client_dir).unwrap();
    let output_dir = client_dir.join("kb");

    let result = {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(generate_playbook(
            &template_path,
            Some(&client_dir.join("details.md")),
            Some(&client_dir.join("discovery.md")),
            &output_dir,
        ))
        .unwrap()
    };

    assert!(result.pages_generated > 0);
    // Should have used default values from the skeleton
    let welcome = fs::read_to_string(output_dir.join("welcome.md")).unwrap();
    assert!(welcome.contains("Your Firm Name"));
}

#[test]
fn missing_template_path_recovery() {
    let dir = tempdir().unwrap();
    let template_dir = dir.path().join("template");
    let client_dir = dir.path().join("client");
    let new_template_dir = dir.path().join("new_template");

    // Create original template and scaffold
    create_test_template(&template_dir, "1.0.0");
    simulate_scaffold(&client_dir, &template_dir).unwrap();

    // Delete the original template directory (simulating moved/renamed)
    fs::remove_dir_all(&template_dir).unwrap();

    // Read .template-path should fail
    let result = read_template_path(&client_dir);
    assert!(result.is_err());
    match result.unwrap_err() {
        kbauto_template::TemplatePathError::Invalid(_) => {}
        other => panic!("expected Invalid, got {other:?}"),
    }

    // Create a new template directory at a different location
    create_test_template(&new_template_dir, "1.0.0");

    // Simulate wizard recovery: update .template-path to the new location
    write_template_path(&client_dir, &new_template_dir).unwrap();

    // Now read should succeed
    let template_path = read_template_path(&client_dir).unwrap();
    assert_eq!(template_path, new_template_dir.canonicalize().unwrap());

    // Generate should work with the new template path
    let output_dir = client_dir.join("kb");
    let result = {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(generate_playbook(
            &template_path,
            Some(&client_dir.join("details.md")),
            Some(&client_dir.join("discovery.md")),
            &output_dir,
        ))
        .unwrap()
    };

    assert!(result.pages_generated > 0);
}

#[test]
fn scaffold_does_not_overwrite_existing_client() {
    let dir = tempdir().unwrap();
    let template_dir = dir.path().join("template");
    let client_dir = dir.path().join("existing");

    create_test_template(&template_dir, "1.0.0");

    // Create a non-empty client directory
    fs::create_dir_all(&client_dir).unwrap();
    fs::write(client_dir.join("important.txt"), "data").unwrap();

    // Scaffold should fail
    let result = simulate_scaffold(&client_dir, &template_dir);
    assert!(result.is_err());
}

#[test]
fn scaffold_creates_valid_skeleton_discovery() {
    let dir = tempdir().unwrap();
    let template_dir = dir.path().join("template");
    let client_dir = dir.path().join("client");

    create_test_template(&template_dir, "1.0.0");
    simulate_scaffold(&client_dir, &template_dir).unwrap();

    let discovery = fs::read_to_string(client_dir.join("discovery.md")).unwrap();
    // Should contain discovery question headings
    assert!(discovery.contains("## "));
    // Should have multiple questions
    let heading_count = discovery.matches("## ").count();
    assert!(heading_count >= 5, "should have at least 5 discovery questions");
}

#[test]
fn template_path_survives_across_calls() {
    let dir = tempdir().unwrap();
    let template_dir = dir.path().join("template");
    let client_dir = dir.path().join("client");

    create_test_template(&template_dir, "1.0.0");
    simulate_scaffold(&client_dir, &template_dir).unwrap();

    // First read
    let path1 = read_template_path(&client_dir).unwrap();

    // Second read (simulating returning days later)
    let path2 = read_template_path(&client_dir).unwrap();

    assert_eq!(path1, path2);
}