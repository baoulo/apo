# AI adoption signals

`apo evidence` inventories **AI adoption** as observational evidence: prompt
libraries, MCP configs, agent definitions, governance docs, and CI/workflow
mentions of AI-assisted work. It does **not** call model APIs for scoring
(optional Ollama is narrative-only and separate).

Detection is path-based plus CI/script text needles. Artifacts are classified
into kinds; findings and an **AI maturity** score are derived from those signals.

## Artifact kinds

| Kind | id | What APO looks for |
|------|----|-------------------|
| Prompt library | `prompt_library` | Paths under `prompts/`, `.cursor/prompts`, `.github/prompts`, `baoulo/prompts`, or a filename/path containing `prompt` / `prompt-` with `.md` / `.txt` / `.yaml` / `.yml` (and not treated as a template) |
| Prompt template | `prompt_template` | Same prompt path rules when the path also contains `template` |
| MCP config | `mcp_config` | `mcp.json` / `mcp.yml` / `mcp.yaml`, `.cursor/mcp*`, `claude_desktop_config*`, or path containing `mcp_config` |
| Agent definition | `agent_definition` | `AGENTS.md` / `agent.md`, `.cursor/agents`, `/agents/`, or `agent.yaml` / `agent.yml` / `agent.json` |
| AI governance | `ai_governance` | Doc (`.md` / `.txt`) whose path includes `ai` / `llm` / `genai` / `generative` **and** `govern` / `policy` / `ethics` / `responsible` |
| AI coding standard | `ai_coding_standard` | `.md` whose path includes `ai` / `copilot` / `cursor` / `llm` **and** `standard` / `guideline` / `playbook` / `rules` |
| AI review workflow | `ai_review_workflow` | CI/workflow text mentioning e.g. `ai review`, `llm review`, `coderabbit`, `chatgpt`, `openai`, `anthropic`, `cursor bugbot`, `gemini` |
| AI-assisted docs | `ai_assisted_docs` | CI mentions of `llm`, `ollama`, `ai generated`, `chatgpt`, `copilot` (default bucket when not clearly tests/release) |
| AI-assisted testing | `ai_assisted_testing` | Same CI AI needles when the matching detail also contains `test` |
| AI-assisted release | `ai_assisted_release` | Same CI AI needles when the matching detail contains `release` or `changelog` |

Prompt paths are checked before MCP/agents/governance. A path matches at most one
file-based kind in that order (CI-derived kinds are added separately).

## Derived flags

| Flag | True when |
|------|-----------|
| `prompt_count` | Count of `prompt_library` + `prompt_template` artifacts |
| `prompt_versioning` | Prompt paths include `/v1/` / `/v2/` / `version` / semver-like `v?\d+\.\d+`, or prompt file content mentions `version:` / `semver` / `changelog`, or `prompts/CHANGELOG.md` / `prompts/VERSION` exists |
| `governance_present` | Any `ai_governance` or `ai_coding_standard` artifact |
| `human_review_signals` | Any `ai_review_workflow`, **or** `.github/pull_request_template.md` / `PULL_REQUEST_TEMPLATE.md` |
| `workflow_adoption` | Any of: assisted docs/tests/release, review workflow, MCP, or agent definition |

## AI findings

| Finding id | Meaning |
|------------|---------|
| `ai.prompt_library` | No prompts (**Missing**); prompts without clear versioning (**Present**); prompts + versioning (**Enforced**) |
| `ai.mcp` | MCP config present or missing |
| `ai.agents` | Agent definition(s) present or missing |
| `ai.governance` | Governance / AI coding standard present or missing |
| `ai.human_review` | Human-review signals (AI review CI and/or PR templates) present or missing |
| `ai.workflow_adoption` | Broader AI workflow signals present or missing |

## AI maturity score

Points accumulate (clamped 0–100):

| Signal | Points |
|--------|-------:|
| ≥1 prompt artifact | +20 |
| ≥3 prompt artifacts | +10 |
| Prompt versioning | +15 |
| Governance or AI coding standard | +20 |
| Human review signals | +15 |
| Workflow adoption | +10 |
| MCP config | +5 |
| Agent definition | +5 |

Reported as `ai_maturity` on the evidence report.

## Risks (examples)

Deterministic risk strings may include:

- Prompts without AI governance docs
- Agents/MCP without clear human-review gates
- AI-assisted CI mentions without a reusable prompt library

## Related

- Command: `apo evidence` / `apo evidence --llm-prompt`
- [Knowledge artifacts](knowledge-artifacts.md) — documentation kinds on the same report
- [Hygiene controls](hygiene-controls.md) — `apo analyze` inventory
- [Language and web packs](language-packs.md) — ecosystem tooling packs
- [AI governance (this repo)](ai-governance.md)
- [Glossary](glossary.md)
- [README — Knowledge + AI evidence](../README.md#knowledge--ai-evidence)
