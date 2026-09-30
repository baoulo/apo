# Hygiene remediation (v1)

Version: 1.0.0

Use this prompt after running `apo analyze --llm-prompt` on a repository.

1. Read the generated `*-repository-hygiene-prompt.md` (or paste its contents).
2. Close **Missing** / **Partial** controls with authentic artifacts only.
3. Prefer docs, CODEOWNERS, templates, CI gates, and settings-as-code that match
   how the project actually works.
4. Re-run `apo analyze` and confirm score movement without inventing evidence.
