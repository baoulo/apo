# Local developer shortcuts (mirrors scripts/ci.sh pieces)

.PHONY: help setup fmt clippy test doc ci apo-self

help:
	@echo "Targets: setup fmt clippy test doc ci apo-self"

setup:
	@command -v rustc >/dev/null || (echo "Install Rust 1.85+ from https://rustup.rs" && exit 1)
	rustup component add rustfmt clippy llvm-tools-preview
	@echo "Optional: pip install pre-commit && pre-commit install"
	@echo "Ready. Run: make ci"

fmt:
	cargo fmt --all -- --check

clippy:
	cargo clippy --all-targets --all-features -- -D warnings

test:
	cargo test --all-targets

doc:
	cargo doc --no-deps --document-private-items

ci:
	./scripts/ci.sh

apo-self:
	mkdir -p docs/badges out/apo-self
	cargo build -q
	./target/debug/apo analyze . --format both --output out/apo-self \
	  --badge-output docs/badges/apo-hygiene.svg
	./target/debug/apo evidence . --format both --output out/apo-self \
	  --badge-output docs/badges/apo-evidence.svg
