# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-07-20

Knowledge Evidence and AI Evidence analyzers, plus observational language/web packs for fairer multi-ecosystem hygiene scoring.

### Added

- `apo evidence` command for Knowledge + AI evidence (offline by default)
- Knowledge analyzer: README/ADRs/architecture/design/runbooks/API docs/diagrams/onboarding/glossaries/prompts, coverage & freshness, ownership signals, broken links, knowledge graph
- AI analyzer: prompt libraries/templates, MCP configs, agent definitions, AI governance, review workflows, maturity scoring
- Optional local Ollama HTTP enrichment (`--ollama`, `--ollama-url`, `--ollama-model` / `APO_OLLAMA_*`) for narrative only — never invents evidence
- `{repo}-repository-evidence.md` / `.json` reports with maturity scores, risks, and executive summary
- `evidence --llm-prompt` / `apo prompt --evidence` write `{repo}-repository-evidence-prompt.md`
- Language/web **pack engine** with 30+ observational packs (Elixir Mix reference, Nim, SQL, web frameworks, …)
- `.apo.toml` `[[tooling]]` overlays; new rule `documentation.doc_tooling`
- [`scripts/ci.sh`](scripts/ci.sh) local CI mirror; [`ROADMAP.md`](ROADMAP.md); [`examples/apo.toml`](examples/apo.toml)
- Enterprise-friendly **static SVG badges** (`{repo}-*-badge.svg`) for hygiene and evidence — relative README embeds work on GitHub Enterprise; `--no-badge` / `--badge-output`
- CI **APO self-analysis** job + `./scripts/ci.sh` dogfood (`analyze` + `evidence` on this repo; badges under `docs/badges/`)

### Changed

- Package version `0.2.0`; description and README cover hygiene, evidence, and packs
- Hygiene rule count **36** (adds `documentation.doc_tooling`)

### Notes

- Packs are observational (configs + CI/script text); APO does not execute toolchains for scores
- Ollama client is HTTP-only
- MSRV: Rust **1.85** (edition 2024)

## [0.1.0] - 2026-07-20

First public release of **APO** — Engineering Evidence Platform — with the Repository Hygiene analyzer.

### Added

- Single-crate Rust 2024 CLI/library `apo` for local and remote Git repository hygiene analysis
- 35 observational rules across six weighted rubric categories (`apo-hygiene-v0.1`):
  - Documentation & Onboarding (20%)
  - Development Hygiene (15%)
  - Quality Assurance (20%)
  - Security & Supply Chain (15%)
  - Automation & Delivery (15%)
  - Project Management & Collaboration (15%)
- Markdown and JSON reports with scores, findings, gaps, recommendations, and evidence appendix
- Remote Git URI support via shallow clone to a temporary checkout
- `apo prompt` / `analyze --llm-prompt` LLM remediation prompts for closing hygiene gaps
- Repo-prefixed default artifacts (`{repo}-repository-hygiene.md|.json|-prompt.md`)
- CI (fmt, clippy, multi-OS test, coverage, audit/`cargo deny`, publish dry-run)
- Release workflow: multi-platform binaries + GitHub Release + crates.io publish on `v*` tags
- Dependabot + scheduled dependency workflow

### Notes

- Branch protection / required status checks cannot be verified from a local clone alone (reported as `Unknown` unless policy-as-code is present)
- MSRV: Rust **1.85** (edition 2024)
- Uses `gix` 0.85 (`revision` + `sha1`) for Git history sampling

[0.2.0]: https://github.com/thanos/apo/releases/tag/v0.2.0
[0.1.0]: https://github.com/thanos/apo/releases/tag/v0.1.0
