//! Tests for TUI missing-value data model.
//!
//! These are pure data-structure tests, not terminal rendering tests.

use kbauto_template::{MissingValue, MissingValueForm};
use std::collections::HashMap;

#[test]
fn missing_value_fields() {
    let mv = MissingValue {
        key: "TEAM_NAME".to_string(),
        description: "The name of the team".to_string(),
        default: Some("Default Team".to_string()),
    };
    assert_eq!(mv.key, "TEAM_NAME");
    assert_eq!(mv.description, "The name of the team");
    assert_eq!(mv.default, Some("Default Team".to_string()));
}

#[test]
fn missing_value_without_default() {
    let mv = MissingValue {
        key: "CLIENT_INDUSTRY".to_string(),
        description: "Industry vertical".to_string(),
        default: None,
    };
    assert!(mv.default.is_none());
}

#[test]
fn form_starts_empty() {
    let form = MissingValueForm::new(vec![]);
    assert!(form.missing.is_empty());
    assert!(form.values.is_empty());
}

#[test]
fn form_with_missing_values() {
    let missing = vec![
        MissingValue {
            key: "TEAM_NAME".to_string(),
            description: "Team name".to_string(),
            default: Some("Default Team".to_string()),
        },
        MissingValue {
            key: "CLIENT_INDUSTRY".to_string(),
            description: "Industry".to_string(),
            default: None,
        },
    ];
    let form = MissingValueForm::new(missing);
    assert_eq!(form.missing.len(), 2);
    assert_eq!(form.missing[0].key, "TEAM_NAME");
    assert_eq!(form.missing[1].key, "CLIENT_INDUSTRY");
}

#[test]
fn form_set_value() {
    let missing = vec![MissingValue {
        key: "TEAM_NAME".to_string(),
        description: "Team name".to_string(),
        default: Some("Default Team".to_string()),
    }];
    let mut form = MissingValueForm::new(missing);
    form.set_value("TEAM_NAME", "Example Corp");
    assert_eq!(
        form.values.get("TEAM_NAME"),
        Some(&"Example Corp".to_string())
    );
}

#[test]
fn form_set_value_overwrites() {
    let missing = vec![MissingValue {
        key: "TEAM_NAME".to_string(),
        description: "Team name".to_string(),
        default: Some("Default Team".to_string()),
    }];
    let mut form = MissingValueForm::new(missing);
    form.set_value("TEAM_NAME", "First");
    form.set_value("TEAM_NAME", "Second");
    assert_eq!(form.values.get("TEAM_NAME"), Some(&"Second".to_string()));
}

#[test]
fn form_collect_defaults_when_no_user_value() {
    let missing = vec![
        MissingValue {
            key: "TEAM_NAME".to_string(),
            description: "Team name".to_string(),
            default: Some("Default Team".to_string()),
        },
        MissingValue {
            key: "CLIENT_INDUSTRY".to_string(),
            description: "Industry".to_string(),
            default: None,
        },
    ];
    let form = MissingValueForm::new(missing);
    // resolve merges default values where no user value is set
    let resolved = form.resolve();
    assert_eq!(resolved.get("TEAM_NAME"), Some(&"Default Team".to_string()));
    assert!(!resolved.contains_key("CLIENT_INDUSTRY"));
}

#[test]
fn form_user_value_overrides_default() {
    let missing = vec![MissingValue {
        key: "TEAM_NAME".to_string(),
        description: "Team name".to_string(),
        default: Some("Default Team".to_string()),
    }];
    let mut form = MissingValueForm::new(missing);
    form.set_value("TEAM_NAME", "Example Corp");
    let resolved = form.resolve();
    assert_eq!(resolved.get("TEAM_NAME"), Some(&"Example Corp".to_string()));
}

#[test]
fn form_empty_missing_returns_empty_map() {
    let form = MissingValueForm::new(vec![]);
    let resolved = form.resolve();
    assert!(resolved.is_empty());
}
