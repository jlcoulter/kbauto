//! AI failure retry and fallback logic.

use crate::rewriter::{AiRewriter, RewriterError};

/// Configuration for retry behaviour.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (default: 3).
    pub max_retries: usize,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self { max_retries: 3 }
    }
}

/// Result of a retry operation.
#[derive(Debug)]
pub enum RetryOutcome<T> {
    /// Operation succeeded with the returned value.
    Success(T),
    /// All retries exhausted; returning fallback value.
    Fallback {
        /// The original text that was being rewritten.
        original: String,
        /// The error that caused all retries to fail.
        error: RewriterError,
    },
}

/// Retry an AI rewrite operation with fallback.
///
/// On success, returns `RetryOutcome::Success(rewritten_text)`.
/// After exhausting all retries, returns `RetryOutcome::Fallback` with
/// the original text and a `RewriterError::Failed` error.
///
/// # Arguments
///
/// * `rewriter` - The AI rewriter to call
/// * `section_name` - Name of the section being rewritten
/// * `original_text` - The original text (used as fallback)
/// * `discovery_context` - Context from the discovery document
/// * `config` - Retry configuration
pub async fn retry_with_fallback(
    rewriter: &dyn AiRewriter,
    section_name: &str,
    original_text: &str,
    discovery_context: &str,
    config: &RetryConfig,
) -> RetryOutcome<String> {
    let mut last_error: Option<RewriterError> = None;

    for _attempt in 0..config.max_retries {
        match rewriter
            .rewrite_section(section_name, original_text, discovery_context)
            .await
        {
            Ok(text) => {
                return RetryOutcome::Success(text);
            }
            Err(e) => {
                eprintln!(
                    "Warning: retry_with_fallback failed for section '{}': {}",
                    section_name, e
                );
                last_error = Some(e);
            }
        }
    }

    // When all retries are exhausted, return a Failed error wrapping the last error
    let failed_error = match last_error {
        Some(RewriterError::EmptyResponse(section)) => {
            // Keep EmptyResponse as-is so callers can distinguish empty responses
            RewriterError::EmptyResponse(section)
        }
        Some(e) => {
            // Wrap any other error as Failed
            RewriterError::Failed {
                retries: config.max_retries,
                message: e.to_string(),
            }
        }
        None => RewriterError::Failed {
            retries: config.max_retries,
            message: "all retries exhausted".to_string(),
        },
    };

    RetryOutcome::Fallback {
        original: original_text.to_string(),
        error: failed_error,
    }
}
