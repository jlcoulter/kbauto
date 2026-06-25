//! Tests for the wizard phase detection logic.
//!
//! These tests verify `detect_phase()` correctly identifies the wizard phase
//! from directory contents, without exercising the interactive TUI portions.

use std::fs;
use tempfile::tempdir;

// We can't import from the binary crate directly, so we replicate the phase
// detection logic here to test the core algorithm. The actual implementation
// lives in `crates/kbauto-cli/src/wizard.rs`.

#[derive(Debug, Clone, PartialEq, Eq)]
enum WizardPhase {
    Scaffold,
    Generate,
    RebaseOrUpdate,
}

fn detect_phase(client_dir: &std::path::Path) -> WizardPhase {
    let has_details = client_dir.join("details.md").exists();
    let has_discovery = client_dir.join("discovery.md").exists();
    let kb_dir = client_dir.join("kb");

    if !has_details && !has_discovery {
        return WizardPhase::Scaffold;
    }

    if has_details && has_discovery {
        if kb_dir.exists() {
            if let Ok(entries) = fs::read_dir(&kb_dir) {
                if entries.filter_map(|e| e.ok()).any(|_| true) {
                    return WizardPhase::RebaseOrUpdate;
                }
            }
        }
        return WizardPhase::Generate;
    }

    if kb_dir.exists() {
        if let Ok(entries) = fs::read_dir(&kb_dir) {
            if entries.filter_map(|e| e.ok()).any(|_| true) {
                return WizardPhase::RebaseOrUpdate;
            }
        }
    }
    WizardPhase::Generate
}

#[test]
fn empty_directory_is_scaffold() {
    let dir = tempdir().unwrap();
    let phase = detect_phase(dir.path());
    assert_eq!(phase, WizardPhase::Scaffold);
}

#[test]
fn nonexistent_directory_is_scaffold() {
    let dir = tempdir().unwrap();
    let client_dir = dir.path().join("nonexistent");
    let phase = detect_phase(&client_dir);
    assert_eq!(phase, WizardPhase::Scaffold);
}

#[test]
fn skeleton_files_empty_kb_is_generate() {
    let dir = tempdir().unwrap();
    let client_dir = dir.path().join("client");
    fs::create_dir_all(&client_dir).unwrap();
    fs::write(client_dir.join("details.md"), "## FIRM_NAME\nExample\n").unwrap();
    fs::write(client_dir.join("discovery.md"), "## Question\nAnswer\n").unwrap();
    fs::create_dir_all(client_dir.join("kb")).unwrap();

    let phase = detect_phase(&client_dir);
    assert_eq!(phase, WizardPhase::Generate);
}

#[test]
fn skeleton_files_no_kb_dir_is_generate() {
    let dir = tempdir().unwrap();
    let client_dir = dir.path().join("client");
    fs::create_dir_all(&client_dir).unwrap();
    fs::write(client_dir.join("details.md"), "## FIRM_NAME\nExample\n").unwrap();
    fs::write(client_dir.join("discovery.md"), "## Question\nAnswer\n").unwrap();
    // No kb/ directory

    let phase = detect_phase(&client_dir);
    assert_eq!(phase, WizardPhase::Generate);
}

#[test]
fn kb_with_content_is_rebase_or_update() {
    let dir = tempdir().unwrap();
    let client_dir = dir.path().join("client");
    fs::create_dir_all(&client_dir).unwrap();
    fs::write(client_dir.join("details.md"), "## FIRM_NAME\nExample\n").unwrap();
    fs::write(client_dir.join("discovery.md"), "## Question\nAnswer\n").unwrap();
    fs::create_dir_all(client_dir.join("kb")).unwrap();
    fs::write(client_dir.join("kb/welcome.md"), "# Welcome\n").unwrap();

    let phase = detect_phase(&client_dir);
    assert_eq!(phase, WizardPhase::RebaseOrUpdate);
}

#[test]
fn only_details_no_kb_is_generate() {
    let dir = tempdir().unwrap();
    let client_dir = dir.path().join("client");
    fs::create_dir_all(&client_dir).unwrap();
    fs::write(client_dir.join("details.md"), "## FIRM_NAME\nExample\n").unwrap();
    // No discovery.md, no kb/

    let phase = detect_phase(&client_dir);
    assert_eq!(phase, WizardPhase::Generate);
}

#[test]
fn only_details_with_kb_content_is_rebase() {
    let dir = tempdir().unwrap();
    let client_dir = dir.path().join("client");
    fs::create_dir_all(&client_dir).unwrap();
    fs::write(client_dir.join("details.md"), "## FIRM_NAME\nExample\n").unwrap();
    fs::create_dir_all(client_dir.join("kb")).unwrap();
    fs::write(client_dir.join("kb/page.md"), "# Page\n").unwrap();

    let phase = detect_phase(&client_dir);
    assert_eq!(phase, WizardPhase::RebaseOrUpdate);
}

#[test]
fn only_discovery_no_kb_is_generate() {
    let dir = tempdir().unwrap();
    let client_dir = dir.path().join("client");
    fs::create_dir_all(&client_dir).unwrap();
    fs::write(client_dir.join("discovery.md"), "## Q\nA\n").unwrap();

    let phase = detect_phase(&client_dir);
    assert_eq!(phase, WizardPhase::Generate);
}

#[test]
fn template_path_file_does_not_affect_detection() {
    let dir = tempdir().unwrap();
    let client_dir = dir.path().join("client");
    fs::create_dir_all(&client_dir).unwrap();
    fs::write(client_dir.join(".template-path"), "/some/template").unwrap();
    // No details.md or discovery.md

    let phase = detect_phase(&client_dir);
    assert_eq!(phase, WizardPhase::Scaffold);
}