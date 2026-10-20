//! Resolve active packs and merge tooling entries.

use std::path::PathBuf;

use tracing::warn;

use serde::{Deserialize, Serialize};

use crate::discovery::RepoContext;
use crate::error::Result;
use crate::packs::catalog::builtin_packs;
use crate::packs::config_file::ApoProjectConfig;
use crate::packs::def::PackDef;
use crate::packs::external::load_external_packs;

/// Tooling entry skipped because `maps_to` was unknown or invalid.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkippedToolingNote {
    /// Pack id, `overlay`, or file path context.
    pub source: String,
    /// Tooling entry id.
    pub id: String,
    /// Requested `maps_to` string that could not be resolved.
    pub maps_to: String,
    /// Human-readable reason.
    pub reason: String,
}

/// Which hygiene rule a tooling signal feeds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MapsTo {
    Formatter,
    Linter,
    TypeChecker,
    DependencyScanning,
    TestFramework,
    StaticAnalysisCi,
    TypeCheckingCi,
    DocTooling,
    SecretScanning,
    PropertyTesting,
    IntegrationTesting,
    UiTesting,
    Sast,
}

impl MapsTo {
    pub fn rule_id(self) -> &'static str {
        match self {
            Self::Formatter => "local_development.formatter",
            Self::Linter => "local_development.linter",
            Self::TypeChecker => "local_development.type_checker",
            Self::DependencyScanning => "security.dependency_scanning",
            Self::TestFramework => "testing.framework",
            Self::StaticAnalysisCi => "testing.static_analysis_ci",
            Self::TypeCheckingCi => "testing.type_checking_ci",
            Self::DocTooling => "documentation.doc_tooling",
            Self::SecretScanning => "security.secret_scanning",
            Self::PropertyTesting => "testing.property",
            Self::IntegrationTesting => "testing.integration",
            Self::UiTesting => "testing.ui",
            Self::Sast => "security.sast",
        }
    }

    pub fn from_rule_id(id: &str) -> Option<Self> {
        match id {
            "local_development.formatter" => Some(Self::Formatter),
            "local_development.linter" => Some(Self::Linter),
            "local_development.type_checker" => Some(Self::TypeChecker),
            "security.dependency_scanning" => Some(Self::DependencyScanning),
            "testing.framework" => Some(Self::TestFramework),
            "testing.static_analysis_ci" => Some(Self::StaticAnalysisCi),
            "testing.type_checking_ci" => Some(Self::TypeCheckingCi),
            "documentation.doc_tooling" => Some(Self::DocTooling),
            "security.secret_scanning" => Some(Self::SecretScanning),
            "testing.property" => Some(Self::PropertyTesting),
            "testing.integration" => Some(Self::IntegrationTesting),
            "testing.ui" => Some(Self::UiTesting),
            "security.sast" => Some(Self::Sast),
            _ => None,
        }
    }
}

/// One tooling detection entry (from a pack or overlay).
#[derive(Debug, Clone)]
pub struct ToolingEntry {
    pub id: String,
    pub pack_id: String,
    pub maps_to: MapsTo,
    pub configs: Vec<String>,
    pub ci_commands: Vec<String>,
}

/// A pack that matched this repository, and why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetectedPack {
    pub id: String,
    pub triggers: Vec<String>,
    pub tooling: Vec<String>,
}

/// Inline knowledge markers copied from active packs.
#[derive(Debug, Clone)]
pub struct ActiveKnowledge {
    pub pack_id: String,
    pub kind: String,
    pub content_needles: Vec<String>,
    pub path_suffixes: Vec<String>,
    pub exclude_needles: Vec<String>,
}

/// Resolved tooling for a repository analysis.
#[derive(Debug, Clone, Default)]
pub struct ResolvedTooling {
    pub active_packs: Vec<String>,
    pub detected_packs: Vec<DetectedPack>,
    pub entries: Vec<ToolingEntry>,
    pub knowledge: Vec<ActiveKnowledge>,
    /// Built-in pack ids replaced by external packs.
    pub overridden_packs: Vec<String>,
    /// Tooling entries skipped (e.g. unknown maps_to).
    pub skipped_tooling: Vec<SkippedToolingNote>,
}

impl ResolvedTooling {
    pub fn entries_for(&self, maps_to: MapsTo) -> impl Iterator<Item = &ToolingEntry> {
        self.entries.iter().filter(move |e| e.maps_to == maps_to)
    }

    pub fn config_names(&self, maps_to: MapsTo) -> Vec<&str> {
        let mut out: Vec<&str> = self
            .entries_for(maps_to)
            .flat_map(|e| e.configs.iter().map(String::as_str))
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }

    pub fn ci_commands(&self, maps_to: MapsTo) -> Vec<&str> {
        let mut out: Vec<&str> = self
            .entries_for(maps_to)
            .flat_map(|e| e.ci_commands.iter().map(String::as_str))
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }
}

/// Merge builtins + external packs. External packs with the same `id` **override** builtins.
/// Returns `(catalog, overridden_builtin_ids)`.
pub fn merge_pack_catalog(external: Vec<PackDef>) -> (Vec<PackDef>, Vec<String>) {
    let mut by_id: std::collections::BTreeMap<String, PackDef> = builtin_packs()
        .into_iter()
        .map(|p| (p.id.clone(), p))
        .collect();
    let mut overridden = Vec::new();
    for pack in external {
        if by_id.contains_key(&pack.id) {
            tracing::info!(
                pack = %pack.id,
                "external pack overrides built-in pack with the same id"
            );
            overridden.push(pack.id.clone());
        }
        by_id.insert(pack.id.clone(), pack);
    }
    overridden.sort();
    overridden.dedup();
    (by_id.into_values().collect(), overridden)
}

/// List pack ids that match the repository inventory (and forced overlays).
pub fn active_pack_ids(
    ctx: &RepoContext,
    config: Option<&ApoProjectConfig>,
    catalog: &[PackDef],
) -> Vec<String> {
    let mut ids = Vec::new();
    for pack in catalog {
        if pack_matches(ctx, pack) {
            ids.push(pack.id.clone());
        }
    }
    if let Some(cfg) = config {
        for id in cfg
            .ecosystem
            .languages
            .iter()
            .chain(cfg.ecosystem.web.iter())
        {
            if !ids.iter().any(|x| x == id) {
                ids.push(id.clone());
            }
        }
    }
    ids.sort();
    ids.dedup();
    ids
}

fn pack_matches(ctx: &RepoContext, pack: &PackDef) -> bool {
    !pack_triggers(ctx, pack).is_empty()
}

fn pack_triggers(ctx: &RepoContext, pack: &PackDef) -> Vec<String> {
    let mut triggers = Vec::new();
    for m in &pack.manifests {
        if ctx.has_file(m) {
            triggers.push(m.clone());
        }
        let manifest_name = std::path::Path::new(m)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(m);
        for entry in ctx.inventory.find_by_basenames(&[manifest_name]) {
            if entry.relative != *m {
                triggers.push(entry.relative.clone());
            }
        }
    }
    for needle in &pack.path_contains {
        if let Some(hit) = ctx.inventory.find_path_contains(needle).first() {
            triggers.push(hit.relative.clone());
        }
    }
    for basename in &pack.basename_any {
        if let Some(hit) = ctx
            .inventory
            .find_by_basenames(&[basename.as_str()])
            .first()
        {
            triggers.push(hit.relative.clone());
        }
    }
    if !pack.package_json_contains.is_empty()
        && let Some(pkg) = ctx.read_text("package.json")
    {
        let lower = pkg.to_ascii_lowercase();
        if pack
            .package_json_contains
            .iter()
            .any(|n| lower.contains(&n.to_ascii_lowercase()))
        {
            triggers.push("package.json".into());
        }
    }
    triggers.sort();
    triggers.dedup();
    triggers
}

/// Resolve built-in + external packs + `.apo.toml` overlays into tooling entries.
pub fn resolve_tooling(
    ctx: &RepoContext,
    config: Option<&ApoProjectConfig>,
    packs_dirs: &[PathBuf],
) -> Result<ResolvedTooling> {
    let (external, mut skipped_tooling) = load_external_packs(&ctx.root, packs_dirs)?;
    let (catalog, overridden_packs) = merge_pack_catalog(external);
    let active = active_pack_ids(ctx, config, &catalog);
    let mut entries = Vec::new();
    let mut detected_packs = Vec::new();
    let mut knowledge = Vec::new();

    for pack in &catalog {
        if !active.iter().any(|id| id == &pack.id) {
            continue;
        }
        let mut triggers = pack_triggers(ctx, pack);
        if triggers.is_empty() {
            triggers.push("forced".into());
        }
        let mut tooling_ids = Vec::new();
        for t in &pack.tooling {
            tooling_ids.push(format!("{}.{}", pack.id, t.id));
            for maps_to in &t.maps_to {
                entries.push(ToolingEntry {
                    id: format!("{}.{}", pack.id, t.id),
                    pack_id: pack.id.clone(),
                    maps_to: *maps_to,
                    configs: t.configs.clone(),
                    ci_commands: t.ci_commands.clone(),
                });
            }
        }
        for signal in &pack.knowledge {
            knowledge.push(ActiveKnowledge {
                pack_id: pack.id.clone(),
                kind: signal.kind.clone(),
                content_needles: signal.content_needles.clone(),
                path_suffixes: signal.path_suffixes.clone(),
                exclude_needles: signal.exclude_needles.clone(),
            });
        }
        detected_packs.push(DetectedPack {
            id: pack.id.clone(),
            triggers,
            tooling: tooling_ids,
        });
    }

    if let Some(cfg) = config {
        for overlay in &cfg.tooling {
            let Some(maps_to) = overlay.maps_to_rule() else {
                warn!(id = %overlay.id, maps_to = %overlay.maps_to, "skipping overlay with unknown maps_to");
                skipped_tooling.push(SkippedToolingNote {
                    source: "overlay".into(),
                    id: overlay.id.clone(),
                    maps_to: overlay.maps_to.clone(),
                    reason: "unknown maps_to rule id".into(),
                });
                continue;
            };
            entries.push(ToolingEntry {
                id: overlay.id.clone(),
                pack_id: "overlay".into(),
                maps_to,
                configs: overlay.configs.clone(),
                ci_commands: overlay.ci_commands.clone(),
            });
        }
    }

    skipped_tooling.sort_by(|a, b| (&a.source, &a.id).cmp(&(&b.source, &b.id)));

    detected_packs.sort_by(|a, b| a.id.cmp(&b.id));

    Ok(ResolvedTooling {
        active_packs: active,
        detected_packs,
        entries,
        knowledge,
        overridden_packs,
        skipped_tooling,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packs::def::{PackDef, PackKind, ToolingDef};

    #[test]
    fn merge_reports_overridden_ids_and_keeps_external_tooling() {
        let external = vec![PackDef {
            id: "rust".into(),
            kind: PackKind::Language,
            manifests: vec!["Cargo.toml".into()],
            path_contains: vec![],
            basename_any: vec![],
            package_json_contains: vec![],
            tooling: vec![ToolingDef {
                id: "lint".into(),
                maps_to: vec![MapsTo::Linter],
                configs: vec!["custom-clippy.toml".into()],
                ci_commands: vec![],
            }],
            knowledge: vec![],
        }];
        let (merged, overridden) = merge_pack_catalog(external);
        assert_eq!(overridden, vec!["rust".to_string()]);
        let rust = merged.iter().find(|p| p.id == "rust").unwrap();
        assert_eq!(
            rust.tooling[0].configs,
            vec!["custom-clippy.toml".to_string()]
        );
    }

    #[test]
    fn merge_new_external_id_is_not_overridden() {
        let external = vec![PackDef {
            id: "crystal".into(),
            kind: PackKind::Language,
            manifests: vec!["shard.yml".into()],
            path_contains: vec![],
            basename_any: vec![],
            package_json_contains: vec![],
            tooling: vec![],
            knowledge: vec![],
        }];
        let (_merged, overridden) = merge_pack_catalog(external);
        assert!(overridden.is_empty());
    }
}
