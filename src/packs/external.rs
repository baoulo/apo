//! Load declarative external packs from TOML files (no rebuild required).

use std::path::{Path, PathBuf};

use serde::Deserialize;
use tracing::warn;

use crate::error::{Error, Result};
use crate::packs::MapsTo;
use crate::packs::def::{PackDef, PackKind, ToolingDef};

/// Default relative directory under a repo root for external packs.
pub const DEFAULT_PACKS_SUBDIR: &str = ".apo/packs";

#[derive(Debug, Deserialize)]
struct ExternalPackFile {
    id: String,
    kind: String,
    #[serde(default)]
    manifests: Vec<String>,
    #[serde(default)]
    path_contains: Vec<String>,
    #[serde(default)]
    basename_any: Vec<String>,
    #[serde(default)]
    package_json_contains: Vec<String>,
    #[serde(default)]
    tooling: Vec<ExternalTooling>,
}

#[derive(Debug, Deserialize)]
struct ExternalTooling {
    id: String,
    maps_to: String,
    #[serde(default)]
    configs: Vec<String>,
    #[serde(default)]
    ci_commands: Vec<String>,
}

impl ExternalPackFile {
    fn into_pack_def(self, path: &Path) -> Result<PackDef> {
        let kind = match self.kind.to_ascii_lowercase().as_str() {
            "language" | "lang" => PackKind::Language,
            "web" => PackKind::Web,
            other => {
                return Err(Error::Config(format!(
                    "external pack {}: unknown kind '{other}' (expected language|web)",
                    path.display()
                )));
            }
        };
        if self.id.trim().is_empty() {
            return Err(Error::Config(format!(
                "external pack {}: id must be non-empty",
                path.display()
            )));
        }

        let mut tooling = Vec::new();
        for t in self.tooling {
            let Some(maps_to) = MapsTo::from_rule_id(&t.maps_to) else {
                warn!(
                    pack = %self.id,
                    maps_to = %t.maps_to,
                    path = %path.display(),
                    "skipping tooling entry with unknown maps_to"
                );
                continue;
            };
            tooling.push(ToolingDef {
                id: t.id,
                maps_to,
                configs: t.configs,
                ci_commands: t.ci_commands,
            });
        }

        Ok(PackDef {
            id: self.id,
            kind,
            manifests: self.manifests,
            path_contains: self.path_contains,
            basename_any: self.basename_any,
            package_json_contains: self.package_json_contains,
            tooling,
        })
    }
}

/// Load all `*.toml` packs from a directory (non-recursive). Missing dir → empty.
pub fn load_packs_from_dir(dir: &Path) -> Result<Vec<PackDef>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    if !dir.is_dir() {
        return Err(Error::Config(format!(
            "packs dir is not a directory: {}",
            dir.display()
        )));
    }

    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("toml"))
        })
        .collect();
    paths.sort();

    let mut packs = Vec::new();
    for path in paths {
        let text = std::fs::read_to_string(&path)
            .map_err(|e| Error::Config(format!("read external pack {}: {e}", path.display())))?;
        let parsed: ExternalPackFile = toml::from_str(&text)
            .map_err(|e| Error::Config(format!("parse external pack {}: {e}", path.display())))?;
        packs.push(parsed.into_pack_def(&path)?);
    }
    Ok(packs)
}

/// Collect external packs from repo `.apo/packs` plus extra directories.
///
/// Extra dirs typically come from `--packs-dir` / `APO_PACKS_DIR`.
pub fn load_external_packs(repo_root: &Path, extra_dirs: &[PathBuf]) -> Result<Vec<PackDef>> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    dirs.push(repo_root.join(DEFAULT_PACKS_SUBDIR));
    for d in extra_dirs {
        dirs.push(d.clone());
    }

    let mut all = Vec::new();
    let mut seen_ids = std::collections::BTreeSet::new();
    for dir in dirs {
        for pack in load_packs_from_dir(&dir)? {
            if !seen_ids.insert(pack.id.clone()) {
                return Err(Error::Config(format!(
                    "duplicate external pack id '{}' (seen while loading {})",
                    pack.id,
                    dir.display()
                )));
            }
            all.push(pack);
        }
    }
    all.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(all)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn loads_crystal_like_pack() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("crystal.toml"),
            r#"
id = "crystal"
kind = "language"
manifests = ["shard.yml"]
path_contains = [".cr"]

[[tooling]]
id = "lint"
maps_to = "local_development.linter"
configs = [".ameba.yml"]
ci_commands = ["ameba"]
"#,
        )
        .unwrap();
        let packs = load_packs_from_dir(dir.path()).unwrap();
        assert_eq!(packs.len(), 1);
        assert_eq!(packs[0].id, "crystal");
        assert_eq!(packs[0].kind, PackKind::Language);
        assert_eq!(packs[0].tooling.len(), 1);
        assert_eq!(packs[0].tooling[0].maps_to, MapsTo::Linter);
    }

    #[test]
    fn rejects_unknown_kind() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("bad.toml"),
            r#"
id = "x"
kind = "nope"
"#,
        )
        .unwrap();
        assert!(load_packs_from_dir(dir.path()).is_err());
    }
}
