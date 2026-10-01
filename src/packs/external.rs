//! Load declarative external packs from TOML files (no rebuild required).

use std::path::{Path, PathBuf};

use serde::Deserialize;
use tracing::warn;

use crate::error::{Error, Result};
use crate::packs::MapsTo;
use crate::packs::SkippedToolingNote;
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
    fn into_pack_def(self, path: &Path) -> Result<(PackDef, Vec<SkippedToolingNote>)> {
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

        let pack_id = self.id.clone();
        let mut tooling = Vec::new();
        let mut skipped = Vec::new();
        for t in self.tooling {
            let Some(maps_to) = MapsTo::from_rule_id(&t.maps_to) else {
                warn!(
                    pack = %pack_id,
                    maps_to = %t.maps_to,
                    path = %path.display(),
                    "skipping tooling entry with unknown maps_to"
                );
                skipped.push(SkippedToolingNote {
                    source: pack_id.clone(),
                    id: t.id,
                    maps_to: t.maps_to,
                    reason: format!("unknown maps_to in {}", path.display()),
                });
                continue;
            };
            tooling.push(ToolingDef {
                id: t.id,
                maps_to,
                configs: t.configs,
                ci_commands: t.ci_commands,
            });
        }

        Ok((
            PackDef {
                id: self.id,
                kind,
                manifests: self.manifests,
                path_contains: self.path_contains,
                basename_any: self.basename_any,
                package_json_contains: self.package_json_contains,
                tooling,
            },
            skipped,
        ))
    }
}

/// Load all `*.toml` packs from a directory (non-recursive). Missing dir → empty.
pub fn load_packs_from_dir(dir: &Path) -> Result<(Vec<PackDef>, Vec<SkippedToolingNote>)> {
    if !dir.exists() {
        return Ok((Vec::new(), Vec::new()));
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
    let mut skipped = Vec::new();
    for path in paths {
        let text = std::fs::read_to_string(&path)
            .map_err(|e| Error::Config(format!("read external pack {}: {e}", path.display())))?;
        let parsed: ExternalPackFile = toml::from_str(&text)
            .map_err(|e| Error::Config(format!("parse external pack {}: {e}", path.display())))?;
        let (pack, pack_skipped) = parsed.into_pack_def(&path)?;
        packs.push(pack);
        skipped.extend(pack_skipped);
    }
    Ok((packs, skipped))
}

/// Collect external packs from repo `.apo/packs` plus extra directories.
///
/// Extra dirs typically come from `--packs-dir` / `APO_PACKS_DIR`.
/// Also returns tooling entries skipped for unknown `maps_to`.
pub fn load_external_packs(
    repo_root: &Path,
    extra_dirs: &[PathBuf],
) -> Result<(Vec<PackDef>, Vec<SkippedToolingNote>)> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    dirs.push(repo_root.join(DEFAULT_PACKS_SUBDIR));
    for d in extra_dirs {
        dirs.push(d.clone());
    }

    let mut all = Vec::new();
    let mut skipped = Vec::new();
    let mut seen_ids = std::collections::BTreeSet::new();
    for dir in dirs {
        let (packs, dir_skipped) = load_packs_from_dir(&dir)?;
        skipped.extend(dir_skipped);
        for pack in packs {
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
    Ok((all, skipped))
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
        let (packs, skipped) = load_packs_from_dir(dir.path()).unwrap();
        assert!(skipped.is_empty());
        assert_eq!(packs.len(), 1);
        assert_eq!(packs[0].id, "crystal");
        assert_eq!(packs[0].kind, PackKind::Language);
        assert_eq!(packs[0].tooling.len(), 1);
        assert_eq!(packs[0].tooling[0].maps_to, MapsTo::Linter);
    }

    #[test]
    fn records_skipped_unknown_maps_to() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("x.toml"),
            r#"
id = "x"
kind = "language"
manifests = ["x.toml"]

[[tooling]]
id = "bad"
maps_to = "not.a.real.rule"
configs = ["a"]
"#,
        )
        .unwrap();
        let (packs, skipped) = load_packs_from_dir(dir.path()).unwrap();
        assert_eq!(packs.len(), 1);
        assert!(packs[0].tooling.is_empty());
        assert_eq!(skipped.len(), 1);
        assert_eq!(skipped[0].id, "bad");
        assert_eq!(skipped[0].maps_to, "not.a.real.rule");
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
