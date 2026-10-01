//! Command-line interface.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::config::{AnalyzerMode, Config, OutputFormat};
use crate::packs::ApoProjectConfig;

/// APO — Engineering Evidence Platform.
#[derive(Debug, Parser)]
#[command(
    name = "apo",
    version,
    about = "APO — Engineering Evidence Platform. Hygiene, knowledge, and AI evidence.",
    long_about = None
)]
pub struct Cli {
    /// Extra directory of external pack TOML files (repeatable). Also loads `{repo}/.apo/packs/`.
    /// Env `APO_PACKS_DIR` is included by default when set.
    #[arg(long = "packs-dir", global = true, action = clap::ArgAction::Append)]
    pub packs_dir: Vec<PathBuf>,

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

    /// Unified hygiene + knowledge + AI evidence pack (v0.3).
    #[command(visible_alias = "all")]
    Report {
        /// Local repository path or remote Git URI.
        #[arg(default_value = ".")]
        target: String,

        /// Output format: markdown, json, both, or sarif.
        #[arg(long, default_value = "markdown")]
        format: String,

        /// Write pack to this path (file) or directory.
        #[arg(long)]
        output: Option<PathBuf>,

        /// Also write a combined LLM remediation prompt.
        #[arg(long)]
        llm_prompt: bool,

        /// Write static SVG badges (hygiene + evidence) next to reports (default: on).
        #[arg(long = "badge", default_value_t = true, action = clap::ArgAction::SetTrue)]
        #[arg(long = "no-badge", action = clap::ArgAction::SetFalse)]
        badge: bool,

        /// Write badge(s) to this path (file) or directory.
        #[arg(long)]
        badge_output: Option<PathBuf>,

        /// Prior pack JSON for score/gap diffs.
        #[arg(long)]
        baseline: Option<PathBuf>,

        /// Also write SARIF (`{repo}-repository-evidence-pack.sarif`).
        #[arg(long)]
        sarif: bool,

        /// Enable optional Ollama semantic enrichment.
        #[arg(long)]
        ollama: bool,

        /// Ollama base URL.
        #[arg(long, default_value = "http://127.0.0.1:11434")]
        ollama_url: String,

        /// Ollama model name.
        #[arg(long, default_value = "llama3.2")]
        ollama_model: String,

        /// Fail if hygiene overall score is below this threshold (0–100).
        #[arg(long)]
        fail_on_score: Option<f64>,
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

        /// Generate a unified pack remediation prompt.
        #[arg(long)]
        pack: bool,
    },

    /// Inspect or export built-in language/web packs.
    Packs {
        #[command(subcommand)]
        command: PacksCommands,
    },
}

/// Subcommands under `apo packs`.
#[derive(Debug, Subcommand)]
pub enum PacksCommands {
    /// Write built-in packs as external TOML files (`{id}.toml`).
    ///
    /// Useful for inspection or as a starting point: place a dumped file under
    /// `.apo/packs/` (same `id`) to **override** the shipped built-in definition.
    Dump {
        /// Output directory for `{id}.toml` files.
        #[arg(long, short = 'o')]
        output: PathBuf,

        /// Dump only this built-in pack id (default: all).
        #[arg(long)]
        id: Option<String>,
    },
}

impl Cli {
    /// Convert CLI args into a [`Config`], merging `.apo.toml` defaults when present.
    pub fn into_config(self) -> Result<Config, String> {
        let extra_packs = self.packs_dir.clone();
        let apply_packs = |cfg: &mut Config| {
            for d in &extra_packs {
                if !cfg.packs_dirs.iter().any(|p| p == d) {
                    cfg.packs_dirs.push(d.clone());
                }
            }
        };
        match self.command {
            Commands::Analyze {
                target,
                format,
                output,
                llm_prompt,
                badge,
                badge_output,
            } => {
                let mut cfg = base_from_project(&target);
                cfg.target = target;
                cfg.format = OutputFormat::parse(&format)?;
                cfg.output = output;
                cfg.llm_prompt = llm_prompt;
                cfg.mode = AnalyzerMode::Hygiene;
                cfg.badge = badge;
                cfg.badge_output = badge_output;
                apply_packs(&mut cfg);
                Ok(cfg)
            }
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
            } => {
                let mut cfg = base_from_project(&target);
                cfg.target = target;
                cfg.format = OutputFormat::parse(&format)?;
                cfg.output = output;
                cfg.llm_prompt = llm_prompt;
                cfg.mode = AnalyzerMode::Evidence;
                cfg.badge = badge;
                cfg.badge_output = badge_output;
                if ollama {
                    cfg.ollama = true;
                }
                if ollama || cfg.ollama {
                    if ollama_url != "http://127.0.0.1:11434" || !cfg.ollama {
                        cfg.ollama_url = ollama_url;
                    }
                    if ollama_model != "llama3.2" || ollama {
                        cfg.ollama_model = ollama_model;
                    }
                }
                apply_packs(&mut cfg);
                Ok(cfg)
            }
            Commands::Report {
                target,
                format,
                output,
                llm_prompt,
                badge,
                badge_output,
                baseline,
                sarif,
                ollama,
                ollama_url,
                ollama_model,
                fail_on_score,
            } => {
                let mut cfg = base_from_project(&target);
                cfg.target = target;
                cfg.format = OutputFormat::parse(&format)?;
                if format == "markdown"
                    && let Some(pf) =
                        ApoProjectConfig::load_from_root(std::path::Path::new(&cfg.target))
                            .and_then(|p| p.report.format)
                    && let Ok(parsed) = OutputFormat::parse(&pf)
                {
                    cfg.format = parsed;
                }
                cfg.output = output;
                cfg.llm_prompt = llm_prompt;
                cfg.mode = AnalyzerMode::Pack;
                cfg.badge = badge;
                cfg.badge_output = badge_output;
                cfg.baseline = baseline;
                cfg.write_sarif = sarif;
                cfg.fail_on_score = fail_on_score;
                if ollama {
                    cfg.ollama = true;
                    cfg.ollama_url = ollama_url;
                    cfg.ollama_model = ollama_model;
                }
                apply_packs(&mut cfg);
                Ok(cfg)
            }
            Commands::Prompt {
                target,
                output,
                quiet,
                evidence,
                pack,
            } => {
                let mode = if pack {
                    AnalyzerMode::Pack
                } else if evidence {
                    AnalyzerMode::Evidence
                } else {
                    AnalyzerMode::Hygiene
                };
                let mut cfg = base_from_project(&target);
                cfg.target = target;
                cfg.format = OutputFormat::Markdown;
                cfg.output = output;
                cfg.llm_prompt = true;
                cfg.prompt_only = true;
                cfg.prompt_stdout = !quiet;
                cfg.badge = false;
                cfg.mode = mode;
                apply_packs(&mut cfg);
                Ok(cfg)
            }
            Commands::Packs { .. } => {
                Err("internal: packs subcommands are handled before into_config".into())
            }
        }
    }
}

fn base_from_project(target: &str) -> Config {
    let mut cfg = Config::default();
    let Some(project) = ApoProjectConfig::load_from_root(std::path::Path::new(target)) else {
        return cfg;
    };
    if let Some(limit) = project.analyze.commit_sample_limit {
        cfg.commit_sample_limit = limit;
    }
    if !project.analyze.rule_disable.is_empty() {
        cfg.rule_disable = project.analyze.rule_disable;
    }
    if project.ollama.enabled {
        cfg.ollama = true;
        if let Some(url) = project.ollama.url {
            cfg.ollama_url = url;
        }
        if let Some(model) = project.ollama.model {
            cfg.ollama_model = model;
        }
    }
    if let Some(fmt) = &project.report.format
        && let Ok(parsed) = OutputFormat::parse(fmt)
    {
        cfg.format = parsed;
    }
    cfg
}
