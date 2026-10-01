# Agent notes for APO

You are working on **APO**, a Rust Engineering Evidence Platform (single crate).

## Constraints

- MSRV **1.85**, edition **2024**; `#![forbid(unsafe_code)]`
- Deterministic analyzers are the source of truth; Ollama is narrative-only
- Language packs are observational — do not execute toolchains for scores
- `baoulo/` is gitignored local planning material — do not commit it
- Do not invent evidence when remediating self-analysis gaps

## Commands

```bash
./scripts/ci.sh
cargo test
cargo run -- report . --format both --output out/apo-self --badge-output docs/badges --sarif
cargo run -- analyze . --format both --output out/apo-self --badge-output docs/badges/apo-hygiene.svg
cargo run -- evidence . --format both --output out/apo-self --badge-output docs/badges/apo-evidence.svg
```

## Layout

- `src/rules/` — hygiene rules
- `src/packs/` — language/web packs + `.apo.toml`
- `src/knowledge/`, `src/ai_evidence/` — evidence analyzers
- `docs/badges/` — committed static SVG badges for README embeds
- `docs/language-packs.md` — pack activation + tooling catalog
- `docs/hygiene-controls.md`, `knowledge-artifacts.md`, `ai-adoption-signals.md` — catalogs

Prefer focused changes. Update `CHANGELOG.md` for user-facing behavior.
