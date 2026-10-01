//! Language and web ecosystem packs — observational tooling detection.

mod catalog;
mod config_file;
mod def;
mod dump;
mod external;
mod resolve;

pub use catalog::{
    StaticPackDef, StaticToolingDef, builtin_packs, catalog_ids, static_builtin_packs,
};
pub use config_file::{
    AnalyzeSection, ApoProjectConfig, EcosystemSection, OllamaSection, ReportSection,
    ToolingOverlay,
};
pub use def::{PackDef, PackKind, ToolingDef};
pub use dump::{dump_builtin_packs, pack_to_toml};
pub use external::{DEFAULT_PACKS_SUBDIR, load_external_packs, load_packs_from_dir};
pub use resolve::{
    MapsTo, ResolvedTooling, SkippedToolingNote, ToolingEntry, active_pack_ids, merge_pack_catalog,
    resolve_tooling,
};

use crate::discovery::RepoContext;
use crate::error::Result;

/// Resolve tooling for a repository (built-ins + `.apo/packs` + optional dirs + `.apo.toml`).
pub fn for_repo(ctx: &RepoContext) -> ResolvedTooling {
    for_repo_with_dirs(ctx, &[])
}

/// Resolve tooling with extra pack directories (`--packs-dir` / `APO_PACKS_DIR`).
pub fn for_repo_with_dirs(ctx: &RepoContext, packs_dirs: &[std::path::PathBuf]) -> ResolvedTooling {
    for_repo_with_dirs_result(ctx, packs_dirs).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "pack resolution failed; using empty tooling");
        ResolvedTooling::default()
    })
}

/// Fallible pack resolution (surfaces config conflicts).
pub fn for_repo_with_dirs_result(
    ctx: &RepoContext,
    packs_dirs: &[std::path::PathBuf],
) -> Result<ResolvedTooling> {
    let overlay = ApoProjectConfig::load_from_root(&ctx.root);
    resolve_tooling(ctx, overlay.as_ref(), packs_dirs)
}

/// Attach resolved tooling onto a context (call after discover).
pub fn attach_tooling(ctx: &mut RepoContext, packs_dirs: &[std::path::PathBuf]) -> Result<()> {
    ctx.tooling = for_repo_with_dirs_result(ctx, packs_dirs)?;
    Ok(())
}
