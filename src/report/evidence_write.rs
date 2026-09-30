//! Markdown + JSON writers for evidence reports.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::report::EvidenceReport;

pub fn write_evidence_json(report: &EvidenceReport, path: &Path) -> Result<()> {
    let file = std::fs::File::create(path)?;
    serde_json::to_writer_pretty(file, report)?;
    Ok(())
}

pub fn evidence_json_to_string(report: &EvidenceReport) -> Result<String> {
    Ok(serde_json::to_string_pretty(report)?)
}

pub fn write_evidence_markdown(report: &EvidenceReport, path: &Path) -> Result<()> {
    std::fs::write(path, render_evidence_markdown(report))?;
    Ok(())
}

pub fn render_evidence_markdown(report: &EvidenceReport) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "# Repository Evidence Report");
    let _ = writeln!(out);
    let _ = writeln!(out, "- **Analyzer:** {}", report.analyzer);
    let _ = writeln!(out, "- **APO version:** {}", report.apo_version);
    let _ = writeln!(out, "- **Repository:** `{}`", report.repository);
    if let Some(uri) = &report.source_uri {
        let _ = writeln!(out, "- **Source URI:** `{uri}`");
    }
    let _ = writeln!(out, "- **Generated:** {}", report.generated_at);
    let _ = writeln!(
        out,
        "- **Knowledge maturity:** {:.1}/100",
        report.knowledge_maturity
    );
    let _ = writeln!(out, "- **AI maturity:** {:.1}/100", report.ai_maturity);
    let _ = writeln!(out);

    let _ = writeln!(out, "## Executive summary");
    let _ = writeln!(out);
    let _ = writeln!(out, "{}", report.executive_summary);
    let _ = writeln!(out);

    let _ = writeln!(out, "## Repository evidence");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "- **Files inventoried:** {}",
        report.repository_evidence.file_count
    );
    let _ = writeln!(
        out,
        "- **Analyzers:** {}",
        report.repository_evidence.analyzers.join(", ")
    );
    let _ = writeln!(out);

    let _ = writeln!(out, "## Knowledge evidence");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "- **Coverage score:** {:.1}/100",
        report.knowledge.coverage_score
    );
    let _ = writeln!(
        out,
        "- **Freshness score:** {:.1}/100",
        report.knowledge.freshness_score
    );
    let _ = writeln!(out, "- **Artifacts:** {}", report.knowledge.artifacts.len());
    let _ = writeln!(
        out,
        "- **Broken local links:** {}",
        report.knowledge.broken_links.len()
    );
    let _ = writeln!(
        out,
        "- **Graph:** {} nodes, {} edges",
        report.knowledge.graph.nodes.len(),
        report.knowledge.graph.edges.len()
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "### Knowledge kinds present");
    let _ = writeln!(out);
    if report.knowledge.kinds_present.is_empty() {
        let _ = writeln!(out, "_None_");
    } else {
        for k in &report.knowledge.kinds_present {
            let _ = writeln!(out, "- `{}`", k.as_str());
        }
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "### Knowledge kinds missing");
    let _ = writeln!(out);
    if report.knowledge.kinds_missing.is_empty() {
        let _ = writeln!(out, "_None_");
    } else {
        for k in &report.knowledge.kinds_missing {
            let _ = writeln!(out, "- `{}`", k.as_str());
        }
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "### Knowledge artifacts");
    let _ = writeln!(out);
    let _ = writeln!(out, "| Path | Kind | Stale | Days |");
    let _ = writeln!(out, "|------|------|:-----:|-----:|");
    for a in report.knowledge.artifacts.iter().take(40) {
        let days = a
            .days_since_modified
            .map(|d| d.to_string())
            .unwrap_or_else(|| "—".into());
        let _ = writeln!(
            out,
            "| `{}` | {} | {} | {} |",
            a.path,
            a.kind.as_str(),
            if a.stale { "yes" } else { "no" },
            days
        );
    }
    if report.knowledge.artifacts.len() > 40 {
        let _ = writeln!(
            out,
            "\n_…and {} more._",
            report.knowledge.artifacts.len() - 40
        );
    }
    let _ = writeln!(out);

    let _ = writeln!(out, "### Knowledge findings");
    let _ = writeln!(out);
    for f in &report.knowledge.findings {
        let _ = writeln!(out, "#### `{}`", f.rule);
        let _ = writeln!(out);
        let _ = writeln!(out, "- **Status:** {:?}", f.status);
        let _ = writeln!(out, "- **Summary:** {}", f.summary);
        if let Some(r) = &f.remediation {
            let _ = writeln!(out, "- **Remediation:** {r}");
        }
        let _ = writeln!(out);
    }

    let _ = writeln!(out, "## AI evidence");
    let _ = writeln!(out);
    let _ = writeln!(out, "- **Prompt artifacts:** {}", report.ai.prompt_count);
    let _ = writeln!(
        out,
        "- **Prompt versioning:** {}",
        report.ai.prompt_versioning
    );
    let _ = writeln!(
        out,
        "- **Governance present:** {}",
        report.ai.governance_present
    );
    let _ = writeln!(
        out,
        "- **Human review signals:** {}",
        report.ai.human_review_signals
    );
    let _ = writeln!(
        out,
        "- **Workflow adoption:** {}",
        report.ai.workflow_adoption
    );
    let _ = writeln!(out);
    let _ = writeln!(out, "### AI artifacts");
    let _ = writeln!(out);
    let _ = writeln!(out, "| Path | Kind |");
    let _ = writeln!(out, "|------|------|");
    for a in report.ai.artifacts.iter().take(40) {
        let _ = writeln!(out, "| `{}` | {} |", a.path, a.kind.as_str());
    }
    let _ = writeln!(out);
    let _ = writeln!(out, "### AI findings");
    let _ = writeln!(out);
    for f in &report.ai.findings {
        let _ = writeln!(out, "#### `{}`", f.rule);
        let _ = writeln!(out);
        let _ = writeln!(out, "- **Status:** {:?}", f.status);
        let _ = writeln!(out, "- **Summary:** {}", f.summary);
        if let Some(r) = &f.remediation {
            let _ = writeln!(out, "- **Remediation:** {r}");
        }
        let _ = writeln!(out);
    }

    let _ = writeln!(out, "## Knowledge maturity");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "Overall knowledge maturity **{:.1}/100** (coverage {:.1}, freshness {:.1}).",
        report.knowledge_maturity,
        report.knowledge.coverage_score,
        report.knowledge.freshness_score
    );
    let _ = writeln!(out);

    let _ = writeln!(out, "## AI maturity");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "Overall AI maturity **{:.1}/100**.",
        report.ai_maturity
    );
    let _ = writeln!(out);

    let _ = writeln!(out, "## Knowledge risks");
    let _ = writeln!(out);
    if report.knowledge_risks.is_empty() {
        let _ = writeln!(out, "_No knowledge risks flagged._");
    } else {
        for r in &report.knowledge_risks {
            let _ = writeln!(out, "- {r}");
        }
    }
    let _ = writeln!(out);

    let _ = writeln!(out, "## AI risks");
    let _ = writeln!(out);
    if report.ai_risks.is_empty() {
        let _ = writeln!(out, "_No AI risks flagged._");
    } else {
        for r in &report.ai_risks {
            let _ = writeln!(out, "- {r}");
        }
    }
    let _ = writeln!(out);

    if let Some(o) = &report.ollama {
        let _ = writeln!(out, "## Ollama enrichment");
        let _ = writeln!(out);
        let _ = writeln!(out, "- **Enabled:** {}", o.enabled);
        if let Some(m) = &o.model {
            let _ = writeln!(out, "- **Model:** `{m}`");
        }
        for n in &o.notes {
            let _ = writeln!(out, "- **Note:** {n}");
        }
        if !o.classifications.is_empty() {
            let _ = writeln!(out);
            let _ = writeln!(out, "### Classifications");
            let _ = writeln!(out);
            for c in &o.classifications {
                let _ = writeln!(out, "- `{}` → **{}** — {}", c.path, c.kind, c.rationale);
            }
        }
        if let Some(s) = &o.architecture_summary {
            let _ = writeln!(out);
            let _ = writeln!(out, "### Architecture summary");
            let _ = writeln!(out);
            let _ = writeln!(out, "{}", s.summary);
            let _ = writeln!(out);
            let _ = writeln!(out, "Evidence: {}", s.evidence_paths.join(", "));
        }
        if !o.suggested_links.is_empty() {
            let _ = writeln!(out);
            let _ = writeln!(out, "### Suggested doc↔code links");
            let _ = writeln!(out);
            for l in &o.suggested_links {
                let _ = writeln!(out, "- `{}` → `{}` — {}", l.from, l.to, l.rationale);
            }
        }
        let _ = writeln!(out);
    }

    let _ = writeln!(out, "## Knowledge graph (sample)");
    let _ = writeln!(out);
    let _ = writeln!(out, "| From | To | Kind |");
    let _ = writeln!(out, "|------|----|------|");
    for e in report.knowledge.graph.edges.iter().take(30) {
        let _ = writeln!(out, "| `{}` | `{}` | {:?} |", e.from, e.to, e.kind);
    }
    let _ = writeln!(out);

    out
}

/// Write evidence report artifacts for the requested format.
pub fn write_evidence_report(
    report: &EvidenceReport,
    format: crate::config::OutputFormat,
    output: Option<&Path>,
    default_dir: &Path,
) -> Result<Vec<PathBuf>> {
    use crate::config::OutputFormat;

    let targets = match (format, output) {
        (OutputFormat::Markdown, None) => {
            vec![(
                OutputFormat::Markdown,
                default_dir.join(report.markdown_filename()),
            )]
        }
        (OutputFormat::Json, None) => {
            vec![(OutputFormat::Json, default_dir.join(report.json_filename()))]
        }
        (OutputFormat::Both, None) => vec![
            (
                OutputFormat::Markdown,
                default_dir.join(report.markdown_filename()),
            ),
            (OutputFormat::Json, default_dir.join(report.json_filename())),
        ],
        (OutputFormat::Markdown, Some(p)) if p.is_dir() || p.extension().is_none() => {
            vec![(OutputFormat::Markdown, p.join(report.markdown_filename()))]
        }
        (OutputFormat::Markdown, Some(p)) => vec![(OutputFormat::Markdown, p.to_path_buf())],
        (OutputFormat::Json, Some(p)) if p.is_dir() || p.extension().is_none() => {
            vec![(OutputFormat::Json, p.join(report.json_filename()))]
        }
        (OutputFormat::Json, Some(p)) => vec![(OutputFormat::Json, p.to_path_buf())],
        (OutputFormat::Both, Some(p)) if p.is_dir() || p.extension().is_none() => vec![
            (OutputFormat::Markdown, p.join(report.markdown_filename())),
            (OutputFormat::Json, p.join(report.json_filename())),
        ],
        (OutputFormat::Both, Some(p)) => {
            let parent = p.parent().unwrap_or(default_dir);
            let stem = p
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("repository-evidence");
            vec![
                (OutputFormat::Markdown, parent.join(format!("{stem}.md"))),
                (OutputFormat::Json, parent.join(format!("{stem}.json"))),
            ]
        }
    };

    let mut written = Vec::new();
    for (fmt, path) in targets {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        match fmt {
            OutputFormat::Markdown => write_evidence_markdown(report, &path)?,
            OutputFormat::Json => write_evidence_json(report, &path)?,
            OutputFormat::Both => unreachable!(),
        }
        written.push(path);
    }
    Ok(written)
}
