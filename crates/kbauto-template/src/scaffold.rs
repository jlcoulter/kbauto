//! Skeleton document generation and client directory scaffolding.
//!
//! This module handles:
//! - Generating skeleton input documents (static details + discovery) from the
//!   template schema and embedded discovery questions (FR-024).
//! - Scaffolding a full client directory structure with auto-generated skeleton
//!   files, an empty `kb/` output directory, and a `.template-path` file
//!   recording the template directory path (FR-025).
//! - Reading and writing the `.template-path` file that persists the template
//!   directory reference across sessions (FR-014).

use kbauto_customise::generate_skeleton_discovery;
use kbauto_placeholder::{DefaultsFile, PlaceholderSchema};
use std::fs;
use std::path::{Path, PathBuf};

/// Which phase the wizard should enter, based on directory contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WizardPhase {
    /// No client files exist yet — scaffold the directory.
    Scaffold,
    /// Skeleton files exist but no KB output — generate the KB.
    Generate,
    /// KB output exists — offer rebase or incremental update.
    RebaseOrUpdate,
}

/// Detect the wizard phase by inspecting the client directory's contents.
///
/// - No `details.md` or `discovery.md` → `Scaffold`
/// - Both skeleton files exist but `kb/` is empty → `Generate`
/// - `kb/` has generated content → `RebaseOrUpdate`
pub fn detect_phase(client_dir: &Path) -> WizardPhase {
    let has_details = client_dir.join("details.md").exists();
    let has_discovery = client_dir.join("discovery.md").exists();
    let kb_dir = client_dir.join("kb");

    if !has_details && !has_discovery {
        return WizardPhase::Scaffold;
    }

    if has_details && has_discovery {
        if kb_dir.exists() {
            if let Ok(entries) = fs::read_dir(&kb_dir) {
                if entries.filter_map(|e| e.ok()).any(|_| true) {
                    return WizardPhase::RebaseOrUpdate;
                }
            }
        }
        return WizardPhase::Generate;
    }

    if kb_dir.exists() {
        if let Ok(entries) = fs::read_dir(&kb_dir) {
            if entries.filter_map(|e| e.ok()).any(|_| true) {
                return WizardPhase::RebaseOrUpdate;
            }
        }
    }
    WizardPhase::Generate
}

/// Errors that can occur during scaffolding.
#[derive(Debug, thiserror::Error)]
pub enum ScaffoldError {
    /// IO error during file or directory operations.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// The client directory already exists and contains files.
    #[error("client directory already exists and is not empty: {0}")]
    DirectoryExists(String),
}

/// Errors that can occur when reading or validating `.template-path`.
#[derive(Debug, thiserror::Error)]
pub enum TemplatePathError {
    /// The `.template-path` file does not exist in the client directory.
    #[error("missing .template-path file in: {0}")]
    Missing(String),

    /// The path in `.template-path` does not point to a valid template directory.
    #[error("invalid template path in .template-path: {0}")]
    Invalid(String),

    /// IO error reading the `.template-path` file.
    #[error("IO error reading .template-path: {0}")]
    Io(#[from] std::io::Error),
}

/// Generate a skeleton static details document from the placeholder schema.
///
/// Creates a markdown file with all placeholder keys as `##` headings, with
/// bodies pre-filled with default values from the defaults file. This gives
/// the user a ready-to-edit starting point so they never need to look up what
/// keys the template expects.
///
/// # Arguments
/// * `schema` - The placeholder schema extracted from the template
/// * `defaults` - The defaults file containing default values
///
/// # Returns
/// A markdown string with heading-value pairs.
pub fn generate_skeleton_details(schema: &PlaceholderSchema, defaults: &DefaultsFile) -> String {
    // Collect and sort keys for stable output
    let mut keys: Vec<&String> = schema.placeholders.keys().collect();
    keys.sort();

    let mut output = String::new();
    for key in keys {
        let default_value = defaults.get_default(key).unwrap_or("");
        output.push_str(&format!("## {key}\n{default_value}\n\n"));
    }
    output
}

/// Generate a skeleton discovery document with embedded canonical questions.
///
/// Delegates to `kbauto_customise::generate_skeleton_discovery()` which uses
/// the `DISCOVERY_QUESTIONS` constant compiled into the binary.
///
/// # Returns
/// A markdown string with discovery question headings and empty bodies.
pub fn generate_skeleton_discovery_doc() -> String {
    generate_skeleton_discovery()
}

/// Scaffold a full client directory structure.
///
/// Creates the following structure:
/// ```text
/// client_dir/
/// ├── details.md       # skeleton with placeholder keys + default values
/// ├── discovery.md     # skeleton with canonical discovery questions
/// ├── kb/              # empty output directory
/// └── .template-path   # absolute path of the template directory
/// ```
///
/// # Arguments
/// * `client_dir` - Path where the client directory will be created
/// * `template_dir` - Path to the template directory (recorded in .template-path)
/// * `schema` - Placeholder schema from the template
/// * `defaults` - Defaults file for the template version
///
/// # Errors
/// Returns an error if the client directory already exists and is non-empty,
/// or if any file or directory cannot be created.
pub fn scaffold_client_dir(
    client_dir: &Path,
    template_dir: &Path,
    schema: &PlaceholderSchema,
    defaults: &DefaultsFile,
) -> Result<(), ScaffoldError> {
    // Don't overwrite an existing non-empty directory
    if client_dir.exists() && fs::read_dir(client_dir)?.next().is_some() {
        return Err(ScaffoldError::DirectoryExists(
            client_dir.display().to_string(),
        ));
    }

    // Create the client directory
    fs::create_dir_all(client_dir)?;

    // Generate and write skeleton details
    let details_content = generate_skeleton_details(schema, defaults);
    fs::write(client_dir.join("details.md"), details_content)?;

    // Generate and write skeleton discovery
    let discovery_content = generate_skeleton_discovery_doc();
    fs::write(client_dir.join("discovery.md"), discovery_content)?;

    // Create empty kb/ output directory
    fs::create_dir_all(client_dir.join("kb"))?;

    // Write .template-path with the absolute path of the template directory
    write_template_path(client_dir, template_dir)?;

    Ok(())
}

/// Write the template directory path to `.template-path` inside the client directory.
///
/// The path is stored as an absolute path so it survives across sessions
/// regardless of the working directory.
///
/// # Arguments
/// * `client_dir` - Path to the client directory
/// * `template_dir` - Path to the template directory to record
///
/// # Errors
/// Returns an error if the file cannot be written.
pub fn write_template_path(client_dir: &Path, template_dir: &Path) -> Result<(), ScaffoldError> {
    let abs_template = template_dir
        .canonicalize()
        .unwrap_or_else(|_| template_dir.to_path_buf());
    let path_file = client_dir.join(".template-path");
    fs::write(&path_file, abs_template.to_string_lossy().as_bytes())?;
    Ok(())
}

/// Read the template directory path from `.template-path` in the client directory.
///
/// Validates that the recorded path exists and contains a `docs/` subdirectory.
///
/// # Arguments
/// * `client_dir` - Path to the client directory
///
/// # Returns
/// The template directory path if valid.
///
/// # Errors
/// - `TemplatePathError::Missing` if the `.template-path` file doesn't exist.
/// - `TemplatePathError::Invalid` if the recorded path doesn't exist or lacks `docs/`.
/// - `TemplatePathError::Io` if the file cannot be read.
pub fn read_template_path(client_dir: &Path) -> Result<PathBuf, TemplatePathError> {
    let path_file = client_dir.join(".template-path");

    if !path_file.exists() {
        return Err(TemplatePathError::Missing(client_dir.display().to_string()));
    }

    let content = fs::read_to_string(&path_file)?;
    let template_path = PathBuf::from(content.trim());

    if !template_path.exists() {
        return Err(TemplatePathError::Invalid(
            template_path.display().to_string(),
        ));
    }

    if !template_path.join("docs").exists() {
        return Err(TemplatePathError::Invalid(
            template_path.display().to_string(),
        ));
    }

    Ok(template_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kbauto_placeholder::PlaceholderSchema;
    use std::collections::HashMap;

    fn make_test_schema() -> PlaceholderSchema {
        let mut placeholders = HashMap::new();
        placeholders.insert(
            "FIRM_NAME".to_string(),
            kbauto_placeholder::Placeholder {
                key: "FIRM_NAME".to_string(),
                format: kbauto_placeholder::PlaceholderFormat::Mustache,
                placeholder_type: kbauto_placeholder::PlaceholderType::Text,
                source_file: "welcome.md".to_string(),
                line_number: 1,
                default_value: Some("Your Firm Name".to_string()),
                description: None,
            },
        );
        placeholders.insert(
            "CONTACT_EMAIL".to_string(),
            kbauto_placeholder::Placeholder {
                key: "CONTACT_EMAIL".to_string(),
                format: kbauto_placeholder::PlaceholderFormat::Mustache,
                placeholder_type: kbauto_placeholder::PlaceholderType::Text,
                source_file: "welcome.md".to_string(),
                line_number: 5,
                default_value: Some("hello@example.com".to_string()),
                description: None,
            },
        );
        PlaceholderSchema {
            version: "1.0.0".to_string(),
            placeholders,
        }
    }

    fn make_test_defaults() -> DefaultsFile {
        DefaultsFile {
            version: "1.0.0".to_string(),
            defaults: vec![
                kbauto_placeholder::DefaultValue {
                    key: "FIRM_NAME".to_string(),
                    value: "Your Firm Name".to_string(),
                    value_type: "text".to_string(),
                    description: None,
                },
                kbauto_placeholder::DefaultValue {
                    key: "CONTACT_EMAIL".to_string(),
                    value: "hello@example.com".to_string(),
                    value_type: "text".to_string(),
                    description: None,
                },
            ],
        }
    }

    #[test]
    fn generate_skeleton_details_has_all_keys() {
        let schema = make_test_schema();
        let defaults = make_test_defaults();
        let details = generate_skeleton_details(&schema, &defaults);
        assert!(details.contains("## FIRM_NAME"));
        assert!(details.contains("## CONTACT_EMAIL"));
    }

    #[test]
    fn generate_skeleton_details_prefills_defaults() {
        let schema = make_test_schema();
        let defaults = make_test_defaults();
        let details = generate_skeleton_details(&schema, &defaults);
        assert!(details.contains("Your Firm Name"));
        assert!(details.contains("hello@example.com"));
    }

    #[test]
    fn generate_skeleton_details_headings_are_uppercase() {
        let schema = make_test_schema();
        let defaults = make_test_defaults();
        let details = generate_skeleton_details(&schema, &defaults);
        // Keys should be uppercase (as stored in schema)
        assert!(details.contains("## FIRM_NAME"));
        assert!(!details.contains("## firm_name"));
    }

    #[test]
    fn generate_skeleton_details_empty_schema() {
        let schema = PlaceholderSchema {
            version: "1.0.0".to_string(),
            placeholders: HashMap::new(),
        };
        let defaults = make_test_defaults();
        let details = generate_skeleton_details(&schema, &defaults);
        // Empty schema produces minimal output (just whitespace)
        assert!(details.trim().is_empty());
    }

    #[test]
    fn generate_skeleton_discovery_has_questions() {
        let discovery = generate_skeleton_discovery_doc();
        assert!(!discovery.is_empty());
        assert!(discovery.contains("## "));
    }

    #[test]
    fn scaffold_creates_all_files() {
        let dir = tempfile::tempdir().unwrap();
        let client_dir = dir.path().join("example");
        let template_dir = dir.path().join("template");
        fs::create_dir_all(template_dir.join("docs")).unwrap();

        let schema = make_test_schema();
        let defaults = make_test_defaults();

        scaffold_client_dir(&client_dir, &template_dir, &schema, &defaults).unwrap();

        assert!(client_dir.exists());
        assert!(client_dir.join("details.md").exists());
        assert!(client_dir.join("discovery.md").exists());
        assert!(client_dir.join("kb").exists());
        assert!(client_dir.join(".template-path").exists());
    }

    #[test]
    fn scaffold_template_path_contains_absolute_path() {
        let dir = tempfile::tempdir().unwrap();
        let client_dir = dir.path().join("example");
        let template_dir = dir.path().join("template");
        fs::create_dir_all(template_dir.join("docs")).unwrap();

        let schema = make_test_schema();
        let defaults = make_test_defaults();

        scaffold_client_dir(&client_dir, &template_dir, &schema, &defaults).unwrap();

        let path_content = fs::read_to_string(client_dir.join(".template-path")).unwrap();
        assert!(path_content.contains("template"));
    }

    #[test]
    fn scaffold_does_not_overwrite_existing() {
        let dir = tempfile::tempdir().unwrap();
        let client_dir = dir.path().join("example");
        fs::create_dir_all(&client_dir).unwrap();
        fs::write(client_dir.join("existing.txt"), "data").unwrap();

        let schema = make_test_schema();
        let defaults = make_test_defaults();

        let result = scaffold_client_dir(&client_dir, dir.path(), &schema, &defaults);
        assert!(result.is_err());
    }

    #[test]
    fn scaffold_skeleton_matches_schema() {
        let dir = tempfile::tempdir().unwrap();
        let client_dir = dir.path().join("example");
        let template_dir = dir.path().join("template");
        fs::create_dir_all(template_dir.join("docs")).unwrap();

        let schema = make_test_schema();
        let defaults = make_test_defaults();

        scaffold_client_dir(&client_dir, &template_dir, &schema, &defaults).unwrap();

        let details = fs::read_to_string(client_dir.join("details.md")).unwrap();
        assert!(details.contains("## FIRM_NAME"));
        assert!(details.contains("## CONTACT_EMAIL"));
    }

    #[test]
    fn write_and_read_template_path() {
        let dir = tempfile::tempdir().unwrap();
        let client_dir = dir.path().join("client");
        let template_dir = dir.path().join("template");
        fs::create_dir_all(&client_dir).unwrap();
        fs::create_dir_all(template_dir.join("docs")).unwrap();

        write_template_path(&client_dir, &template_dir).unwrap();
        let read_path = read_template_path(&client_dir).unwrap();
        assert!(read_path.join("docs").exists());
    }

    #[test]
    fn read_template_path_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let client_dir = dir.path().join("client");
        fs::create_dir_all(&client_dir).unwrap();

        let result = read_template_path(&client_dir);
        match result {
            Err(TemplatePathError::Missing(_)) => {}
            other => panic!("expected Missing, got {other:?}"),
        }
    }

    #[test]
    fn read_template_path_invalid_directory() {
        let dir = tempfile::tempdir().unwrap();
        let client_dir = dir.path().join("client");
        fs::create_dir_all(&client_dir).unwrap();

        // Write a path that doesn't exist
        fs::write(client_dir.join(".template-path"), "/nonexistent/path").unwrap();

        let result = read_template_path(&client_dir);
        match result {
            Err(TemplatePathError::Invalid(_)) => {}
            other => panic!("expected Invalid, got {other:?}"),
        }
    }

    #[test]
    fn read_template_path_no_docs_dir() {
        let dir = tempfile::tempdir().unwrap();
        let client_dir = dir.path().join("client");
        let template_dir = dir.path().join("template");
        fs::create_dir_all(&client_dir).unwrap();
        fs::create_dir_all(&template_dir).unwrap();
        // No docs/ subdirectory

        fs::write(
            client_dir.join(".template-path"),
            template_dir.display().to_string(),
        )
        .unwrap();

        let result = read_template_path(&client_dir);
        match result {
            Err(TemplatePathError::Invalid(_)) => {}
            other => panic!("expected Invalid, got {other:?}"),
        }
    }
}
