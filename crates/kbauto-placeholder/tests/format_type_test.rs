//! Tests for PlaceholderFormat and PlaceholderType enums.

use kbauto_placeholder::{PlaceholderFormat, PlaceholderType};

// --- PlaceholderFormat tests ---

#[test]
fn format_bare_serializes_to_snake_case() {
    let json = serde_json::to_string(&PlaceholderFormat::Bare).expect("serialize");
    assert_eq!(json, "\"bare\"");
}

#[test]
fn format_angle_bracket_serializes() {
    let json = serde_json::to_string(&PlaceholderFormat::AngleBracket).expect("serialize");
    assert_eq!(json, "\"angle_bracket\"");
}

#[test]
fn format_mustache_serializes() {
    let json = serde_json::to_string(&PlaceholderFormat::Mustache).expect("serialize");
    assert_eq!(json, "\"mustache\"");
}

#[test]
fn format_round_trip() {
    for fmt in [
        PlaceholderFormat::Bare,
        PlaceholderFormat::AngleBracket,
        PlaceholderFormat::Mustache,
    ] {
        let json = serde_json::to_string(&fmt).expect("serialize");
        let back: PlaceholderFormat = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(fmt, back, "round-trip for {json}");
    }
}

#[test]
fn format_debug_output() {
    assert!(format!("{:?}", PlaceholderFormat::Bare).contains("Bare"));
}

// --- PlaceholderType tests ---

#[test]
fn type_text_serializes() {
    let json = serde_json::to_string(&PlaceholderType::Text).expect("serialize");
    assert_eq!(json, "\"text\"");
}

#[test]
fn type_long_text_serializes() {
    let json = serde_json::to_string(&PlaceholderType::LongText).expect("serialize");
    assert_eq!(json, "\"long_text\"");
}

#[test]
fn type_number_serializes() {
    let json = serde_json::to_string(&PlaceholderType::Number).expect("serialize");
    assert_eq!(json, "\"number\"");
}

#[test]
fn type_boolean_serializes() {
    let json = serde_json::to_string(&PlaceholderType::Boolean).expect("serialize");
    assert_eq!(json, "\"boolean\"");
}

#[test]
fn type_list_serializes() {
    let json = serde_json::to_string(&PlaceholderType::List).expect("serialize");
    assert_eq!(json, "\"list\"");
}

#[test]
fn type_date_serializes() {
    let json = serde_json::to_string(&PlaceholderType::Date).expect("serialize");
    assert_eq!(json, "\"date\"");
}

#[test]
fn type_round_trip() {
    for t in [
        PlaceholderType::Text,
        PlaceholderType::LongText,
        PlaceholderType::Number,
        PlaceholderType::Boolean,
        PlaceholderType::List,
        PlaceholderType::Date,
    ] {
        let json = serde_json::to_string(&t).expect("serialize");
        let back: PlaceholderType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(t, back, "round-trip for {json}");
    }
}
