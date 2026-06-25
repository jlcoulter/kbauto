//! Tests for ProvenanceClassification enum.

use kbauto_provenance::ProvenanceClassification;

#[test]
fn template_serializes() {
    let json = serde_json::to_string(&ProvenanceClassification::Template).expect("serialize");
    assert_eq!(json, "\"template\"");
}

#[test]
fn substituted_serializes() {
    let json = serde_json::to_string(&ProvenanceClassification::Substituted).expect("serialize");
    assert_eq!(json, "\"substituted\"");
}

#[test]
fn rewritten_serializes() {
    let json = serde_json::to_string(&ProvenanceClassification::Rewritten).expect("serialize");
    assert_eq!(json, "\"rewritten\"");
}

#[test]
fn classification_round_trip() {
    for c in [
        ProvenanceClassification::Template,
        ProvenanceClassification::Substituted,
        ProvenanceClassification::Rewritten,
    ] {
        let json = serde_json::to_string(&c).expect("serialize");
        let back: ProvenanceClassification = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(c, back, "round-trip for {json}");
    }
}

#[test]
fn display_template() {
    assert_eq!(
        format!("{}", ProvenanceClassification::Template),
        "template"
    );
}

#[test]
fn display_substituted() {
    assert_eq!(
        format!("{}", ProvenanceClassification::Substituted),
        "substituted"
    );
}

#[test]
fn display_rewritten() {
    assert_eq!(
        format!("{}", ProvenanceClassification::Rewritten),
        "rewritten"
    );
}

#[test]
fn ordering_rewritten_greater_than_substituted() {
    assert!(ProvenanceClassification::Rewritten > ProvenanceClassification::Substituted);
}

#[test]
fn ordering_substituted_greater_than_template() {
    assert!(ProvenanceClassification::Substituted > ProvenanceClassification::Template);
}

#[test]
fn ordering_rewritten_greater_than_template() {
    assert!(ProvenanceClassification::Rewritten > ProvenanceClassification::Template);
}
