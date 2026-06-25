//! Conflict resolution application.

use crate::conflict::{Conflict, ConflictResolution};

/// Apply a conflict resolution to a page's content.
///
/// Updates the paragraph at the conflict's index with the resolved text
/// and adjusts the provenance classification accordingly.
///
/// # Errors
///
/// Returns `ResolveError::IndexOutOfBounds` if the conflict's paragraph
/// index is beyond the length of the content's paragraphs.
pub fn resolve_conflict(
    content: &str,
    conflict: &Conflict,
    resolution: &ConflictResolution,
) -> Result<String, ResolveError> {
    let paragraphs = kbauto_provenance::split_paragraphs(content);

    if conflict.paragraph_index >= paragraphs.len() {
        return Err(ResolveError::IndexOutOfBounds);
    }

    let resolved_text = match resolution {
        ConflictResolution::KeepClient => conflict.client_text.clone(),
        ConflictResolution::KeepNewBase => conflict.new_base_text.clone(),
        ConflictResolution::ManualMerge { merged_text } => merged_text.clone(),
    };

    // Rebuild the content by replacing the paragraph at the conflict index
    let mut parts: Vec<String> = paragraphs.iter().map(|p| p.text.clone()).collect();
    parts[conflict.paragraph_index] = resolved_text;
    Ok(parts.join("\n\n"))
}

/// Errors from conflict resolution.
#[derive(Debug, thiserror::Error)]
pub enum ResolveError {
    /// Feature not yet implemented.
    #[error("not yet implemented")]
    NotImplemented,
    /// The paragraph index in the conflict is beyond the document length.
    #[error("paragraph index out of bounds")]
    IndexOutOfBounds,
}
