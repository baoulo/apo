# Eating Our Own Dog Food

*A short account of running APO against APO.*

We built APO to turn repository practice into evidence — and to tell you, without
hand-waving, what is missing. The honest test of that claim is whether we would
trust the same reports on our own tree.

So we did.

## First look in the mirror

CI already dogfoods the binary: `./scripts/ci.sh` and the **APO self-analysis**
job write hygiene and evidence reports under `out/apo-self/`, and refresh the
static badges in [`docs/badges/`](badges/). Those badges sit in the README so the
scores are visible without leaving the forge.

On the first deliberate pass, the numbers were blunt:

| Metric | Score |
|--------|------:|
| Hygiene | 58.7 |
| Knowledge maturity | 43.8 |
| AI maturity | 0.0 |

Sixteen hygiene gaps. Missing ADRs, architecture notes, runbooks, CODEOWNERS,
templates, secret scanning, editor defaults, setup automation. Knowledge kinds
empty for almost everything past a README. AI maturity at zero — no agents file,
no governance, no versioned prompts.

That was the point of the tool. APO did not scold; it listed paths and
remediations. With `--llm-prompt`, it also wrote paste-ready prompts so an
assistant could close gaps *from the report*, not from vibes.

## Closing gaps without inventing evidence

The rule we held ourselves to is the same one we ask of users: **do not invent
controls**. Every artifact had to match how this repo actually works.

We added architecture and design docs that describe the real pipeline. An ADR
for the single-crate, observational-packs decision. A release runbook that
tracks the existing publish flow. Glossary and API notes. CODEOWNERS, PR and
issue templates, settings-as-code for branch protection and required checks.
`.editorconfig`, pre-commit hooks, gitleaks in CI, Conventional Commits config.
`Makefile` and `scripts/setup.sh` so onboarding is more than prose.
`AGENTS.md`, AI governance, and a small versioned `prompts/` library for the
remediation loop itself. The Rust pack gained a `cargo doc` needle so
documentation tooling scores when docs are actually gated.

Then we ran the binary again.

## Second look

| Metric | Before | After |
|--------|-------:|------:|
| Hygiene | 58.7 | **84.8** |
| Knowledge maturity | 43.8 | **100** |
| AI maturity | 0.0 | **95.0** |

Two hygiene findings stayed intentionally imperfect. Conventional Commits is
still **Partial** because history is what it is — a config file does not rewrite
the past. Branch protection is **Partial** because a checked-in
`.github/settings.yml` is a signal, not proof that the forge enforces it. APO
refuses to pretend otherwise. That refusal is a feature.

## What we learned

1. **Reports beat checklists.** The gap list was more useful than a generic
   “improve docs” aspiration; remediations pointed at concrete files.
2. **Prompts are accelerators, not oracles.** The LLM prompts helped apply
   changes quickly; the scores only moved when the filesystem changed.
3. **Dogfooding keeps the product honest.** Living with `Unknown` and `Partial`
   where local clones cannot see the platform stopped us from gaming the rubric.
4. **Badges close the loop.** Relative SVG embeds refresh in CI and show up next
   to the other project shields — enterprise-friendly, no external CDN.

If you want the same loop on your repository:

```bash
apo analyze . --llm-prompt --badge-output docs/badges/apo-hygiene.svg
apo evidence . --llm-prompt --badge-output docs/badges/apo-evidence.svg
# apply remediations from the *-prompt.md files
# re-run until the badges tell the story you intend
```

We eat this dog food on every `./scripts/ci.sh` run. The badges above the fold
are not marketing — they are the latest measured leftovers.
