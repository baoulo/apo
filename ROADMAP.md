# APO Roadmap

APO is an **Engineering Evidence Platform**. Deterministic analyzers and language packs are the source of truth; optional LLMs only narrate.

## Shipped

| Version | Focus |
|---------|--------|
| **0.1** | Repository Hygiene (observational rules, weighted rubric, remote URIs, LLM remediation prompts) |
| **0.2** | Knowledge + AI Evidence, optional Ollama narrative enrichment, evidence LLM prompts; language/web **pack engine**, `.apo.toml` overlays; **static SVG badges** (enterprise-friendly) |
| **0.3** | Unified evidence pack (`apo report`), `evidence_schema: apo-v0.3`, baseline diffs, SARIF, GitHub Action, QA depth rules, COBOL/FORTRAN/Pascal packs |

## Next

| Milestone | Focus |
|-----------|--------|
| **0.4** | Optional host platform API signals (branch protection) |
| **0.5** | Supply-chain observational depth (SBOM/provenance hints) |
| **0.6** | Portfolio / multi-repo batch |
| **1.0** | Freeze evidence + pack schemas; library API polish |

## Non-goals

- Grading code quality or executing linters/tests for pass/fail scoring
- Replacing SAST/dependency scanners
- Inventing evidence via LLMs
