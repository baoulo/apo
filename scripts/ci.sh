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
run cargo test --all-targets
run cargo llvm-cov --all-features --lcov --output-path lcov.info \
  --ignore-filename-regex 'src/main\.rs' \
  --fail-under-lines 80
run cargo deny check

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
run ./target/debug/apo analyze . --format both --output out/apo-self \
  --badge-output docs/badges/apo-hygiene.svg
run ./target/debug/apo evidence . --format both --output out/apo-self \
  --badge-output docs/badges/apo-evidence.svg

echo ""
echo "All CI checks passed (including APO self-analysis → out/apo-self/, docs/badges/)."
