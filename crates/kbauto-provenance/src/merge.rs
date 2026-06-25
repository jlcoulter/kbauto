//! Provenance merge logic (never downgrade).

use crate::classification::ProvenanceClassification;

/// Merge two provenance classifications, never downgrading.
///
/// The result is the higher-priority classification:
/// `Rewritten > Substituted > Template`.
///
/// This ensures that once a paragraph has been rewritten by AI,
/// it cannot be "downgraded" to merely substituted by a rebase.
#[must_use]
pub fn merge_provenance(
    current: &ProvenanceClassification,
    incoming: &ProvenanceClassification,
) -> ProvenanceClassification {
    if incoming > current {
        incoming.clone()
    } else {
        current.clone()
    }
}
