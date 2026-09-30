# Architecture

APO is a single Rust crate (`apo`) that observes a Git repository and emits
evidence reports. Analyzers are deterministic and offline-first; optional Ollama
enrichment narrates from analyzer outputs and never invents evidence.

## High-level flow

```text
target (path | git URI)
        │
        ▼
  source::resolve ──► Workspace (local or shallow clone)
        │
        ▼
  discovery::discover ──► RepoContext (inventory, git sample, CI text)
        │
        ├──────────────┬──────────────────┐
        ▼              ▼                  ▼
   rules::*      knowledge::*       ai_evidence::*
   (hygiene)     (docs graph)       (prompts/MCP/agents)
        │              │                  │
        ▼              └────────┬─────────┘
   policy::evaluate             ▼
        │              EvidenceReport
        ▼
   Report (+ optional badge / LLM prompt)
```

## Modules

| Module | Role |
|--------|------|
| `cli` / `main` | Clap entrypoints: `analyze`, `evidence`, `prompt` |
| `source` | Local path or remote shallow clone |
| `discovery` | File inventory, CI/script text, git commit sample |
| `rules` | Hygiene rule set (docs, local_dev, testing, security, delivery, collaboration) |
| `policy` | Category / overall scores from finding statuses |
| `packs` | Observational language/web tooling packs + `.apo.toml` overlays |
| `knowledge` | Knowledge kinds, freshness, ownership, links, graph |
| `ai_evidence` | Prompt / MCP / agent / governance signals + AI maturity |
| `ollama` | HTTP-only local LLM client for narrative enrichment |
| `report` | Markdown/JSON reports, badges, remediation prompts |

## Design constraints

- **Observational packs** — configs and CI/script text only; APO does not run
  toolchains to produce scores.
- **Evidence before opinion** — rules emit `Finding` + `EvidenceItem`; scoring
  lives in `policy`.
- **No invented evidence** — Ollama and LLM prompts summarize analyzer output.
- **Single crate** — library + CLI; MSRV Rust 1.85 (edition 2024).

## Outputs

Hygiene and evidence writers produce `{repo}-repository-hygiene.*` and
`{repo}-repository-evidence.*` (md/json), optional `*-prompt.md`, and static
SVG badges under a path you choose (this repo uses `docs/badges/`).
