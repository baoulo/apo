//! Write unified evidence pack artifacts.

use std::path::{Path, PathBuf};

use crate::config::OutputFormat;
use crate::error::Result;
use crate::report::pack::EvidencePack;
use crate::report::pack_diff::render_diff_markdown;
use crate::report::{
    evidence_json_to_string, json_to_string, render_evidence_markdown, write_evidence_badge,
    write_hygiene_badge,
};

/// Serialize pack JSON.
pub fn pack_json_to_string(pack: &EvidencePack) -> Result<String> {
    Ok(serde_json::to_string_pretty(pack)?)
}

/// Write pack JSON to path.
pub fn write_pack_json(pack: &EvidencePack, path: &Path) -> Result<()> {
    std::fs::write(path, pack_json_to_string(pack)?)?;
    Ok(())
}

/// Render combined markdown for the unified pack.
pub fn render_pack_markdown(pack: &EvidencePack) -> String {
    let mut out = String::new();
    out.push_str("# APO repository evidence pack\n\n");
    out.push_str(&format!(
        "- **Schema:** `{}`\n- **APO:** {}\n- **Repository:** `{}`\n- **Generated:** {}\n\n",
        pack.evidence_schema, pack.apo_version, pack.repository, pack.generated_at
    ));
    out.push_str("## Executive summary\n\n");
    out.push_str(&pack.executive_summary);
    out.push_str("\n\n");

    if let Some(diff) = &pack.diff {
        out.push_str("## Baseline diff\n\n");
        out.push_str(&diff.summary);
        out.push_str("\n\n");
    }

    out.push_str("---\n\n");
    out.push_str("## Hygiene\n\n");
    // Reuse hygiene markdown body without duplicating the top-level title heavily.
    let hygiene_md = crate::report::markdown::render_markdown(&pack.hygiene);
    out.push_str(&hygiene_md);
    out.push_str("\n\n---\n\n");
    out.push_str("## Knowledge & AI evidence\n\n");
    out.push_str(&render_evidence_markdown(&pack.evidence));
    out
}

/// Write pack markdown.
pub fn write_pack_markdown(pack: &EvidencePack, path: &Path) -> Result<()> {
    std::fs::write(path, render_pack_markdown(pack))?;
    Ok(())
}

fn resolve_pack_outputs(
    pack: &EvidencePack,
    format: OutputFormat,
    output: Option<&Path>,
    cwd: &Path,
) -> Result<Vec<(OutputFormat, PathBuf)>> {
    let default_md = cwd.join(pack.markdown_filename());
    let default_json = cwd.join(pack.json_filename());
    let default_sarif = cwd.join(pack.sarif_filename());

    match (format, output) {
        (OutputFormat::Markdown, None) => Ok(vec![(OutputFormat::Markdown, default_md)]),
        (OutputFormat::Json, None) => Ok(vec![(OutputFormat::Json, default_json)]),
        (OutputFormat::Sarif, None) => Ok(vec![(OutputFormat::Sarif, default_sarif)]),
        (OutputFormat::Both, None) => Ok(vec![
            (OutputFormat::Markdown, default_md),
            (OutputFormat::Json, default_json),
        ]),
        (OutputFormat::Markdown, Some(p)) => {
            if p.is_dir() || p.extension().is_none() {
                Ok(vec![(
                    OutputFormat::Markdown,
                    p.join(pack.markdown_filename()),
                )])
            } else {
                Ok(vec![(OutputFormat::Markdown, p.to_path_buf())])
            }
        }
        (OutputFormat::Json, Some(p)) => {
            if p.is_dir() || (p.extension().is_none() && !p.to_string_lossy().ends_with(".json")) {
                Ok(vec![(OutputFormat::Json, p.join(pack.json_filename()))])
            } else {
                Ok(vec![(OutputFormat::Json, p.to_path_buf())])
            }
        }
        (OutputFormat::Sarif, Some(p)) => {
            if p.is_dir() || p.extension().is_none() {
                Ok(vec![(OutputFormat::Sarif, p.join(pack.sarif_filename()))])
            } else {
                Ok(vec![(OutputFormat::Sarif, p.to_path_buf())])
            }
        }
        (OutputFormat::Both, Some(p)) => {
            if p.is_file()
                || p.extension()
                    .is_some_and(|e| e == "md" || e == "json" || e == "markdown" || e == "sarif")
            {
                let parent = p.parent().unwrap_or(cwd);
                let stem = p
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(str::to_string)
                    .unwrap_or_else(|| {
                        format!("{}-repository-evidence-pack", pack.artifact_prefix())
                    });
                Ok(vec![
                    (OutputFormat::Markdown, parent.join(format!("{stem}.md"))),
                    (OutputFormat::Json, parent.join(format!("{stem}.json"))),
                ])
            } else {
                Ok(vec![
                    (OutputFormat::Markdown, p.join(pack.markdown_filename())),
                    (OutputFormat::Json, p.join(pack.json_filename())),
                ])
            }
        }
    }
}

/// Write pack report files for the requested format.
pub fn write_pack_report(
    pack: &EvidencePack,
    format: OutputFormat,
    output: Option<&Path>,
    default_dir: &Path,
) -> Result<Vec<PathBuf>> {
    let targets = resolve_pack_outputs(pack, format, output, default_dir)?;
    let mut written = Vec::new();
    for (fmt, path) in targets {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        match fmt {
            OutputFormat::Markdown => write_pack_markdown(pack, &path)?,
            OutputFormat::Json => write_pack_json(pack, &path)?,
            OutputFormat::Sarif => crate::report::sarif::write_sarif(pack, &path)?,
            OutputFormat::Both => unreachable!("resolved to concrete formats"),
        }
        written.push(path);
    }
    Ok(written)
}

/// Write hygiene + evidence badges for a pack.
pub fn write_pack_badges(
    pack: &EvidencePack,
    badge_output: Option<&Path>,
    output: Option<&Path>,
    default_dir: &Path,
) -> Result<Vec<PathBuf>> {
    Ok(vec![
        write_hygiene_badge(&pack.hygiene, badge_output, output, default_dir)?,
        write_evidence_badge(&pack.evidence, badge_output, output, default_dir)?,
    ])
}

/// Resolve LLM prompt path for the pack.
pub fn resolve_pack_prompt_path(
    pack: &EvidencePack,
    output: Option<&Path>,
    default_dir: &Path,
) -> PathBuf {
    match output {
        Some(p) if p.extension().is_some_and(|e| e == "md") => p.to_path_buf(),
        Some(p) if p.is_dir() || p.extension().is_none() => p.join(pack.prompt_filename()),
        Some(p) => p
            .parent()
            .unwrap_or(default_dir)
            .join(pack.prompt_filename()),
        None => default_dir.join(pack.prompt_filename()),
    }
}

/// Combined remediation prompt covering hygiene + evidence gaps.
pub fn render_pack_llm_prompt(pack: &EvidencePack) -> String {
    let mut out = String::from(
        "# APO unified evidence pack remediation prompt\n\n\
         You are remediating repository gaps from an APO observational analysis. \
         Prefer adding real artifacts and CI gates. Do not invent evidence that is not present.\n\n",
    );
    out.push_str(&crate::report::render_llm_prompt(&pack.hygiene));
    out.push_str("\n\n---\n\n");
    out.push_str(&crate::report::render_evidence_llm_prompt(&pack.evidence));
    out
}

pub fn write_pack_llm_prompt(pack: &EvidencePack, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, render_pack_llm_prompt(pack))?;
    Ok(())
}

pub fn write_pack_diff_markdown(
    pack: &EvidencePack,
    default_dir: &Path,
) -> Result<Option<PathBuf>> {
    let Some(diff) = &pack.diff else {
        return Ok(None);
    };
    let path = default_dir.join(pack.diff_markdown_filename());
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, render_diff_markdown(diff))?;
    Ok(Some(path))
}

/// Helper re-exports used by tests / debugging.
#[allow(dead_code)]
pub fn hygiene_and_evidence_json_lens(pack: &EvidencePack) -> Result<(String, String)> {
    Ok((
        json_to_string(&pack.hygiene)?,
        evidence_json_to_string(&pack.evidence)?,
    ))
}
