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
	cargo test --workspace

examples:
	@for d in crates/*/examples/*.rs; do \
		name=$$(basename $$d .rs); \
		crate=$$(basename $$(dirname $$(dirname $$d))); \
		echo "\n▶ Running $$crate::$$name ..."; \
		if [ "$$name" = "hot_reload" ]; then \
			timeout 5 cargo run -p $$crate --example $$name --all-features || true; \
		else \
			cargo run -p $$crate --example $$name --all-features; \
		fi; \
	done

verify: fmt lint test examples
	@echo "Verify successful"
