# Design notes

## Scoring model

Each hygiene rule returns a `Status` (`Missing`, `Partial`, `Present`,
`Enforced`, `Unknown`, …). The policy layer maps statuses to category scores and
an overall score. Knowledge and AI maturity use separate coverage / risk
formulas over classified artifacts.

## Pack overlays

Built-in packs activate from manifests (e.g. `Cargo.toml` → `rust`). Projects
may force packs or add needles in `.apo.toml`:

```toml
[[tooling]]
id = "custom.docs"
maps_to = "documentation.doc_tooling"
ci_commands = ["cargo doc"]
```

## Remediation prompts

`apo analyze --llm-prompt` / `evidence --llm-prompt` write paste-ready prompts
that list gaps with remediations. Prompts must not invent controls that the
report did not observe.
