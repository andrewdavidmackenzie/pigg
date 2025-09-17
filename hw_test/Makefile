all: clippy test

.PHONY: clippy
clippy:
	cargo clippy --tests

.PHONY: build
build:
	cargo build --tests

.PHONY: test
test:
	cargo test -- --show-output