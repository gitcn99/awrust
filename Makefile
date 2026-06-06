.PHONY: help env fmt lint

help:
	@echo ""

env:
	pnpm add -DE prettier
	[ -f config/config.toml ] || cp config/config.toml.example config/config.toml

fmt:
	pnpm exec prettier . --write --ignore-unknown --log-level warn
	cargo fmt --all

lint: fmt
	pnpm exec prettier . --check --ignore-unknown
	cargo check --workspace --all-targets 2>&1
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace --all-targets

examples:
	@for d in crates/*/examples/*.rs; do \
		name=$$(basename $$d .rs); \
		crate=$$(basename $$(dirname $$(dirname $$d))); \
		echo "\n▶ Running $$crate::$$name ..."; \
		cargo run -p $$crate --example $$name; \
	done
