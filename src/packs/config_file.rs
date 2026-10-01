//! `.apo.toml` project overlays for tooling detection and analysis defaults.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::packs::MapsTo;

/// Root project config loaded from `.apo.toml` when present.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApoProjectConfig {
    #[serde(default)]
    pub ecosystem: EcosystemSection,
    #[serde(default)]
    pub tooling: Vec<ToolingOverlay>,
    #[serde(default)]
    pub report: ReportSection,
    #[serde(default)]
    pub analyze: AnalyzeSection,
    #[serde(default)]
    pub ollama: OllamaSection,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EcosystemSection {
    /// Force-enable pack ids (in addition to auto-detect).
    #[serde(default)]
    pub languages: Vec<String>,
    /// Force-enable web pack ids.
    #[serde(default)]
    pub web: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReportSection {
    /// Default output format: md | json | both | sarif.
    #[serde(default)]
    pub format: Option<String>,
    /// Schema label (informational; packs always emit apo-v0.3).
    #[serde(default)]
    pub schema: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalyzeSection {
    #[serde(default)]
    pub commit_sample_limit: Option<usize>,
    /// Rule ids to exclude from hygiene evaluation.
    #[serde(default)]
    pub rule_disable: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OllamaSection {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
}

/// User-defined tooling detection overlay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolingOverlay {
    pub id: String,
    /// Target hygiene rule id, e.g. `local_development.linter`.
    pub maps_to: String,
    #[serde(default)]
    pub configs: Vec<String>,
    #[serde(default)]
    pub ci_commands: Vec<String>,
}

impl ToolingOverlay {
    pub fn maps_to_rule(&self) -> Option<MapsTo> {
        MapsTo::from_rule_id(&self.maps_to)
    }
}

impl ApoProjectConfig {
    pub fn load_from_root(root: &Path) -> Option<Self> {
        let path = root.join(".apo.toml");
        let text = std::fs::read_to_string(path).ok()?;
        toml::from_str(&text).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn maps_known_and_unknown_rules() {
        let ok = ToolingOverlay {
            id: "rust.docs".into(),
            maps_to: "documentation.doc_tooling".into(),
            configs: vec![],
            ci_commands: vec!["cargo doc".into()],
        };
        assert_eq!(ok.maps_to_rule(), Some(MapsTo::DocTooling));

        let bad = ToolingOverlay {
            id: "x".into(),
            maps_to: "not.a.rule".into(),
            configs: vec![],
            ci_commands: vec![],
        };
        assert!(bad.maps_to_rule().is_none());
    }

    #[test]
    fn load_from_root_reads_overlay() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join(".apo.toml"),
            r#"
[ecosystem]
languages = ["rust"]

[report]
format = "both"
schema = "apo-v0.3"

[analyze]
commit_sample_limit = 50
rule_disable = ["collaboration.maintenance_activity"]

[ollama]
enabled = false
url = "http://127.0.0.1:11434"
model = "llama3.2"

[[tooling]]
id = "custom.docs"
maps_to = "documentation.doc_tooling"
ci_commands = ["cargo doc"]
"#,
        )
        .unwrap();
        let cfg = ApoProjectConfig::load_from_root(dir.path()).expect("load");
        assert_eq!(cfg.ecosystem.languages, vec!["rust"]);
        assert_eq!(cfg.tooling.len(), 1);
        assert_eq!(cfg.tooling[0].id, "custom.docs");
        assert_eq!(cfg.report.format.as_deref(), Some("both"));
        assert_eq!(cfg.analyze.commit_sample_limit, Some(50));
        assert_eq!(
            cfg.analyze.rule_disable,
            vec!["collaboration.maintenance_activity"]
        );
    }

    #[test]
    fn load_from_root_missing_is_none() {
        let dir = tempfile::tempdir().unwrap();
        assert!(ApoProjectConfig::load_from_root(dir.path()).is_none());
    }
}
