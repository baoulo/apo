# Design notes

## Scoring model

Each hygiene rule returns a `Status` (`Missing`, `Partial`, `Present`,
`Enforced`, `Unknown`, …). The policy layer maps statuses to category scores and
an overall score. The full rule list is in [hygiene-controls.md](hygiene-controls.md).
Knowledge maturity mixes coverage, freshness, link health, and CODEOWNERS
([knowledge-artifacts.md](knowledge-artifacts.md)). AI maturity awards points for
prompts, versioning, governance, human review, workflows, MCP, and agents
([ai-adoption-signals.md](ai-adoption-signals.md)).

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
