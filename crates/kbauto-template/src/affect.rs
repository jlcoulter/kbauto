//! Affected page identification.
//!
//! Given a template and a set of changed placeholder keys, determine
//! which pages contain those placeholders and therefore need to be
//! regenerated during an incremental update.

use crate::template::PlaybookTemplate;

/// Find pages in the template that contain any of the changed placeholder keys.
///
/// Returns a sorted, deduplicated list of page filenames.
/// Placeholder keys are compared case-insensitively (canonical UPPERCASE form).
#[must_use = "the list of affected pages should be used"]
pub fn find_affected_pages(template: &PlaybookTemplate, changed_keys: &[String]) -> Vec<String> {
    let changed_set: std::collections::HashSet<String> = changed_keys
        .iter()
        .map(|k| kbauto_placeholder::canonical_key(k))
        .collect();

    let mut affected = Vec::new();

    for page in &template.pages {
        let page_has_changed = page
            .placeholders
            .iter()
            .any(|p| changed_set.contains(&kbauto_placeholder::canonical_key(p)));

        if page_has_changed {
            affected.push(page.filename.clone());
        }
    }

    affected.sort();
    affected.dedup();
    affected
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::TemplatePage;

    fn make_template(pages: Vec<TemplatePage>) -> PlaybookTemplate {
        PlaybookTemplate {
            version: "1.0.0".to_string(),
            source_dir: std::path::PathBuf::from("/tmp/test"),
            parsed_version: kbauto_placeholder::PlaybookVersion {
                major: 1,
                minor: 0,
                patch: 0,
            },
            pages,
        }
    }

    #[test]
    fn find_affected_pages_basic() {
        let template = make_template(vec![
            TemplatePage {
                filename: "welcome.md".to_string(),
                content: "Welcome to TEAM_NAME.".to_string(),
                placeholders: vec!["TEAM_NAME".to_string(), "TEAM_EMAIL".to_string()],
            },
            TemplatePage {
                filename: "services.md".to_string(),
                content: "TEAM_PHONE number.".to_string(),
                placeholders: vec!["TEAM_PHONE".to_string()],
            },
            TemplatePage {
                filename: "about.md".to_string(),
                content: "Static content.".to_string(),
                placeholders: vec![],
            },
        ]);

        let changed = vec!["TEAM_NAME".to_string()];
        let affected = find_affected_pages(&template, &changed);
        assert_eq!(affected, vec!["welcome.md"]);
    }

    #[test]
    fn find_affected_pages_multiple_keys() {
        let template = make_template(vec![
            TemplatePage {
                filename: "welcome.md".to_string(),
                content: "TEAM_NAME TEAM_EMAIL".to_string(),
                placeholders: vec!["TEAM_NAME".to_string(), "TEAM_EMAIL".to_string()],
            },
            TemplatePage {
                filename: "services.md".to_string(),
                content: "TEAM_PHONE".to_string(),
                placeholders: vec!["TEAM_PHONE".to_string()],
            },
        ]);

        let changed = vec!["TEAM_NAME".to_string(), "TEAM_PHONE".to_string()];
        let affected = find_affected_pages(&template, &changed);
        assert_eq!(affected, vec!["services.md", "welcome.md"]);
    }

    #[test]
    fn find_affected_pages_no_matches() {
        let template = make_template(vec![TemplatePage {
            filename: "static.md".to_string(),
            content: "No placeholders".to_string(),
            placeholders: vec![],
        }]);

        let changed = vec!["TEAM_NAME".to_string()];
        let affected = find_affected_pages(&template, &changed);
        assert!(affected.is_empty());
    }

    #[test]
    fn find_affected_pages_empty_keys() {
        let template = make_template(vec![TemplatePage {
            filename: "page.md".to_string(),
            content: "TEAM_NAME".to_string(),
            placeholders: vec!["TEAM_NAME".to_string()],
        }]);

        let affected = find_affected_pages(&template, &[]);
        assert!(affected.is_empty());
    }

    #[test]
    fn find_affected_pages_case_insensitive() {
        let template = make_template(vec![TemplatePage {
            filename: "welcome.md".to_string(),
            content: "TEAM_NAME".to_string(),
            placeholders: vec!["TEAM_NAME".to_string()],
        }]);

        let changed = vec!["team_name".to_string()];
        let affected = find_affected_pages(&template, &changed);
        assert_eq!(affected, vec!["welcome.md"]);
    }
}
