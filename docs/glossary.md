# Glossary

| Term | Meaning |
|------|---------|
| **Hygiene** | v0.1 analyzer: docs, local_dev, testing, security, delivery, collaboration ([control catalog](hygiene-controls.md)) |
| **Evidence** | Observational artifacts (paths, CI mentions, git samples) backing a finding |
| **Finding** | One rule evaluation with status, confidence, summary, evidence, remediation |
| **Policy** | Scoring layer that turns finding statuses into category / overall scores |
| **Pack** | Observational language or web ecosystem tooling definitions ([pack catalog](language-packs.md)) |
| **Overlay** | Project `.apo.toml` tooling needles merged on top of packs |
| **Knowledge maturity** | Coverage of knowledge kinds, freshness, ownership, link health ([artifact catalog](knowledge-artifacts.md)) |
| **AI maturity** | Prompt / MCP / agent / governance / workflow adoption signals ([signal catalog](ai-adoption-signals.md)) |
| **Dogfood** | Running `apo` against this repository in CI / `scripts/ci.sh` |
