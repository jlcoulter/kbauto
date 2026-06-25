//! Playbook version parsing and comparison.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;

/// Semantic version for a playbook template.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlaybookVersion {
    /// Major version component.
    pub major: u32,
    /// Minor version component.
    pub minor: u32,
    /// Patch version component.
    pub patch: u32,
}

impl PlaybookVersion {
    /// Parse a version string like `"1.0.0"` or `"v1.0.0"`.
    ///
    /// # Errors
    ///
    /// Returns an error if the string is not a valid semantic version.
    #[must_use = "parsing a version must be checked for errors"]
    pub fn parse(s: &str) -> Result<Self, VersionParseError> {
        let trimmed = s.trim_start_matches('v');
        let parts: Vec<&str> = trimmed.split('.').collect();
        if parts.len() != 3 {
            return Err(VersionParseError::InvalidFormat(s.to_string()));
        }
        let major = parts[0]
            .parse::<u32>()
            .map_err(|_| VersionParseError::InvalidFormat(s.to_string()))?;
        let minor = parts[1]
            .parse::<u32>()
            .map_err(|_| VersionParseError::InvalidFormat(s.to_string()))?;
        let patch = parts[2]
            .parse::<u32>()
            .map_err(|_| VersionParseError::InvalidFormat(s.to_string()))?;
        Ok(Self {
            major,
            minor,
            patch,
        })
    }
}

impl fmt::Display for PlaybookVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Ord for PlaybookVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.major
            .cmp(&other.major)
            .then_with(|| self.minor.cmp(&other.minor))
            .then_with(|| self.patch.cmp(&other.patch))
    }
}

impl PartialOrd for PlaybookVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::str::FromStr for PlaybookVersion {
    type Err = VersionParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

/// Errors that can occur when parsing a playbook version.
#[derive(Debug, thiserror::Error)]
pub enum VersionParseError {
    /// The version string does not match the expected `MAJOR.MINOR.PATCH` format.
    #[error("invalid version format: {0}")]
    InvalidFormat(String),
}
