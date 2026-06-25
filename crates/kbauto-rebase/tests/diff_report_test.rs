//! T059: Diff report output contract tests.
//!
//! Tests for: DiffReport display formatting, JSON serialization,
//! empty diff, page-level change categorisation.

use kbauto_rebase::{DiffReport, PageChange, diff_playbooks};
use std::fs;

fn setup_version_dir(dir: &std::path::Path, version: &str, files: &[(&str, &str)]) {
    let docs_dir = dir.join("docs");
    fs::create_dir_all(&docs_dir).expect("should create docs dir");
    let defaults_json = format!(r#"{{"version":"{version}","defaults":[]}}"#);
    fs::write(dir.join("defaults.json"), defaults_json).expect("should write defaults.json");
    for (name, content) in files {
        fs::write(docs_dir.join(name), content).expect("should write file");
    }
}

// --- Human-readable output format ---

#[test]
fn diff_report_display_shows_summary() {
    let report = DiffReport {
        old_version: "1.0.0".to_string(),
        new_version: "2.0.0".to_string(),
        changes: vec![
            PageChange::Added {
                filename: "services.md".to_string(),
            },
            PageChange::Modified {
                filename: "welcome.md".to_string(),
                paragraph_changes: vec![],
            },
            PageChange::Removed {
                filename: "old-page.md".to_string(),
            },
        ],
    };

    let display = format!("{report}");
    assert!(display.contains("1.0.0"), "should show old version");
    assert!(display.contains("2.0.0"), "should show new version");
    assert!(
        display.contains("added") || display.contains("Added"),
        "should mention added pages"
    );
    assert!(
        display.contains("removed") || display.contains("Removed"),
        "should mention removed pages"
    );
    assert!(
        display.contains("modified") || display.contains("Modified"),
        "should mention modified pages"
    );
}

#[test]
fn diff_report_display_empty_diff() {
    let report = DiffReport {
        old_version: "1.0.0".to_string(),
        new_version: "1.0.1".to_string(),
        changes: vec![],
    };

    let display = format!("{report}");
    assert!(
        display.contains("no changes") || display.contains("No changes") || display.contains("0"),
        "should indicate no changes"
    );
}

#[test]
fn diff_report_display_page_categorisation() {
    let report = DiffReport {
        old_version: "1.0.0".to_string(),
        new_version: "2.0.0".to_string(),
        changes: vec![
            PageChange::Added {
                filename: "new-page.md".to_string(),
            },
            PageChange::Removed {
                filename: "deprecated.md".to_string(),
            },
            PageChange::Modified {
                filename: "welcome.md".to_string(),
                paragraph_changes: vec![],
            },
        ],
    };

    let display = format!("{report}");
    assert!(display.contains("new-page.md"), "should list added page");
    assert!(
        display.contains("deprecated.md"),
        "should list removed page"
    );
    assert!(display.contains("welcome.md"), "should list modified page");
}

// --- JSON output format ---

#[test]
fn diff_report_json_serialization() {
    let report = DiffReport {
        old_version: "1.0.0".to_string(),
        new_version: "2.0.0".to_string(),
        changes: vec![PageChange::Added {
            filename: "services.md".to_string(),
        }],
    };

    let json = serde_json::to_string(&report).expect("should serialize");
    assert!(json.contains("1.0.0"), "should contain old version");
    assert!(json.contains("2.0.0"), "should contain new version");
    assert!(json.contains("services.md"), "should contain filename");
    assert!(json.contains("added"), "should contain change type");
}

#[test]
fn diff_report_json_roundtrip() {
    let report = DiffReport {
        old_version: "1.0.0".to_string(),
        new_version: "2.0.0".to_string(),
        changes: vec![
            PageChange::Added {
                filename: "a.md".to_string(),
            },
            PageChange::Removed {
                filename: "b.md".to_string(),
            },
            PageChange::Modified {
                filename: "c.md".to_string(),
                paragraph_changes: vec![],
            },
        ],
    };

    let json = serde_json::to_string(&report).expect("should serialize");
    let deserialized: DiffReport = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(report.old_version, deserialized.old_version);
    assert_eq!(report.new_version, deserialized.new_version);
    assert_eq!(report.changes.len(), deserialized.changes.len());
}

// --- Empty diff (no changes) ---

#[test]
fn diff_playbooks_empty_diff() {
    let old_dir = tempfile::tempdir().unwrap();
    let new_dir = tempfile::tempdir().unwrap();

    setup_version_dir(
        old_dir.path(),
        "1.0.0",
        &[("welcome.md", "# Welcome\n\nHello.\n")],
    );
    setup_version_dir(
        new_dir.path(),
        "1.0.1",
        &[("welcome.md", "# Welcome\n\nHello.\n")],
    );

    let report = diff_playbooks(old_dir.path(), new_dir.path()).expect("should diff");
    assert!(
        report.changes.is_empty(),
        "identical content should have no changes"
    );
}

// --- Page-level change categorisation via diff_playbooks ---

#[test]
fn diff_playbooks_categorises_added_pages() {
    let old_dir = tempfile::tempdir().unwrap();
    let new_dir = tempfile::tempdir().unwrap();

    setup_version_dir(
        old_dir.path(),
        "1.0.0",
        &[("welcome.md", "# Welcome\n\nHello.\n")],
    );
    setup_version_dir(
        new_dir.path(),
        "2.0.0",
        &[
            ("welcome.md", "# Welcome\n\nHello.\n"),
            ("services.md", "# Services\n\nOur services.\n"),
        ],
    );

    let report = diff_playbooks(old_dir.path(), new_dir.path()).expect("should diff");
    let added: Vec<_> = report
        .changes
        .iter()
        .filter_map(|c| {
            if let PageChange::Added { filename } = c {
                Some(filename.clone())
            } else {
                None
            }
        })
        .collect();
    assert!(
        added.contains(&"services.md".to_string()),
        "should categorise services.md as added"
    );
}

#[test]
fn diff_playbooks_categorises_removed_pages() {
    let old_dir = tempfile::tempdir().unwrap();
    let new_dir = tempfile::tempdir().unwrap();

    setup_version_dir(
        old_dir.path(),
        "1.0.0",
        &[
            ("welcome.md", "# Welcome\n\nHello.\n"),
            ("old-page.md", "# Old\n\nOld content.\n"),
        ],
    );
    setup_version_dir(
        new_dir.path(),
        "2.0.0",
        &[("welcome.md", "# Welcome\n\nHello.\n")],
    );

    let report = diff_playbooks(old_dir.path(), new_dir.path()).expect("should diff");
    let removed: Vec<_> = report
        .changes
        .iter()
        .filter_map(|c| {
            if let PageChange::Removed { filename } = c {
                Some(filename.clone())
            } else {
                None
            }
        })
        .collect();
    assert!(
        removed.contains(&"old-page.md".to_string()),
        "should categorise old-page.md as removed"
    );
}

#[test]
fn diff_playbooks_categorises_modified_pages() {
    let old_dir = tempfile::tempdir().unwrap();
    let new_dir = tempfile::tempdir().unwrap();

    setup_version_dir(
        old_dir.path(),
        "1.0.0",
        &[("welcome.md", "# Welcome\n\nOld text.\n")],
    );
    setup_version_dir(
        new_dir.path(),
        "2.0.0",
        &[("welcome.md", "# Welcome\n\nNew text.\n")],
    );

    let report = diff_playbooks(old_dir.path(), new_dir.path()).expect("should diff");
    let modified: Vec<_> = report
        .changes
        .iter()
        .filter_map(|c| {
            if let PageChange::Modified { filename, .. } = c {
                Some(filename.clone())
            } else {
                None
            }
        })
        .collect();
    assert!(
        modified.contains(&"welcome.md".to_string()),
        "should categorise welcome.md as modified"
    );
}
