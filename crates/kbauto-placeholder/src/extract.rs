//! Placeholder extraction from template content.

use crate::types::{Placeholder, PlaceholderFormat, PlaceholderSchema, PlaceholderType};
use regex::Regex;

/// Extract placeholders from markdown content.
///
/// Detects placeholders in three formats:
/// - Bare: `TEAM_NAME` (matches `[A-Z][A-Z0-9_]{2,}`)
/// - AngleBracket: `<<team_name>>`
/// - Mustache: `{{TEAM_NAME}}`
///
/// Placeholders inside fenced code blocks (``` blocks) are skipped.
/// Placeholders inside inline code (`` ` ``) are also skipped.
///
/// # Arguments
///
/// * `content` - The markdown content to scan
/// * `source_file` - The filename for provenance tracking
#[must_use = "extracted placeholders should be used"]
pub fn extract_placeholders(content: &str, source_file: &str) -> Vec<Placeholder> {
    let mut placeholders = Vec::new();

    // Regex patterns for the three formats
    let bare_re = Regex::new(r"[A-Z][A-Z0-9_]{2,}").unwrap();
    let angle_re = Regex::new(r"<<([^>]+)>>").unwrap();
    let mustache_re = Regex::new(r"\{\{([A-Z][A-Z0-9_]*)\}\}").unwrap();

    // Code block ranges are used by the line-by-line tracking below,
    // so we don't need the byte-offset ranges here (we track in_code_block inline).
    let _code_block_ranges = compute_fenced_code_block_ranges(content);

    let mut in_code_block = false;
    for (line_num, line) in content.lines().enumerate() {
        let line_num_1based = line_num + 1;

        // Track fenced code block boundaries
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }

        // Find inline code spans in this line
        let inline_spans = find_inline_code_spans_in_line(line);
        // Collect ranges of mustache and angle-bracket matches (to exclude from bare matching)
        let mut excluded_ranges: Vec<(usize, usize)> = Vec::new();

        // Extract mustache placeholders
        for cap in mustache_re.captures_iter(line) {
            let full_mat = cap.get(0).unwrap();
            if !is_in_inline_code(full_mat.start(), &inline_spans) {
                excluded_ranges.push((full_mat.start(), full_mat.end()));
                placeholders.push(Placeholder {
                    key: cap.get(1).unwrap().as_str().to_string(),
                    format: PlaceholderFormat::Mustache,
                    placeholder_type: PlaceholderType::Text,
                    source_file: source_file.to_string(),
                    line_number: line_num_1based,
                    default_value: None,
                    description: None,
                });
            }
        }

        // Extract angle-bracket placeholders
        for cap in angle_re.captures_iter(line) {
            let full_mat = cap.get(0).unwrap();
            if !is_in_inline_code(full_mat.start(), &inline_spans) {
                excluded_ranges.push((full_mat.start(), full_mat.end()));
                placeholders.push(Placeholder {
                    key: canonical_key(cap.get(1).unwrap().as_str()),
                    format: PlaceholderFormat::AngleBracket,
                    placeholder_type: PlaceholderType::Text,
                    source_file: source_file.to_string(),
                    line_number: line_num_1based,
                    default_value: None,
                    description: None,
                });
            }
        }

        // Extract bare placeholders, but skip any that overlap with mustache/angle-bracket matches
        for mat in bare_re.find_iter(line) {
            if !is_in_inline_code(mat.start(), &inline_spans)
                && !overlaps_any_range(mat.start(), mat.end(), &excluded_ranges)
            {
                placeholders.push(Placeholder {
                    key: mat.as_str().to_string(),
                    format: PlaceholderFormat::Bare,
                    placeholder_type: PlaceholderType::Text,
                    source_file: source_file.to_string(),
                    line_number: line_num_1based,
                    default_value: None,
                    description: None,
                });
            }
        }
    }

    placeholders
}

/// Check if a range [start, end) overlaps with any range in the list.
fn overlaps_any_range(start: usize, end: usize, ranges: &[(usize, usize)]) -> bool {
    ranges.iter().any(|&(s, e)| start < e && end > s)
}

/// Compute character ranges that are inside fenced code blocks.
fn compute_fenced_code_block_ranges(content: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut in_code = false;
    let mut code_start: Option<usize> = None;

    // Track byte offset position for each line
    let mut offset = 0usize;
    for line in content.lines() {
        if line.trim().starts_with("```") {
            if in_code {
                // End of code block
                if let Some(start) = code_start.take() {
                    ranges.push((start, offset + line.len()));
                }
                in_code = false;
            } else {
                // Start of code block
                code_start = Some(offset);
                in_code = true;
            }
        }
        offset += line.len() + 1; // +1 for newline
    }

    ranges
}

/// Find inline code spans within a single line.
fn find_inline_code_spans_in_line(line: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut in_code = false;
    let mut start: Option<usize> = None;

    for (i, ch) in line.char_indices() {
        if ch == '`' {
            if in_code {
                // End of inline code
                if let Some(s) = start.take() {
                    spans.push((s, i + 1));
                }
                in_code = false;
            } else {
                // Start of inline code
                start = Some(i);
                in_code = true;
            }
        }
    }

    spans
}

/// Check if a position is inside an inline code span.
fn is_in_inline_code(pos: usize, spans: &[(usize, usize)]) -> bool {
    spans.iter().any(|&(s, e)| pos >= s && pos < e)
}

/// Build a placeholder schema from extracted placeholders.
///
/// Deduplicates by canonical (UPPERCASE) key, preserving the first
/// occurrence's format and location information.
#[must_use = "the built schema should be used"]
pub fn build_schema(placeholders: Vec<Placeholder>, version: &str) -> PlaceholderSchema {
    let mut map = std::collections::HashMap::new();

    for p in placeholders {
        let key = canonical_key(&p.key);
        // Only insert if key doesn't exist yet (first occurrence wins)
        map.entry(key).or_insert(p);
    }

    PlaceholderSchema {
        version: version.to_string(),
        placeholders: map,
    }
}

/// Normalise a placeholder key to its canonical UPPERCASE form.
///
/// - Bare keys are already uppercase.
/// - AngleBracket keys like `<<team_name>>` become `TEAM_NAME`.
/// - Mustache keys like `{{TEAM_NAME}}` become `TEAM_NAME`.
#[must_use]
pub fn canonical_key(raw_key: &str) -> String {
    // Strip delimiters if present
    let key = if raw_key.starts_with("<<") && raw_key.ends_with(">>") {
        &raw_key[2..raw_key.len() - 2]
    } else if raw_key.starts_with("{{") && raw_key.ends_with("}}") {
        &raw_key[2..raw_key.len() - 2]
    } else {
        raw_key
    };

    key.to_uppercase()
}

/// Detect the format of a placeholder from its raw text.
///
/// Returns `None` if the text does not match any known placeholder format.
#[must_use]
pub fn detect_format(raw: &str) -> Option<PlaceholderFormat> {
    if raw.starts_with("<<") && raw.ends_with(">>") {
        Some(PlaceholderFormat::AngleBracket)
    } else if raw.starts_with("{{") && raw.ends_with("}}") {
        Some(PlaceholderFormat::Mustache)
    } else if raw.chars().all(|c| c.is_ascii_uppercase() || c == '_') && raw.len() >= 3 {
        Some(PlaceholderFormat::Bare)
    } else {
        None
    }
}
