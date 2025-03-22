all: clippy test

.PHONY: clippy
clippy:
	cargo clippy --tests

.PHONY: test
test:
	cargo test --no-fail-fast -- --show-output