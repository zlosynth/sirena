.PHONY: all
all: format lint check test

.PHONY: format
format:
	cargo fmt --all

.PHONY: lint
lint:
	cargo clippy --all --features defmt

.PHONY: check
check:
	cargo check --all --features defmt
	cargo check --benches --all

.PHONY: test
test:
	cargo test --all --features defmt
ifdef SLOW
	cargo test --all --features defmt -- --ignored
endif
