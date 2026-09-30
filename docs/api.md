# Library API (crate `apo`)

Public surface used by the CLI and embedders:

| Item | Purpose |
|------|---------|
| `Config` | Target path/URI, formats, Ollama options, badge paths |
| `analyze` / `analyze_with_workspace` | Hygiene report |
| `analyze_evidence` / `analyze_evidence_with_workspace` | Knowledge + AI report |
| `Report` / `EvidenceReport` | Structured results + writers |
| `Workspace` | Resolved local/remote clone handle |
| `Error` / `Result` | Crate error type |

See module docs (`cargo doc --open`) and [ARCHITECTURE.md](../ARCHITECTURE.md)
for the internal pipeline. The CLI commands `analyze`, `evidence`, and `prompt`
are thin wrappers around these entrypoints.
