# Knowledge artifacts

`apo evidence` inventories **knowledge** as observational evidence: documentation
paths classified by kind, plus freshness, local markdown link health, ownership
signals, and a knowledge graph. It does **not** grade prose quality or invent
docs.

Classification is path-based (deterministic). Optional Ollama enrichment may
narrate from these artifacts; it never creates kinds that were not observed.

## Expected kinds (coverage)

Coverage scoring expects these kinds. Missing ones appear in `kinds_missing` and
drive `knowledge.coverage`:

| Kind | `kinds_*` id | Path signals (examples) |
|------|--------------|-------------------------|
| README | `readme` | Filename starts with `readme` |
| ADR | `adr` | Under `/adr/` or `/adrs/`, or name `adr-` / `adr_` + doc extension |
| Architecture | `architecture` | Path contains `architect` + doc extension |
| Design | `design` | Path contains `design`, or `design.md` |
| Runbook | `runbook` | Path contains `runbook` + doc extension |
| API | `api` | `/api/`, `openapi`, `swagger`, or `api.md` (docs or yaml/yml/json) |
| Onboarding | `onboarding` | `onboard`, `getting-started` / `getting_started`, or `contributing*` + doc extension |
| Glossary | `glossary` | Path contains `glossary`, or `glossary.md` |
| Prompt library | `prompt_library` | `prompts/`, `.cursor/prompts`, `.github/prompts`, `baoulo/prompts`, or path with `prompt` + doc extension |

**Coverage score** = (distinct expected kinds present ÷ 9) × 100.

## Also classified (not required for coverage)

These kinds are recorded when found but are **not** in the expected set, so they
do not count toward `kinds_missing`:

| Kind | `kinds_*` id | Path signals |
|------|--------------|--------------|
| Business rules | `business_rules` | `business-rule`, `business_rule`, or `domain-rule` + doc extension |
| Diagram | `diagram` | `.mmd` / `.puml` / `.plantuml` / `.drawio`; or `diagram`/`architecture` SVG; or path containing `diagram` with doc/png/svg |
| Other doc | `other_doc` | Other `.md`/`.mdx`/`.rst`/`.adoc`/`.txt` under `docs/` or `doc/` that did not match a sharper kind |

## Doc extensions

Unless noted, “doc extension” means: `.md`, `.mdx`, `.rst`, `.adoc`, `.txt`.

## Freshness

Each artifact gets `days_since_modified` from filesystem mtime when available.

| Days since modified | Freshness contribution |
|--------------------:|------------------------|
| ≤ 30 | 100 |
| ≤ 90 | 80 |
| ≤ 180 | 55 |
| ≤ 365 | 30 |
| \> 365 | 10 |

Artifacts older than **180 days** are flagged `stale: true` and feed
`knowledge.freshness`. Aggregate **freshness score** averages per-artifact
contributions (default 50 if no dates).

## Link health

APO samples local relative links in markdown docs. Broken targets become
`broken_links` and `knowledge.links`.

| Broken local links | Link health score |
|-------------------:|------------------:|
| 0 | 100 |
| 1–3 | 70 |
| 4–10 | 40 |
| \> 10 | 15 |

## Ownership signal

**Ownership bonus** is 100 if `CODEOWNERS`, `.github/CODEOWNERS`, or
`docs/CODEOWNERS` exists; otherwise 0. Used only in the maturity mix below.

## Knowledge maturity

```text
maturity = 0.45 × coverage
         + 0.25 × freshness
         + 0.15 × link_health
         + 0.15 × ownership_bonus
```

Clamped to 0–100. Reported as `knowledge_maturity` on the evidence report.

## Knowledge findings

| Finding id | Meaning |
|------------|---------|
| `knowledge.coverage` | Whether all expected kinds are present (**Enforced**) or some missing (**Partial**) |
| `knowledge.freshness` | No artifacts (**Missing**), recently maintained (**Enforced**), or stale/partial |
| `knowledge.links` | Broken local markdown links (**Partial**) or none found (**Enforced**) |
| `knowledge.architecture` | Architecture and/or ADR kinds present (**Present**) or missing |
| `knowledge.operational_readiness` | Runbook kind present (**Present**) or missing |

## Knowledge graph

From discovered artifacts and inventory, APO builds a graph linking docs, code,
tests, ADRs, and runbooks (edges such as doc↔code and doc↔doc). Used in the
evidence report; not a separate score.

## Related

- Command: `apo evidence` / `apo evidence --llm-prompt`
- [Hygiene controls](hygiene-controls.md) — separate `apo analyze` inventory
- [Language and web packs](language-packs.md) — ecosystem tooling packs
- [AI adoption signals](ai-adoption-signals.md) — AI side of `apo evidence`
- [Glossary](glossary.md)
- [README — Knowledge + AI evidence](../README.md#knowledge--ai-evidence)
