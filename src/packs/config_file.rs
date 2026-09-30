//! `.apo.toml` project overlays for tooling detection.

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
