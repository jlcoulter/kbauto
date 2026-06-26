//! AI rewriter trait definition.

use async_trait::async_trait;

/// Trait for AI section rewriting (steps 6-10 of customisation).
///
/// Implementations provide the backend-specific logic for rewriting
/// a section of text to match a client's voice and tone.
#[async_trait]
pub trait AiRewriter: Send + Sync {
    /// Rewrite a section of text for the client's voice and tone.
    ///
    /// # Arguments
    ///
    /// * `section_name` - The name/heading of the section to rewrite
    /// * `original_text` - The original section text
    /// * `discovery_context` - Relevant context from the discovery document
    ///
    /// # Errors
    ///
    /// Returns an error if the AI backend is unavailable or the
    /// rewriting fails after all retries are exhausted.
    async fn rewrite_section(
        &self,
        section_name: &str,
        original_text: &str,
        discovery_context: &str,
    ) -> Result<String, RewriterError>;
}

/// Errors from AI rewriting operations.
#[derive(Debug, thiserror::Error)]
pub enum RewriterError {
    /// The AI backend service is not available.
    #[error("AI backend unavailable: {0}")]
    Unavailable(String),
    /// The AI rewriting failed after the specified number of retries.
    #[error("AI rewriting failed after {retries} retries: {message}")]
    Failed {
        /// Number of retry attempts.
        retries: usize,
        /// Error message from the last failed attempt.
        message: String,
    },
    /// The AI returned an empty response for the given section.
    #[error("AI response was empty for section: {0}")]
    EmptyResponse(String),
}
