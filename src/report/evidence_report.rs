//! Combined Knowledge + AI evidence report.

use serde::{Deserialize, Serialize};

use crate::ai_evidence::AiEvidence;
use crate::discovery::RepoContext;
use crate::knowledge::KnowledgeEvidence;
use crate::ollama::OllamaEnrichment;
use crate::source::Workspace;

/// Full repository evidence report (v0.2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceReport {
    pub apo_version: String,
    pub analyzer: String,
    pub repository: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkout_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_uri: Option<String>,
    pub generated_at: String,
    pub executive_summary: String,
    pub repository_evidence: RepositoryEvidenceMeta,
    pub knowledge: KnowledgeEvidence,
    pub ai: AiEvidence,
    pub knowledge_maturity: f64,
    pub ai_maturity: f64,
    pub knowledge_risks: Vec<String>,
    pub ai_risks: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ollama: Option<OllamaEnrichment>,
}

/// High-level repository identity for the evidence report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryEvidenceMeta {
    pub label: String,
    pub file_count: usize,
    pub analyzers: Vec<String>,
}

impl EvidenceReport {
    pub fn build(
        ctx: &RepoContext,
        workspace: &Workspace,
        knowledge: KnowledgeEvidence,
        ai: AiEvidence,
        ollama: Option<OllamaEnrichment>,
    ) -> Self {
        let knowledge_maturity = knowledge.maturity_score;
        let ai_maturity = ai.maturity_score;
        let knowledge_risks = knowledge.risks.clone();
        let ai_risks = ai.risks.clone();

        let mut executive_summary = format!(
            "Knowledge maturity {:.0}/100 ({} artifacts, {} missing kinds, {} broken links). \
             AI maturity {:.0}/100 ({} prompt artifacts; governance {}; human review {}).",
            knowledge_maturity,
            knowledge.artifacts.len(),
            knowledge.kinds_missing.len(),
            knowledge.broken_links.len(),
            ai_maturity,
            ai.prompt_count,
            if ai.governance_present {
                "present"
            } else {
                "absent"
            },
            if ai.human_review_signals {
                "signaled"
            } else {
                "unclear"
            },
        );

        if let Some(enrichment) = &ollama {
            if enrichment.enabled {
                if let Some(narr) = &enrichment.executive_narrative {
                    executive_summary = format!("{}\n\n{}", executive_summary, narr.summary);
                }
            } else if let Some(note) = enrichment.notes.first() {
                executive_summary = format!("{executive_summary} Ollama: {note}");
            }
        }

        Self {
            apo_version: env!("CARGO_PKG_VERSION").to_string(),
            analyzer: "repository-evidence".into(),
            repository: workspace.label.clone(),
            checkout_path: Some(ctx.root.display().to_string()),
            source_uri: workspace.source_uri.clone(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            executive_summary,
            repository_evidence: RepositoryEvidenceMeta {
                label: workspace.label.clone(),
                file_count: ctx.inventory.len(),
                analyzers: vec![
                    "knowledge".into(),
                    "ai".into(),
                    if ollama.as_ref().is_some_and(|o| o.enabled) {
                        "ollama".into()
                    } else {
                        "deterministic-only".into()
                    },
                ],
            },
            knowledge,
            ai,
            knowledge_maturity,
            ai_maturity,
            knowledge_risks,
            ai_risks,
            ollama,
        }
    }

    pub fn artifact_prefix(&self) -> String {
        if let Some(uri) = &self.source_uri {
            return crate::report::repo_name_from_label(uri, None);
        }
        if let Some(path) = &self.checkout_path {
            return crate::report::repo_name_from_label(path, None);
        }
        crate::report::repo_name_from_label(&self.repository, None)
    }

    pub fn markdown_filename(&self) -> String {
        format!("{}-repository-evidence.md", self.artifact_prefix())
    }

    pub fn json_filename(&self) -> String {
        format!("{}-repository-evidence.json", self.artifact_prefix())
    }

    /// Default LLM remediation prompt filename.
    pub fn prompt_filename(&self) -> String {
        format!("{}-repository-evidence-prompt.md", self.artifact_prefix())
    }
}
