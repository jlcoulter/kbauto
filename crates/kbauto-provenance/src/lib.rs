//! kbauto-provenance: Provenance tracking and frontmatter management
//!
//! This crate handles reading/writing provenance metadata in Docusaurus
//! frontmatter, paragraph splitting and anchor computation, and
//! provenance classification (template, substituted, rewritten).

pub mod classification;
pub mod frontmatter;
pub mod merge;
pub mod paragraph;

// Re-exports
pub use classification::ProvenanceClassification;
pub use frontmatter::{FrontmatterData, parse_frontmatter, write_frontmatter};
pub use merge::merge_provenance;
pub use paragraph::{Paragraph, compute_anchor, split_paragraphs};
