//! Runtime (owned) pack definitions shared by builtins and external packs.

use crate::packs::MapsTo;
use crate::packs::catalog::{StaticPackDef, StaticToolingDef};

/// Pack category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackKind {
    Language,
    Web,
}

/// Owned pack definition (builtins converted + external TOML).
#[derive(Debug, Clone)]
pub struct PackDef {
    pub id: String,
    pub kind: PackKind,
    pub manifests: Vec<String>,
    pub path_contains: Vec<String>,
    pub basename_any: Vec<String>,
    pub package_json_contains: Vec<String>,
    pub tooling: Vec<ToolingDef>,
}

/// Owned tooling entry within a pack.
#[derive(Debug, Clone)]
pub struct ToolingDef {
    pub id: String,
    pub maps_to: MapsTo,
    pub configs: Vec<String>,
    pub ci_commands: Vec<String>,
}

impl PackDef {
    pub fn from_static(s: &StaticPackDef) -> Self {
        Self {
            id: s.id.to_string(),
            kind: s.kind,
            manifests: s.manifests.iter().map(|x| (*x).to_string()).collect(),
            path_contains: s.path_contains.iter().map(|x| (*x).to_string()).collect(),
            basename_any: s.basename_any.iter().map(|x| (*x).to_string()).collect(),
            package_json_contains: s
                .package_json_contains
                .iter()
                .map(|x| (*x).to_string())
                .collect(),
            tooling: s.tooling.iter().map(ToolingDef::from_static).collect(),
        }
    }
}

impl ToolingDef {
    pub fn from_static(s: &StaticToolingDef) -> Self {
        Self {
            id: s.id.to_string(),
            maps_to: s.maps_to,
            configs: s.configs.iter().map(|x| (*x).to_string()).collect(),
            ci_commands: s.ci_commands.iter().map(|x| (*x).to_string()).collect(),
        }
    }
}
