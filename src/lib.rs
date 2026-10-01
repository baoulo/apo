//! APO — Engineering Evidence Platform.
//!
//! - **v0.1** Repository Hygiene analyzer
//! - **v0.2** Knowledge Evidence + AI Evidence analyzers (optional Ollama enrichment);
//!   observational language/web packs and `.apo.toml` tooling overlays
//! - **v0.3** Unified evidence pack (`apo report`), baseline diffs, SARIF

#![forbid(unsafe_code)]

pub mod ai_evidence;
pub mod cli;
pub mod config;
pub mod discovery;
pub mod error;
pub mod evidence;
pub mod git;
pub mod knowledge;
pub mod ollama;
pub mod packs;
pub mod policy;
pub mod report;
pub mod rules;
pub mod source;

pub use config::{AnalyzerMode, Config, OutputFormat};
pub use error::{Error, Result};
pub use report::{EvidencePack, EvidenceReport, Report};
pub use source::Workspace;

use tracing::{info, warn};

/// Merge `.apo.toml` `[analyze]` defaults when the caller left them unset.
fn with_project_analyze(config: &Config, root: &std::path::Path) -> Config {
    let mut cfg = config.clone();
    let Some(project) = packs::ApoProjectConfig::load_from_root(root) else {
        return cfg;
    };
    if cfg.rule_disable.is_empty() && !project.analyze.rule_disable.is_empty() {
        cfg.rule_disable = project.analyze.rule_disable;
    }
    if let Some(limit) = project.analyze.commit_sample_limit
        && cfg.commit_sample_limit == Config::default().commit_sample_limit
    {
        cfg.commit_sample_limit = limit;
    }
    cfg
}

/// Analyze a repository and produce a hygiene report.
///
/// `target` may be a local path or a remote Git URI. Remote URIs are shallow-cloned
/// into a temporary directory that is removed when this function returns.
pub fn analyze(config: &Config) -> Result<Report> {
    let (report, _workspace) = analyze_with_workspace(config)?;
    Ok(report)
}

/// Analyze and retain the workspace until the caller drops it.
pub fn analyze_with_workspace(config: &Config) -> Result<(Report, Workspace)> {
    info!(target = %config.target, "resolving repository");
    let workspace = source::resolve(&config.target, config.commit_sample_limit)?;
    let config = with_project_analyze(config, &workspace.path);

    info!(
        path = %workspace.path.display(),
        label = %workspace.label,
        remote = workspace.is_remote(),
        "discovering repository"
    );
    let mut ctx = discovery::discover(&workspace.path, config.commit_sample_limit)?;
    packs::attach_tooling(&mut ctx, &config.packs_dirs)?;

    info!(files = ctx.inventory.len(), "evaluating hygiene rules");
    let (findings, transparency) = filter_disabled(rules::evaluate_all(&ctx), &config.rule_disable);

    info!(count = findings.len(), "computing policy scores");
    let policy = policy::evaluate(&findings);

    let report = Report::build(&ctx, findings, policy, &workspace, transparency);
    Ok((report, workspace))
}

fn filter_disabled(
    findings: Vec<evidence::Finding>,
    disabled: &[String],
) -> (Vec<evidence::Finding>, report::Transparency) {
    let mut transparency = report::Transparency {
        disabled_rules: disabled.to_vec(),
        ..Default::default()
    };
    transparency.sort();
    if disabled.is_empty() {
        return (findings, transparency);
    }
    let findings = findings
        .into_iter()
        .filter(|f| !disabled.iter().any(|d| d == &f.rule))
        .collect();
    (findings, transparency)
}

/// Analyze knowledge + AI evidence (v0.2).
pub fn analyze_evidence(config: &Config) -> Result<EvidenceReport> {
    let (report, _workspace) = analyze_evidence_with_workspace(config)?;
    Ok(report)
}

/// Analyze evidence and retain the workspace until drop.
pub fn analyze_evidence_with_workspace(config: &Config) -> Result<(EvidenceReport, Workspace)> {
    info!(target = %config.target, "resolving repository for evidence");
    let workspace = source::resolve(&config.target, config.commit_sample_limit)?;
    let mut ctx = discovery::discover(&workspace.path, config.commit_sample_limit)?;
    packs::attach_tooling(&mut ctx, &config.packs_dirs)?;

    info!(files = ctx.inventory.len(), "collecting knowledge evidence");
    let knowledge = knowledge::analyze(&ctx);

    info!("collecting AI evidence");
    let ai = ai_evidence::analyze(&ctx);

    let ollama_enrichment = if config.ollama {
        info!(url = %config.ollama_url, model = %config.ollama_model, "ollama enrichment enabled");
        let client = ollama::OllamaClient::new(&config.ollama_url, &config.ollama_model);
        let doc_paths: Vec<_> = knowledge.artifacts.iter().map(|a| a.path.clone()).collect();
        let code_paths: Vec<_> = ctx
            .inventory
            .iter()
            .filter(|e| {
                let l = e.relative.to_ascii_lowercase();
                l.ends_with(".rs")
                    || l.ends_with(".go")
                    || l.ends_with(".py")
                    || l.ends_with(".ts")
                    || l.ends_with(".js")
                    || l.ends_with(".ex")
            })
            .take(40)
            .map(|e| e.relative.clone())
            .collect();
        let findings_summary = format!(
            "Knowledge maturity {:.0}; AI maturity {:.0}; missing kinds: {}; AI governance: {}; prompts: {}.",
            knowledge.maturity_score,
            ai.maturity_score,
            knowledge
                .kinds_missing
                .iter()
                .map(|k| k.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            ai.governance_present,
            ai.prompt_count
        );
        Some(ollama::OllamaEnrichment::enrich(
            &client,
            &doc_paths,
            &code_paths,
            &findings_summary,
        ))
    } else {
        None
    };

    if let Some(ref o) = ollama_enrichment {
        if !o.enabled {
            warn!(notes = ?o.notes, "ollama enrichment disabled/fallback");
        }
    }

    let report = EvidenceReport::build(&ctx, &workspace, knowledge, ai, ollama_enrichment);
    Ok((report, workspace))
}

/// Unified hygiene + knowledge + AI pack (v0.3).
pub fn analyze_pack(config: &Config) -> Result<EvidencePack> {
    let (pack, _workspace) = analyze_pack_with_workspace(config)?;
    Ok(pack)
}

/// Unified pack analysis retaining the workspace.
pub fn analyze_pack_with_workspace(config: &Config) -> Result<(EvidencePack, Workspace)> {
    info!(target = %config.target, "resolving repository for unified pack");
    let workspace = source::resolve(&config.target, config.commit_sample_limit)?;
    let config = with_project_analyze(config, &workspace.path);
    let mut ctx = discovery::discover(&workspace.path, config.commit_sample_limit)?;
    packs::attach_tooling(&mut ctx, &config.packs_dirs)?;

    let (findings, transparency) = filter_disabled(rules::evaluate_all(&ctx), &config.rule_disable);
    let policy = policy::evaluate(&findings);
    let hygiene = Report::build(&ctx, findings, policy, &workspace, transparency);

    let knowledge = knowledge::analyze(&ctx);
    let ai = ai_evidence::analyze(&ctx);
    let ollama_enrichment = if config.ollama {
        let client = ollama::OllamaClient::new(&config.ollama_url, &config.ollama_model);
        let doc_paths: Vec<_> = knowledge.artifacts.iter().map(|a| a.path.clone()).collect();
        let code_paths: Vec<_> = ctx
            .inventory
            .iter()
            .filter(|e| {
                let l = e.relative.to_ascii_lowercase();
                l.ends_with(".rs")
                    || l.ends_with(".go")
                    || l.ends_with(".py")
                    || l.ends_with(".ts")
                    || l.ends_with(".js")
                    || l.ends_with(".ex")
            })
            .take(40)
            .map(|e| e.relative.clone())
            .collect();
        let findings_summary = format!(
            "Knowledge maturity {:.0}; AI maturity {:.0}.",
            knowledge.maturity_score, ai.maturity_score
        );
        Some(ollama::OllamaEnrichment::enrich(
            &client,
            &doc_paths,
            &code_paths,
            &findings_summary,
        ))
    } else {
        None
    };
    let evidence = EvidenceReport::build(&ctx, &workspace, knowledge, ai, ollama_enrichment);

    let mut pack = EvidencePack::build(hygiene, evidence, &workspace);
    if let Some(baseline) = &config.baseline {
        report::apply_baseline(&mut pack, baseline)?;
    }
    Ok((pack, workspace))
}

/// Analyze and write reports (and optional LLM prompt) to disk.
pub fn analyze_and_write(config: &Config) -> Result<(Report, Vec<std::path::PathBuf>)> {
    let (report, workspace) = analyze_with_workspace(config)?;

    let cwd = std::env::current_dir()?;
    let write_dir = resolve_write_dir(config, &workspace, &cwd);

    let mut written = Vec::new();

    if !config.prompt_only {
        written.extend(report::write_report(
            &report,
            config.format,
            config.output.as_deref(),
            write_dir,
        )?);
    }

    if config.badge && !config.prompt_only {
        let badge_path = report::write_hygiene_badge(
            &report,
            config.badge_output.as_deref(),
            config.output.as_deref(),
            write_dir,
        )?;
        written.push(badge_path);
    }

    if config.llm_prompt {
        let prompt_path = report::resolve_prompt_path(&report, config.output.as_deref(), write_dir);
        report::write_llm_prompt(&report, &prompt_path)?;
        written.push(prompt_path);
    }

    drop(workspace);
    Ok((report, written))
}

/// Run the evidence analyzer and write `{repo}-repository-evidence.*` artifacts.
pub fn evidence_and_write(config: &Config) -> Result<(EvidenceReport, Vec<std::path::PathBuf>)> {
    let (report, workspace) = analyze_evidence_with_workspace(config)?;
    let cwd = std::env::current_dir()?;
    let write_dir = resolve_write_dir(config, &workspace, &cwd);

    let mut written = Vec::new();
    if !config.prompt_only {
        written.extend(report::write_evidence_report(
            &report,
            config.format,
            config.output.as_deref(),
            write_dir,
        )?);
    }

    if config.badge && !config.prompt_only {
        let badge_path = report::write_evidence_badge(
            &report,
            config.badge_output.as_deref(),
            config.output.as_deref(),
            write_dir,
        )?;
        written.push(badge_path);
    }

    if config.llm_prompt {
        let prompt_path =
            report::resolve_evidence_prompt_path(&report, config.output.as_deref(), write_dir);
        report::write_evidence_llm_prompt(&report, &prompt_path)?;
        written.push(prompt_path);
    }

    drop(workspace);
    Ok((report, written))
}

/// Run unified pack analysis and write pack artifacts.
pub fn pack_and_write(config: &Config) -> Result<(EvidencePack, Vec<std::path::PathBuf>)> {
    let (pack, workspace) = analyze_pack_with_workspace(config)?;
    let cwd = std::env::current_dir()?;
    let write_dir = resolve_write_dir(config, &workspace, &cwd);

    let mut written = Vec::new();
    if !config.prompt_only {
        written.extend(report::write_pack_report(
            &pack,
            config.format,
            config.output.as_deref(),
            write_dir,
        )?);

        if config.write_sarif && config.format != OutputFormat::Sarif {
            let sarif_path = write_dir.join(pack.sarif_filename());
            report::write_sarif(&pack, &sarif_path)?;
            written.push(sarif_path);
        }

        if let Some(diff_path) = report::write_pack_diff_markdown(&pack, write_dir)? {
            written.push(diff_path);
        }
    }

    if config.badge && !config.prompt_only {
        written.extend(report::write_pack_badges(
            &pack,
            config.badge_output.as_deref(),
            config.output.as_deref(),
            write_dir,
        )?);
    }

    if config.llm_prompt {
        let prompt_path =
            report::resolve_pack_prompt_path(&pack, config.output.as_deref(), write_dir);
        report::write_pack_llm_prompt(&pack, &prompt_path)?;
        written.push(prompt_path);
    }

    drop(workspace);
    Ok((pack, written))
}

fn resolve_write_dir<'a>(
    config: &'a Config,
    workspace: &'a Workspace,
    cwd: &'a std::path::Path,
) -> &'a std::path::Path {
    if let Some(p) = config.output.as_ref() {
        // Treat missing extension-less paths as directories (may not exist yet).
        if p.is_dir() || p.extension().is_none() {
            return p.as_path();
        }
        return p.parent().unwrap_or(cwd);
    }
    if workspace.is_remote() {
        cwd
    } else {
        workspace.path.as_path()
    }
}
