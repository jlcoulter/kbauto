//! Tests for DiscoveryDocument parser.

use kbauto_template::{DiscoveryDocument, DiscoveryQuestion};
use std::path::Path;

#[test]
fn parse_discovery_with_qa_pairs() {
    let md = r#"## What is your firm's specialty?
We focus on small business accounting and tax preparation.

## How many staff do you have?
We have 12 accountants and 3 admin staff.

## What software do you use?
Xero, QuickBooks, and MYOB.
"#;
    let doc = DiscoveryDocument::from_markdown(md, "test").unwrap();
    assert_eq!(doc.len(), 3);
    assert_eq!(doc.questions[0].question, "What is your firm's specialty?");
    assert_eq!(
        doc.questions[0].answer,
        "We focus on small business accounting and tax preparation."
    );
    assert_eq!(doc.questions[1].question, "How many staff do you have?");
    assert_eq!(
        doc.questions[1].answer,
        "We have 12 accountants and 3 admin staff."
    );
    assert_eq!(doc.questions[2].question, "What software do you use?");
    assert_eq!(
        doc.questions[2].answer,
        "Xero, QuickBooks, and MYOB."
    );
}

#[test]
fn h3_headings_parsed_as_questions() {
    let md = "### Contact preference?\nEmail preferred.\n";
    let doc = DiscoveryDocument::from_markdown(md, "test").unwrap();
    assert_eq!(doc.len(), 1);
    assert_eq!(doc.questions[0].question, "Contact preference?");
    assert_eq!(doc.questions[0].answer, "Email preferred.");
}

#[test]
fn empty_answer_is_valid() {
    let md = "## Optional question?\n\n## Next question?\nSome answer.\n";
    let doc = DiscoveryDocument::from_markdown(md, "test").unwrap();
    assert_eq!(doc.len(), 2);
    assert_eq!(doc.questions[0].answer, "");
    assert_eq!(doc.questions[1].answer, "Some answer.");
}

#[test]
fn empty_discovery_returns_error() {
    let result = DiscoveryDocument::from_markdown("no headings here", "test");
    assert!(result.is_err());
}

#[test]
fn from_file_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("discovery.md");
    std::fs::write(
        &path,
        "## What is your specialty?\nTax and advisory.\n\n## How many staff?\n15\n",
    )
    .unwrap();

    let doc = DiscoveryDocument::from_markdown_file(&path).unwrap();
    assert_eq!(doc.len(), 2);
    assert_eq!(doc.questions[0].question, "What is your specialty?");
    assert_eq!(doc.questions[0].answer, "Tax and advisory.");
}

#[test]
fn missing_file_returns_error() {
    let result = DiscoveryDocument::from_markdown_file(Path::new("/nonexistent/discovery.md"));
    assert!(result.is_err());
}

#[test]
fn question_with_multiline_answer() {
    let md = "## Describe your services?
We offer:
- Accounting
- Tax preparation
- Advisory

## Anything else?
Nope.
";
    let doc = DiscoveryDocument::from_markdown(md, "test").unwrap();
    assert_eq!(doc.len(), 2);
    assert!(doc.questions[0].answer.contains("Accounting"));
    assert!(doc.questions[0].answer.contains("Advisory"));
    assert_eq!(doc.questions[1].answer, "Nope.");
}

#[test]
fn is_empty() {
    let md = "## Q?\nA.\n";
    let doc = DiscoveryDocument::from_markdown(md, "test").unwrap();
    assert!(!doc.is_empty());
}