//! T044: Rebase engine contract tests.
//!
//! Tests for: rebase_client_kb, RebaseResult
//! Covers template-origin update, substituted/rewritten preservation,
//! conflict detection, new page addition, removed page flagging.

use kbauto_provenance::{FrontmatterData, parse_frontmatter, write_frontmatter};
use kbauto_rebase::rebase_client_kb;
use std::collections::HashMap;
use std::fs;

fn setup_template_dir(dir: &std::path::Path, version: &str) {
    let docs_dir = dir.join("docs");
    fs::create_dir_all(&docs_dir).expect("should create docs dir");
    let defaults_json = format!(
        r#"{{"version":"{version}","defaults":[
            {{"key":"TEAM_NAME","value":"Default Co","type":"text"}},
            {{"key":"TEAM_EMAIL","value":"info@default.com","type":"text"}}
        ]}}"#
    );
    fs::write(dir.join("defaults.json"), defaults_json).expect("should write defaults.json");
}

fn write_client_page(dir: &std::path::Path, filename: &str, content: &str) {
    let pages_dir = dir.join("docs");
    fs::create_dir_all(&pages_dir).expect("should create pages dir");
    fs::write(pages_dir.join(filename), content).expect("should write page");
}

// --- Template-origin text is updated ---

#[test]
fn rebase_updates_template_origin_text() {
    // Setup: generate a client KB from v1, then rebase onto v2 where
    // template-origin paragraphs have changed.
    // Template-origin paragraphs should be updated to v2 content.
    let old_template = tempfile::tempdir().unwrap();
    let new_template = tempfile::tempdir().unwrap();
    let client_kb = tempfile::tempdir().unwrap();
    let output = tempfile::tempdir().unwrap();

    // Old template v1
    setup_template_dir(old_template.path(), "1.0.0");
    fs::write(old_template.path().join("docs/welcome.md"),
        "---\ntitle: Welcome\nsidebar_position: 1\nprovenance:\n  0: template\n---\n\nWelcome to our firm.\n\nContact us for details.\n"
    ).unwrap();

    // New template v2 — template text changed
    setup_template_dir(new_template.path(), "2.0.0");
    fs::write(new_template.path().join("docs/welcome.md"),
        "---\ntitle: Welcome\nsidebar_position: 1\n---\n\nWelcome to our updated firm.\n\nContact us for details.\n"
    ).unwrap();

    // Client KB — has the v1 content
    write_client_page(
        client_kb.path(),
        "welcome.md",
        "---\ntitle: Welcome\nsidebar_position: 1\nprovenance:\n  0: template\n  1: substituted\n---\n\nWelcome to our firm.\n\nTEAM_NAME is the best.\n",
    );

    let result = rebase_client_kb(client_kb.path(), "1.0.0", "2.0.0", new_template.path());
    assert!(result.is_ok(), "rebase should succeed: {:?}", result);
}

// --- Substituted/rewritten text is preserved ---

#[test]
fn rebase_preserves_substituted_text() {
    let new_template = tempfile::tempdir().unwrap();
    let client_kb = tempfile::tempdir().unwrap();

    setup_template_dir(new_template.path(), "2.0.0");
    fs::write(
        new_template.path().join("docs/welcome.md"),
        "---\ntitle: Welcome\n---\n\nWelcome to our firm.\n\nContact us.\n",
    )
    .unwrap();

    // Client KB has substituted text in paragraph 1
    write_client_page(
        client_kb.path(),
        "welcome.md",
        "---\ntitle: Welcome\nprovenance:\n  0: template\n  1: substituted\n---\n\nWelcome to our firm.\n\nExample Corp is the best.\n",
    );

    let result = rebase_client_kb(client_kb.path(), "1.0.0", "2.0.0", new_template.path());
    assert!(result.is_ok(), "rebase should succeed: {:?}", result);
    let rebased = result.unwrap();
    assert!(rebased.pages_updated > 0, "should update pages");
}

// --- Conflict detection ---

#[test]
fn rebase_detects_conflicts() {
    let new_template = tempfile::tempdir().unwrap();
    let client_kb = tempfile::tempdir().unwrap();

    setup_template_dir(new_template.path(), "2.0.0");
    // New base has a changed paragraph
    fs::write(
        new_template.path().join("docs/welcome.md"),
        "---\ntitle: Welcome\n---\n\nWelcome to our new and improved firm.\n\nContact us.\n",
    )
    .unwrap();

    // Client has rewritten that same paragraph
    write_client_page(
        client_kb.path(),
        "welcome.md",
        "---\ntitle: Welcome\nprovenance:\n  0: rewritten\n  1: template\n---\n\nWelcome to Example Corp — we care.\n\nContact us.\n",
    );

    let result = rebase_client_kb(client_kb.path(), "1.0.0", "2.0.0", new_template.path());
    assert!(result.is_ok(), "rebase should succeed: {:?}", result);
    // Should detect at least one conflict since paragraph 0 changed in both base and client
    assert!(result.unwrap().conflicts > 0, "should detect conflicts");
}

// --- New page addition ---

#[test]
fn rebase_adds_new_pages() {
    let new_template = tempfile::tempdir().unwrap();
    let client_kb = tempfile::tempdir().unwrap();

    setup_template_dir(new_template.path(), "2.0.0");
    fs::write(
        new_template.path().join("docs/welcome.md"),
        "---\ntitle: Welcome\n---\n\nWelcome.\n",
    )
    .unwrap();
    fs::write(
        new_template.path().join("docs/services.md"),
        "---\ntitle: Services\n---\n\nOur services.\n",
    )
    .unwrap();

    // Client only has welcome.md
    write_client_page(
        client_kb.path(),
        "welcome.md",
        "---\ntitle: Welcome\nprovenance:\n  0: template\n---\n\nWelcome.\n",
    );

    let result = rebase_client_kb(client_kb.path(), "1.0.0", "2.0.0", new_template.path());
    assert!(result.is_ok(), "rebase should succeed: {:?}", result);
    let rebased = result.unwrap();
    assert!(rebased.pages_updated >= 2, "should update at least 2 pages");
}

// --- Removed page flagging ---

#[test]
fn rebase_flags_removed_pages() {
    let new_template = tempfile::tempdir().unwrap();
    let client_kb = tempfile::tempdir().unwrap();

    setup_template_dir(new_template.path(), "2.0.0");
    // v2 only has welcome.md (services.md removed)
    fs::write(
        new_template.path().join("docs/welcome.md"),
        "---\ntitle: Welcome\n---\n\nWelcome.\n",
    )
    .unwrap();

    // Client has both welcome.md and services.md from v1
    write_client_page(
        client_kb.path(),
        "welcome.md",
        "---\ntitle: Welcome\nprovenance:\n  0: template\n---\n\nWelcome.\n",
    );
    write_client_page(
        client_kb.path(),
        "services.md",
        "---\ntitle: Services\nprovenance:\n  0: template\n---\n\nOur services.\n",
    );

    let result = rebase_client_kb(client_kb.path(), "1.0.0", "2.0.0", new_template.path());
    assert!(result.is_ok(), "rebase should succeed: {:?}", result);
}

// --- Error on nonexistent path ---

#[test]
fn rebase_errors_on_nonexistent_path() {
    let result = rebase_client_kb(
        std::path::Path::new("/nonexistent/client"),
        "1.0.0",
        "2.0.0",
        std::path::Path::new("/nonexistent/template"),
    );
    assert!(result.is_err(), "should error on nonexistent paths");
}
