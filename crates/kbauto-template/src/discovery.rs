//! Discovery document parsing.
//!
//! Parses a discovery document (Q&A pairs) from markdown. Each heading
//! (## or ###) is treated as a question, and the body text between headings
//! is the answer. Discovery documents provide context for AI rewriting.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// A single question-answer pair from a discovery document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscoveryQuestion {
    /// The question text (from the markdown heading).
    pub question: String,
    /// The answer text (body content between headings).
    pub answer: String,
}

/// A parsed discovery document containing Q&A pairs.
///
/// The order of questions is preserved from the source document.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscoveryDocument {
    /// Ordered list of question-answer pairs.
    pub questions: Vec<DiscoveryQuestion>,
}

/// Errors that can occur when parsing a discovery document.
#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    /// IO error reading the discovery file.
    #[error("IO error reading discovery document: {0}")]
    Io(#[from] std::io::Error),
    /// The discovery file contains no parseable Q&A pairs.
    #[error("discovery document is empty: {0}")]
    EmptyFile(String),
}

impl DiscoveryDocument {
    /// Parse a discovery document from a markdown file.
    ///
    /// # Errors
    ///
    /// Returns `DiscoveryError` if the file cannot be read or has no Q&A pairs.
    pub fn from_markdown_file(path: &Path) -> Result<Self, DiscoveryError> {
        let content = std::fs::read_to_string(path)?;
        Self::from_markdown(&content, &path.display().to_string())
    }

    /// Parse a discovery document from a markdown string.
    ///
    /// Each heading (## or ###) is a question, body text between headings
    /// is the answer. The question text preserves its original case and spacing.
    ///
    /// # Errors
    ///
    /// Returns `DiscoveryError::EmptyFile` if no Q&A pairs are found.
    pub fn from_markdown(content: &str, source: &str) -> Result<Self, DiscoveryError> {
        let mut questions = Vec::new();
        let mut current_question = String::new();
        let mut current_answer = String::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("### ") || trimmed.starts_with("## ") {
                // Save the previous Q&A pair
                if !current_question.is_empty() {
                    questions.push(DiscoveryQuestion {
                        question: current_question.trim().to_string(),
                        answer: current_answer.trim().to_string(),
                    });
                }
                // Start new question — preserve original heading text
                let heading = if trimmed.starts_with("### ") {
                    &trimmed[4..]
                } else {
                    &trimmed[3..]
                };
                current_question = heading.trim().to_string();
                current_answer.clear();
            } else {
                // Accumulate answer content
                if !current_question.is_empty() {
                    current_answer.push_str(line);
                    current_answer.push('\n');
                }
            }
        }

        // Save the last Q&A pair
        if !current_question.is_empty() {
            questions.push(DiscoveryQuestion {
                question: current_question.trim().to_string(),
                answer: current_answer.trim().to_string(),
            });
        }

        if questions.is_empty() {
            return Err(DiscoveryError::EmptyFile(source.to_string()));
        }

        Ok(Self { questions })
    }

    /// Return the number of Q&A pairs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.questions.len()
    }

    /// Return true if there are no Q&A pairs.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.questions.is_empty()
    }

    /// Format all Q&A pairs as context text for AI rewriting.
    ///
    /// Returns a single string with each question followed by its answer,
    /// suitable for inclusion in an AI rewriting prompt.
    #[must_use]
    pub fn as_context(&self) -> String {
        self.questions
            .iter()
            .map(|qa| format!("Q: {}\nA: {}", qa.question, qa.answer))
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}