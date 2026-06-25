//! kbauto-customise: Client brief parsing, AI rewriting, and customisation logic
//!
//! This crate handles parsing client briefs, applying placeholder substitutions
//! (steps 1-5 of the 10-step customisation process), invoking the AI trait for
//! section rewrites (steps 6-10), and retry/fallback logic for AI failures.

pub mod discovery_questions;
pub mod ollama;
pub mod prompt;
pub mod retry;
pub mod rewriter;

// Re-exports
pub use discovery_questions::{DISCOVERY_QUESTIONS, generate_skeleton_discovery};
pub use ollama::OllamaRewriter;
pub use prompt::CUSTOMISATION_PROMPT;
pub use retry::{RetryConfig, RetryOutcome, retry_with_fallback};
pub use rewriter::{AiRewriter, RewriterError};
