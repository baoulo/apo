//! Knowledge evidence: documentation inventory, freshness, links, and graph.

mod graph;
mod links;

pub use graph::{EdgeKind, KnowledgeEdge, KnowledgeGraph, KnowledgeNode, NodeKind};

use std::collections::BTreeSet;
use std::path::Path;
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::discovery::RepoContext;
use crate::evidence::{Confidence, EvidenceItem, Finding, Status};

/// Knowledge artifact kinds discovered deterministically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeKind {
    Readme,
    Adr,
    Architecture,
    Design,
    BusinessRules,
    Runbook,
    Api,
    Diagram,
    Onboarding,
    Glossary,
    PromptLibrary,
    OtherDoc,
}

impl KnowledgeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Readme => "readme",
            Self::Adr => "adr",
            Self::Architecture => "architecture",
            Self::Design => "design",
            Self::BusinessRules => "business_rules",
            Self::Runbook => "runbook",
            Self::Api => "api",
            Self::Diagram => "diagram",
            Self::Onboarding => "onboarding",
            Self::Glossary => "glossary",
            Self::PromptLibrary => "prompt_library",
            Self::OtherDoc => "other_doc",
        }
    }
}

/// A discovered knowledge artifact with observable metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeArtifact {
    pub path: String,
    pub kind: KnowledgeKind,
    /// Days since last modification (git commit mtime when available, else filesystem).
    pub days_since_modified: Option<u64>,
    pub stale: bool,
    pub size_bytes: u64,
}

/// Broken local markdown link.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrokenLink {
    pub source: String,
    pub target: String,
    pub detail: String,
}

/// Aggregated knowledge evidence section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEvidence {
    pub artifacts: Vec<KnowledgeArtifact>,
    pub kinds_present: Vec<KnowledgeKind>,
    pub kinds_missing: Vec<KnowledgeKind>,
    pub coverage_score: f64,
    pub freshness_score: f64,
    pub broken_links: Vec<BrokenLink>,
    pub findings: Vec<Finding>,
    pub graph: KnowledgeGraph,
    pub risks: Vec<String>,
    pub maturity_score: f64,
}

/// Analyze knowledge artifacts in the repository (deterministic).
pub fn analyze(ctx: &RepoContext) -> KnowledgeEvidence {
    let artifacts = discover_artifacts(ctx);
    let kinds_present: BTreeSet<_> = artifacts.iter().map(|a| a.kind).collect();
    let expected = expected_kinds();
    let kinds_missing: Vec<_> = expected
        .iter()
        .copied()
        .filter(|k| !kinds_present.contains(k))
        .collect();

    let coverage_score = if expected.is_empty() {
        0.0
    } else {
        (kinds_present.len() as f64 / expected.len() as f64) * 100.0
    };

    let freshness_score = compute_freshness_score(&artifacts);
    let broken_links = links::find_broken_markdown_links(ctx);
    let graph = graph::build_graph(ctx, &artifacts);
    let findings = build_findings(&artifacts, &kinds_missing, &broken_links, freshness_score);
    let risks = build_risks(&kinds_missing, &broken_links, &artifacts);
    let maturity_score = (coverage_score * 0.45)
        + (freshness_score * 0.25)
        + (link_health_score(&broken_links) * 0.15)
        + (ownership_bonus(ctx) * 0.15);

    KnowledgeEvidence {
        artifacts,
        kinds_present: kinds_present.into_iter().collect(),
        kinds_missing,
        coverage_score,
        freshness_score,
        broken_links,
        findings,
        graph,
        risks,
        maturity_score: maturity_score.clamp(0.0, 100.0),
    }
}

fn expected_kinds() -> Vec<KnowledgeKind> {
    vec![
        KnowledgeKind::Readme,
        KnowledgeKind::Adr,
        KnowledgeKind::Architecture,
        KnowledgeKind::Design,
        KnowledgeKind::Runbook,
        KnowledgeKind::Api,
        KnowledgeKind::Onboarding,
        KnowledgeKind::Glossary,
        KnowledgeKind::PromptLibrary,
    ]
}

fn discover_artifacts(ctx: &RepoContext) -> Vec<KnowledgeArtifact> {
    let mut out = Vec::new();
    for entry in ctx.inventory.iter() {
        let Some(kind) = classify_path(&entry.relative) else {
            continue;
        };
        let days = days_since_modified(&entry.absolute);
        let stale = days.is_some_and(|d| d > 180);
        out.push(KnowledgeArtifact {
            path: entry.relative.clone(),
            kind,
            days_since_modified: days,
            stale,
            size_bytes: entry.size,
        });
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

fn classify_path(rel: &str) -> Option<KnowledgeKind> {
    let l = rel.to_ascii_lowercase().replace('\\', "/");
    let name = Path::new(&l)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    if name.starts_with("readme") {
        return Some(KnowledgeKind::Readme);
    }
    if (l.contains("/adr/")
        || l.contains("/adrs/")
        || name.starts_with("adr-")
        || name.starts_with("adr_"))
        && is_doc_ext(&l)
    {
        return Some(KnowledgeKind::Adr);
    }
    if l.contains("architect") && is_doc_ext(&l) {
        return Some(KnowledgeKind::Architecture);
    }
    if (l.contains("design") || name == "design.md") && is_doc_ext(&l) {
        return Some(KnowledgeKind::Design);
    }
    if (l.contains("business-rule") || l.contains("business_rule") || l.contains("domain-rule"))
        && is_doc_ext(&l)
    {
        return Some(KnowledgeKind::BusinessRules);
    }
    if l.contains("runbook") && is_doc_ext(&l) {
        return Some(KnowledgeKind::Runbook);
    }
    if (l.contains("/api/") || l.contains("openapi") || l.contains("swagger") || name == "api.md")
        && (is_doc_ext(&l) || l.ends_with(".yaml") || l.ends_with(".yml") || l.ends_with(".json"))
    {
        return Some(KnowledgeKind::Api);
    }
    if is_diagram(&l) {
        return Some(KnowledgeKind::Diagram);
    }
    if (l.contains("onboard")
        || l.contains("getting-started")
        || l.contains("getting_started")
        || name.starts_with("contributing"))
        && is_doc_ext(&l)
    {
        return Some(KnowledgeKind::Onboarding);
    }
    if (l.contains("glossary") || name == "glossary.md") && is_doc_ext(&l) {
        return Some(KnowledgeKind::Glossary);
    }
    if is_prompt_library_path(&l) {
        return Some(KnowledgeKind::PromptLibrary);
    }
    // Generic docs under docs/ that weren't classified
    if (l.starts_with("docs/") || l.starts_with("doc/")) && is_doc_ext(&l) {
        return Some(KnowledgeKind::OtherDoc);
    }
    None
}

fn is_doc_ext(l: &str) -> bool {
    l.ends_with(".md")
        || l.ends_with(".mdx")
        || l.ends_with(".rst")
        || l.ends_with(".adoc")
        || l.ends_with(".txt")
}

fn is_diagram(l: &str) -> bool {
    l.ends_with(".mmd")
        || l.ends_with(".puml")
        || l.ends_with(".plantuml")
        || l.ends_with(".drawio")
        || l.ends_with(".svg") && (l.contains("diagram") || l.contains("architecture"))
        || l.contains("diagram") && (is_doc_ext(l) || l.ends_with(".png") || l.ends_with(".svg"))
}

fn is_prompt_library_path(l: &str) -> bool {
    l.contains("/prompts/")
        || l.starts_with("prompts/")
        || l.contains("baoulo/prompts")
        || l.contains(".cursor/prompts")
        || l.contains(".github/prompts")
        || (l.contains("prompt") && is_doc_ext(l) && l.contains('/'))
}

fn days_since_modified(path: &Path) -> Option<u64> {
    let meta = std::fs::metadata(path).ok()?;
    let modified = meta.modified().ok()?;
    let now = SystemTime::now();
    let dur = now.duration_since(modified).ok()?;
    Some(dur.as_secs() / 86_400)
}

fn compute_freshness_score(artifacts: &[KnowledgeArtifact]) -> f64 {
    if artifacts.is_empty() {
        return 0.0;
    }
    let mut sum = 0.0;
    let mut n = 0usize;
    for a in artifacts {
        let Some(days) = a.days_since_modified else {
            continue;
        };
        let score = if days <= 30 {
            100.0
        } else if days <= 90 {
            80.0
        } else if days <= 180 {
            55.0
        } else if days <= 365 {
            30.0
        } else {
            10.0
        };
        sum += score;
        n += 1;
    }
    if n == 0 { 50.0 } else { sum / n as f64 }
}

fn link_health_score(broken: &[BrokenLink]) -> f64 {
    if broken.is_empty() {
        100.0
    } else if broken.len() <= 3 {
        70.0
    } else if broken.len() <= 10 {
        40.0
    } else {
        15.0
    }
}

fn ownership_bonus(ctx: &RepoContext) -> f64 {
    if ctx.has_file("CODEOWNERS")
        || ctx.has_file(".github/CODEOWNERS")
        || ctx.has_file("docs/CODEOWNERS")
    {
        100.0
    } else {
        0.0
    }
}

fn build_findings(
    artifacts: &[KnowledgeArtifact],
    missing: &[KnowledgeKind],
    broken: &[BrokenLink],
    freshness: f64,
) -> Vec<Finding> {
    use crate::evidence::Category;

    let mut findings = Vec::new();

    // Coverage finding
    let present: Vec<_> = artifacts.iter().map(|a| a.path.clone()).collect();
    if missing.is_empty() {
        findings.push(
            Finding::builder("knowledge.coverage", Category::Documentation)
                .status(Status::Enforced)
                .confidence(Confidence::High)
                .summary("Core knowledge artifact kinds are present.")
                .evidence(
                    present
                        .into_iter()
                        .take(12)
                        .map(EvidenceItem::path)
                        .collect(),
                )
                .build(),
        );
    } else {
        let miss: Vec<_> = missing.iter().map(|k| k.as_str().to_string()).collect();
        findings.push(
            Finding::builder("knowledge.coverage", Category::Documentation)
                .status(Status::Partial)
                .confidence(Confidence::High)
                .summary(format!(
                    "Knowledge coverage incomplete; missing kinds: {}.",
                    miss.join(", ")
                ))
                .push_evidence(EvidenceItem::detail(format!("missing: {}", miss.join(", "))))
                .remediation("Add documentation for missing knowledge kinds (ADRs, runbooks, API docs, etc.).")
                .build(),
        );
    }

    // Freshness
    let stale: Vec<_> = artifacts.iter().filter(|a| a.stale).collect();
    if artifacts.is_empty() {
        findings.push(
            Finding::builder("knowledge.freshness", Category::Documentation)
                .status(Status::Missing)
                .confidence(Confidence::Medium)
                .summary("No knowledge artifacts found to assess freshness.")
                .build(),
        );
    } else if stale.is_empty() && freshness >= 70.0 {
        findings.push(
            Finding::builder("knowledge.freshness", Category::Documentation)
                .status(Status::Enforced)
                .confidence(Confidence::Medium)
                .summary("Documentation appears recently maintained.")
                .build(),
        );
    } else {
        let mut b = Finding::builder("knowledge.freshness", Category::Documentation)
            .status(Status::Partial)
            .confidence(Confidence::Medium)
            .summary(format!(
                "{} stale knowledge artifact(s) (>180 days); freshness score {:.0}/100.",
                stale.len(),
                freshness
            ))
            .remediation("Refresh stale docs or archive intentionally obsolete material.");
        for s in stale.into_iter().take(8) {
            b = b.push_evidence(EvidenceItem::path_detail(
                &s.path,
                format!("{} days since modified", s.days_since_modified.unwrap_or(0)),
            ));
        }
        findings.push(b.build());
    }

    // Broken links
    if broken.is_empty() {
        findings.push(
            Finding::builder("knowledge.links", Category::Documentation)
                .status(Status::Enforced)
                .confidence(Confidence::High)
                .summary("No broken local markdown links detected in sampled docs.")
                .build(),
        );
    } else {
        let mut b = Finding::builder("knowledge.links", Category::Documentation)
            .status(Status::Partial)
            .confidence(Confidence::High)
            .summary(format!(
                "{} broken local markdown link(s) detected.",
                broken.len()
            ))
            .remediation("Fix or remove broken relative links in documentation.");
        for link in broken.iter().take(15) {
            b = b.push_evidence(EvidenceItem::path_detail(
                &link.source,
                format!("→ {} ({})", link.target, link.detail),
            ));
        }
        findings.push(b.build());
    }

    // Architecture coverage
    let has_arch = artifacts
        .iter()
        .any(|a| matches!(a.kind, KnowledgeKind::Architecture | KnowledgeKind::Adr));
    if has_arch {
        findings.push(
            Finding::builder("knowledge.architecture", Category::Documentation)
                .status(Status::Present)
                .confidence(Confidence::High)
                .summary("Architecture and/or ADR documentation detected.")
                .evidence(
                    artifacts
                        .iter()
                        .filter(|a| {
                            matches!(a.kind, KnowledgeKind::Architecture | KnowledgeKind::Adr)
                        })
                        .take(8)
                        .map(|a| EvidenceItem::path(&a.path))
                        .collect(),
                )
                .build(),
        );
    } else {
        findings.push(
            Finding::builder("knowledge.architecture", Category::Documentation)
                .status(Status::Missing)
                .confidence(Confidence::High)
                .summary("No architecture docs or ADRs detected.")
                .remediation("Add ARCHITECTURE.md and/or docs/adr/ decision records.")
                .build(),
        );
    }

    // Operational readiness (runbooks)
    let runbooks: Vec<_> = artifacts
        .iter()
        .filter(|a| a.kind == KnowledgeKind::Runbook)
        .collect();
    if runbooks.is_empty() {
        findings.push(
            Finding::builder("knowledge.operational_readiness", Category::Documentation)
                .status(Status::Missing)
                .confidence(Confidence::Medium)
                .summary("No runbooks detected.")
                .remediation("Add operational runbooks under docs/runbooks/.")
                .build(),
        );
    } else {
        findings.push(
            Finding::builder("knowledge.operational_readiness", Category::Documentation)
                .status(Status::Present)
                .confidence(Confidence::High)
                .summary("Runbook documentation detected.")
                .evidence(
                    runbooks
                        .into_iter()
                        .take(8)
                        .map(|a| EvidenceItem::path(&a.path))
                        .collect(),
                )
                .build(),
        );
    }

    findings
}

fn build_risks(
    missing: &[KnowledgeKind],
    broken: &[BrokenLink],
    artifacts: &[KnowledgeArtifact],
) -> Vec<String> {
    let mut risks = Vec::new();
    if missing.contains(&KnowledgeKind::Adr) {
        risks.push("No ADRs — architectural decisions may be tacit and hard to onboard.".into());
    }
    if missing.contains(&KnowledgeKind::Runbook) {
        risks.push(
            "No runbooks — operational incident response may rely on tribal knowledge.".into(),
        );
    }
    if missing.contains(&KnowledgeKind::Architecture) {
        risks.push("No architecture docs — system boundaries may be unclear.".into());
    }
    if broken.len() > 5 {
        risks.push(format!(
            "High broken-link count ({}) — documentation trust and navigation degraded.",
            broken.len()
        ));
    }
    let stale = artifacts.iter().filter(|a| a.stale).count();
    if stale > 5 {
        risks.push(format!(
            "{stale} stale docs (>180 days) — knowledge may no longer match the codebase."
        ));
    }
    if artifacts.is_empty() {
        risks.push(
            "No knowledge artifacts discovered — repository knowledge is effectively absent."
                .into(),
        );
    }
    risks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_common_paths() {
        assert_eq!(classify_path("README.md"), Some(KnowledgeKind::Readme));
        assert_eq!(
            classify_path("docs/adr/0001-record.md"),
            Some(KnowledgeKind::Adr)
        );
        assert_eq!(
            classify_path("docs/runbooks/restart.md"),
            Some(KnowledgeKind::Runbook)
        );
        assert_eq!(
            classify_path("baoulo/prompts/foo.md"),
            Some(KnowledgeKind::PromptLibrary)
        );
    }
}
