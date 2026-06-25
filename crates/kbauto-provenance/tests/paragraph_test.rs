//! T021: Paragraph splitting contract tests
//!
//! Tests for: split_paragraphs, compute_anchor, Paragraph
//! Covers paragraph boundary detection (blank lines), anchor computation
//! (first N words lowercased hyphen-joined), None for empty/whitespace
//! paragraphs, and preserving original text.

use kbauto_provenance::{compute_anchor, split_paragraphs};

// --- Detect paragraph boundaries (blank lines) ---

#[test]
fn split_simple_paragraphs() {
    let content = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
    let paras = split_paragraphs(content);
    assert_eq!(paras.len(), 3, "should split into 3 paragraphs");
    assert_eq!(paras[0].index, 0);
    assert_eq!(paras[1].index, 1);
    assert_eq!(paras[2].index, 2);
}

#[test]
fn split_single_paragraph() {
    let content = "Just one paragraph with no blank lines.";
    let paras = split_paragraphs(content);
    assert_eq!(paras.len(), 1, "should have 1 paragraph");
    assert_eq!(paras[0].index, 0);
}

#[test]
fn split_handles_multiple_blank_lines() {
    let content = "First para.\n\n\n\nSecond para.";
    let paras = split_paragraphs(content);
    assert_eq!(
        paras.len(),
        2,
        "multiple blank lines should still be 2 paragraphs"
    );
}

#[test]
fn split_empty_content() {
    let content = "";
    let paras = split_paragraphs(content);
    assert_eq!(paras.len(), 0, "empty content should have 0 paragraphs");
}

#[test]
fn split_whitespace_only_content() {
    let content = "   \n   \n   ";
    let paras = split_paragraphs(content);
    assert_eq!(
        paras.len(),
        0,
        "whitespace-only content should have 0 paragraphs"
    );
}

#[test]
fn split_leading_trailing_blank_lines() {
    let content = "\n\nFirst para.\n\nSecond para.\n\n";
    let paras = split_paragraphs(content);
    assert_eq!(
        paras.len(),
        2,
        "leading/trailing blank lines should be ignored"
    );
}

// --- Anchor computation ---

#[test]
fn compute_anchor_from_words() {
    // First 5 words, lowercased, hyphen-joined
    let anchor = compute_anchor("Welcome to Our Accounting Services Page");
    assert_eq!(
        anchor,
        Some("welcome-to-our-accounting-services".to_string())
    );
}

#[test]
fn compute_anchor_fewer_than_five_words() {
    let anchor = compute_anchor("Short Title");
    assert_eq!(anchor, Some("short-title".to_string()));
}

#[test]
fn compute_anchor_single_word() {
    let anchor = compute_anchor("Introduction");
    assert_eq!(anchor, Some("introduction".to_string()));
}

#[test]
fn compute_anchor_returns_none_for_empty() {
    let anchor = compute_anchor("");
    assert_eq!(anchor, None, "empty string should return None");
}

#[test]
fn compute_anchor_returns_none_for_whitespace() {
    let anchor = compute_anchor("   \t  \n  ");
    assert_eq!(anchor, None, "whitespace-only string should return None");
}

#[test]
fn compute_anchor_lowercases_and_hyphen_joins() {
    let anchor = compute_anchor("The Quick Brown Fox Jumps Over");
    assert_eq!(anchor, Some("the-quick-brown-fox-jumps".to_string()));
}

#[test]
fn compute_anchor_strips_punctuation() {
    let anchor = compute_anchor("Welcome! To: Our; Services.");
    // Punctuation should be stripped from anchor computation
    assert!(anchor.is_some(), "should produce an anchor");
    let anchor_str = anchor.unwrap();
    assert!(
        !anchor_str.contains('!'),
        "anchor should not contain exclamation"
    );
    assert!(!anchor_str.contains(':'), "anchor should not contain colon");
    assert!(
        !anchor_str.contains(';'),
        "anchor should not contain semicolon"
    );
}

// --- Preserve original text ---

#[test]
fn paragraph_preserves_original_text() {
    let content = "Hello world.\n\nGoodbye world.";
    let paras = split_paragraphs(content);
    assert_eq!(paras.len(), 2);
    assert_eq!(
        paras[0].text, "Hello world.",
        "first paragraph text should be preserved"
    );
    assert_eq!(
        paras[1].text, "Goodbye world.",
        "second paragraph text should be preserved"
    );
}

#[test]
fn paragraph_preserves_multiline_text() {
    let content = "Line one\nLine two\n\nNext para.";
    let paras = split_paragraphs(content);
    assert_eq!(paras.len(), 2);
    assert!(
        paras[0].text.contains("Line one"),
        "should preserve line one"
    );
    assert!(
        paras[0].text.contains("Line two"),
        "should preserve line two"
    );
}

#[test]
fn paragraph_anchor_is_computed() {
    let content = "Welcome to Our Services Page\n\nContact Us Today";
    let paras = split_paragraphs(content);
    assert_eq!(paras.len(), 2);
    // First paragraph: "Welcome to Our Services Page" → anchor
    assert!(
        paras[0].anchor.is_some(),
        "first paragraph should have an anchor"
    );
    // Second paragraph: "Contact Us Today" → anchor
    assert!(
        paras[1].anchor.is_some(),
        "second paragraph should have an anchor"
    );
}

#[test]
fn paragraph_indices_are_zero_based() {
    let content = "Para one.\n\nPara two.\n\nPara three.";
    let paras = split_paragraphs(content);
    for (i, para) in paras.iter().enumerate() {
        assert_eq!(para.index, i, "paragraph index should be zero-based");
    }
}

#[test]
fn code_block_treated_as_single_paragraph() {
    let content = "Intro text.\n\n```\ncode line one\ncode line two\n```\n\nAfter code.";
    let paras = split_paragraphs(content);
    // The fenced code block should be treated as a single paragraph
    assert!(
        paras.len() >= 3,
        "should have at least 3 paragraphs (intro, code block, after)"
    );
    // Find the code block paragraph
    let code_para = paras.iter().find(|p| p.text.contains("code line one"));
    assert!(
        code_para.is_some(),
        "code block should be a single paragraph"
    );
    assert!(
        code_para.unwrap().text.contains("code line two"),
        "code block should include all lines"
    );
}
