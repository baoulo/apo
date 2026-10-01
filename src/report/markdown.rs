//! Markdown report writer.

use std::fmt::Write as _;
use std::path::Path;

use crate::error::Result;
use crate::evidence::Status;
use crate::report::Report;

pub fn write_markdown(report: &Report, path: &Path) -> Result<()> {
    std::fs::write(path, render_markdown(report))?;
    Ok(())
}

pub fn render_markdown(report: &Report) -> String {
    let mut out = String::new();

    let _ = writeln!(out, "# Repository Hygiene Report");
    let _ = writeln!(out);
    let _ = writeln!(out, "- **Analyzer:** {}", report.analyzer);
    let _ = writeln!(out, "- **APO version:** {}", report.apo_version);
    let _ = writeln!(out, "- **Repository:** `{}`", report.repository);
    if let Some(uri) = &report.source_uri {
        let _ = writeln!(out, "- **Source URI:** `{uri}`");
    }
    if let Some(path) = &report.checkout_path
        && report.source_uri.is_some()
    {
        let _ = writeln!(out, "- **Checkout path:** `{path}`");
    }
    let _ = writeln!(out, "- **Generated:** {}", report.generated_at);
    if let Some(score) = report.policy.overall_score {
        let _ = writeln!(out, "- **Overall score:** {score:.1}/100");
    }
    let _ = writeln!(out);

    let _ = writeln!(out, "## Executive summary");
    let _ = writeln!(out);
    let _ = writeln!(out, "{}", report.executive_summary);
    let _ = writeln!(out);

    let _ = writeln!(out, "## Category scores");
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "| Category | Weight | Score | Contribution | Enforced | Present | Partial | Missing | Unknown |"
    );
    let _ = writeln!(
        out,
        "|----------|-------:|------:|-------------:|---------:|--------:|--------:|--------:|--------:|"
    );
    for c in &report.policy.categories {
        let score = c
            .score
            .map(|s| format!("{s:.1}"))
            .unwrap_or_else(|| "n/a".into());
        let contrib = c
            .weighted_contribution
            .map(|s| format!("{s:.1}"))
            .unwrap_or_else(|| "—".into());
        let _ = writeln!(
            out,
            "| {} | {:.0}% | {} | {} | {} | {} | {} | {} | {} |",
            c.name,
            c.weight * 100.0,
            score,
            contrib,
            c.enforced,
            c.present,
            c.partial,
            c.missing,
            c.unknown
        );
    }
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "_Rubric `{}`: weighted overall from category scores. Notes: {}._",
        report.policy.rubric,
        report
            .policy
            .categories
            .iter()
            .map(|c| format!("{} — {}", c.name, c.note))
            .collect::<Vec<_>>()
            .join("; ")
    );
    let _ = writeln!(out);

    let _ = writeln!(out, "## Findings");
    let _ = writeln!(out);
    for f in &report.findings {
        let _ = writeln!(out, "### `{}`", f.rule);
        let _ = writeln!(out);
        let _ = writeln!(out, "- **Category:** {}", f.category.display_name());
        let _ = writeln!(out, "- **Status:** {:?}", f.status);
        let _ = writeln!(out, "- **Confidence:** {:?}", f.confidence);
        let _ = writeln!(out, "- **Summary:** {}", f.summary);
        if let Some(r) = &f.remediation {
            let _ = writeln!(out, "- **Remediation:** {r}");
        }
        if !f.evidence.is_empty() {
            let _ = writeln!(out, "- **Evidence:**");
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
        let _ = writeln!(out);
    }

    let _ = writeln!(out, "## Missing controls");
    let _ = writeln!(out);
    if report.missing_controls.is_empty() {
        let _ = writeln!(out, "No gap signals recorded.");
    } else {
        for g in &report.missing_controls {
            let _ = writeln!(out, "- `{g}`");
        }
    }
    let _ = writeln!(out);

    let _ = writeln!(out, "## Recommendations");
    let _ = writeln!(out);
    if report.recommendations.is_empty() {
        let _ = writeln!(out, "No recommendations.");
    } else {
        for r in &report.recommendations {
            let _ = writeln!(out, "- {r}");
        }
    }
    let _ = writeln!(out);

    if !report.transparency.is_empty() {
        let _ = writeln!(out, "## Transparency");
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "_Adjustments that affect which checks run or which pack tooling applies._"
        );
        let _ = writeln!(out);
        if !report.transparency.disabled_rules.is_empty() {
            let _ = writeln!(out, "### Disabled rules");
            let _ = writeln!(out);
            for id in &report.transparency.disabled_rules {
                let _ = writeln!(out, "- `{id}`");
            }
            let _ = writeln!(out);
        }
        if !report.transparency.overridden_packs.is_empty() {
            let _ = writeln!(out, "### Overridden packs");
            let _ = writeln!(out);
            for id in &report.transparency.overridden_packs {
                let _ = writeln!(
                    out,
                    "- `{id}` — built-in pack replaced by an external pack with the same id"
                );
            }
            let _ = writeln!(out);
        }
        if !report.transparency.skipped_tooling.is_empty() {
            let _ = writeln!(out, "### Skipped tooling");
            let _ = writeln!(out);
            for s in &report.transparency.skipped_tooling {
                let _ = writeln!(
                    out,
                    "- `{}.{}` (`maps_to`=`{}`) — {}",
                    s.source, s.id, s.maps_to, s.reason
                );
            }
            let _ = writeln!(out);
        }
        if !report.transparency.not_applicable_rules.is_empty() {
            let _ = writeln!(out, "### Not applicable");
            let _ = writeln!(out);
            for id in &report.transparency.not_applicable_rules {
                let _ = writeln!(out, "- `{id}`");
            }
            let _ = writeln!(out);
        }
    }

    let _ = writeln!(out, "## Evidence appendix");
    let _ = writeln!(out);
    let _ = writeln!(out, "| Rule | Status | Paths |");
    let _ = writeln!(out, "|------|--------|-------|");
    for f in &report.findings {
        let paths: Vec<_> = f
            .evidence
            .iter()
            .filter_map(|e| e.path.as_deref())
            .collect();
        let path_str = if paths.is_empty() {
            "—".into()
        } else {
            paths.join(", ")
        };
        let status = match f.status {
            Status::Enforced => "Enforced",
            Status::Present => "Present",
            Status::Partial => "Partial",
            Status::Missing => "Missing",
            Status::NotApplicable => "NotApplicable",
            Status::Unknown => "Unknown",
        };
        let _ = writeln!(out, "| `{}` | {status} | {path_str} |", f.rule);
    }
    let _ = writeln!(out);

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::{Category, Confidence, Finding, Status};
    use crate::policy;
    use crate::report::{Report, SkippedToolingNote, Transparency};

    #[test]
    fn markdown_includes_transparency_sections() {
        let findings = vec![
            Finding::builder("documentation.readme", Category::Documentation)
                .status(Status::Present)
                .confidence(Confidence::High)
                .summary("README present.")
                .build(),
        ];
        let report = Report {
            apo_version: "0.3.0".into(),
            analyzer: "repository-hygiene".into(),
            repository: "/tmp/demo".into(),
            checkout_path: None,
            source_uri: None,
            generated_at: "2026-01-01T00:00:00Z".into(),
            executive_summary: "test".into(),
            policy: policy::evaluate(&findings),
            missing_controls: vec![],
            recommendations: vec![],
            findings,
            transparency: Transparency {
                disabled_rules: vec!["documentation.license".into()],
                overridden_packs: vec!["rust".into()],
                skipped_tooling: vec![SkippedToolingNote {
                    source: "crystal".into(),
                    id: "bogus".into(),
                    maps_to: "not.a.rule".into(),
                    reason: "unknown maps_to".into(),
                }],
                not_applicable_rules: vec![],
            },
        };
        let md = render_markdown(&report);
        assert!(md.contains("## Transparency"));
        assert!(md.contains("### Disabled rules"));
        assert!(md.contains("`documentation.license`"));
        assert!(md.contains("### Overridden packs"));
        assert!(md.contains("`rust`"));
        assert!(md.contains("### Skipped tooling"));
        assert!(md.contains("crystal.bogus"));
    }
}
