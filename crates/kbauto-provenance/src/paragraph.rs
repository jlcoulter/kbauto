//! Paragraph splitting and anchor computation.

use serde::{Deserialize, Serialize};

/// A paragraph extracted from markdown content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Paragraph {
    /// 0-based index of this paragraph in the document.
    pub index: usize,
    /// The paragraph text content.
    pub text: String,
    /// Anchor for matching across versions: first N words lowercased and hyphen-joined.
    pub anchor: Option<String>,
}

/// Split markdown content into paragraphs.
///
/// Paragraphs are separated by one or more blank lines.
/// Code blocks (fenced with ```) are treated as single paragraphs.
///
/// # Arguments
///
/// * `content` - The markdown content to split
///
/// # Returns
///
/// A vector of paragraphs with their indices and computed anchors.
#[must_use = "split paragraphs should be used"]
pub fn split_paragraphs(content: &str) -> Vec<Paragraph> {
    if content.trim().is_empty() {
        return Vec::new();
    }

    let mut paragraphs = Vec::new();
    let mut current_lines: Vec<String> = Vec::new();
    let mut in_code_block = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            if in_code_block {
                // End of code block - add closing ``` to current paragraph
                current_lines.push(line.to_string());
                in_code_block = false;
            } else {
                // Starting a code block - if we have accumulated lines, save them as a paragraph
                // unless they're blank
                in_code_block = true;
                current_lines.push(line.to_string());
            }
            continue;
        }

        if in_code_block {
            current_lines.push(line.to_string());
            continue;
        }

        if trimmed.is_empty() {
            // Blank line - paragraph boundary
            if !current_lines.is_empty() {
                let text = current_lines.join("\n");
                if !text.trim().is_empty() {
                    paragraphs.push(text);
                }
                current_lines.clear();
            }
        } else {
            current_lines.push(line.to_string());
        }
    }

    // Flush remaining lines
    if !current_lines.is_empty() {
        let text = current_lines.join("\n");
        if !text.trim().is_empty() {
            paragraphs.push(text);
        }
    }

    // Build Paragraph structs with anchors
    paragraphs
        .into_iter()
        .enumerate()
        .map(|(i, text)| Paragraph {
            index: i,
            anchor: compute_anchor(&text),
            text,
        })
        .collect()
}

/// Compute an anchor for a paragraph.
///
/// Takes the first 5 words, lowercases them, and joins with hyphens.
/// Returns `None` for empty or whitespace-only text.
#[must_use]
pub fn compute_anchor(text: &str) -> Option<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Split into words, stripping punctuation
    let words: Vec<&str> = trimmed
        .split_whitespace()
        .take(5)
        .map(|w| {
            // Strip leading/trailing punctuation
            w.trim_matches(|c: char| !c.is_alphanumeric())
        })
        .filter(|w| !w.is_empty())
        .collect();

    if words.is_empty() {
        return None;
    }

    let anchor = words
        .iter()
        .map(|w| w.to_lowercase())
        .collect::<Vec<_>>()
        .join("-");

    if anchor.is_empty() {
        None
    } else {
        Some(anchor)
    }
}
