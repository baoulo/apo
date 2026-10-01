//! Unified evidence pack (v0.3): hygiene + knowledge + AI.

use serde::{Deserialize, Serialize};

use crate::report::{EvidenceReport, Report};
use crate::source::Workspace;

/// Schema version for unified pack JSON.
pub const EVIDENCE_SCHEMA: &str = "apo-v0.3";

/// Combined hygiene + knowledge + AI evidence pack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidencePack {
    pub evidence_schema: String,
    pub apo_version: String,
    pub analyzer: String,
    pub repository: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkout_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_uri: Option<String>,
    pub generated_at: String,
    pub executive_summary: String,
    pub hygiene: Report,
    pub evidence: EvidenceReport,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff: Option<PackDiff>,
}

/// Deterministic score/gap diff vs a prior pack JSON.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackDiff {
    pub baseline_schema: String,
    pub baseline_generated_at: Option<String>,
    pub hygiene_score_delta: Option<f64>,
    pub category_deltas: Vec<CategoryDelta>,
    pub knowledge_maturity_delta: f64,
    pub ai_maturity_delta: f64,
    pub newly_gapped: Vec<String>,
    pub newly_satisfied: Vec<String>,
    pub unchanged_gaps: Vec<String>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryDelta {
    pub category: String,
    pub before: Option<f64>,
    pub after: Option<f64>,
    pub delta: Option<f64>,
}

impl EvidencePack {
    pub fn build(hygiene: Report, evidence: EvidenceReport, workspace: &Workspace) -> Self {
        let hygiene_score = hygiene
            .policy
            .overall_score
            .map(|s| format!("{s:.0}"))
            .unwrap_or_else(|| "n/a".into());
        let executive_summary = format!(
            "Unified evidence pack for `{}`: hygiene {hygiene_score}/100; \
             knowledge maturity {:.0}/100; AI maturity {:.0}/100. \
             Schema {EVIDENCE_SCHEMA}.",
            workspace.label, evidence.knowledge_maturity, evidence.ai_maturity
        );

        Self {
            evidence_schema: EVIDENCE_SCHEMA.into(),
            apo_version: env!("CARGO_PKG_VERSION").into(),
            analyzer: "repository-evidence-pack".into(),
            repository: workspace.label.clone(),
            checkout_path: hygiene.checkout_path.clone(),
            source_uri: workspace.source_uri.clone(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            executive_summary,
            hygiene,
            evidence,
            diff: None,
        }
    }

    pub fn artifact_prefix(&self) -> String {
        self.hygiene.artifact_prefix()
    }

    pub fn markdown_filename(&self) -> String {
        format!("{}-repository-evidence-pack.md", self.artifact_prefix())
    }

    pub fn json_filename(&self) -> String {
        format!("{}-repository-evidence-pack.json", self.artifact_prefix())
    }

    pub fn prompt_filename(&self) -> String {
        format!(
            "{}-repository-evidence-pack-prompt.md",
            self.artifact_prefix()
        )
    }

    pub fn sarif_filename(&self) -> String {
        format!("{}-repository-evidence-pack.sarif", self.artifact_prefix())
    }

    pub fn diff_markdown_filename(&self) -> String {
        format!(
            "{}-repository-evidence-pack-diff.md",
            self.artifact_prefix()
        )
    }
}
