//! Tests for CLI error handling and output formatting.
//!
//! Verifies FR-017 (structured errors) and FR-020 (TEMPLATE_DIR validation).

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn generate_errors_when_template_dir_missing() {
    let mut cmd = Command::cargo_bin("kbauto").unwrap();
    cmd.arg("generate")
        .arg("--template-dir")
        .arg("/nonexistent/path")
        .assert()
        .failure();
}

#[test]
fn generate_errors_when_docs_subdir_missing() {
    let dir = tempfile::tempdir().unwrap();
    // Create defaults.json but no docs/ subdirectory
    std::fs::write(dir.path().join("defaults.json"), "{}").unwrap();

    let mut cmd = Command::cargo_bin("kbauto").unwrap();
    cmd.arg("generate")
        .arg("--template-dir")
        .arg(dir.path())
        .assert()
        .failure();
}

#[test]
fn generate_errors_when_defaults_json_missing() {
    let dir = tempfile::tempdir().unwrap();
    // Create docs/ subdirectory but no defaults.json
    std::fs::create_dir(dir.path().join("docs")).unwrap();

    let mut cmd = Command::cargo_bin("kbauto").unwrap();
    cmd.arg("generate")
        .arg("--template-dir")
        .arg(dir.path())
        .assert()
        .failure();
}

#[test]
fn schema_errors_when_template_dir_missing() {
    let mut cmd = Command::cargo_bin("kbauto").unwrap();
    cmd.arg("schema")
        .arg("--template-dir")
        .arg("/nonexistent/path")
        .assert()
        .failure();
}

#[test]
fn schema_errors_when_docs_subdir_missing() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("defaults.json"), "{}").unwrap();

    let mut cmd = Command::cargo_bin("kbauto").unwrap();
    cmd.arg("schema")
        .arg("--template-dir")
        .arg(dir.path())
        .assert()
        .failure();
}

#[test]
fn schema_errors_when_defaults_json_missing() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("docs")).unwrap();

    let mut cmd = Command::cargo_bin("kbauto").unwrap();
    cmd.arg("schema")
        .arg("--template-dir")
        .arg(dir.path())
        .assert()
        .failure();
}

#[test]
fn generate_error_message_human_mode() {
    let mut cmd = Command::cargo_bin("kbauto").unwrap();
    cmd.arg("generate")
        .arg("--template-dir")
        .arg("/nonexistent/path")
        .assert()
        .stderr(predicate::str::contains("Template directory not found"));
}

#[test]
fn schema_error_message_human_mode() {
    let mut cmd = Command::cargo_bin("kbauto").unwrap();
    cmd.arg("schema")
        .arg("--template-dir")
        .arg("/nonexistent/path")
        .assert()
        .stderr(predicate::str::contains("Template directory not found"));
}

#[test]
fn generate_errors_when_brief_file_missing() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("docs")).unwrap();
    std::fs::write(
        dir.path().join("defaults.json"),
        r#"{"version": "1.0.0", "defaults": {}}"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("kbauto").unwrap();
    cmd.arg("generate")
        .arg("--template-dir")
        .arg(dir.path())
        .arg("--brief")
        .arg("/nonexistent/brief.md")
        .assert()
        .failure();
}

#[test]
fn help_flag_works() {
    let mut cmd = Command::cargo_bin("kbauto").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Generate, rebase, and diff"));
}

#[test]
fn generate_help_flag_works() {
    let mut cmd = Command::cargo_bin("kbauto").unwrap();
    cmd.arg("generate")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("template-dir"));
}

#[test]
fn schema_help_flag_works() {
    let mut cmd = Command::cargo_bin("kbauto").unwrap();
    cmd.arg("schema")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("template-dir"));
}
