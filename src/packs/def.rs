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
    pub knowledge: Vec<KnowledgeDef>,
}

/// Owned tooling entry within a pack.
#[derive(Debug, Clone)]
pub struct ToolingDef {
    pub id: String,
    pub maps_to: Vec<MapsTo>,
    pub configs: Vec<String>,
    pub ci_commands: Vec<String>,
}

/// Inline documentation markers a pack contributes to the knowledge inventory.
#[derive(Debug, Clone)]
pub struct KnowledgeDef {
    pub kind: String,
    pub content_needles: Vec<String>,
    pub path_suffixes: Vec<String>,
    pub exclude_needles: Vec<String>,
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
            knowledge: builtin_knowledge(s.id),
        }
    }
}

fn builtin_knowledge(pack_id: &str) -> Vec<KnowledgeDef> {
    match pack_id {
        "elixir" => vec![
            KnowledgeDef {
                kind: "api".into(),
                content_needles: vec!["@moduledoc".into(), "@doc".into()],
                path_suffixes: vec![".ex".into()],
                exclude_needles: vec!["@moduledoc false".into()],
            },
            KnowledgeDef {
                kind: "onboarding".into(),
                content_needles: vec!["iex>".into(), "doctest".into()],
                path_suffixes: vec![".ex".into(), ".exs".into()],
                exclude_needles: vec![],
            },
        ],
        _ => Vec::new(),
    }
}

impl ToolingDef {
    pub fn from_static(s: &StaticToolingDef) -> Self {
        let mut maps_to = vec![s.maps_to];
        maps_to.extend(s.also_maps_to.iter().copied());
        Self {
            id: s.id.to_string(),
            maps_to,
            configs: s.configs.iter().map(|x| (*x).to_string()).collect(),
            ci_commands: s.ci_commands.iter().map(|x| (*x).to_string()).collect(),
        }
    }
}
