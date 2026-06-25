//! T045: Conflict resolution contract tests.
//!
//! Tests for: Conflict, ConflictResolution, resolve_conflict
//! Covers KeepClient, KeepNewBase, ManualMerge resolutions,
//! frontmatter update after resolution.

use kbauto_rebase::{Conflict, ConflictResolution, resolve_conflict};

// --- KeepClient resolution ---

#[test]
fn resolve_conflict_keep_client() {
    let conflict = Conflict {
        filename: "welcome.md".to_string(),
        paragraph_index: 0,
        anchor: Some("welcome-to".to_string()),
        new_base_text: "Welcome to our new firm.\n".to_string(),
        client_text: "Welcome to Example Corp.\n".to_string(),
    };

    let content = "Welcome to Example Corp.\n\nContact us.\n";
    let result = resolve_conflict(content, &conflict, &ConflictResolution::KeepClient);
    assert!(result.is_ok(), "KeepClient should succeed: {:?}", result);
    // Should keep the client text unchanged
    assert!(
        result.unwrap().contains("Example Corp"),
        "should keep client text"
    );
}

#[test]
fn resolve_conflict_keep_client_preserves_other_paragraphs() {
    let conflict = Conflict {
        filename: "welcome.md".to_string(),
        paragraph_index: 0,
        anchor: Some("welcome-to".to_string()),
        new_base_text: "Welcome to our new firm.\n".to_string(),
        client_text: "Welcome to Example.\n".to_string(),
    };

    let content = "Welcome to old.\n\nSecond paragraph.\n\nThird paragraph.\n";
    let result = resolve_conflict(content, &conflict, &ConflictResolution::KeepClient);
    assert!(result.is_ok());
    let resolved = result.unwrap();
    assert!(resolved.contains("Example"), "should have client text");
    assert!(
        resolved.contains("Second paragraph"),
        "should preserve other paragraphs"
    );
    assert!(
        resolved.contains("Third paragraph"),
        "should preserve other paragraphs"
    );
}

// --- KeepNewBase resolution ---

#[test]
fn resolve_conflict_keep_new_base() {
    let conflict = Conflict {
        filename: "welcome.md".to_string(),
        paragraph_index: 0,
        anchor: Some("welcome-to".to_string()),
        new_base_text: "Welcome to our updated firm.\n".to_string(),
        client_text: "Welcome to Example.\n".to_string(),
    };

    let content = "Welcome to Example.\n\nContact us.\n";
    let result = resolve_conflict(content, &conflict, &ConflictResolution::KeepNewBase);
    assert!(result.is_ok(), "KeepNewBase should succeed");
    let resolved = result.unwrap();
    assert!(
        resolved.contains("updated firm"),
        "should have new base text"
    );
    assert!(!resolved.contains("Example"), "should not have client text");
}

// --- ManualMerge resolution ---

#[test]
fn resolve_conflict_manual_merge() {
    let conflict = Conflict {
        filename: "welcome.md".to_string(),
        paragraph_index: 1,
        anchor: Some("contact-us".to_string()),
        new_base_text: "Contact us at info@new.com.\n".to_string(),
        client_text: "Contact us at hello@example.com.\n".to_string(),
    };

    let merged_text = "Contact us at hello@example.com or info@new.com.\n".to_string();
    let content = "Welcome.\n\nContact us at hello@example.com.\n\nMore info.\n";

    let result = resolve_conflict(
        content,
        &conflict,
        &ConflictResolution::ManualMerge {
            merged_text: merged_text.clone(),
        },
    );
    assert!(result.is_ok(), "ManualMerge should succeed");
    let resolved = result.unwrap();
    assert!(
        resolved.contains(&merged_text.trim()),
        "should contain merged text"
    );
    assert!(
        resolved.contains("Welcome"),
        "should preserve surrounding paragraphs"
    );
}

// --- Conflict at different indices ---

#[test]
fn resolve_conflict_at_last_paragraph() {
    let conflict = Conflict {
        filename: "welcome.md".to_string(),
        paragraph_index: 2,
        anchor: None,
        new_base_text: "New closing.\n".to_string(),
        client_text: "Old closing.\n".to_string(),
    };

    let content = "First.\n\nSecond.\n\nOld closing.\n";
    let result = resolve_conflict(content, &conflict, &ConflictResolution::KeepNewBase);
    assert!(result.is_ok());
    assert!(
        result.unwrap().contains("New closing"),
        "should replace last paragraph"
    );
}

// --- Error on invalid paragraph index ---

#[test]
fn resolve_conflict_out_of_bounds_index() {
    let conflict = Conflict {
        filename: "welcome.md".to_string(),
        paragraph_index: 99, // out of bounds
        anchor: None,
        new_base_text: "Should not appear.\n".to_string(),
        client_text: "Also should not appear.\n".to_string(),
    };

    let content = "Only one paragraph.\n";
    let result = resolve_conflict(content, &conflict, &ConflictResolution::KeepNewBase);
    // Should either error or return original content unchanged
    // Implementation choice: return error for out-of-bounds
    assert!(
        result.is_err() || result.unwrap().contains("Only one paragraph"),
        "out of bounds should error or leave content unchanged"
    );
}

// --- Conflict serialization ---

#[test]
fn conflict_serialization_roundtrip() {
    let conflict = Conflict {
        filename: "welcome.md".to_string(),
        paragraph_index: 0,
        anchor: Some("welcome-to".to_string()),
        new_base_text: "New base.\n".to_string(),
        client_text: "Client text.\n".to_string(),
    };

    let json = serde_json::to_string(&conflict).expect("should serialize");
    let deserialized: Conflict = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(conflict, deserialized, "roundtrip should be equal");
}

#[test]
fn conflict_resolution_serialization_roundtrip() {
    let resolution = ConflictResolution::ManualMerge {
        merged_text: "Combined text.".to_string(),
    };

    let json = serde_json::to_string(&resolution).expect("should serialize");
    let deserialized: ConflictResolution = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(resolution, deserialized, "roundtrip should be equal");
}
