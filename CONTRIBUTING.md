# Contributing to APO

Thanks for helping improve APO.

## Development setup

Requirements:

- Rust **1.85+** (edition 2024)
- Git (used for remote clones and history sampling)

```bash
git clone https://github.com/baoulo/apo.git
cd apo
./scripts/setup.sh   # rustup components + optional pre-commit hooks
# or: make setup
./scripts/ci.sh      # fmt, clippy, doc, test, llvm-cov (≥80%), deny, audit, apo self-analysis
```

Commit messages should follow [Conventional Commits](https://www.conventionalcommits.org/)
(see `.commitlintrc.json`).

Or run checks individually:

```bash
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo run -- analyze . --badge-output docs/badges/apo-hygiene.svg
cargo run -- evidence . --badge-output docs/badges/apo-evidence.svg
```

## Project layout

- `src/` — library + CLI (`analyze`, `prompt`)
- `tests/` — CLI and integration fixtures
- `.github/workflows/` — CI, release, dependency updates

## Coding guidelines

- Prefer observational evidence over opinions in rules
- Keep policy scoring separate from rule evaluation
- Match existing module style; avoid drive-by refactors
- Run `cargo deny check` when changing dependencies

## Pull requests

1. Open a PR against `main`
2. Ensure CI is green (fmt, clippy, tests, coverage gate)
3. Update `CHANGELOG.md` under an `[Unreleased]` section when user-facing behavior changes
4. Keep the diff focused on one concern

## Release (maintainers)

1. Bump version in `Cargo.toml` and add a `CHANGELOG.md` section
2. Merge to `main`
3. Ensure repository secrets are set: `CARGO_REGISTRY_TOKEN` (crates.io) and `RELEASE_PUSH_TOKEN` (push packaging checksums to `main`)
4. Tag and push: `git tag v0.3.2 && git push origin v0.3.2`
5. The Release workflow: build binaries → GitHub Release + `SHA256SUMS` → fill Homebrew/Scoop checksums on `main` → smoke-test `install.sh` (Ubuntu + macOS) and Homebrew formula (macOS) → `cargo publish`
6. Copy updated `packaging/homebrew/apo.rb` and `packaging/scoop/apo.json` to the tap/bucket repos when ready

Manual publish (optional):

```bash
cargo publish --dry-run
cargo publish
```
