//! Tests for OllamaRewriter::rewrite_section() and helper functions.
//!
//! These tests verify code-fence stripping, prompt building, and retry logic.
//! Actual Ollama calls are not made in unit tests (that would require a running
//! server and is covered by integration tests).

use kbauto_customise::ollama::OllamaRewriter;
use kbauto_customise::retry::RetryConfig;

#[test]
fn strip_code_fences_markdown() {
    let input = "```markdown\n# Hello\nWorld\n```";
    let result = OllamaRewriter::strip_code_fences(input);
    assert_eq!(result, "# Hello\nWorld");
}

#[test]
fn strip_code_fences_md() {
    let input = "```md\nSome text\n```";
    let result = OllamaRewriter::strip_code_fences(input);
    assert_eq!(result, "Some text");
}

#[test]
fn strip_code_fences_plain() {
    let input = "```\nPlain code block\n```";
    let result = OllamaRewriter::strip_code_fences(input);
    assert_eq!(result, "Plain code block");
}

#[test]
fn strip_code_fences_no_fence() {
    let input = "Just plain text";
    let result = OllamaRewriter::strip_code_fences(input);
    assert_eq!(result, "Just plain text");
}

#[test]
fn strip_code_fences_nested_content() {
    let input = "```markdown\n## Heading\n\nParagraph\n\n- List item\n```";
    let result = OllamaRewriter::strip_code_fences(input);
    assert!(result.contains("## Heading"));
    assert!(result.contains("- List item"));
}

#[test]
fn retry_config_default() {
    let config = RetryConfig::default();
    assert_eq!(config.max_retries, 3);
}

#[test]
fn ollama_rewriter_new() {
    let rewriter = OllamaRewriter::new(
        "deepseek-v4-flash:cloud".to_string(),
        "http://localhost:11434".to_string(),
        5,
    );
    assert_eq!(rewriter.model, "deepseek-v4-flash:cloud");
    assert_eq!(rewriter.url, "http://localhost:11434");
    assert_eq!(rewriter.max_retries, 5);
}

#[test]
fn build_prompt_contains_section_name() {
    let prompt = OllamaRewriter::build_prompt(
        "Welcome",
        "Welcome to our firm",
        "Example Corp is a mid-size firm",
    );
    assert!(prompt.contains("Welcome"));
    assert!(prompt.contains("Welcome to our firm"));
    assert!(prompt.contains("Example Corp is a mid-size firm"));
}
