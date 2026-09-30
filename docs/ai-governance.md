# AI governance for APO

## Principles

1. **Analyzers are source of truth.** AI assistants must not invent hygiene or
   evidence findings that `apo` did not observe.
2. **Observational only.** Do not “fix” scores by faking CI gates, CODEOWNERS,
   or docs that do not describe real practice.
3. **Secrets stay out.** Never commit tokens, `.env` credentials, or private keys.
4. **Prefer small diffs.** Match existing module style; avoid drive-by refactors.

## Allowed AI uses

- Closing gaps listed in `*-repository-hygiene-prompt.md` /
  `*-repository-evidence-prompt.md`
- Writing tests, docs, and remediations grounded in existing code
- Explaining report output

## Disallowed

- Fabricating coverage, branch protection, or secret-scanning evidence
- Running destructive git operations unless explicitly requested
- Expanding scope beyond the user’s request

## Agents

See [AGENTS.md](../AGENTS.md) for assistant operating notes on this repository.
