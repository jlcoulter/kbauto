//! Conflict types and resolutions.

use serde::{Deserialize, Serialize};

/// A conflict between template-origin text and client-origin text
/// during a rebase operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Conflict {
    /// Page filename where the conflict occurs.
    pub filename: String,
    /// Paragraph index within the page.
    pub paragraph_index: usize,
    /// Anchor for matching across versions.
    pub anchor: Option<String>,
    /// The template-origin (new base) text.
    pub new_base_text: String,
    /// The client-origin (substituted/rewritten) text.
    pub client_text: String,
}

/// Resolution strategy for a conflict.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictResolution {
    /// Keep the client version (discard new base changes).
    KeepClient,
    /// Keep the new base version (discard client changes).
    KeepNewBase,
    /// Manually merge — requires user intervention.
    ManualMerge {
        /// The manually merged text.
        merged_text: String,
    },
}
