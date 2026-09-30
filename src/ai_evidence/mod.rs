//! AI evidence: prompts, MCP, agents, governance, and AI-assisted workflows.

use serde::{Deserialize, Serialize};

use crate::discovery::RepoContext;
use crate::evidence::{Category, Confidence, EvidenceItem, Finding, Status};
use crate::rules::helpers;

/// Categories of AI-related artifacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiArtifactKind {
    PromptLibrary,
    PromptTemplate,
    McpConfig,
    AgentDefinition,
    AiCodingStandard,
    AiGovernance,
    AiReviewWorkflow,
    AiAssistedDocs,
    AiAssistedTesting,
    AiAssistedRelease,
}

impl AiArtifactKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PromptLibrary => "prompt_library",
            Self::PromptTemplate => "prompt_template",
            Self::McpConfig => "mcp_config",
            Self::AgentDefinition => "agent_definition",
            Self::AiCodingStandard => "ai_coding_standard",
            Self::AiGovernance => "ai_governance",
            Self::AiReviewWorkflow => "ai_review_workflow",
            Self::AiAssistedDocs => "ai_assisted_docs",
            Self::AiAssistedTesting => "ai_assisted_testing",
            Self::AiAssistedRelease => "ai_assisted_release",
        }
    }
}

/// Discovered AI artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiArtifact {
    pub path: String,
    pub kind: AiArtifactKind,
    pub detail: Option<String>,
}

/// Aggregated AI evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiEvidence {
    pub artifacts: Vec<AiArtifact>,
    pub findings: Vec<Finding>,
    pub prompt_count: usize,
    pub prompt_versioning: bool,
    pub governance_present: bool,
    pub human_review_signals: bool,
    pub workflow_adoption: bool,
    pub maturity_score: f64,
    pub risks: Vec<String>,
}

/// Analyze AI adoption and governance evidence (deterministic).
pub fn analyze(ctx: &RepoContext) -> AiEvidence {
    let artifacts = discover(ctx);
    let prompt_count = artifacts
        .iter()
        .filter(|a| {
            matches!(
                a.kind,
                AiArtifactKind::PromptLibrary | AiArtifactKind::PromptTemplate
            )
        })
        .count();
    let prompt_versioning = detect_prompt_versioning(ctx, &artifacts);
    let governance_present = artifacts.iter().any(|a| {
        matches!(
            a.kind,
            AiArtifactKind::AiGovernance | AiArtifactKind::AiCodingStandard
        )
    });
    let human_review_signals = artifacts
        .iter()
        .any(|a| a.kind == AiArtifactKind::AiReviewWorkflow)
        || ctx.has_file(".github/pull_request_template.md")
        || ctx.has_file(".github/PULL_REQUEST_TEMPLATE.md");
    let workflow_adoption = artifacts.iter().any(|a| {
        matches!(
            a.kind,
            AiArtifactKind::AiAssistedDocs
                | AiArtifactKind::AiAssistedTesting
                | AiArtifactKind::AiAssistedRelease
                | AiArtifactKind::AiReviewWorkflow
                | AiArtifactKind::McpConfig
                | AiArtifactKind::AgentDefinition
        )
    });

    let findings = build_findings(
        &artifacts,
        prompt_count,
        prompt_versioning,
        governance_present,
        human_review_signals,
        workflow_adoption,
    );
    let risks = build_risks(
        prompt_count,
        governance_present,
        human_review_signals,
        &artifacts,
    );

    let maturity_score = score_maturity(
        prompt_count,
        prompt_versioning,
        governance_present,
        human_review_signals,
        workflow_adoption,
        &artifacts,
    );

    AiEvidence {
        artifacts,
        findings,
        prompt_count,
        prompt_versioning,
        governance_present,
        human_review_signals,
        workflow_adoption,
        maturity_score,
        risks,
    }
}

fn discover(ctx: &RepoContext) -> Vec<AiArtifact> {
    let mut out = Vec::new();

    for entry in ctx.inventory.iter() {
        let l = entry.relative.to_ascii_lowercase().replace('\\', "/");

        if is_prompt_path(&l) {
            let kind = if l.contains("template") {
                AiArtifactKind::PromptTemplate
            } else {
                AiArtifactKind::PromptLibrary
            };
            out.push(AiArtifact {
                path: entry.relative.clone(),
                kind,
                detail: None,
            });
            continue;
        }

        if is_mcp_path(&l) {
            out.push(AiArtifact {
                path: entry.relative.clone(),
                kind: AiArtifactKind::McpConfig,
                detail: None,
            });
            continue;
        }

        if is_agent_path(&l) {
            out.push(AiArtifact {
                path: entry.relative.clone(),
                kind: AiArtifactKind::AgentDefinition,
                detail: None,
            });
            continue;
        }

        if is_governance_path(&l) {
            out.push(AiArtifact {
                path: entry.relative.clone(),
                kind: AiArtifactKind::AiGovernance,
                detail: None,
            });
            continue;
        }

        if is_ai_standard_path(&l) {
            out.push(AiArtifact {
                path: entry.relative.clone(),
                kind: AiArtifactKind::AiCodingStandard,
                detail: None,
            });
        }
    }

    // CI / workflow AI signals
    let review = helpers::ci_mentions(
        ctx,
        &[
            "ai review",
            "llm review",
            "coderabbit",
            "chatgpt",
            "openai",
            "anthropic",
            "cursor bugbot",
            "gemini",
        ],
    );
    for item in review {
        if let Some(path) = item.path {
            out.push(AiArtifact {
                path,
                kind: AiArtifactKind::AiReviewWorkflow,
                detail: item.detail,
            });
        }
    }

    let docs_ai = helpers::ci_mentions(
        ctx,
        &["llm", "ollama", "ai generated", "chatgpt", "copilot"],
    );
    for item in docs_ai {
        if let Some(path) = item.path.clone() {
            let lower = item.detail.unwrap_or_default().to_ascii_lowercase();
            let kind = if lower.contains("test") {
                AiArtifactKind::AiAssistedTesting
            } else if lower.contains("release") || lower.contains("changelog") {
                AiArtifactKind::AiAssistedRelease
            } else {
                AiArtifactKind::AiAssistedDocs
            };
            out.push(AiArtifact {
                path,
                kind,
                detail: Some(lower),
            });
        }
    }

    out.sort_by(|a, b| (&a.path, a.kind.as_str()).cmp(&(&b.path, b.kind.as_str())));
    out.dedup_by(|a, b| a.path == b.path && a.kind == b.kind);
    out
}

fn is_prompt_path(l: &str) -> bool {
    let name = std::path::Path::new(l)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    l.contains("/prompts/")
        || l.starts_with("prompts/")
        || l.contains("baoulo/prompts")
        || l.contains(".cursor/prompts")
        || l.contains(".github/prompts")
        || ((name.contains("prompt") || l.contains("prompt-"))
            && (l.ends_with(".md")
                || l.ends_with(".txt")
                || l.ends_with(".yaml")
                || l.ends_with(".yml")))
}

fn is_mcp_path(l: &str) -> bool {
    l.ends_with("mcp.json")
        || l.ends_with("mcp.yml")
        || l.ends_with("mcp.yaml")
        || l.contains(".cursor/mcp")
        || l.contains("claude_desktop_config")
        || l.contains("mcp_config")
}

fn is_agent_path(l: &str) -> bool {
    let name = std::path::Path::new(l)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    name == "agents.md"
        || name == "agent.md"
        || l.contains(".cursor/agents")
        || l.contains("/agents/")
        || l.ends_with("agent.yaml")
        || l.ends_with("agent.yml")
        || l.ends_with("agent.json")
}

fn is_governance_path(l: &str) -> bool {
    (l.contains("ai") || l.contains("llm") || l.contains("genai") || l.contains("generative"))
        && (l.contains("govern")
            || l.contains("policy")
            || l.contains("ethics")
            || l.contains("responsible"))
        && (l.ends_with(".md") || l.ends_with(".txt"))
}

fn is_ai_standard_path(l: &str) -> bool {
    (l.contains("ai") || l.contains("copilot") || l.contains("cursor") || l.contains("llm"))
        && (l.contains("standard")
            || l.contains("guideline")
            || l.contains("playbook")
            || l.contains("rules"))
        && l.ends_with(".md")
}

fn detect_prompt_versioning(ctx: &RepoContext, artifacts: &[AiArtifact]) -> bool {
    // Version dirs, CHANGELOG near prompts, or semver in filenames
    for a in artifacts {
        let l = a.path.to_ascii_lowercase();
        if l.contains("/v1/")
            || l.contains("/v2/")
            || l.contains("version")
            || regex_is_match(r"v?\d+\.\d+", &l)
        {
            return true;
        }
        if let Some(content) = ctx.read_text(&a.path) {
            let c = content.to_ascii_lowercase();
            if c.contains("version:") || c.contains("semver") || c.contains("changelog") {
                return true;
            }
        }
    }
    ctx.has_file("prompts/CHANGELOG.md") || ctx.has_file("prompts/VERSION")
}

fn regex_is_match(pat: &str, hay: &str) -> bool {
    Regex::new(pat).ok().is_some_and(|re| re.is_match(hay))
}

use regex::Regex;

fn build_findings(
    artifacts: &[AiArtifact],
    prompt_count: usize,
    versioning: bool,
    governance: bool,
    human_review: bool,
    workflows: bool,
) -> Vec<Finding> {
    let mut findings = Vec::new();

    if prompt_count == 0 {
        findings.push(
            Finding::builder("ai.prompt_library", Category::Collaboration)
                .status(Status::Missing)
                .confidence(Confidence::High)
                .summary("No prompt library or prompt templates detected.")
                .remediation(
                    "Add a versioned prompts/ directory or .cursor/prompts for reusable prompts.",
                )
                .build(),
        );
    } else {
        let status = if versioning {
            Status::Enforced
        } else {
            Status::Present
        };
        findings.push(
            Finding::builder("ai.prompt_library", Category::Collaboration)
                .status(status)
                .confidence(Confidence::High)
                .summary(format!(
                    "{prompt_count} prompt artifact(s) detected; versioning {}.",
                    if versioning {
                        "signals present"
                    } else {
                        "not clearly evidenced"
                    }
                ))
                .evidence(
                    artifacts
                        .iter()
                        .filter(|a| {
                            matches!(
                                a.kind,
                                AiArtifactKind::PromptLibrary | AiArtifactKind::PromptTemplate
                            )
                        })
                        .take(10)
                        .map(|a| EvidenceItem::path(&a.path))
                        .collect(),
                )
                .remediation(if versioning {
                    String::new()
                } else {
                    "Add prompt versioning (semver paths, VERSION file, or changelog).".into()
                })
                .build(),
        );
    }

    let mcp: Vec<_> = artifacts
        .iter()
        .filter(|a| a.kind == AiArtifactKind::McpConfig)
        .collect();
    if mcp.is_empty() {
        findings.push(
            Finding::builder("ai.mcp", Category::Collaboration)
                .status(Status::Missing)
                .confidence(Confidence::Medium)
                .summary("No MCP configuration detected.")
                .remediation("Add MCP config (e.g. .cursor/mcp.json) if agents use tools.")
                .build(),
        );
    } else {
        findings.push(
            Finding::builder("ai.mcp", Category::Collaboration)
                .status(Status::Present)
                .confidence(Confidence::High)
                .summary("MCP configuration detected.")
                .evidence(
                    mcp.into_iter()
                        .map(|a| EvidenceItem::path(&a.path))
                        .collect(),
                )
                .build(),
        );
    }

    let agents: Vec<_> = artifacts
        .iter()
        .filter(|a| a.kind == AiArtifactKind::AgentDefinition)
        .collect();
    if agents.is_empty() {
        findings.push(
            Finding::builder("ai.agents", Category::Collaboration)
                .status(Status::Missing)
                .confidence(Confidence::Medium)
                .summary("No agent definitions detected.")
                .remediation("Add AGENTS.md or agent config describing AI agent roles.")
                .build(),
        );
    } else {
        findings.push(
            Finding::builder("ai.agents", Category::Collaboration)
                .status(Status::Present)
                .confidence(Confidence::High)
                .summary("Agent definition artifact(s) detected.")
                .evidence(
                    agents
                        .into_iter()
                        .map(|a| EvidenceItem::path(&a.path))
                        .collect(),
                )
                .build(),
        );
    }

    if governance {
        findings.push(
            Finding::builder("ai.governance", Category::Collaboration)
                .status(Status::Present)
                .confidence(Confidence::High)
                .summary("AI governance / coding-standard documentation detected.")
                .evidence(
                    artifacts
                        .iter()
                        .filter(|a| {
                            matches!(
                                a.kind,
                                AiArtifactKind::AiGovernance | AiArtifactKind::AiCodingStandard
                            )
                        })
                        .map(|a| EvidenceItem::path(&a.path))
                        .collect(),
                )
                .build(),
        );
    } else {
        findings.push(
            Finding::builder("ai.governance", Category::Collaboration)
                .status(Status::Missing)
                .confidence(Confidence::Medium)
                .summary("No AI governance or AI coding standards documentation detected.")
                .remediation(
                    "Publish AI usage policy / coding standards for assistants and agents.",
                )
                .build(),
        );
    }

    if human_review {
        findings.push(
            Finding::builder("ai.human_review", Category::Collaboration)
                .status(Status::Present)
                .confidence(Confidence::Medium)
                .summary("Human review signals present (AI review workflow and/or PR templates).")
                .build(),
        );
    } else {
        findings.push(
            Finding::builder("ai.human_review", Category::Collaboration)
                .status(Status::Missing)
                .confidence(Confidence::Low)
                .summary("No clear human-review gate for AI-assisted changes detected.")
                .remediation(
                    "Require human PR review for AI-assisted changes; document the policy.",
                )
                .build(),
        );
    }

    if workflows {
        findings.push(
            Finding::builder("ai.workflow_adoption", Category::Collaboration)
                .status(Status::Present)
                .confidence(Confidence::Medium)
                .summary("AI workflow adoption signals detected (CI, MCP, agents, or assisted docs/tests).")
                .build(),
        );
    } else {
        findings.push(
            Finding::builder("ai.workflow_adoption", Category::Collaboration)
                .status(Status::Missing)
                .confidence(Confidence::Medium)
                .summary("Limited AI workflow adoption signals in CI or tooling configs.")
                .remediation("Adopt AI-assisted workflows deliberately (review bots, MCP, documented agents).")
                .build(),
        );
    }

    // Clear empty remediations
    for f in &mut findings {
        if f.remediation.as_deref() == Some("") {
            f.remediation = None;
        }
    }

    findings
}

fn build_risks(
    prompt_count: usize,
    governance: bool,
    human_review: bool,
    artifacts: &[AiArtifact],
) -> Vec<String> {
    let mut risks = Vec::new();
    if prompt_count > 0 && !governance {
        risks.push(
            "Prompts exist without AI governance docs — usage norms may be inconsistent.".into(),
        );
    }
    if artifacts
        .iter()
        .any(|a| a.kind == AiArtifactKind::AgentDefinition || a.kind == AiArtifactKind::McpConfig)
        && !human_review
    {
        risks
            .push("Agents/MCP configured without clear human-review gates — autonomy risk.".into());
    }
    if prompt_count == 0
        && artifacts.iter().any(|a| {
            matches!(
                a.kind,
                AiArtifactKind::AiAssistedDocs | AiArtifactKind::AiAssistedTesting
            )
        })
    {
        risks.push(
            "AI-assisted workflows referenced without a reusable prompt library — low reuse."
                .into(),
        );
    }
    risks
}

fn score_maturity(
    prompt_count: usize,
    versioning: bool,
    governance: bool,
    human_review: bool,
    workflows: bool,
    artifacts: &[AiArtifact],
) -> f64 {
    let mut score = 0.0_f64;
    if prompt_count > 0 {
        score += 20.0;
    }
    if prompt_count >= 3 {
        score += 10.0;
    }
    if versioning {
        score += 15.0;
    }
    if governance {
        score += 20.0;
    }
    if human_review {
        score += 15.0;
    }
    if workflows {
        score += 10.0;
    }
    if artifacts
        .iter()
        .any(|a| a.kind == AiArtifactKind::McpConfig)
    {
        score += 5.0;
    }
    if artifacts
        .iter()
        .any(|a| a.kind == AiArtifactKind::AgentDefinition)
    {
        score += 5.0;
    }
    score.clamp(0.0, 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_prompt_paths() {
        assert!(is_prompt_path("baoulo/prompts/foo.md"));
        assert!(is_prompt_path(".cursor/prompts/bar.md"));
        assert!(is_mcp_path(".cursor/mcp.json"));
        assert!(is_agent_path("AGENTS.md"));
    }
}
