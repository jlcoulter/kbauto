//! Canonical discovery questions embedded at compile time.
//!
//! These questions provide the voice/tone context for AI rewriting (steps 6-10).
//! They are the same for every client — only the answers vary per client.
//! The questions align with the customisation prompt in `prompt.rs`.

/// The canonical set of discovery questions used to generate skeleton
/// discovery documents and to guide AI rewriting.
///
/// These questions capture the client's voice, tone, positioning, and
/// preferences. They are embedded in the binary at compile time per FR-024.
pub const DISCOVERY_QUESTIONS: &[&str] = &[
    "What is the firm's primary service focus and target client?",
    "What tone should the knowledge base convey?",
    "What sets this firm apart from competitors?",
    "What is the firm's approach to client communication?",
    "What are the firm's core values and philosophy?",
    "How does the firm describe its pricing or fee structure?",
    "What geographic area or jurisdictions does the firm serve?",
    "What is the firm's history or founding story?",
    "What client success stories or testimonials reflect the firm's impact?",
    "What is the firm's vision for the future?",
];

/// Generate a skeleton discovery document with all canonical questions as
/// `##` headings and empty bodies for the user to fill in.
///
/// The output is valid markdown ready for the user to edit. Each question
/// becomes a `##` heading with an empty line beneath it.
pub fn generate_skeleton_discovery() -> String {
    DISCOVERY_QUESTIONS
        .iter()
        .map(|q| format!("## {q}\n\n"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovery_questions_is_non_empty() {
        assert!(!DISCOVERY_QUESTIONS.is_empty());
        assert!(DISCOVERY_QUESTIONS.len() >= 5);
    }

    #[test]
    fn each_question_is_non_empty() {
        for q in DISCOVERY_QUESTIONS {
            assert!(!q.is_empty(), "discovery question must not be empty");
            assert!(q.len() > 5, "discovery question should be meaningful");
        }
    }

    #[test]
    fn questions_cover_voice_and_tone() {
        // Questions should address tone, positioning, and differentiation
        let all_questions = DISCOVERY_QUESTIONS.join(" ").to_lowercase();
        assert!(all_questions.contains("tone"), "must cover tone");
        assert!(
            all_questions.contains("apart") || all_questions.contains("differentiat"),
            "must cover differentiation"
        );
        assert!(
            all_questions.contains("service") || all_questions.contains("focus"),
            "must cover service focus"
        );
    }

    #[test]
    fn generate_skeleton_produces_valid_markdown() {
        let skeleton = generate_skeleton_discovery();
        assert!(!skeleton.is_empty());

        // Every question should appear as a ## heading
        for q in DISCOVERY_QUESTIONS {
            assert!(
                skeleton.contains(&format!("## {q}")),
                "skeleton must contain heading for: {q}"
            );
        }
    }

    #[test]
    fn generate_skeleton_has_empty_bodies() {
        let skeleton = generate_skeleton_discovery();
        // Each heading should be followed by an empty line, not text
        for q in DISCOVERY_QUESTIONS {
            let heading = format!("## {q}\n\n");
            assert!(
                skeleton.contains(&heading),
                "skeleton must have empty body after: {q}"
            );
        }
    }
}