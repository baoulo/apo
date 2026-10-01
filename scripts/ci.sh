#!/usr/bin/env bash
# Run local checks that mirror .github/workflows/ci.yml (fmt, clippy, test, coverage, audit/deny)
# plus APO dogfooding on this repository.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

run() {
  echo ""
  echo "==> $*"
  "$@"
}

run cargo fmt --all -- --check
run cargo clippy --all-targets --all-features -- -D warnings
run cargo doc --no-deps --document-private-items
run cargo test --all-targets

echo ""
echo "==> cargo llvm-cov (CI gate: ≥80% lines; fails locally the same way)"
if ! cargo llvm-cov --all-features --lcov --output-path lcov.info \
  --ignore-filename-regex 'src/main\.rs' \
  --fail-under-lines 80; then
  echo "error: line coverage is below 80% — fix tests before pushing (CI Coverage job will fail)." >&2
  exit 1
fi

run cargo deny check

if command -v gitleaks >/dev/null 2>&1; then
  run gitleaks detect --source . --config .gitleaks.toml --no-banner
else
  echo ""
  echo "==> gitleaks (skipped — install from https://github.com/gitleaks/gitleaks)"
fi

# Optional but present in CI audit job:
if command -v cargo-audit >/dev/null 2>&1 || cargo audit -h >/dev/null 2>&1; then
  run cargo audit
else
  echo ""
  echo "==> cargo audit (skipped — install with: cargo install cargo-audit)"
fi

# Dogfood: analyze this repo with the just-built apo binary.
mkdir -p docs/badges out/apo-self
run cargo build -q
run ./target/debug/apo report . --format both --output out/apo-self \
  --badge-output docs/badges --sarif
# Keep README-canonical badge filenames in sync
cp -f docs/badges/*-repository-hygiene-badge.svg docs/badges/apo-hygiene.svg
cp -f docs/badges/*-repository-evidence-badge.svg docs/badges/apo-evidence.svg
rm -f docs/badges/*-repository-hygiene-badge.svg docs/badges/*-repository-evidence-badge.svg

echo ""
echo "All CI checks passed (including APO self-analysis → out/apo-self/, docs/badges/)."
