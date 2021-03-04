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
	cargo check --examples --all
	cargo check --benches --all

.PHONY: test
test:
ifdef SLOW
	cargo test --all
	cargo test --all -- --ignored
else
	cargo test --all
endif
