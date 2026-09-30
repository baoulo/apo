//! Markdown local-link checker.

use std::path::{Component, Path, PathBuf};

use regex::Regex;

use crate::discovery::RepoContext;
use crate::knowledge::BrokenLink;

/// Find broken relative markdown links in documentation files.
pub fn find_broken_markdown_links(ctx: &RepoContext) -> Vec<BrokenLink> {
    let re = Regex::new(r"\[([^\]]*)\]\(([^)]+)\)").expect("link regex");
    let mut broken = Vec::new();

    for entry in ctx.inventory.iter() {
        let l = entry.relative.to_ascii_lowercase();
        if !(l.ends_with(".md") || l.ends_with(".mdx") || l.ends_with(".markdown")) {
            continue;
        }
        // Skip huge files
        if entry.size > 512_000 {
            continue;
        }
        let Some(content) = ctx.read_text(&entry.relative) else {
            continue;
        };

        for cap in re.captures_iter(&content) {
            let target = cap.get(2).map(|m| m.as_str().trim()).unwrap_or("");
            if target.is_empty() || !is_local_link(target) {
                continue;
            }
            let path_part = target.split(['#', '?']).next().unwrap_or(target);
            if path_part.is_empty() {
                continue;
            }
            let resolved = resolve_relative(&entry.relative, path_part);
            if !ctx.has_file(&resolved) && !ctx.has_dir(&resolved) {
                // Also try URL-decoded / without leading ./
                let alt = resolved.trim_start_matches("./").to_string();
                if !ctx.has_file(&alt) && !ctx.has_dir(&alt) {
                    broken.push(BrokenLink {
                        source: entry.relative.clone(),
                        target: path_part.to_string(),
                        detail: format!("resolved to `{resolved}` (not found)"),
                    });
                }
            }
        }
    }

    broken.sort_by(|a, b| (&a.source, &a.target).cmp(&(&b.source, &b.target)));
    broken.dedup_by(|a, b| a.source == b.source && a.target == b.target);
    broken
}

fn is_local_link(target: &str) -> bool {
    let t = target.trim();
    if t.starts_with('#') {
        return false;
    }
    if t.contains("://") || t.starts_with("mailto:") || t.starts_with("tel:") {
        return false;
    }
    true
}

fn resolve_relative(from_file: &str, link: &str) -> String {
    let base = Path::new(from_file)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(""));
    let joined = if link.starts_with('/') {
        PathBuf::from(link.trim_start_matches('/'))
    } else {
        base.join(link)
    };
    normalize_path(&joined)
}

fn normalize_path(path: &Path) -> String {
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            Component::Normal(s) => out.push(s),
            Component::RootDir => {}
            Component::Prefix(_) => {}
        }
    }
    out.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_relative() {
        assert_eq!(
            resolve_relative("docs/guide.md", "../README.md"),
            "README.md"
        );
        assert_eq!(
            resolve_relative("docs/guide.md", "adr/0001.md"),
            "docs/adr/0001.md"
        );
    }
}
