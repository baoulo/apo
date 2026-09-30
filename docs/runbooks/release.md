# Runbook: Release APO

## Preconditions

- Changes merged to `main`; CI green
- `CHANGELOG.md` section for the version
- Version bumped in `Cargo.toml`
- Repository secret `CARGO_REGISTRY_TOKEN` set for crates.io

## Steps

1. Confirm dry-run: `cargo publish --dry-run`
2. Tag and push: `git tag vX.Y.Z && git push origin vX.Y.Z`
3. Release workflow builds binaries, creates the GitHub Release (with
   `SHA256SUMS`), and runs `cargo publish`
4. Verify crates.io / docs.rs / release assets

## Rollback

- Yank a bad crates.io version if needed: `cargo yank --vers X.Y.Z`
- Prefer a follow-up patch release over force-moving tags already consumed by
  installers

## Related

- [CONTRIBUTING.md](../../CONTRIBUTING.md) — release checklist
- [.github/workflows/](../../.github/workflows/) — CI and release workflows
