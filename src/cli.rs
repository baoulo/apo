//! Command-line interface.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::config::{AnalyzerMode, Config, OutputFormat};

/// APO — Engineering Evidence Platform.
#[derive(Debug, Parser)]
#[command(
    name = "apo",
    version,
    about = "APO — Engineering Evidence Platform. Hygiene, knowledge, and AI evidence.",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Top-level commands.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Analyze repository hygiene evidence (v0.1).
    Analyze {
        /// Local repository path or remote Git URI
        /// (https://…, git@…, ssh://…, file://…).
        #[arg(default_value = ".")]
        target: String,

        /// Output format: markdown, json, or both.
        #[arg(long, default_value = "markdown")]
        format: String,

        /// Write report to this path (file) or directory.
        #[arg(long)]
        output: Option<PathBuf>,

        /// Also write an LLM remediation prompt (`{repo}-repository-hygiene-prompt.md`)
        /// that instructs a model to add missing artifacts and close gaps.
        #[arg(long)]
        llm_prompt: bool,

        /// Write a static SVG hygiene badge next to reports (default: on).
        /// Enterprise-friendly (no external CDN). Use `--no-badge` to skip.
        #[arg(long = "badge", default_value_t = true, action = clap::ArgAction::SetTrue)]
        #[arg(long = "no-badge", action = clap::ArgAction::SetFalse)]
        badge: bool,

        /// Write badge to this path (file) or directory (defaults beside report artifacts).
        #[arg(long)]
        badge_output: Option<PathBuf>,
    },

    /// Analyze knowledge + AI evidence (v0.2).
    Evidence {
        /// Local repository path or remote Git URI.
        #[arg(default_value = ".")]
        target: String,

        /// Output format: markdown, json, or both.
        #[arg(long, default_value = "markdown")]
        format: String,

        /// Write report to this path (file) or directory.
        #[arg(long)]
        output: Option<PathBuf>,

        /// Also write an LLM remediation prompt (`{repo}-repository-evidence-prompt.md`)
        /// that instructs a model to close knowledge and AI evidence gaps.
        #[arg(long)]
        llm_prompt: bool,

        /// Write a static SVG evidence badge next to reports (default: on).
        /// Enterprise-friendly (no external CDN). Use `--no-badge` to skip.
        #[arg(long = "badge", default_value_t = true, action = clap::ArgAction::SetTrue)]
        #[arg(long = "no-badge", action = clap::ArgAction::SetFalse)]
        badge: bool,

        /// Write badge to this path (file) or directory (defaults beside report artifacts).
        #[arg(long)]
        badge_output: Option<PathBuf>,

        /// Enable optional Ollama semantic enrichment (local HTTP API).
        #[arg(long)]
        ollama: bool,

        /// Ollama base URL.
        #[arg(long, default_value = "http://127.0.0.1:11434")]
        ollama_url: String,

        /// Ollama model name.
        #[arg(long, default_value = "llama3.2")]
        ollama_model: String,
    },

    /// Analyze and emit only an LLM remediation prompt (stdout + file).
    Prompt {
        /// Local repository path or remote Git URI.
        #[arg(default_value = ".")]
        target: String,

        /// Write prompt to this path (file) or directory.
        #[arg(long)]
        output: Option<PathBuf>,

        /// Write the file only; do not print the prompt to stdout.
        #[arg(long)]
        quiet: bool,

        /// Generate a knowledge/AI evidence remediation prompt instead of hygiene.
        #[arg(long)]
        evidence: bool,
    },
}

impl Cli {
    /// Convert CLI args into a [`Config`].
    pub fn into_config(self) -> Result<Config, String> {
        match self.command {
            Commands::Analyze {
                target,
                format,
                output,
                llm_prompt,
                badge,
                badge_output,
            } => Ok(Config {
                target,
                format: OutputFormat::parse(&format)?,
                output,
                llm_prompt,
                prompt_only: false,
                prompt_stdout: false,
                mode: AnalyzerMode::Hygiene,
                badge,
                badge_output,
                ..Config::default()
            }),
            Commands::Evidence {
                target,
                format,
                output,
                llm_prompt,
                badge,
                badge_output,
                ollama,
                ollama_url,
                ollama_model,
            } => Ok(Config {
                target,
                format: OutputFormat::parse(&format)?,
                output,
                llm_prompt,
                prompt_only: false,
                prompt_stdout: false,
                mode: AnalyzerMode::Evidence,
                badge,
                badge_output,
                ollama,
                ollama_url,
                ollama_model,
                ..Config::default()
            }),
            Commands::Prompt {
                target,
                output,
                quiet,
                evidence,
            } => Ok(Config {
                target,
                format: OutputFormat::Markdown,
                output,
                llm_prompt: true,
                prompt_only: true,
                prompt_stdout: !quiet,
                badge: false,
                mode: if evidence {
                    AnalyzerMode::Evidence
                } else {
                    AnalyzerMode::Hygiene
                },
                ..Config::default()
            }),
        }
    }
}
