# Variables used to talk to your pi. Set these up in your env, or set them on the command
# line when invoking make
# Which Pi to copy files to and ssh into
#PI_TARGET := pizero2w0.local
# The User name of your user on the pi, to be able to copy files and ssh into it
#PI_USER := andrew

# default target: "make" ran on macos or linux host should build these binaries:
# target/debug/piggui - GUI version without GPIO, to enable UI development on a host
# target/aarch64-unknown-linux-gnu/release/piggui - GUI version for Pi with GPIO, can be run natively from RPi command line
# target/aarch64-unknown-linux-gnu/release/piglet - Headless version for Pi with GPIO, can be run natively from RPi command line
# target/armv7-unknown-linux-gnueabihf/release/piggui - GUI version for armv7 based architecture with GPIO, can be run natively
# target/armv7-unknown-linux-gnueabihf/release/piglet - Headless version for armv7 based architecture with GPIO, can be run natively

# Detect if on a Raspberry Pi
$(eval PI = $(shell cat /proc/cpuinfo 2>&1 | grep "Raspberry Pi"))

.PHONY: all
all: clippy build test

.PHONY: cross
cross: cross-clippy cross-build cross-test cross-release-build cross-build-armv7 cross-release-build-armv7

release: release-build

.PHONY: clippy
clippy:
	cargo clippy  --tests --no-deps

# Enable the "iced" feature so we only build the "piggui" binary on the current host (macos, linux or raspberry pi)
# To build both binaries on a Pi directly, we will need to modify this
.PHONY: build
build:
	cargo build

.PHONY: run
run:
	cargo run --bin piggui

.PHONY: run-release
run-release:
	cargo run --bin piggui --release

.PHONY: run-piglet
run-piglet:
	RUST_LOG=piglet=info cargo run --bin piglet

.PHONY: run-release-piglet
run-release-piglet:
	RUST_LOG=piglet=info cargo run --bin piglet --release

# This will build all binaries on the current host, be it macos, linux or raspberry pi - with release profile
.PHONY: build-release
build-release:
	cargo build --release

# This will only test GUI tests in piggui on the local host, whatever that is
# We'd need to think how to run tests on RPi, on piggui with GUI and GPIO functionality, and piglet with GPIO functionality
.PHONY: test
test:
	cargo test

.PHONY: cross-clippy
cross-clippy:
	CROSS_CONTAINER_OPTS="--platform linux/amd64" cross clippy --tests --no-deps --target=aarch64-unknown-linux-gnu

.PHONY: cross-build
cross-build:
	CROSS_CONTAINER_OPTS="--platform linux/amd64" cross build --target=aarch64-unknown-linux-gnu

.PHONY: cross-build-armv7
cross-build-armv7:
	CROSS_CONTAINER_OPTS="--platform linux/amd64" cross build --target=armv7-unknown-linux-gnueabihf

.PHONY: cross-release-build
cross-release-build:
	CROSS_CONTAINER_OPTS="--platform linux/amd64" cross build --release --target=aarch64-unknown-linux-gnu

.PHONY: cross-release-build-armv7
cross-release-build-armv7:
	CROSS_CONTAINER_OPTS="--platform linux/amd64" cross build --release --target=armv7-unknown-linux-gnueabihf

.PHONY: cross-test
cross-test:
	CROSS_CONTAINER_OPTS="--platform linux/amd64" cross test --target=aarch64-unknown-linux-gnu

.PHONY: copy
copy: cross-build
	scp target/aarch64-unknown-linux-gnu/debug/piggui $(PI_USER)@$(PI_TARGET):~/
	scp target/aarch64-unknown-linux-gnu/debug/piglet $(PI_USER)@$(PI_TARGET):~/

.PHONY: copy-release
copy-release: cross-release-build
	scp target/aarch64-unknown-linux-gnu/release/piggui $(PI_USER)@$(PI_TARGET):~/
	scp target/aarch64-unknown-linux-gnu/release/piglet $(PI_USER)@$(PI_TARGET):~/

.PHONY: copy-armv7
copy-armv7: cross-build
	scp target/armv7-unknown-linux-gnueabihf/debug/piggui $(PI_USER)@$(PI_TARGET):~/
	scp target/armv7-unknown-linux-gnueabihf/debug/piglet $(PI_USER)@$(PI_TARGET):~/

.PHONY: copy-release-armv7
copy-release-armv7: cross-build-armv7
	scp target/armv7-unknown-linux-gnueabihf/release/piggui $(PI_USER)@$(PI_TARGET):~/
	scp target/armv7-unknown-linux-gnueabihf/release/piglet $(PI_USER)@$(PI_TARGET):~/

.PHONY: ssh
ssh:
	ssh $(PI_USER)@$(PI_TARGET)

.PHONY: web-build
web-build:
	rustup target add wasm32-unknown-unknown
	cargo build --bin piggui --no-default-features --target wasm32-unknown-unknown

.PHONY: web-run
web-run: web-build
	trunk serve