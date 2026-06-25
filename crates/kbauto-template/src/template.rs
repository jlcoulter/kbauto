//! Template directory loading.

use crate::page::TemplatePage;
use kbauto_placeholder::{DefaultsError, DefaultsFile, PlaybookVersion};
use std::fs;
use std::path::Path;

/// A loaded playbook template set.
#[derive(Debug, Clone)]
pub struct PlaybookTemplate {
    /// Version string from the defaults.json.
    pub version: String,
    /// Directory path this template was loaded from.
    pub source_dir: std::path::PathBuf,
    /// Parsed version structure.
    pub parsed_version: PlaybookVersion,
    /// Template pages loaded from the docs/ subdirectory.
    pub pages: Vec<TemplatePage>,
}

/// Errors that can occur when loading templates.
#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    /// The template directory was not found.
    #[error("template directory not found: {0}")]
    NotFound(String),
    /// The required `docs/` subdirectory is missing.
    #[error("missing docs/ subdirectory in: {0}")]
    MissingDocsDir(String),
    /// The required `defaults.json` file is missing.
    #[error("missing defaults.json in: {0}")]
    MissingDefaults(String),
    /// The playbook version in defaults.json does not match the expected version.
    #[error("version mismatch: expected {expected}, found {found}")]
    VersionMismatch {
        /// The expected version string.
        expected: String,
        /// The version found in the defaults file.
        found: String,
    },
    /// An IO error occurred while reading template files.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// A JSON parsing error occurred.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    /// A YAML parsing error occurred.
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    /// An error parsing defaults.json.
    #[error("defaults error: {0}")]
    Defaults(#[from] DefaultsError),
}

/// Load a playbook template from a directory.
///
/// Expects the directory to contain:
/// - `docs/` subdirectory with markdown page files
/// - `defaults.json` with default placeholder values
///
/// # Errors
///
/// Returns `TemplateError` if the directory doesn't exist, is missing
/// required subdirectories, or if files cannot be parsed.
pub fn load_template(dir: &Path) -> Result<PlaybookTemplate, TemplateError> {
    // Check that the directory exists
    if !dir.exists() {
        return Err(TemplateError::NotFound(dir.display().to_string()));
    }

    let docs_dir = dir.join("docs");
    if !docs_dir.exists() {
        return Err(TemplateError::MissingDocsDir(dir.display().to_string()));
    }

    let defaults_path = dir.join("defaults.json");
    if !defaults_path.exists() {
        return Err(TemplateError::MissingDefaults(dir.display().to_string()));
    }

    // Parse defaults.json
    let defaults_content = fs::read_to_string(&defaults_path)?;
    let defaults = DefaultsFile::from_json(&defaults_content)?;

    // Validate version
    let parsed_version =
        PlaybookVersion::parse(&defaults.version).map_err(|_| TemplateError::VersionMismatch {
            expected: "valid semver".to_string(),
            found: defaults.version.clone(),
        })?;

    // Load page files
    let page_files = list_page_files(dir)?;
    let mut pages = Vec::new();

    for page_path in &page_files {
        let content = fs::read_to_string(page_path)?;
        let filename = page_path
            .strip_prefix(&docs_dir)
            .unwrap_or(page_path)
            .to_string_lossy()
            .to_string();

        // Extract placeholders from the content (ignoring frontmatter)
        let placeholders = kbauto_placeholder::extract_placeholders(&content, &filename);
        let placeholder_keys: Vec<String> = placeholders.iter().map(|p| p.key.clone()).collect();

        pages.push(TemplatePage {
            filename,
            content,
            placeholders: placeholder_keys,
        });
    }

    Ok(PlaybookTemplate {
        version: defaults.version,
        source_dir: dir.to_path_buf(),
        parsed_version,
        pages,
    })
}

/// List all markdown page files in a template's docs/ directory.
///
/// # Errors
///
/// Returns `TemplateError` if the directory cannot be read.
#[must_use = "the list of page files should be used"]
pub fn list_page_files(dir: &Path) -> Result<Vec<std::path::PathBuf>, TemplateError> {
    let docs_dir = dir.join("docs");

    if !docs_dir.exists() {
        return Err(TemplateError::MissingDocsDir(dir.display().to_string()));
    }

    let mut md_files = Vec::new();

    let entries = fs::read_dir(&docs_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "md") {
            md_files.push(path);
        }
    }

    // Sort for deterministic ordering
    md_files.sort();

    Ok(md_files)
}
