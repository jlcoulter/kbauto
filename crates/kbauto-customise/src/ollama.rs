//! Ollama-based AI rewriter implementation.

use crate::prompt::CUSTOMISATION_PROMPT;
use crate::rewriter::{AiRewriter, RewriterError};
use ollama_rs::generation::completion::request::GenerationRequest;

/// Ollama-based AI rewriter using the ollama-rs client.
pub struct OllamaRewriter {
    /// Ollama model name (e.g. "deepseek-v4-flash:cloud").
    pub model: String,
    /// Ollama server URL (e.g. "http://localhost:11434").
    pub url: String,
    /// Maximum number of retry attempts (default: 3).
    pub max_retries: usize,
}

impl OllamaRewriter {
    /// Create a new OllamaRewriter with the given model, URL, and retry count.
    pub fn new(model: String, url: String, max_retries: usize) -> Self {
        Self {
            model,
            url,
            max_retries,
        }
    }

    /// Strip markdown code fences from an AI response.
    ///
    /// AI models often wrap responses in ```markdown ... ``` or ``` ... ```
    /// code fences. This function removes them, returning the inner content.
    pub fn strip_code_fences(response: &str) -> String {
        let trimmed = response.trim();
        // Try ```markdown ... ``` first, then ``` ... ```
        for fence in ["```markdown", "```md", "```"] {
            if trimmed.starts_with(fence) {
                // Find the opening fence end (newline after fence tag)
                let after_fence = if let Some(idx) = trimmed[fence.len()..].find('\n') {
                    &trimmed[fence.len() + idx + 1..]
                } else {
                    // No newline — fence is the entire prefix
                    &trimmed[fence.len()..]
                };
                // Strip the closing ```
                if let Some(end) = after_fence.rfind("```") {
                    return after_fence[..end].trim().to_string();
                } else {
                    return after_fence.trim().to_string();
                }
            }
        }
        trimmed.to_string()
    }

    /// Build the prompt for the AI model.
    pub fn build_prompt(
        section_name: &str,
        original_text: &str,
        discovery_context: &str,
    ) -> String {
        format!(
            "{CUSTOMISATION_PROMPT}\n\n\
            ## Section: {section_name}\n\n\
            Original text:\n\
            ---\n\
            {original_text}\n\
            ---\n\n\
            Client discovery context:\n\
            ---\n\
            {discovery_context}\n\
            ---\n\n\
            Rewrite this section for the client, adapting tone and positioning \
            while preserving the factual content and structure. Output ONLY the \
            rewritten text, no explanations or meta-commentary."
        )
    }
}

#[async_trait::async_trait]
impl AiRewriter for OllamaRewriter {
    async fn rewrite_section(
        &self,
        section_name: &str,
        original_text: &str,
        discovery_context: &str,
    ) -> Result<String, RewriterError> {
        let prompt = Self::build_prompt(section_name, original_text, discovery_context);

        let mut last_error = String::new();
        let max_attempts = self.max_retries.max(1);

        for attempt in 0..max_attempts {
            match self.call_ollama(&prompt).await {
                Ok(response) => {
                    let cleaned = Self::strip_code_fences(&response);
                    if cleaned.trim().is_empty() {
                        // Empty response after stripping — retry unless last attempt
                        if attempt + 1 < max_attempts {
                            last_error = format!("empty response on attempt {}", attempt + 1);
                            continue;
                        }
                        return Err(RewriterError::EmptyResponse(section_name.to_string()));
                    }
                    return Ok(cleaned);
                }
                Err(e) => {
                    last_error = e;
                    if attempt + 1 < max_attempts {
                        continue;
                    }
                }
            }
        }

        Err(RewriterError::Failed {
            retries: max_attempts as usize,
            message: last_error,
        })
    }
}

impl OllamaRewriter {
    /// Call the Ollama API with the given prompt.
    async fn call_ollama(&self, prompt: &str) -> Result<String, String> {
        // Parse url string (e.g. "http://localhost:11434") into host + port.
        let (host, port) = parse_ollama_url(&self.url)?;
        let ollama = ollama_rs::Ollama::builder().host(host).port(port).build();
        let request = GenerationRequest::new(self.model.clone(), prompt.to_string());

        match ollama.generate(request).await {
            Ok(response) => Ok(response.response),
            Err(e) => Err(format!("Ollama error: {e}")),
        }
    }
}

/// Parse an Ollama URL string (e.g. "http://localhost:11434") into (host, port).
///
/// Returns the host (without scheme) and the port number. Defaults to
/// `http://localhost` and port `11434` if parsing fails.
fn parse_ollama_url(url: &str) -> Result<(String, u16), String> {
    let trimmed = url.trim();
    // Strip scheme
    let without_scheme = trimmed
        .strip_prefix("http://")
        .or_else(|| trimmed.strip_prefix("https://"))
        .unwrap_or(trimmed);
    // Split host:port
    if let Some((host, port_str)) = without_scheme.rsplit_once(':') {
        let port: u16 = port_str
            .parse()
            .map_err(|_| format!("invalid port in Ollama URL: {url}"))?;
        Ok((host.to_string(), port))
    } else {
        // No port — default to 11434
        Ok((without_scheme.to_string(), 11434))
    }
}
