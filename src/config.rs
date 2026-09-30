//! Runtime configuration for analysis.

use serde::{Deserialize, Serialize};

/// Output format for reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Markdown report (default).
    #[default]
    Markdown,
    /// JSON report.
    Json,
    /// Emit both Markdown and JSON.
    Both,
}

impl OutputFormat {
    /// Parse from CLI string.
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.to_ascii_lowercase().as_str() {
            "md" | "markdown" => Ok(Self::Markdown),
            "json" => Ok(Self::Json),
            "both" => Ok(Self::Both),
            other => Err(format!(
                "unknown format '{other}'; expected markdown, json, or both"
            )),
        }
    }
}

/// Which analyzer pipeline to run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalyzerMode {
    /// Repository hygiene (v0.1).
    #[default]
    Hygiene,
    /// Knowledge + AI evidence (v0.2).
    Evidence,
}

/// Analysis configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Local path or remote Git URI to analyze.
    pub target: String,
    /// Desired output format.
    pub format: OutputFormat,
    /// Optional explicit output path (file or directory).
    pub output: Option<std::path::PathBuf>,
    /// Maximum commits to inspect for maintenance signals (also clone depth for remotes).
    pub commit_sample_limit: usize,
    /// Write an LLM remediation prompt from gap findings.
    pub llm_prompt: bool,
    /// Skip hygiene report files; only emit the LLM prompt.
    pub prompt_only: bool,
    /// Print the LLM prompt to stdout.
    pub prompt_stdout: bool,
    /// Analyzer pipeline to run.
    pub mode: AnalyzerMode,
    /// Enable optional Ollama semantic enrichment (evidence mode).
    pub ollama: bool,
    /// Ollama base URL.
    pub ollama_url: String,
    /// Ollama model name.
    pub ollama_model: String,
    /// Write a static SVG status badge next to report artifacts (enterprise-friendly).
    pub badge: bool,
    /// Optional explicit badge output path (file or directory).
    pub badge_output: Option<std::path::PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            target: ".".into(),
            format: OutputFormat::Markdown,
            output: None,
            commit_sample_limit: 100,
            llm_prompt: false,
            prompt_only: false,
            prompt_stdout: false,
            mode: AnalyzerMode::Hygiene,
            ollama: false,
            ollama_url: std::env::var("APO_OLLAMA_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:11434".into()),
            ollama_model: std::env::var("APO_OLLAMA_MODEL").unwrap_or_else(|_| "llama3.2".into()),
            badge: true,
            badge_output: None,
        }
    }
}
