#!/usr/bin/env bash
# Reproducible local setup for APO contributors.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if ! command -v rustc >/dev/null 2>&1; then
  echo "Install Rust 1.85+ from https://rustup.rs" >&2
  exit 1
fi

rustup component add rustfmt clippy llvm-tools-preview

if command -v pre-commit >/dev/null 2>&1; then
  pre-commit install
else
  echo "Optional: install pre-commit (pipx/pip) then re-run scripts/setup.sh"
fi

echo "Setup complete. Try: ./scripts/ci.sh"
