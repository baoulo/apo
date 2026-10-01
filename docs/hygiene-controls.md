# Hygiene controls

`apo analyze` inventories **repository hygiene** as observational evidence: files,
directories, and CI/script text. It does **not** run formatters, linters, tests,
or forge APIs to produce scores.

There are **39 rules** in **6** weighted categories (`apo-hygiene-v0.1`). Language
and web **packs** only contribute config/CI needles into existing rule ids (for
example formatters, linters, and `documentation.doc_tooling`).

For scoring status weights and finding fields, see the [README — What gets
measured](../README.md#what-gets-measured) section.

## Scoring rubric

| Category | Weight | Role |
|----------|-------:|------|
| Documentation & Onboarding | 20% | Foundation |
| Development Hygiene | 15% | Daily developer experience |
| Quality Assurance | 20% | Code health |
| Security & Supply Chain | 15% | Risk reduction |
| Automation & Delivery | 15% | Velocity + reliability |
| Project Management & Collaboration | 15% | Process maturity |

## 1. Documentation & Onboarding (20%)

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
| `documentation.doc_tooling` | Docs quality gates from language packs (e.g. `mix doctor`, `mix docs --warnings-as-errors`, `cargo doc`). |

## 2. Development Hygiene (15%)

| Rule id | What APO looks for |
|---------|-------------------|
| `local_development.gitignore` | `.gitignore` |
| `local_development.editorconfig` | `.editorconfig` |
| `local_development.formatter` | Formatter configs (`rustfmt.toml`, Prettier, Black/Ruff in `pyproject.toml`, `.clang-format`, etc.) or format-related `package.json` signals. |
| `local_development.linter` | Linter configs (Clippy, ESLint, Ruff, golangci-lint, Flake8, Pylint, Credo, …) or lint scripts in `package.json`. |
| `local_development.type_checker` | `tsconfig.json` / MyPy / Pyright, or a strongly typed toolchain (`Cargo.toml`, `go.mod`). |
| `local_development.pre_commit_hooks` | `.pre-commit-config.yaml`, Husky, Lefthook, or related `package.json` deps. |
| `local_development.dev_environment` | Dev containers, Docker/Compose, Makefile/Justfile, or setup scripts. **Partial** if only CONTRIBUTING exists without setup automation. |

## 3. Quality Assurance (20%)

| Rule id | What APO looks for |
|---------|-------------------|
| `testing.framework` | Test dirs (`tests/`, `spec/`, `__tests__/`, …), test file naming patterns, or framework config (`package.json` test runners, pytest, Cargo tests). |
| `testing.coverage_config` | Coverage configs (`codecov.yml`, `.coveragerc`, …), coverage tooling in manifests, or coverage mentions in CI. |
| `testing.coverage_enforcement` | CI/config signals that coverage is gated (thresholds, `--fail-under`, codecov targets). **Partial** if tooling exists but enforcement is unclear. |
| `testing.static_analysis_ci` | Linters/SAST invoked in CI (Clippy, ESLint, Ruff, CodeQL, Semgrep, Sonar, …). |
| `testing.type_checking_ci` | Typecheck/compile steps in CI (`tsc`, MyPy, `cargo check`/`build`, `go build`/`vet`, …). |
| `testing.property` | Property-based testing (proptest, Hypothesis, QuickCheck, StreamData, PropEr, fast-check, …) in manifests, CI, or `prop_*` / `*_property_test.*` files. |
| `testing.integration` | Integration test dirs (`tests/integration`, …), `*_integration_test.*`, Testcontainers/Cucumber, or CI jobs mentioning integration. |
| `testing.ui` | When a UI/web ecosystem is present (React/Vue/Next, Playwright/Cypress, …): UI/E2E signals. **NotApplicable** when no UI ecosystem is detected. |

## 4. Security & Supply Chain (15%)

| Rule id | What APO looks for |
|---------|-------------------|
| `security.dependency_scanning` | Audit/deny configs or CI steps (`cargo audit`/`deny`, `npm audit`, Snyk, OSV, Trivy, CodeQL paths, …). |
| `security.secret_scanning` | Gitleaks / TruffleHog / detect-secrets configs or CI mentions; `.secrets.baseline`. |
| `security.policy` | Security policy document (`SECURITY.md`, etc.) — scored in the security category. |
| `security.dependency_updates` | Dependabot or Renovate configuration. |

## 5. Automation & Delivery (15%)

| Rule id | What APO looks for |
|---------|-------------------|
| `delivery.ci_workflows` | CI configs: `.github/workflows/*.yml`, GitLab CI, CircleCI, Azure Pipelines, Jenkinsfile, Travis, Buildkite, … |
| `delivery.release_workflows` | Release/publish workflows (by name or body: release, publish, goreleaser, `cargo`/`npm` publish, …) or release-tooling configs. |
| `delivery.branch_protection` | Policy-as-code hints (e.g. `.github/settings.yml`). **Unknown** when nothing is checked in — forge branch protection cannot be verified from a local clone. |
| `delivery.required_status_checks` | CI presence plus optional settings-as-code for required checks. **Unknown** if workflows exist but platform enforcement cannot be verified locally. |

## 6. Project Management & Collaboration (15%)

| Rule id | What APO looks for |
|---------|-------------------|
| `collaboration.commit_convention` | Commitlint / convention docs, and/or Conventional Commits patterns in sampled commit subjects (≥70% → **Enforced**, some adoption → **Partial**). |
| `collaboration.issue_linkage` | Issue refs in commit messages (`#123`, `PROJ-1`) and/or PR templates that ask for issue linkage. |
| `collaboration.codeowners` | Same CODEOWNERS paths as documentation, scored under collaboration. |
| `collaboration.review_configuration` | PR templates, CODEOWNERS, settings-as-code, or auto-assign/reviewer configs. |
| `collaboration.maintenance_activity` | Age of latest commit from Git history: ≤30 days **Enforced**, ≤90 **Present**, ≤180 **Partial**, older **Missing**. **Unknown** if the repo has no commits or Git metadata is unavailable. |

## Disabling rules

Via `.apo.toml` `[analyze].rule_disable` (or CLI/`Config::rule_disable`), listed rule
ids are excluded from findings and scoring. Disabled ids, pack overrides, skipped
tooling, and NotApplicable rules appear under report **Transparency** so exclusions
are not silent.

## Git history signals

Collaboration rules sample up to `commit_sample_limit` commits from `HEAD`
(default **100**; also used as shallow-clone `--depth` for remote URIs) and may
record days since last commit, Conventional Commits adoption, issue-id linkage,
and a small sample of recent subjects as evidence.

## Related

- [README — Rules catalog](../README.md#rules-catalog) (same tables mirrored for discoverability)
- [Language and web packs](language-packs.md) — ecosystem activation + tooling needles
- [Knowledge artifacts](knowledge-artifacts.md) — documentation kinds
- [AI adoption signals](ai-adoption-signals.md) — prompts, MCP, agents, governance
- [Glossary](glossary.md)
- [Design notes](design.md)
- [Eating Our Own Dog Food](eating-our-own-dog-food.md)
