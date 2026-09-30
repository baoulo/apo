# ADR 0001: Single crate, observational evidence

- **Status:** Accepted
- **Date:** 2026-09-30

## Context

APO needs to score repository hygiene and knowledge/AI maturity across many
ecosystems without becoming a multi-package monorepo or executing arbitrary
toolchains in analyzed repos.

## Decision

1. Ship as **one Rust crate** (`apo`) with library + CLI.
2. Treat **deterministic analyzers** as the source of truth for findings and scores.
3. Detect ecosystem tooling via **observational packs** (manifests, config files,
   CI/script text) and optional `.apo.toml` overlays — never by running the
   project's formatters, linters, or tests for scoring.
4. Keep optional Ollama enrichment **narrative-only** over analyzer outputs.

## Consequences

- Fairer multi-language scoring without installing each language toolchain.
- Scores can lag true CI enforcement when policy lives only on the forge API
  (branch protection, required checks) — mitigated with settings-as-code files.
- Extending coverage means adding packs/rules, not new crates.
