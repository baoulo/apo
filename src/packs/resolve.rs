//! Resolve active packs and merge tooling entries.

use crate::discovery::RepoContext;
use crate::packs::catalog::{PackDef, builtin_packs};
use crate::packs::config_file::ApoProjectConfig;

/// Pack category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackKind {
    Language,
    Web,
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

/// Resolved tooling for a repository analysis.
#[derive(Debug, Clone, Default)]
pub struct ResolvedTooling {
    pub active_packs: Vec<String>,
    pub entries: Vec<ToolingEntry>,
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

/// List pack ids that match the repository inventory (and forced overlays).
pub fn active_pack_ids(ctx: &RepoContext, config: Option<&ApoProjectConfig>) -> Vec<String> {
    let mut ids = Vec::new();
    for pack in builtin_packs() {
        if pack_matches(ctx, pack) {
            ids.push(pack.id.to_string());
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
    for m in pack.manifests {
        if ctx.has_file(m) {
            return true;
        }
    }
    for needle in pack.path_contains {
        if !ctx.inventory.find_path_contains(needle).is_empty() {
            return true;
        }
    }
    for basename in pack.basename_any {
        if !ctx.inventory.find_by_basenames(&[basename]).is_empty() {
            return true;
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
            return true;
        }
    }
    false
}

/// Resolve built-in packs + `.apo.toml` overlays into tooling entries.
pub fn resolve_tooling(ctx: &RepoContext, config: Option<&ApoProjectConfig>) -> ResolvedTooling {
    let active = active_pack_ids(ctx, config);
    let mut entries = Vec::new();

    for pack in builtin_packs() {
        if !active.iter().any(|id| id == pack.id) {
            continue;
        }
        for t in pack.tooling {
            entries.push(ToolingEntry {
                id: format!("{}.{}", pack.id, t.id),
                pack_id: pack.id.to_string(),
                maps_to: t.maps_to,
                configs: t.configs.iter().map(|s| (*s).to_string()).collect(),
                ci_commands: t.ci_commands.iter().map(|s| (*s).to_string()).collect(),
            });
        }
    }

    if let Some(cfg) = config {
        for overlay in &cfg.tooling {
            let Some(maps_to) = overlay.maps_to_rule() else {
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

    ResolvedTooling {
        active_packs: active,
        entries,
    }
}
