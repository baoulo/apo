//! Local Ollama HTTP client for optional semantic enrichment.
//!
//! Deterministic analyzers remain the source of truth. Ollama may only classify,
//! summarize, or link artifacts that already appear in evidence — never invent facts.

use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, warn};

use crate::error::{Error, Result};

/// Default Ollama endpoint.
pub const DEFAULT_URL: &str = "http://127.0.0.1:11434";

/// Client for a local Ollama daemon.
#[derive(Debug, Clone)]
pub struct OllamaClient {
    base_url: String,
    model: String,
    timeout_secs: u64,
}

impl OllamaClient {
    /// Create a client targeting `base_url` with the given model.
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            model: model.into(),
            timeout_secs: 120,
        }
    }

    /// Probe whether Ollama is reachable and the configured model exists.
    pub fn healthcheck(&self) -> Result<OllamaStatus> {
        let url = format!("{}/api/tags", self.base_url);
        let resp = ureq::get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .call()
            .map_err(|e| Error::Ollama(format!("unreachable at {}: {e}", self.base_url)))?;

        let tags: TagsResponse = resp
            .into_json()
            .map_err(|e| Error::Ollama(format!("invalid /api/tags response: {e}")))?;

        let models: Vec<String> = tags.models.into_iter().map(|m| m.name).collect();
        let model_available = models.iter().any(|m| {
            m == &self.model
                || m.starts_with(&format!("{}:", self.model))
                || m.starts_with(&self.model)
        });

        Ok(OllamaStatus {
            reachable: true,
            model: self.model.clone(),
            model_available,
            available_models: models,
        })
    }

    /// Generate a completion. Returns model text only — caller must validate against evidence.
    pub fn generate(&self, system: &str, prompt: &str) -> Result<String> {
        let url = format!("{}/api/generate", self.base_url);
        let body = json!({
            "model": self.model,
            "system": system,
            "prompt": prompt,
            "stream": false,
            "options": { "temperature": 0.1 },
        });

        debug!(model = %self.model, "ollama generate");
        let resp = ureq::post(&url)
            .timeout(std::time::Duration::from_secs(self.timeout_secs))
            .send_json(body)
            .map_err(|e| Error::Ollama(format!("generate failed: {e}")))?;

        let parsed: GenerateResponse = resp
            .into_json()
            .map_err(|e| Error::Ollama(format!("invalid generate response: {e}")))?;

        let text = parsed.response.trim().to_string();
        if text.is_empty() {
            return Err(Error::Ollama("empty model response".into()));
        }
        Ok(text)
    }

    /// Ask the model to produce JSON constrained by the provided schema description.
    ///
    /// On parse failure, returns an error — callers should fall back to deterministic output.
    pub fn generate_json<T: for<'de> Deserialize<'de>>(
        &self,
        system: &str,
        prompt: &str,
    ) -> Result<T> {
        let system =
            format!("{system}\n\nRespond with ONLY valid JSON. No markdown fences. No commentary.");
        let text = self.generate(&system, prompt)?;
        let cleaned = strip_json_fences(&text);
        serde_json::from_str(cleaned).map_err(|e| {
            Error::Ollama(format!(
                "model returned non-JSON ({e}): {}",
                truncate(&text, 200)
            ))
        })
    }
}

/// Result of an Ollama health probe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaStatus {
    pub reachable: bool,
    pub model: String,
    pub model_available: bool,
    pub available_models: Vec<String>,
}

/// Optional semantic enrichment produced by Ollama (always tied to evidence ids).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OllamaEnrichment {
    /// Whether enrichment ran successfully.
    pub enabled: bool,
    /// Model used, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Notes / warnings (e.g. model missing).
    pub notes: Vec<String>,
    /// Documentation classifications referencing evidence artifact paths.
    pub classifications: Vec<DocClassification>,
    /// Architecture summary grounded in listed evidence paths.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture_summary: Option<GroundedSummary>,
    /// Suggested doc↔code links (must cite existing paths).
    pub suggested_links: Vec<SuggestedLink>,
    /// Executive narrative grounded in evidence ids / paths.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub executive_narrative: Option<GroundedSummary>,
}

impl OllamaEnrichment {
    /// Build a disabled enrichment with an explanatory note.
    pub fn disabled(note: impl Into<String>) -> Self {
        Self {
            enabled: false,
            notes: vec![note.into()],
            ..Self::default()
        }
    }

    /// Attempt enrichment; never invents artifacts — only interprets provided inventory.
    pub fn enrich(
        client: &OllamaClient,
        doc_paths: &[String],
        code_paths: &[String],
        findings_summary: &str,
    ) -> Self {
        match client.healthcheck() {
            Err(e) => {
                warn!(error = %e, "ollama unavailable");
                return Self::disabled(format!("Ollama unavailable: {e}"));
            }
            Ok(status) if !status.model_available => {
                return Self::disabled(format!(
                    "model '{}' not found; available: {}",
                    status.model,
                    status.available_models.join(", ")
                ));
            }
            Ok(_) => {}
        }

        let mut out = Self {
            enabled: true,
            model: Some(client.model.clone()),
            notes: Vec::new(),
            ..Self::default()
        };

        // Classify a sample of docs
        let sample_docs: Vec<_> = doc_paths.iter().take(12).cloned().collect();
        if !sample_docs.is_empty() {
            let prompt = format!(
                "Classify each documentation path into one of: readme, adr, architecture, design, \
                 business_rules, runbook, api, diagram, onboarding, glossary, prompt_library, other.\n\
                 Only use paths from this list:\n{}\n\
                 Return JSON: {{\"items\":[{{\"path\":\"...\",\"kind\":\"...\",\"rationale\":\"...\"}}]}}",
                sample_docs.join("\n")
            );
            match client.generate_json::<ClassificationResponse>(
                "You classify repository documentation. Never invent paths.",
                &prompt,
            ) {
                Ok(parsed) => {
                    out.classifications = parsed
                        .items
                        .into_iter()
                        .filter(|c| sample_docs.iter().any(|p| p == &c.path))
                        .collect();
                }
                Err(e) => out.notes.push(format!("classification skipped: {e}")),
            }
        }

        let arch_docs: Vec<_> = doc_paths
            .iter()
            .filter(|p| {
                let l = p.to_ascii_lowercase();
                l.contains("architect") || l.contains("design") || l.contains("/adr")
            })
            .take(8)
            .cloned()
            .collect();
        if !arch_docs.is_empty() {
            let prompt = format!(
                "Summarize the repository architecture using ONLY these evidence paths as grounding:\n{}\n\
                 Return JSON: {{\"summary\":\"...\",\"evidence_paths\":[\"...\"]}}",
                arch_docs.join("\n")
            );
            match client.generate_json::<GroundedSummary>(
                "You summarize architecture from cited files only. Do not invent components.",
                &prompt,
            ) {
                Ok(mut s) => {
                    s.evidence_paths
                        .retain(|p| arch_docs.iter().any(|d| d == p));
                    if s.evidence_paths.is_empty() {
                        s.evidence_paths = arch_docs.clone();
                    }
                    out.architecture_summary = Some(s);
                }
                Err(e) => out.notes.push(format!("architecture summary skipped: {e}")),
            }
        }

        let code_sample: Vec<_> = code_paths.iter().take(20).cloned().collect();
        if !arch_docs.is_empty() && !code_sample.is_empty() {
            let prompt = format!(
                "Suggest links from documentation to source files. Only use paths from these lists.\n\
                 Docs:\n{}\nCode:\n{}\n\
                 Return JSON: {{\"links\":[{{\"from\":\"doc\",\"to\":\"code\",\"rationale\":\"...\"}}]}}",
                arch_docs.join("\n"),
                code_sample.join("\n")
            );
            match client.generate_json::<SuggestedLinksResponse>(
                "You link docs to code. Never invent paths.",
                &prompt,
            ) {
                Ok(parsed) => {
                    out.suggested_links = parsed
                        .links
                        .into_iter()
                        .filter(|l| {
                            arch_docs.iter().any(|d| d == &l.from)
                                && code_sample.iter().any(|c| c == &l.to)
                        })
                        .take(20)
                        .collect();
                }
                Err(e) => out.notes.push(format!("doc-code linking skipped: {e}")),
            }
        }

        let prompt = format!(
            "Write a short executive narrative (3-5 sentences) for a CTO about knowledge and AI evidence.\n\
             Ground every claim in this deterministic summary; do not invent controls:\n{findings_summary}\n\
             Return JSON: {{\"summary\":\"...\",\"evidence_paths\":[]}}"
        );
        match client.generate_json::<GroundedSummary>(
            "You write executive summaries from provided evidence only.",
            &prompt,
        ) {
            Ok(s) => out.executive_narrative = Some(s),
            Err(e) => out.notes.push(format!("executive narrative skipped: {e}")),
        }

        out
    }
}

#[derive(Debug, Deserialize)]
struct TagsResponse {
    #[serde(default)]
    models: Vec<ModelTag>,
}

#[derive(Debug, Deserialize)]
struct ModelTag {
    name: String,
}

#[derive(Debug, Deserialize)]
struct GenerateResponse {
    #[serde(default)]
    response: String,
}

#[derive(Debug, Deserialize)]
struct ClassificationResponse {
    #[serde(default)]
    items: Vec<DocClassification>,
}

/// LLM classification of a documentation path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocClassification {
    pub path: String,
    pub kind: String,
    #[serde(default)]
    pub rationale: String,
}

/// Summary text that cites evidence paths.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundedSummary {
    pub summary: String,
    #[serde(default)]
    pub evidence_paths: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SuggestedLinksResponse {
    #[serde(default)]
    links: Vec<SuggestedLink>,
}

/// Suggested documentation→code link (paths must already exist).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedLink {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub rationale: String,
}

fn strip_json_fences(s: &str) -> &str {
    let s = s.trim();
    if let Some(rest) = s.strip_prefix("```json") {
        return rest
            .trim_end_matches('`')
            .trim()
            .trim_end_matches("```")
            .trim();
    }
    if let Some(rest) = s.strip_prefix("```") {
        return rest
            .trim_end_matches('`')
            .trim()
            .trim_end_matches("```")
            .trim();
    }
    s
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max).collect::<String>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_fences() {
        let raw = "```json\n{\"a\":1}\n```";
        assert!(strip_json_fences(raw).contains('a'));
    }
}
