//! kbauto-placeholder: Placeholder extraction, schema building, and resolution
//!
//! This crate handles extracting placeholders from markdown templates
//! in three formats (Bare, AngleBracket, Mustache), building placeholder
//! schemas, and resolving placeholders with values from defaults or
//! client briefs.

pub mod defaults;
pub mod extract;
pub mod resolve;
pub mod types;
pub mod version;

// Re-exports
pub use defaults::{DefaultValue, DefaultsError, DefaultsFile};
pub use extract::{build_schema, canonical_key, detect_format, extract_placeholders};
pub use resolve::{ResolutionResult, ResolvedPlaceholder, format_value, resolve_placeholders};
pub use types::{Placeholder, PlaceholderFormat, PlaceholderSchema, PlaceholderType};
pub use version::PlaybookVersion;
