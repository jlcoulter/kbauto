//! Configuration file loading and CLI override merging for kbauto.
//!
//! Supports TOML config file at `~/.config/kbauto/config.toml` (XDG convention),
//! CLI flag overrides, and sensible hardcoded defaults for zero-config operation.
//!
//! Precedence: CLI flags > config file > compiled defaults

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration loaded from config file and merged with CLI args.
///
/// All fields use `#[serde(default)]` so partial config files work correctly —
/// missing keys fall back to compiled defaults.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    /// Ollama server URL (e.g. "http://localhost:11434").
    #[serde(default = "default_ollama_url")]
    pub ollama_url: String,

    /// Ollama model name (e.g. "deepseek-v4-flash:cloud").
    #[serde(default = "default_ollama_model")]
    pub ollama_model: String,

    /// Number of retry attempts for AI rewriting.
    #[serde(default = "default_retry_count")]
    pub retry_count: u32,

    /// Output format: text (human-readable) or json.
    #[serde(default = "default_output_format")]
    pub output_format: OutputFormat,
}

/// Output format for CLI results.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum OutputFormat {
    #[default]
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "json")]
    Json,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Text => write!(f, "text"),
            OutputFormat::Json => write!(f, "json"),
        }
    }
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" | "human" => Ok(OutputFormat::Text),
            "json" => Ok(OutputFormat::Json),
            _ => Err(format!("unknown output format: {s}")),
        }
    }
}

/// CLI override values. Fields set to `None` mean "use config file or default".
#[derive(Debug, Clone, Default)]
pub struct CliOverrides {
    pub ollama_url: Option<String>,
    pub ollama_model: Option<String>,
    pub retry_count: Option<u32>,
    pub output_format: Option<OutputFormat>,
}

/// Errors that can occur when loading configuration.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// The config file exists but contains invalid TOML.
    #[error("invalid config file at {path}: {reason}")]
    InvalidToml { path: String, reason: String },

    /// IO error reading the config file.
    #[error("IO error reading config file: {0}")]
    Io(#[from] std::io::Error),
}

// Default value functions for serde defaults

fn default_ollama_url() -> String {
    "http://localhost:11434".to_string()
}

fn default_ollama_model() -> String {
    "deepseek-v4-flash:cloud".to_string()
}

fn default_retry_count() -> u32 {
    3
}

fn default_output_format() -> OutputFormat {
    OutputFormat::Text
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ollama_url: default_ollama_url(),
            ollama_model: default_ollama_model(),
            retry_count: default_retry_count(),
            output_format: default_output_format(),
        }
    }
}

impl AppConfig {
    /// Load configuration from the XDG config directory.
    ///
    /// Returns compiled defaults if the config file doesn't exist.
    /// Returns an error if the file exists but contains invalid TOML.
    pub fn load() -> Result<Self, ConfigError> {
        let path = Self::config_path();
        Self::load_from(path)
    }

    /// Load configuration from a specific file path.
    ///
    /// Returns compiled defaults if the file doesn't exist.
    /// Returns an error if the file exists but contains invalid TOML.
    pub fn load_from(path: PathBuf) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = std::fs::read_to_string(&path)?;
        let config: AppConfig = toml::from_str(&content).map_err(|e| ConfigError::InvalidToml {
            path: path.display().to_string(),
            reason: e.to_string(),
        })?;

        Ok(config)
    }

    /// Get the XDG config path for kbauto.
    ///
    /// Uses `dirs::config_dir()` which follows platform conventions:
    /// - Linux: `~/.config/kbauto/config.toml`
    /// - macOS: `~/Library/Application Support/kbauto/config.toml`
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("kbauto")
            .join("config.toml")
    }

    /// Auto-create the config file with default values if it does not exist.
    ///
    /// If the config file already exists, this is a no-op (the file is NOT overwritten).
    /// If the config directory does not exist, it is created.
    ///
    /// # Errors
    /// Returns an error if the directory or file cannot be created.
    pub fn ensure_config_exists() -> Result<(), ConfigError> {
        Self::ensure_config_exists_at(Self::config_path())
    }

    /// Auto-create the config file at a specific path with default values if it does not exist.
    ///
    /// # Errors
    /// Returns an error if the directory or file cannot be created.
    pub fn ensure_config_exists_at(path: PathBuf) -> Result<(), ConfigError> {
        if path.exists() {
            return Ok(());
        }

        // Create parent directory if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Serialize default config to TOML
        let default_config = Self::default();
        let toml_content =
            toml::to_string_pretty(&default_config).map_err(|e| ConfigError::InvalidToml {
                path: path.display().to_string(),
                reason: format!("failed to serialize default config: {e}"),
            })?;

        std::fs::write(&path, toml_content)?;
        Ok(())
    }

    /// Merge CLI overrides into this config.
    ///
    /// CLI flags take precedence: any `Some(_)` value in `overrides`
    /// replaces the corresponding config value.
    pub fn merge_with_cli_args(mut self, overrides: CliOverrides) -> Self {
        if let Some(url) = overrides.ollama_url {
            self.ollama_url = url;
        }
        if let Some(model) = overrides.ollama_model {
            self.ollama_model = model;
        }
        if let Some(count) = overrides.retry_count {
            self.retry_count = count;
        }
        if let Some(fmt) = overrides.output_format {
            self.output_format = fmt;
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_sensible_defaults() {
        let config = AppConfig::default();
        assert_eq!(config.ollama_url, "http://localhost:11434");
        assert_eq!(config.ollama_model, "deepseek-v4-flash:cloud");
        assert_eq!(config.retry_count, 3);
        assert_eq!(config.output_format, OutputFormat::Text);
    }

    #[test]
    fn merge_cli_overrides_config() {
        let config = AppConfig::default();
        let overrides = CliOverrides {
            ollama_url: Some("http://custom:1234".to_string()),
            ollama_model: None,
            retry_count: Some(5),
            output_format: Some(OutputFormat::Json),
        };
        let merged = config.merge_with_cli_args(overrides);
        assert_eq!(merged.ollama_url, "http://custom:1234");
        assert_eq!(merged.ollama_model, "deepseek-v4-flash:cloud"); // unchanged
        assert_eq!(merged.retry_count, 5);
        assert_eq!(merged.output_format, OutputFormat::Json);
    }

    #[test]
    fn merge_none_overrides_preserve_config() {
        let config = AppConfig {
            ollama_url: "http://custom:1234".to_string(),
            ollama_model: "my-model".to_string(),
            retry_count: 7,
            output_format: OutputFormat::Json,
        };
        let overrides = CliOverrides::default(); // all None
        let merged = config.merge_with_cli_args(overrides);
        assert_eq!(merged.ollama_url, "http://custom:1234");
        assert_eq!(merged.ollama_model, "my-model");
        assert_eq!(merged.retry_count, 7);
        assert_eq!(merged.output_format, OutputFormat::Json);
    }

    #[test]
    fn load_from_missing_file_returns_defaults() {
        let config = AppConfig::load_from(PathBuf::from("/nonexistent/config.toml")).unwrap();
        assert_eq!(config, AppConfig::default());
    }

    #[test]
    fn load_from_valid_toml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            r#"
ollama_url = "http://myserver:8080"
ollama_model = "llama3"
retry_count = 5
output_format = "json"
"#,
        )
        .unwrap();

        let config = AppConfig::load_from(path).unwrap();
        assert_eq!(config.ollama_url, "http://myserver:8080");
        assert_eq!(config.ollama_model, "llama3");
        assert_eq!(config.retry_count, 5);
        assert_eq!(config.output_format, OutputFormat::Json);
    }

    #[test]
    fn load_from_partial_toml_uses_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            r#"
ollama_url = "http://myserver:8080"
"#,
        )
        .unwrap();

        let config = AppConfig::load_from(path).unwrap();
        assert_eq!(config.ollama_url, "http://myserver:8080");
        assert_eq!(config.ollama_model, "deepseek-v4-flash:cloud"); // default
        assert_eq!(config.retry_count, 3); // default
        assert_eq!(config.output_format, OutputFormat::Text); // default
    }

    #[test]
    fn load_from_invalid_toml_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "not valid toml [[[").unwrap();

        let result = AppConfig::load_from(path);
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InvalidToml { .. } => {} // expected
            other => panic!("expected InvalidToml, got {other}"),
        }
    }

    #[test]
    fn output_format_from_str() {
        assert_eq!("text".parse::<OutputFormat>().unwrap(), OutputFormat::Text);
        assert_eq!("json".parse::<OutputFormat>().unwrap(), OutputFormat::Json);
        assert_eq!("human".parse::<OutputFormat>().unwrap(), OutputFormat::Text);
        assert!("xml".parse::<OutputFormat>().is_err());
    }

    #[test]
    fn output_format_display() {
        assert_eq!(OutputFormat::Text.to_string(), "text");
        assert_eq!(OutputFormat::Json.to_string(), "json");
    }

    #[test]
    fn output_format_serde_roundtrip() {
        let fmt = OutputFormat::Json;
        let serialized = serde_json::to_string(&fmt).unwrap();
        assert_eq!(serialized, "\"json\"");
        let deserialized: OutputFormat = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, OutputFormat::Json);
    }

    #[test]
    fn config_path_uses_xdg() {
        let path = AppConfig::config_path();
        assert!(path.to_string_lossy().contains("kbauto"));
        assert!(path.to_string_lossy().ends_with("config.toml"));
    }

    #[test]
    fn ensure_config_creates_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("subdir").join("config.toml");
        AppConfig::ensure_config_exists_at(path.clone()).unwrap();
        assert!(path.exists());

        // Verify the written file is valid TOML with all four fields
        let config = AppConfig::load_from(path).unwrap();
        assert_eq!(config.ollama_url, "http://localhost:11434");
        assert_eq!(config.ollama_model, "deepseek-v4-flash:cloud");
        assert_eq!(config.retry_count, 3);
        assert_eq!(config.output_format, OutputFormat::Text);
    }

    #[test]
    fn ensure_config_does_not_overwrite_existing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, r#"ollama_url = "http://custom:9999""#).unwrap();

        AppConfig::ensure_config_exists_at(path.clone()).unwrap();

        // The existing file should be unchanged
        let config = AppConfig::load_from(path).unwrap();
        assert_eq!(config.ollama_url, "http://custom:9999");
    }

    #[test]
    fn ensure_config_written_file_is_valid_toml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        AppConfig::ensure_config_exists_at(path.clone()).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        // Should contain all four fields
        assert!(content.contains("ollama_url"));
        assert!(content.contains("ollama_model"));
        assert!(content.contains("retry_count"));
        assert!(content.contains("output_format"));
    }

    #[test]
    fn ensure_config_creates_parent_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a").join("b").join("c").join("config.toml");
        AppConfig::ensure_config_exists_at(path.clone()).unwrap();
        assert!(path.exists());
    }
}
