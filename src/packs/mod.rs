//! Language and web ecosystem packs — observational tooling detection.

mod catalog;
mod config_file;
mod resolve;

pub use catalog::{PackDef, ToolingDef, builtin_packs, catalog_ids};
pub use config_file::{ApoProjectConfig, ToolingOverlay};
pub use resolve::{
    MapsTo, PackKind, ResolvedTooling, ToolingEntry, active_pack_ids, resolve_tooling,
};

use crate::discovery::RepoContext;

/// Resolve tooling for a repository (built-in packs + optional `.apo.toml`).
pub fn for_repo(ctx: &RepoContext) -> ResolvedTooling {
    let overlay = ApoProjectConfig::load_from_root(&ctx.root);
    resolve_tooling(ctx, overlay.as_ref())
}
