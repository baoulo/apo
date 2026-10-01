# APO

[![CI](https://github.com/baoulo/apo/actions/workflows/ci.yml/badge.svg)](https://github.com/baoulo/apo/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/apo.svg)](https://crates.io/crates/apo)
[![docs.rs](https://docs.rs/apo/badge.svg)](https://docs.rs/apo)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.85-blue)](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/)
![APO hygiene](docs/badges/apo-hygiene.svg)
![APO evidence](docs/badges/apo-evidence.svg)

**APO** (from Greek *apothiki* — "storehouse") is an **Engineering Evidence Tool**.

You use `apo` to scan a git repository to make an inventory of Engineering evidence — what the repo already has on disk and in sampled Git/CI text:

**Hygiene controls** — docs, editor/setup, tests/coverage gates, dependency & secret scanning signals, CI/release automation, CODEOWNERS/templates/commit conventions.
**Knowledge artifacts** - README, ADRs, architecture/design, runbooks, API docs, glossaries, diagrams, onboarding, prompt libraries.
**AI adoption signals** — prompts, agents, MCP configs, AI governance, AI-assisted workflow hints
Then it turns that inventory into findings, scores, badges, and optional remediation prompts — without inventing controls or running your toolchains.

`apo` with local or remote  Git repository. 

Hygiene rules emit evidence only. A separate policy layer turns those observations into category and overall scores. Knowledge and AI analyzers stay deterministic by default; Ollama never invents evidence — it only narrates from analyzer outputs. Language packs observe configs and CI/script text; they do not run toolchains. APO does not grade “code quality” or run your tests.

- **v0.1 — Repository Hygiene** — documentation, local development controls, testing gates, security/supply-chain signals, delivery automation, and collaboration practices
- **v0.2 — Knowledge + AI Evidence** — knowledge coverage/freshness/ownership, knowledge graph, prompt/MCP/agent artifacts, and AI governance maturity (optional local Ollama enrichment); observational **language/web packs** and `.apo.toml` tooling overlays for fairer multi-ecosystem scoring
- **v0.3 — Unified Evidence Pack** — `apo report` / `apo all` combines hygiene + knowledge + AI into one pack (`evidence_schema: apo-v0.3`), baseline diffs, SARIF export, and an in-repo GitHub Action



## Install

Release binaries ship on [GitHub Releases](https://github.com/baoulo/apo/releases) (with `SHA256SUMS`). Prefer a package manager or `install.sh` for day-to-day use; `cargo install` is the contributor path. Packaging is scaffolded with [sastri](https://crates.io/crates/sastri) (`Sastri.toml`).

### Homebrew

From [baoulo/homebrew-tap](https://github.com/baoulo/homebrew-tap) (fill formula checksums from `SHA256SUMS` after each release):

```bash
brew install baoulo/tap/apo
```

### Scoop (Windows)

From [baoulo/scoop-bucket](https://github.com/baoulo/scoop-bucket):

```bash
scoop bucket add baoulo https://github.com/baoulo/scoop-bucket
scoop install apo
```

### mise

See [packaging/mise/README.md](packaging/mise/README.md).

```bash
mise use -g github:baoulo/apo@0.2.0
```

### asdf

From [baoulo/asdf-apo](https://github.com/baoulo/asdf-apo) (push `packaging/asdf-apo` there):

```bash
asdf plugin add apo https://github.com/baoulo/asdf-apo.git
asdf install apo 0.2.0
```

### curl

Installs the latest tag into `~/.local/bin` (or `APO_INSTALL_DIR`). Pin with `APO_VERSION=v0.2.0`.

```bash
curl -fsSL https://raw.githubusercontent.com/baoulo/apo/main/scripts/install.sh | sh
```

### crates.io / from source

The crates.io package remains under the [`thanos`](https://crates.io/users/thanos) publisher; source and issues live at [baoulo/apo](https://github.com/baoulo/apo).

```bash
cargo install apo
cargo install --git https://github.com/baoulo/apo
# or from a checkout:
cargo install --path .
```

### CI

```yaml
- uses: baoulo/apo/.github/actions/setup-apo@main
  # with:
  #   version: "0.3.0"   # optional; omit for latest

# Unified pack + PR summary + artifacts (preferred for v0.3+)
- uses: baoulo/apo/.github/actions/apo-report@main
  with:
    format: both
    write-sarif: "true"
    # baseline: path/to/prior-pack.json
    # fail-on-score: "70"
```

Templates under [`packaging/`](packaging/) are regenerated with `sastri generate --force` (CI/release workflows are owned by this repo and stay out of Sastri generation).
## Usage

```bash
# Unified evidence pack (v0.3) — hygiene + knowledge + AI
apo report .
apo all . --format both --output ./out
apo report . --baseline prior-pack.json --sarif
apo report . --fail-on-score 70 --llm-prompt

# Hygiene (v0.1)
apo analyze .
apo analyze . --format json
apo analyze . --output report.md
apo analyze . --format both --output ./out

# Remote Git URI (shallow-cloned to a temp dir, then cleaned up)
apo analyze https://github.com/thanos/ex_arrow
apo analyze git@github.com:thanos/ex_arrow.git --format both

# LLM remediation prompt (paste into Cursor/ChatGPT/etc. to close gaps)
apo analyze . --llm-prompt
apo prompt .
apo prompt --pack .
apo prompt https://github.com/thanos/ex_arrow --output ./out

# Knowledge + AI evidence (v0.2)
apo evidence .
apo evidence . --format both
apo evidence . --llm-prompt
apo evidence . --ollama --ollama-model llama3.2
```

Default artifacts:

- Local targets: written next to the analyzed repository
- Remote URIs: written to the current working directory

Hygiene files:

- `{repo}-repository-hygiene.md`
- `{repo}-repository-hygiene.json` (when `--format json` or `both`)
- `{repo}-repository-hygiene-prompt.md` (when `--llm-prompt` or `apo prompt`)
- `{repo}-repository-hygiene-badge.svg` (default; skip with `--no-badge`)

Unified pack files (`apo report` / `apo all`):

- `{repo}-repository-evidence-pack.md` / `.json` (`evidence_schema: "apo-v0.3"`)
- `{repo}-repository-evidence-pack.sarif` (when `--sarif` or `--format sarif`)
- `{repo}-repository-evidence-pack-diff.md` (when `--baseline` is set)
- `{repo}-repository-evidence-pack-prompt.md` (when `--llm-prompt`)
- Hygiene + evidence badges (same as below)

Evidence files:

- `{repo}-repository-evidence.md`
- `{repo}-repository-evidence.json` (when `--format json` or `both`)
- `{repo}-repository-evidence-prompt.md` (when `evidence --llm-prompt`)
- `{repo}-repository-evidence-badge.svg` (default; skip with `--no-badge`)

`{repo}` is the repository directory name (local) or the remote repo basename (e.g. `ex_arrow` from `https://github.com/thanos/ex_arrow`).

### Badges (enterprise-friendly)

APO writes **self-contained SVG badges** next to reports — no shields.io or other external CDN. That works on **GitHub.com, GitHub Enterprise, GitLab EE**, and other forges when you embed a **relative** path (the forge serves the file with the same auth as the README).

```markdown
![APO hygiene](docs/badges/apo-hygiene.svg)
![APO evidence](docs/badges/apo-evidence.svg)
```

```bash
apo analyze . --badge-output docs/badges/apo-hygiene.svg
apo evidence . --badge-output docs/badges/apo-evidence.svg
# or default names beside the repo / --output directory
apo analyze .          # writes {repo}-repository-hygiene-badge.svg
apo evidence . --no-badge
```

Color bands: ≥80 green, ≥50 yellow, &lt;50 red (`n/a` gray). Hygiene shows overall score; evidence shows knowledge + AI maturity side by side.

Example CI step (GitHub.com or GitHub Enterprise Actions) to refresh a tracked badge:

```yaml
- name: APO hygiene badge
  run: |
    cargo install apo --locked
    apo analyze . --format json --badge-output docs/badges/apo-hygiene.svg
    # optional: commit docs/badges/ or upload as a workflow artifact
```

### Knowledge + AI evidence

`apo evidence` inventories documentation and AI artifacts, builds a knowledge graph (docs ↔ code ↔ tests ↔ ADRs ↔ runbooks), scores knowledge and AI maturity, and lists risks. Everything is offline-capable.

Knowledge kinds and scoring: [docs/knowledge-artifacts.md](docs/knowledge-artifacts.md).
AI adoption signals and scoring: [docs/ai-adoption-signals.md](docs/ai-adoption-signals.md).

Optional `--ollama` calls a local Ollama HTTP API for semantic classification, architecture summary, doc↔code linking hints, and an executive narrative. Deterministic analyzer data remains the source of truth; if Ollama is unreachable, APO falls back with notes and still writes the report. The client speaks plain HTTP (default `http://127.0.0.1:11434`); HTTPS endpoints are not supported.

```bash
apo evidence . --format both
apo evidence . --llm-prompt
apo evidence . --ollama --ollama-url http://127.0.0.1:11434 --ollama-model llama3.2
# or: APO_OLLAMA_URL / APO_OLLAMA_MODEL
```

### LLM remediation prompt

`apo prompt` / `analyze --llm-prompt` and `evidence --llm-prompt` generate paste-ready instructions for an LLM coding agent.

**Hygiene** (`apo prompt` / `analyze --llm-prompt`):

- Repository identity and current weighted score
- Rubric priorities
- Controls already satisfied (do not redo)
- Enumerated gaps (`Missing` / `Partial` / `Unknown`) with remediations and evidence
- Constraints and a required changelog deliverable mapping files → APO rule ids

**Evidence** (`evidence --llm-prompt`):

- Knowledge and AI maturity snapshot
- Present artifacts and satisfied findings (do not redo)
- Missing knowledge kinds, risks, and broken links
- Knowledge and AI gap findings with remediations
- Changelog deliverable mapping files → APO rule ids (`knowledge.*`, `ai.*`)

Example:

```bash
apo prompt . > /tmp/fix-hygiene.md   # also writes {repo}-repository-hygiene-prompt.md
apo prompt . --evidence               # writes {repo}-repository-evidence-prompt.md
apo evidence . --llm-prompt           # same evidence prompt alongside the report
# then paste into your LLM agent against the repo checkout
```

### Language and web packs

Hygiene rules use **observational language/web packs** so Elixir, SQL, Nim, COBOL, FORTRAN, Pascal, and other ecosystems are not under-scored relative to Rust/Node. Packs activate from manifests (e.g. `mix.exs`, `go.mod`, `fpm.toml`, `next.config.*`, `*.cob`) and contribute config paths + CI/Makefile needles into existing rule ids.

Full pack catalog (activation signals + tooling → rule mappings): [docs/language-packs.md](docs/language-packs.md).
Add a pack without rebuilding: drop a TOML file in `.apo/packs/` (see [external packs](docs/language-packs.md#external-packs) and [`examples/packs/crystal.toml`](examples/packs/crystal.toml)), or pass `--packs-dir` / `APO_PACKS_DIR`.

Optional repo overlay [`.apo.toml`](examples/apo.toml):

```toml
[ecosystem]
languages = ["elixir"]
web = ["nextjs"]

[report]
format = "both"

[analyze]
# rule_disable = ["collaboration.maintenance_activity"]

[[tooling]]
id = "elixir.extra"
maps_to = "local_development.linter"
configs = [".credo.exs"]
ci_commands = ["mix credo --strict"]
```

Packs never execute toolchains; they only observe files and scripts. See [ROADMAP.md](ROADMAP.md).

## Pipeline

```text
# Unified pack (v0.3)
Repository → Discovery → Hygiene + Knowledge + AI → EvidencePack (apo-v0.3)
  → optional baseline diff → Markdown + JSON + SARIF + badges

# Hygiene (v0.1)
Repository → Discovery → Hygiene Rules → Evidence → Policy/Scoring → Markdown + JSON

# Knowledge + AI (v0.2)
Repository → Discovery → Knowledge + AI analyzers → (optional Ollama) → Evidence report
```

1. **Discovery** — walk the tree (respecting `.gitignore`), index files, detect ecosystem signals (`Cargo.toml`, `package.json`, CI workflows, etc.), and sample recent Git history.
2. **Rules / analyzers** — hygiene rules emit findings; knowledge and AI analyzers emit structured evidence (artifacts, graph edges, risks, maturity scores).
3. **Policy** (hygiene) — maps finding statuses to numeric weights, averages them per category, then applies the weighted rubric for the overall score.
4. **Report** — writes Markdown and/or JSON.

---

## What gets measured

APO v0.3+ runs **39 rules** across **6 categories** (packs feed several of them). Full control catalog: [Hygiene controls](docs/hygiene-controls.md). Pack catalog: [Language and web packs](docs/language-packs.md).

Every finding includes:

| Field | Meaning |
|-------|---------|
| `rule` | Stable id, e.g. `documentation.readme` |
| `category` | One of the six hygiene categories |
| `status` | Observational outcome (see below) |
| `confidence` | `High` / `Medium` / `Low` how sure the observation is |
| `summary` | Short description of what was observed |
| `evidence` | Paths and/or detail strings backing the finding |
| `remediation` | Optional guidance when the control looks weak or missing |

### Status values

| Status | Meaning | Policy weight |
|--------|---------|---------------|
| `Enforced` | Control present and appears actively applied (e.g. keywords in docs, CI steps) | 100 |
| `Present` | Control artifact exists | 80 |
| `Partial` | Weak or incomplete signal | 45 |
| `Unknown` | Cannot decide from a local clone alone (or insufficient Git history) | 25 |
| `Missing` | No evidence found | 0 |
| `NotApplicable` | Control does not apply (excluded from averages) | — |

**Category score** = average status weight of that category’s findings (excluding `NotApplicable`).

**Overall score** = weighted average of category scores using the v0.1 rubric below. If a category has no scorable findings, its weight is redistributed proportionally among the remaining categories.

### Scoring rubric (`apo-hygiene-v0.1`)

| Category | Weight | Notes |
|----------|-------:|-------|
| Documentation & Onboarding | 20% | Foundation |
| Development Hygiene | 15% | Daily developer experience |
| Quality Assurance | 20% | Code health |
| Security & Supply Chain | 15% | Risk reduction |
| Automation & Delivery | 15% | Velocity + reliability |
| Project Management & Collaboration | 15% | Process maturity |

Gaps (`Missing`, `Partial`, `Unknown`) appear under **Missing controls** and feed **Recommendations** when a remediation string is set.

---

## Report contents

Both Markdown and JSON include:

| Section | Contents |
|---------|----------|
| Header / metadata | APO version, analyzer name (`repository-hygiene`), repository label, optional `source_uri` / `checkout_path`, timestamp |
| Executive summary | Overall score and counts of enforced / present / gap findings |
| Category scores | Per-category score plus counts of each status |
| Findings | All 36 rule results with evidence |
| Missing controls | Rule ids with gap status |
| Recommendations | Remediation text derived from gap findings |
| Evidence appendix (Markdown) | Compact table of rule → status → evidence paths |

---

## Rules catalog

Detailed tables for every hygiene rule live in [docs/hygiene-controls.md](docs/hygiene-controls.md). Summary below.

### 1. Documentation & Onboarding (20% — Foundation)

| Rule id | What APO looks for |
|---------|-------------------|
| `documentation.readme` | `README.md` (or common variants). **Enforced** if content mentions quickstart/install/usage-style keywords. |
| `documentation.contributing` | `CONTRIBUTING.md` (root, `.github/`, or `docs/`). |
| `documentation.security` | `SECURITY.md` (root, `.github/`, or `docs/`). |
| `documentation.license` | `LICENSE` / `LICENCE` / `COPYING` (and common extensions). |
| `documentation.architecture` | `ARCHITECTURE.md`, `DESIGN.md`, or docs paths containing “architecture”. |
| `documentation.adrs` | ADR directories (`docs/adr`, `adr`, …) or ADR markdown files. |
| `documentation.runbooks` | Paths/dirs containing “runbook”. |
| `documentation.codeowners` | `CODEOWNERS` (root, `.github/`, `docs/`, `.gitlab/`). |
| `documentation.issue_templates` | `.github/ISSUE_TEMPLATE/` or issue template files. |
| `documentation.pr_templates` | Pull request template files (e.g. `.github/pull_request_template.md`). |
| `documentation.doc_tooling` | Docs quality gates from language packs (e.g. `mix doctor`, `mix docs --warnings-as-errors`). |

### 2. Development Hygiene (15% — Daily developer experience)

| Rule id | What APO looks for |
|---------|-------------------|
| `local_development.gitignore` | `.gitignore` |
| `local_development.editorconfig` | `.editorconfig` |
| `local_development.formatter` | Formatter configs (`rustfmt.toml`, Prettier, Black/Ruff in `pyproject.toml`, `.clang-format`, etc.) or format-related `package.json` signals. |
| `local_development.linter` | Linter configs (Clippy, ESLint, Ruff, golangci-lint, Flake8, Pylint, …) or lint scripts in `package.json`. |
| `local_development.type_checker` | `tsconfig.json` / MyPy / Pyright, or a strongly typed toolchain (`Cargo.toml`, `go.mod`). |
| `local_development.pre_commit_hooks` | `.pre-commit-config.yaml`, Husky, Lefthook, or related `package.json` deps. |
| `local_development.dev_environment` | Dev containers, Docker/Compose, Makefile/Justfile, or setup scripts. **Partial** if only CONTRIBUTING exists without setup automation. |

### 3. Quality Assurance (20% — Code health)

| Rule id | What APO looks for |
|---------|-------------------|
| `testing.framework` | Test dirs (`tests/`, `spec/`, `__tests__/`, …), test file naming patterns, or framework config (`package.json` test runners, pytest, Cargo tests). |
| `testing.coverage_config` | Coverage configs (`codecov.yml`, `.coveragerc`, …), coverage tooling in manifests, or coverage mentions in CI. |
| `testing.coverage_enforcement` | CI/config signals that coverage is gated (thresholds, `--fail-under`, codecov targets). **Partial** if tooling exists but enforcement is unclear. |
| `testing.static_analysis_ci` | Linters/SAST invoked in CI (Clippy, ESLint, Ruff, CodeQL, Semgrep, Sonar, …). |
| `testing.type_checking_ci` | Typecheck/compile steps in CI (`tsc`, MyPy, `cargo check`/`build`, `go build`/`vet`, …). |

### 4. Security & Supply Chain (15% — Risk reduction)

| Rule id | What APO looks for |
|---------|-------------------|
| `security.dependency_scanning` | Audit/deny configs or CI steps (`cargo audit`/`deny`, `npm audit`, Snyk, OSV, Trivy, CodeQL paths, …). |
| `security.secret_scanning` | Gitleaks / TruffleHog / detect-secrets configs or CI mentions; `.secrets.baseline`. |
| `security.policy` | Security policy document (`SECURITY.md`, etc.) — scored in the security category. |
| `security.dependency_updates` | Dependabot or Renovate configuration. |

### 5. Automation & Delivery (15% — Velocity + reliability)

| Rule id | What APO looks for |
|---------|-------------------|
| `delivery.ci_workflows` | CI configs: `.github/workflows/*.yml`, GitLab CI, CircleCI, Azure Pipelines, Jenkinsfile, Travis, Buildkite, … |
| `delivery.release_workflows` | Release/publish workflows (by name or body: release, publish, goreleaser, `cargo`/`npm` publish, …) or release-tooling configs. |
| `delivery.branch_protection` | Policy-as-code hints (e.g. `.github/settings.yml`). **Unknown** when nothing is checked in — platform branch protection cannot be verified from a local clone. |
| `delivery.required_status_checks` | CI presence plus optional settings-as-code for required checks. **Unknown** if workflows exist but platform enforcement cannot be verified locally. |

### 6. Project Management & Collaboration (15% — Process maturity)

| Rule id | What APO looks for |
|---------|-------------------|
| `collaboration.commit_convention` | Commitlint / convention docs, and/or Conventional Commits patterns in sampled commit subjects (≥70% → **Enforced**, some adoption → **Partial**). |
| `collaboration.issue_linkage` | Issue refs in commit messages (`#123`, `PROJ-1`) and/or PR templates that ask for issue linkage. |
| `collaboration.codeowners` | Same CODEOWNERS paths as documentation, scored under collaboration. |
| `collaboration.review_configuration` | PR templates, CODEOWNERS, settings-as-code, or auto-assign/reviewer configs. |
| `collaboration.maintenance_activity` | Age of latest commit from Git history: ≤30 days **Enforced**, ≤90 **Present**, ≤180 **Partial**, older **Missing**. **Unknown** if the repo has no commits or Git metadata is unavailable. |

---

## Git history signals

For collaboration rules, APO samples up to `commit_sample_limit` commits from `HEAD` (default **100**; also used as shallow-clone `--depth` for remote URIs) and records:

- Days since last commit
- Share of subjects matching Conventional Commits
- Share of commits referencing an issue id
- A small sample of recent commit subjects (as evidence detail)

---

## Design

- One crate, one binary (`apo`)
- Rules/analyzers produce evidence; hygiene policy computes scores
- Detect tools rather than hard-code a single ecosystem
- Local-clone limits: branch protection and required checks on the hosting platform are reported as `Unknown` unless policy-as-code is checked in
- Ollama is optional enrichment only; analyzers work fully offline

## Development

```bash
./scripts/ci.sh   # mirrors CI: fmt, clippy, test, llvm-cov (≥80%), deny, audit, apo self-analysis
cargo run -- analyze . --format both --badge-output docs/badges/apo-hygiene.svg
cargo run -- evidence . --format both --badge-output docs/badges/apo-evidence.svg
```

`./scripts/ci.sh` dogfoods APO on this repo (writes `out/apo-self/` reports and refreshes `docs/badges/`). CI runs the same as the **APO self-analysis** job. See [Eating Our Own Dog Food](docs/eating-our-own-dog-food.md) for a short narrative of the first self-remediation pass.

Requires `cargo-llvm-cov` and `cargo-deny` on PATH (same as CI). See [CONTRIBUTING.md](CONTRIBUTING.md) for PR and release workflow details. See [CHANGELOG.md](CHANGELOG.md) for release notes.

## License

MIT — see [LICENSE](LICENSE).
