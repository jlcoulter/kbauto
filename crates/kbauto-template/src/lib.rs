//! kbauto-template: Template loading, defaults management, and playbook generation
//!
//! This crate handles loading template directories (TEMPLATE_DIR convention),
//! parsing defaults JSON, managing page-level versioning, parsing client briefs
//! (deprecated: use StaticDetails + DiscoveryDocument), static details, and
//! discovery documents, and orchestrating the generation pipeline
//! (substitution + AI rewriting).

pub mod affect;
pub mod brief;
pub mod brief_diff;
pub mod details;
pub mod discovery;
pub mod generate;
pub mod incremental;
pub mod page;
pub mod scaffold;
pub mod template;
pub mod tui_model;

// Re-exports
pub use affect::find_affected_pages;
pub use brief_diff::{BriefDiff, brief_diff};
pub use details::{DetailsError, StaticDetails};
pub use discovery::{DiscoveryDocument, DiscoveryError, DiscoveryQuestion};
pub use generate::{GenerationResult, generate_playbook};
pub use incremental::{IncrementalResult, incremental_update};
pub use page::TemplatePage;
pub use scaffold::{
    ScaffoldError, TemplatePathError, WizardPhase, detect_phase, generate_skeleton_details,
    generate_skeleton_discovery_doc, read_template_path, scaffold_client_dir, write_template_path,
};
pub use template::{PlaybookTemplate, TemplateError, list_page_files, load_template};
pub use tui_model::{MissingValue, MissingValueForm};
