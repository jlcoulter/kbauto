//! Provenance classification (template, substituted, rewritten).

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Classification of a paragraph's provenance.
///
/// Steps 1-5 of the 10-step customisation process produce "substituted"
/// provenance. Steps 6-10 produce "rewritten" provenance. Unchanged
/// template text is "template".
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvenanceClassification {
    /// Unchanged from the template defaults.
    Template,
    /// Placeholders filled from the brief (steps 1-5).
    Substituted,
    /// AI-rewritten for voice/tone (steps 6-10).
    Rewritten,
}

impl ProvenanceClassification {
    /// Numeric priority for merge-never-downgrade logic.
    fn priority(&self) -> u8 {
        match self {
            Self::Template => 0,
            Self::Substituted => 1,
            Self::Rewritten => 2,
        }
    }
}

impl std::fmt::Display for ProvenanceClassification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Template => write!(f, "template"),
            Self::Substituted => write!(f, "substituted"),
            Self::Rewritten => write!(f, "rewritten"),
        }
    }
}

impl Ord for ProvenanceClassification {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority().cmp(&other.priority())
    }
}

impl PartialOrd for ProvenanceClassification {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
