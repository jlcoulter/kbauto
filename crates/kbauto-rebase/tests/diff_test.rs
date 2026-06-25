//! T043: Diff engine contract tests.
//!
//! Tests for: diff_pages, diff_playbooks, PageChange, ParagraphChange, DiffReport
//! Covers paragraph-level diff, page-level diff, identical pages, added/removed pages.

use kbauto_rebase::{DiffReport, PageChange, diff_pages, diff_playbooks};
use std::fs;

fn setup_version_dir(dir: &std::path::Path, files: &[(&str, &str)]) {
    fs::create_dir_all(dir).expect("should create dir");
    for (name, content) in files {
        fs::write(dir.join(name), content).expect("should write file");
    }
}

// --- diff_pages: paragraph-level diff ---

#[test]
fn diff_pages_identical_content() {
    let content = "# Welcome\n\nThis is paragraph one.\n\nThis is paragraph two.\n";
    let changes = diff_pages(content, content);
    assert!(
        changes.is_empty(),
        "identical content should have no changes"
    );
}

#[test]
fn diff_pages_single_paragraph_changed() {
    let old = "# Welcome\n\nOld paragraph one.\n\nParagraph two.\n";
    let new = "# Welcome\n\nNew paragraph one.\n\nParagraph two.\n";
    let changes = diff_pages(old, new);
    assert_eq!(changes.len(), 1, "should detect one changed paragraph");
    assert_eq!(changes[0].index, 0, "first paragraph changed");
    assert!(changes[0].old_text.contains("Old"));
    assert!(changes[0].new_text.contains("New"));
}

#[test]
fn diff_pages_multiple_paragraphs_changed() {
    let old = "# Title\n\nParagraph A.\n\nParagraph B.\n\nParagraph C.\n";
    let new = "# Title\n\nParagraph X.\n\nParagraph B.\n\nParagraph Y.\n";
    let changes = diff_pages(old, new);
    // Paragraph B is unchanged, A→X and C→Y are changed
    assert_eq!(changes.len(), 2, "should detect two changed paragraphs");
    let indices: Vec<usize> = changes.iter().map(|c| c.index).collect();
    assert!(indices.contains(&0), "paragraph 0 should change");
    assert!(indices.contains(&2), "paragraph 2 should change");
    assert!(!indices.contains(&1), "paragraph 1 should be unchanged");
}

#[test]
fn diff_pages_anchor_matching() {
    // Even if paragraphs move, they should match by anchor (first N words)
    let old = "# Welcome\n\nContact us at old@email.com for info.\n\nWe provide services.\n";
    let new = "# Welcome\n\nContact us at new@email.com for info.\n\nWe provide services.\n";
    let changes = diff_pages(old, new);
    assert_eq!(
        changes.len(),
        1,
        "should match by anchor and find one change"
    );
    assert_eq!(changes[0].index, 0);
}

#[test]
fn diff_pages_all_new_paragraphs() {
    let old = "# Title\n\nOld content here.\n";
    let new = "# Title\n\nCompletely different text.\n\nAdditional paragraph.\n";
    let changes = diff_pages(old, new);
    // Should detect changes — old has 1 content paragraph, new has 2
    assert!(!changes.is_empty(), "should detect differences");
}

// --- diff_playbooks: page-level diff ---

#[test]
fn diff_playbooks_identical_directories() {
    let old_dir = tempfile::tempdir().unwrap();
    let new_dir = tempfile::tempdir().unwrap();

    setup_version_dir(
        old_dir.path().join("docs").as_path(),
        &[
            ("welcome.md", "# Welcome\n\nHello world.\n"),
            ("services.md", "# Services\n\nWe offer services.\n"),
        ],
    );
    // Create defaults.json so it's a valid template dir
    fs::write(
        old_dir.path().join("defaults.json"),
        r#"{"version":"1.0.0","defaults":[]}"#,
    )
    .unwrap();

    setup_version_dir(
        new_dir.path().join("docs").as_path(),
        &[
            ("welcome.md", "# Welcome\n\nHello world.\n"),
            ("services.md", "# Services\n\nWe offer services.\n"),
        ],
    );
    fs::write(
        new_dir.path().join("defaults.json"),
        r#"{"version":"1.0.0","defaults":[]}"#,
    )
    .unwrap();

    let report = diff_playbooks(old_dir.path(), new_dir.path()).expect("should diff");
    assert!(
        report.changes.is_empty(),
        "identical dirs should have no changes"
    );
}

#[test]
fn diff_playbooks_added_page() {
    let old_dir = tempfile::tempdir().unwrap();
    let new_dir = tempfile::tempdir().unwrap();

    setup_version_dir(
        old_dir.path().join("docs").as_path(),
        &[("welcome.md", "# Welcome\n\nHello.\n")],
    );
    fs::write(
        old_dir.path().join("defaults.json"),
        r#"{"version":"1.0.0","defaults":[]}"#,
    )
    .unwrap();

    setup_version_dir(
        new_dir.path().join("docs").as_path(),
        &[
            ("welcome.md", "# Welcome\n\nHello.\n"),
            ("services.md", "# Services\n\nWe offer services.\n"),
        ],
    );
    fs::write(
        new_dir.path().join("defaults.json"),
        r#"{"version":"1.0.0","defaults":[]}"#,
    )
    .unwrap();

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
        "services.md should be added"
    );
}

#[test]
fn diff_playbooks_removed_page() {
    let old_dir = tempfile::tempdir().unwrap();
    let new_dir = tempfile::tempdir().unwrap();

    setup_version_dir(
        old_dir.path().join("docs").as_path(),
        &[
            ("welcome.md", "# Welcome\n\nHello.\n"),
            ("services.md", "# Services\n\nWe offer services.\n"),
        ],
    );
    fs::write(
        old_dir.path().join("defaults.json"),
        r#"{"version":"1.0.0","defaults":[]}"#,
    )
    .unwrap();

    setup_version_dir(
        new_dir.path().join("docs").as_path(),
        &[("welcome.md", "# Welcome\n\nHello.\n")],
    );
    fs::write(
        new_dir.path().join("defaults.json"),
        r#"{"version":"1.0.0","defaults":[]}"#,
    )
    .unwrap();

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
        removed.contains(&"services.md".to_string()),
        "services.md should be removed"
    );
}

#[test]
fn diff_playbooks_modified_page() {
    let old_dir = tempfile::tempdir().unwrap();
    let new_dir = tempfile::tempdir().unwrap();

    setup_version_dir(
        old_dir.path().join("docs").as_path(),
        &[("welcome.md", "# Welcome\n\nOld content.\n")],
    );
    fs::write(
        old_dir.path().join("defaults.json"),
        r#"{"version":"1.0.0","defaults":[]}"#,
    )
    .unwrap();

    setup_version_dir(
        new_dir.path().join("docs").as_path(),
        &[("welcome.md", "# Welcome\n\nNew content.\n")],
    );
    fs::write(
        new_dir.path().join("defaults.json"),
        r#"{"version":"1.0.0","defaults":[]}"#,
    )
    .unwrap();

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
        "welcome.md should be modified"
    );
}

#[test]
fn diff_playbooks_report_contains_versions() {
    let old_dir = tempfile::tempdir().unwrap();
    let new_dir = tempfile::tempdir().unwrap();

    setup_version_dir(
        old_dir.path().join("docs").as_path(),
        &[("welcome.md", "# Welcome\n\nHello.\n")],
    );
    fs::write(
        old_dir.path().join("defaults.json"),
        r#"{"version":"1.0.0","defaults":[]}"#,
    )
    .unwrap();

    setup_version_dir(
        new_dir.path().join("docs").as_path(),
        &[("welcome.md", "# Welcome\n\nHello.\n")],
    );
    fs::write(
        new_dir.path().join("defaults.json"),
        r#"{"version":"2.0.0","defaults":[]}"#,
    )
    .unwrap();

    let report = diff_playbooks(old_dir.path(), new_dir.path()).expect("should diff");
    assert_eq!(report.old_version, "1.0.0");
    assert_eq!(report.new_version, "2.0.0");
}
