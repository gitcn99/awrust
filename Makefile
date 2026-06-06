.PHONY: help env fmt lint

help:
	@echo ""

env:
	pnpm add -DE prettier

fmt:
	pnpm exec prettier . --write --ignore-unknown --log-level warn
	cargo fmt --all

lint: fmt
	pnpm exec prettier . --check --ignore-unknown
	cargo check --workspace --all-targets 2>&1
	cargo clippy --workspace --all-targets -- -D warnings
