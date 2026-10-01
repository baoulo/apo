//! Serialize built-in packs to TOML for external editing / overlays.

use std::fmt::Write as _;
use std::path::Path;

use crate::error::{Error, Result};
use crate::packs::PackKind;
use crate::packs::builtin_packs;
use crate::packs::def::PackDef;

/// Render one pack as TOML matching the external pack schema.
pub fn pack_to_toml(pack: &PackDef) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "id = {}", toml_string(&pack.id));
    let kind = match pack.kind {
        PackKind::Language => "language",
        PackKind::Web => "web",
    };
    let _ = writeln!(out, "kind = {kind:?}");
    write_string_array(&mut out, "manifests", &pack.manifests);
    write_string_array(&mut out, "path_contains", &pack.path_contains);
    write_string_array(&mut out, "basename_any", &pack.basename_any);
    write_string_array(
        &mut out,
        "package_json_contains",
        &pack.package_json_contains,
    );
    for t in &pack.tooling {
        let _ = writeln!(out);
        let _ = writeln!(out, "[[tooling]]");
        let _ = writeln!(out, "id = {}", toml_string(&t.id));
        let _ = writeln!(out, "maps_to = {}", toml_string(t.maps_to.rule_id()));
        write_string_array(&mut out, "configs", &t.configs);
        write_string_array(&mut out, "ci_commands", &t.ci_commands);
    }
    out
}

fn toml_string(s: &str) -> String {
    // Use debug quoting; sufficient for pack ids / paths / needles.
    format!("{s:?}")
}

fn write_string_array(out: &mut String, key: &str, values: &[String]) {
    if values.is_empty() {
        let _ = writeln!(out, "{key} = []");
        return;
    }
    let _ = write!(out, "{key} = [");
    for (i, v) in values.iter().enumerate() {
        if i > 0 {
            let _ = write!(out, ", ");
        }
        let _ = write!(out, "{}", toml_string(v));
    }
    let _ = writeln!(out, "]");
}

/// Dump built-in packs as `{id}.toml` files under `output_dir`.
///
/// If `only_id` is set, dump that single pack (error if unknown).
/// Returns paths written.
pub fn dump_builtin_packs(
    output_dir: &Path,
    only_id: Option<&str>,
) -> Result<Vec<std::path::PathBuf>> {
    std::fs::create_dir_all(output_dir)?;
    let packs = builtin_packs();
    let selected: Vec<&PackDef> = if let Some(id) = only_id {
        match packs.iter().find(|p| p.id == id) {
            Some(p) => vec![p],
            None => {
                return Err(Error::Config(format!("unknown built-in pack id '{id}'")));
            }
        }
    } else {
        packs.iter().collect()
    };

    let mut written = Vec::new();
    for pack in selected {
        let path = output_dir.join(format!("{}.toml", pack.id));
        std::fs::write(&path, pack_to_toml(pack))?;
        written.push(path);
    }
    written.sort();
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packs::load_packs_from_dir;

    #[test]
    fn dump_roundtrips_through_loader() {
        let dir = tempfile::tempdir().unwrap();
        let written = dump_builtin_packs(dir.path(), Some("rust")).unwrap();
        assert_eq!(written.len(), 1);
        let (loaded, skipped) = load_packs_from_dir(dir.path()).unwrap();
        assert!(skipped.is_empty());
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, "rust");
        assert!(!loaded[0].tooling.is_empty());
        assert_eq!(loaded[0].manifests, vec!["Cargo.toml".to_string()]);
    }
}
