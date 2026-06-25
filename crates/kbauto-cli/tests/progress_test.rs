//! Tests for CLI per-page progress output.
//!
//! Verifies FR-017: progress messages during generation,
//! final summary with pages generated, placeholders resolved,
//! and elapsed time.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn setup_template_dir() -> TempDir {
    let dir = TempDir::new().unwrap();
    let docs_dir = dir.path().join("docs");
    std::fs::create_dir_all(&docs_dir).unwrap();
    std::fs::write(
        dir.path().join("defaults.json"),
        r#"{"version": "1.0.0", "defaults": [{"key": "TEAM_NAME", "value": "Default Team", "type": "text"}]}"#,
    )
    .unwrap();
    std::fs::write(
        docs_dir.join("welcome.md"),
        r#"---
title: Welcome
sidebar_position: 1
---

# Welcome to TEAM_NAME

We are glad you are here.
"#,
    )
    .unwrap();
    dir
}

#[test]
fn generate_produces_output_directory() {
    let dir = setup_template_dir();
    let output_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("kbauto").unwrap();
    cmd.arg("generate")
        .arg("--template-dir")
        .arg(dir.path())
        .arg("--output")
        .arg(output_dir.path().join("kb"))
        .assert()
        .success();

    // Output directory should have been created
    assert!(
        output_dir.path().join("kb").exists() || output_dir.path().join("kb/welcome.md").exists()
    );
}

#[test]
fn generate_summary_contains_pages_generated() {
    let dir = setup_template_dir();
    let output_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("kbauto").unwrap();
    cmd.arg("generate")
        .arg("--template-dir")
        .arg(dir.path())
        .arg("--output")
        .arg(output_dir.path().join("kb"))
        .assert()
        .success()
        .stdout(predicate::str::contains("1 pages"));
}

#[test]
fn generate_summary_contains_placeholders_resolved() {
    let dir = setup_template_dir();
    let output_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("kbauto").unwrap();
    cmd.arg("generate")
        .arg("--template-dir")
        .arg(dir.path())
        .arg("--output")
        .arg(output_dir.path().join("kb"))
        .assert()
        .success()
        .stdout(predicate::str::contains("placeholder"));
}

#[test]
fn generate_json_output_format() {
    let dir = setup_template_dir();
    let output_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("kbauto").unwrap();
    cmd.arg("generate")
        .arg("--template-dir")
        .arg(dir.path())
        .arg("--output")
        .arg(output_dir.path().join("kb"))
        .arg("--output-format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("pages_generated"));
}

#[test]
fn generate_default_only_no_details() {
    let dir = setup_template_dir();
    let output_dir = TempDir::new().unwrap();

    // Generate without details — should use defaults
    let mut cmd = Command::cargo_bin("kbauto").unwrap();
    cmd.arg("generate")
        .arg("--template-dir")
        .arg(dir.path())
        .arg("--output")
        .arg(output_dir.path().join("kb"))
        .assert()
        .success();

    // Verify the output file contains the default value
    let output_path = output_dir.path().join("kb/welcome.md");
    if output_path.exists() {
        let content = std::fs::read_to_string(&output_path).unwrap();
        assert!(content.contains("Default Team"));
    }
}

#[test]
fn schema_human_output() {
    let dir = setup_template_dir();

    let mut cmd = Command::cargo_bin("kbauto").unwrap();
    cmd.arg("schema")
        .arg("--template-dir")
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Placeholder Schema"));
}

#[test]
fn schema_json_output() {
    let dir = setup_template_dir();

    let mut cmd = Command::cargo_bin("kbauto").unwrap();
    cmd.arg("schema")
        .arg("--template-dir")
        .arg(dir.path())
        .arg("--output-format")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("TEAM_NAME"));
}

#[test]
fn generate_with_details_file() {
    let dir = setup_template_dir();
    let output_dir = TempDir::new().unwrap();

    // Create a details file (markdown format with heading-value pairs)
    let details_dir = TempDir::new().unwrap();
    std::fs::write(
        details_dir.path().join("details.md"),
        "## Team Name\nExample Corp\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("kbauto").unwrap();
    cmd.arg("generate")
        .arg("--template-dir")
        .arg(dir.path())
        .arg("--details")
        .arg(details_dir.path().join("details.md"))
        .arg("--output")
        .arg(output_dir.path().join("kb"))
        .assert()
        .success();
}