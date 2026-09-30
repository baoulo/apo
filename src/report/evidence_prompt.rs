//! LLM remediation prompt generation from knowledge + AI evidence.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::evidence::{Finding, Status};
use crate::report::EvidenceReport;

/// Render a paste-ready prompt to close knowledge and AI evidence gaps.
pub fn render_evidence_llm_prompt(report: &EvidenceReport) -> String {
    let mut out = String::new();

    let _ = writeln!(out, "# Repository knowledge & AI evidence remediation task");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "You are an expert software engineer improving **engineering knowledge** and **AI adoption/governance** in a repository."
    );
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "APO (Engineering Evidence Platform) analyzed this repository for Knowledge Evidence and AI Evidence."
    );
    let _ = writeln!(
        out,
        "Your job is to **update the repository** so missing or weak knowledge and AI controls become observable — real docs, prompts, configs, and workflows — not aspirational text."
    );
    let _ = writeln!(out);

    let _ = writeln!(out, "## Repository");
    let _ = writeln!(out);
    let _ = writeln!(out, "- **Identity:** `{}`", report.repository);
    if let Some(uri) = &report.source_uri {
        let _ = writeln!(out, "- **Source URI:** `{uri}`");
        let _ = writeln!(
            out,
            "- **Important:** Analysis used a temporary clone. Apply all changes to the real working tree / open PR for `{uri}`."
        );
    }
    if let Some(path) = &report.checkout_path
        && report.source_uri.is_some()
    {
        let _ = writeln!(out, "- **Analysis checkout (ephemeral):** `{path}`");
    }
    let _ = writeln!(out, "- **APO version:** {}", report.apo_version);
    let _ = writeln!(out, "- **Analyzer:** {}", report.analyzer);
    let _ = writeln!(out, "- **Generated:** {}", report.generated_at);
    let _ = writeln!(
        out,
        "- **Knowledge maturity:** {:.1}/100",
        report.knowledge_maturity
    );
    let _ = writeln!(out, "- **AI maturity:** {:.1}/100", report.ai_maturity);
    let _ = writeln!(out);

    let _ = writeln!(out, "## Executive summary (from APO)");
    let _ = writeln!(out);
    let _ = writeln!(out, "{}", report.executive_summary.trim());
    let _ = writeln!(out);

    let _ = writeln!(out, "## Objectives");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "1. Close the **gap findings** below (Missing / Partial / Unknown) for knowledge and AI evidence."
    );
    let _ = writeln!(
        out,
        "2. Prefer the smallest change set that creates **observable evidence** APO can detect (paths, docs, configs, CI)."
    );
    let _ = writeln!(
        out,
        "3. Match the repository’s existing language, tooling, and style."
    );
    let _ = writeln!(
        out,
        "4. Do **not** invent owners, architecture decisions, AI usage, or governance that does not exist. Prefer honest stubs with TODOs over fiction."
    );
    let _ = writeln!(
        out,
        "5. Fix broken local links when listed; do not invent remote URLs."
    );
    let _ = writeln!(
        out,
        "6. After changes, summarize what you added/updated and which APO finding rule ids each change targets."
    );
    let _ = writeln!(out);

    let _ = writeln!(out, "## Constraints");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "- Keep documentation accurate and short; avoid filler."
    );
    let _ = writeln!(
        out,
        "- Do not remove existing working knowledge or AI controls."
    );
    let _ = writeln!(out, "- Do not commit secrets, tokens, or private keys.");
    let _ = writeln!(
        out,
        "- Prompt libraries and agent configs must be real templates usable by the team’s tools — not empty placeholders."
    );
    let _ = writeln!(
        out,
        "- If a gap is truly not applicable, say so and skip rather than adding fake workflows."
    );
    let _ = writeln!(out);

    write_maturity_snapshot(&mut out, report);
    write_present_knowledge(&mut out, report);
    write_present_ai(&mut out, report);
    write_risks(&mut out, report);
    write_broken_links(&mut out, report);
    write_findings_section(&mut out, "Knowledge findings", &report.knowledge.findings);
    write_findings_section(&mut out, "AI findings", &report.ai.findings);

    let _ = writeln!(out, "## Deliverable format");
    let _ = writeln!(out);
    let _ = writeln!(out, "1. Make the file/config/CI edits in the repository.");
    let _ = writeln!(
        out,
        "2. Reply with a short changelog mapping each touched path → APO rule id(s) (e.g. `knowledge.coverage`, `ai.governance`)."
    );
    let _ = writeln!(
        out,
        "3. Call out any gaps you could not close with exact follow-up steps."
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "---");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "_This prompt was generated by APO from observational evidence. Treat findings as facts to act on, not as opinions to debate._"
    );

    out
}

fn write_maturity_snapshot(out: &mut String, report: &EvidenceReport) {
    let _ = writeln!(out, "## Maturity snapshot");
    let _ = writeln!(out);
    let _ = writeln!(out, "| Domain | Score | Notes |");
    let _ = writeln!(out, "|--------|------:|-------|");
    let _ = writeln!(
        out,
        "| Knowledge | {:.1} | coverage {:.1}, freshness {:.1}, {} artifacts, {} missing kinds |",
        report.knowledge_maturity,
        report.knowledge.coverage_score,
        report.knowledge.freshness_score,
        report.knowledge.artifacts.len(),
        report.knowledge.kinds_missing.len()
    );
    let _ = writeln!(
        out,
        "| AI | {:.1} | {} prompts; versioning {}; governance {}; human review {}; workflows {} |",
        report.ai_maturity,
        report.ai.prompt_count,
        yn(report.ai.prompt_versioning),
        yn(report.ai.governance_present),
        yn(report.ai.human_review_signals),
        yn(report.ai.workflow_adoption)
    );
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "Prioritize the lower of the two maturity scores when sequencing work."
    );
    let _ = writeln!(out);
}

fn write_present_knowledge(out: &mut String, report: &EvidenceReport) {
    let _ = writeln!(out, "## Knowledge already present (do not redo)");
    let _ = writeln!(out);
    if report.knowledge.artifacts.is_empty() {
        let _ = writeln!(
            out,
            "_None — treat knowledge docs as greenfield for expected kinds._"
        );
    } else {
        for a in &report.knowledge.artifacts {
            let stale = if a.stale { " stale" } else { "" };
            let days = a
                .days_since_modified
                .map(|d| format!(", {d}d"))
                .unwrap_or_default();
            let _ = writeln!(out, "- `{}` — `{}`{stale}{days}", a.path, a.kind.as_str());
        }
    }
    if !report.knowledge.kinds_missing.is_empty() {
        let _ = writeln!(out);
        let missing: Vec<_> = report
            .knowledge
            .kinds_missing
            .iter()
            .map(|k| k.as_str())
            .collect();
        let _ = writeln!(out, "**Missing knowledge kinds:** {}", missing.join(", "));
    }
    let _ = writeln!(out);

    let satisfied: Vec<_> = report
        .knowledge
        .findings
        .iter()
        .filter(|f| matches!(f.status, Status::Enforced | Status::Present))
        .collect();
    if !satisfied.is_empty() {
        let _ = writeln!(out, "### Satisfied knowledge findings");
        let _ = writeln!(out);
        for f in satisfied {
            write_finding_bullet(out, f);
        }
        let _ = writeln!(out);
    }
}

fn write_present_ai(out: &mut String, report: &EvidenceReport) {
    let _ = writeln!(out, "## AI artifacts already present (do not redo)");
    let _ = writeln!(out);
    if report.ai.artifacts.is_empty() {
        let _ = writeln!(
            out,
            "_None — treat prompts, agents, MCP, and AI governance as greenfield._"
        );
    } else {
        for a in &report.ai.artifacts {
            match &a.detail {
                Some(d) => {
                    let _ = writeln!(out, "- `{}` — `{}` — {d}", a.path, a.kind.as_str());
                }
                None => {
                    let _ = writeln!(out, "- `{}` — `{}`", a.path, a.kind.as_str());
                }
            }
        }
    }
    let _ = writeln!(out);

    let satisfied: Vec<_> = report
        .ai
        .findings
        .iter()
        .filter(|f| matches!(f.status, Status::Enforced | Status::Present))
        .collect();
    if !satisfied.is_empty() {
        let _ = writeln!(out, "### Satisfied AI findings");
        let _ = writeln!(out);
        for f in satisfied {
            write_finding_bullet(out, f);
        }
        let _ = writeln!(out);
    }
}

fn write_risks(out: &mut String, report: &EvidenceReport) {
    let _ = writeln!(out, "## Risks called out by APO");
    let _ = writeln!(out);
    if report.knowledge_risks.is_empty() && report.ai_risks.is_empty() {
        let _ = writeln!(out, "_No explicit risks listed._");
    } else {
        for r in &report.knowledge_risks {
            let _ = writeln!(out, "- **Knowledge:** {r}");
        }
        for r in &report.ai_risks {
            let _ = writeln!(out, "- **AI:** {r}");
        }
    }
    let _ = writeln!(out);
}

fn write_broken_links(out: &mut String, report: &EvidenceReport) {
    let _ = writeln!(out, "## Broken local links");
    let _ = writeln!(out);
    if report.knowledge.broken_links.is_empty() {
        let _ = writeln!(out, "_None detected._");
    } else {
        let _ = writeln!(
            out,
            "Fix these local markdown links (create the target or correct the reference):"
        );
        let _ = writeln!(out);
        for link in &report.knowledge.broken_links {
            let _ = writeln!(
                out,
                "- `{}` → `{}` — {}",
                link.source, link.target, link.detail
            );
        }
    }
    let _ = writeln!(out);
}

fn write_findings_section(out: &mut String, title: &str, findings: &[Finding]) {
    let gaps: Vec<_> = findings.iter().filter(|f| f.status.is_gap()).collect();
    let _ = writeln!(out, "## {title} — gaps to remediate");
    let _ = writeln!(out);
    if gaps.is_empty() {
        let _ = writeln!(
            out,
            "_No gap findings in this domain. Optionally suggest polish only if clearly valuable._"
        );
        let _ = writeln!(out);
        return;
    }

    let _ = writeln!(
        out,
        "Work through these. For each gap: create or update concrete artifacts, then tick it off."
    );
    let _ = writeln!(out);

    for (i, f) in gaps.into_iter().enumerate() {
        let _ = writeln!(out, "### {}. `{}` — {:?}", i + 1, f.rule, f.status);
        let _ = writeln!(out);
        let _ = writeln!(out, "- **Summary:** {}", f.summary);
        let _ = writeln!(out, "- **Confidence:** {:?}", f.confidence);
        if let Some(r) = &f.remediation {
            let _ = writeln!(out, "- **Suggested remediation:** {r}");
        }
        if !f.evidence.is_empty() {
            let _ = writeln!(out, "- **Evidence / notes:**");
            for e in &f.evidence {
                match (&e.path, &e.detail) {
                    (Some(p), Some(d)) => {
                        let _ = writeln!(out, "  - `{p}` — {d}");
                    }
                    (Some(p), None) => {
                        let _ = writeln!(out, "  - `{p}`");
                    }
                    (None, Some(d)) => {
                        let _ = writeln!(out, "  - {d}");
                    }
                    (None, None) => {}
                }
            }
        }
        let _ = writeln!(
            out,
            "- **Your task:** Implement changes that would make APO report this finding as `Present` or `Enforced` (unless genuinely Not Applicable)."
        );
        let _ = writeln!(out);
    }
}

fn write_finding_bullet(out: &mut String, f: &Finding) {
    let paths: Vec<_> = f
        .evidence
        .iter()
        .filter_map(|e| e.path.as_deref())
        .collect();
    if paths.is_empty() {
        let _ = writeln!(out, "- `{}` — {:?} — {}", f.rule, f.status, f.summary);
    } else {
        let _ = writeln!(
            out,
            "- `{}` — {:?} — {} (evidence: {})",
            f.rule,
            f.status,
            f.summary,
            paths.join(", ")
        );
    }
}

fn yn(v: bool) -> &'static str {
    if v { "yes" } else { "no" }
}

/// Write the evidence LLM remediation prompt to `path`.
pub fn write_evidence_llm_prompt(report: &EvidenceReport, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, render_evidence_llm_prompt(report))?;
    Ok(())
}

/// Resolve where to write the evidence LLM prompt given `--output` and default directory.
pub fn resolve_evidence_prompt_path(
    report: &EvidenceReport,
    output: Option<&Path>,
    default_dir: &Path,
) -> PathBuf {
    let default_name = report.prompt_filename();
    match output {
        None => default_dir.join(&default_name),
        Some(p) if p.is_dir() || p.extension().is_none() => p.join(&default_name),
        Some(p) if p.extension().is_some_and(|e| e == "md" || e == "markdown") => {
            let name = p
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("repository-evidence");
            if name.contains("prompt") {
                p.to_path_buf()
            } else {
                p.with_file_name(format!("{name}-prompt.md"))
            }
        }
        Some(p) => {
            let parent = p.parent().unwrap_or(default_dir);
            parent.join(default_name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai_evidence::{AiArtifact, AiArtifactKind, AiEvidence};
    use crate::evidence::{Category, Confidence, Finding, Status};
    use crate::knowledge::{KnowledgeEvidence, KnowledgeKind};
    use crate::report::RepositoryEvidenceMeta;

    fn sample_evidence_report() -> EvidenceReport {
        EvidenceReport {
            apo_version: "0.2.0".into(),
            analyzer: "repository-evidence".into(),
            repository: "/tmp/demo".into(),
            checkout_path: None,
            source_uri: None,
            generated_at: "2026-01-01T00:00:00Z".into(),
            executive_summary: "Knowledge maturity 40/100. AI maturity 10/100.".into(),
            repository_evidence: RepositoryEvidenceMeta {
                label: "/tmp/demo".into(),
                file_count: 10,
                analyzers: vec!["knowledge".into(), "ai".into()],
            },
            knowledge: KnowledgeEvidence {
                artifacts: vec![],
                kinds_present: vec![KnowledgeKind::Readme],
                kinds_missing: vec![KnowledgeKind::Adr, KnowledgeKind::Runbook],
                coverage_score: 20.0,
                freshness_score: 100.0,
                broken_links: vec![],
                findings: vec![
                    Finding::builder("knowledge.coverage", Category::Documentation)
                        .status(Status::Partial)
                        .confidence(Confidence::High)
                        .summary("Missing ADRs and runbooks.")
                        .remediation("Add docs/adr and runbooks.")
                        .build(),
                ],
                graph: Default::default(),
                risks: vec!["No ADRs.".into()],
                maturity_score: 40.0,
            },
            ai: AiEvidence {
                artifacts: vec![AiArtifact {
                    path: "prompts/review.md".into(),
                    kind: AiArtifactKind::PromptLibrary,
                    detail: None,
                }],
                findings: vec![
                    Finding::builder("ai.governance", Category::Collaboration)
                        .status(Status::Missing)
                        .confidence(Confidence::High)
                        .summary("No AI governance docs.")
                        .remediation("Add AI governance documentation.")
                        .build(),
                    Finding::builder("ai.prompt_library", Category::Documentation)
                        .status(Status::Present)
                        .confidence(Confidence::High)
                        .summary("Prompt library present.")
                        .build(),
                ],
                prompt_count: 1,
                prompt_versioning: false,
                governance_present: false,
                human_review_signals: false,
                workflow_adoption: false,
                maturity_score: 10.0,
                risks: vec!["Prompts without governance.".into()],
            },
            knowledge_maturity: 40.0,
            ai_maturity: 10.0,
            knowledge_risks: vec!["No ADRs.".into()],
            ai_risks: vec!["Prompts without governance.".into()],
            ollama: None,
        }
    }

    #[test]
    fn evidence_prompt_includes_gaps_and_present() {
        let text = render_evidence_llm_prompt(&sample_evidence_report());
        assert!(text.contains("knowledge & AI evidence remediation"));
        assert!(text.contains("knowledge.coverage"));
        assert!(text.contains("ai.governance"));
        assert!(text.contains("prompts/review.md"));
        assert!(text.contains("Add AI governance"));
        assert!(text.contains("Already present") || text.contains("do not redo"));
    }
}
