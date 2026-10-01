# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Pack catalog: [docs/language-packs.md](docs/language-packs.md) (47 language/web packs)
- **External packs:** load declarative TOML from `.apo/packs/`, `--packs-dir`, or `APO_PACKS_DIR` without rebuilding ([`examples/packs/crystal.toml`](examples/packs/crystal.toml)); builtin id conflicts are errors

## [0.3.0] - 2026-10-01

Unified Evidence Pack: one report for hygiene + knowledge + AI, baseline diffs, SARIF, GitHub Action, deeper QA rules, and legacy language packs.

### Added

- `apo report` / `apo all` — unified pack with `evidence_schema: "apo-v0.3"` (`{repo}-repository-evidence-pack.{md,json}`)
- `--baseline <prior.json>` score/gap diffs (`diff` object + `*-diff.md`)
- SARIF 2.1.0 export (`--format sarif` / `--sarif`) for GitHub Code Scanning
- In-repo GitHub Action [`.github/actions/apo-report`](.github/actions/apo-report/) (PR summary + artifacts); dogfood CI uses it
- `.apo.toml` `[report]`, `[analyze]` (`commit_sample_limit`, `rule_disable`), `[ollama]` sections ([`examples/apo.toml`](examples/apo.toml))
- QA rules: `testing.property`, `testing.integration`, `testing.ui` (NotApplicable without UI ecosystem)
- Observational packs: `cobol`, `fortran`, `pascal`
- `--fail-on-score` on `apo report`; `apo prompt --pack`

### Changed

- Package version `0.3.0`; hygiene rule count **39**
- Dogfood CI runs unified `apo report` via the Action

### Notes

- `analyze` / `evidence` / `prompt` remain for backward compatibility
- Packs stay observational; Ollama remains narrative-only

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
- Multi-channel install packaging via [sastri](https://crates.io/crates/sastri) (`Sastri.toml`): `scripts/install.sh`, Homebrew / Scoop / asdf / mise under `packaging/`, and `.github/actions/setup-apo`
- Hygiene / knowledge / AI catalogs under [`docs/`](docs/)
- Narrative write-up: [Eating Our Own Dog Food](docs/eating-our-own-dog-food.md)

### Changed

- Package version `0.2.0`; description and README cover hygiene, evidence, and packs
- Hygiene rule count **36** (adds `documentation.doc_tooling`)
- Repository moved to the [baoulo](https://github.com/baoulo/apo) GitHub org; crates.io publisher remains [`thanos`](https://crates.io/users/thanos)

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

[0.3.0]: https://github.com/baoulo/apo/releases/tag/v0.3.0
[0.2.0]: https://github.com/baoulo/apo/releases/tag/v0.2.0
[0.1.0]: https://github.com/baoulo/apo/releases/tag/v0.1.0
