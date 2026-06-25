//! kbauto-rebase: Rebase engine, conflict detection, and diff reports
//!
//! This crate handles diffing playbook versions, rebasing client KBs onto
//! new versions while preserving substituted/rewritten text, detecting
//! conflicts, and generating diff reports.

pub mod conflict;
pub mod diff;
pub mod rebase;
pub mod resolve;

// Re-exports
pub use conflict::{Conflict, ConflictResolution};
pub use diff::{DiffReport, PageChange, diff_pages, diff_playbooks};
pub use rebase::rebase_client_kb;
pub use resolve::resolve_conflict;
