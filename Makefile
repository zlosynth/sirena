.PHONY: all
all: format lint check test

.PHONY: format
format:
	cargo fmt --all

.PHONY: lint
lint:
	cargo clippy --all

.PHONY: check
check:
	cargo check --all
	cargo check --benches --all
	cargo check --no-default-features

.PHONY: test
test:
	cargo test --all
ifdef SLOW
	cargo test --all -- --ignored
endif
