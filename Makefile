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

# Detect if on a Raspberry Pi
$(eval PI = $(shell cat /proc/cpuinfo 2>&1 | grep "Raspberry Pi"))

.PHONY: all
all: clippy build test

release: release-build

.PHONY: clippy
clippy:
ifneq ($(PI),)
	@echo "Detected as running on Raspberry Pi"
	# Native compile on pi, targeting real hardware
	cargo clippy --features "gui","pi" --bin piggui --tests --no-deps
	cargo clippy --features "pi" --bin piglet --tests --no-deps
else
	# Compile for host, targeting fake hardware
	cargo clippy --bin piggui --features "gui","fake" --tests --no-deps
	cargo clippy --bin piglet --features "fake" --tests --no-deps
	# Cross compile for pi, targeting real hardware
	CROSS_CONTAINER_OPTS="--platform linux/amd64" cross clippy --bin piggui --release --features "gui","pi" --tests --no-deps --target=aarch64-unknown-linux-gnu
	CROSS_CONTAINER_OPTS="--platform linux/amd64" cross clippy --bin piglet --release --features "pi" --tests --no-deps --target=aarch64-unknown-linux-gnu
endif

# Enable the "iced" feature so we only build the "piggui" binary on the current host (macos, linux or raspberry pi)
# To build both binaries on a Pi directly, we will need to modify this
.PHONY: build
build:
ifneq ($(PI),)
	@echo "Detected as running on Raspberry Pi"
	# Native compile on pi, targeting real hardware
	cargo build --bin piggui --features "gui","pi"
	cargo build --bin piglet --features "pi"
else
	# Compile for host, targeting fake hardware
	cargo build --bin piggui --features "gui","fake"
	cargo build --bin piglet --features "fake"
	# Cross compile for pi, targeting real hardware
	CROSS_CONTAINER_OPTS="--platform linux/amd64" cross build --bin piggui --release --features "gui","pi" --target=aarch64-unknown-linux-gnu
	CROSS_CONTAINER_OPTS="--platform linux/amd64" cross build --bin piglet --release --features "pi" --target=aarch64-unknown-linux-gnu
endif

.PHONY: run
run:
ifneq ($(PI),)
	@echo "Detected as running on Raspberry Pi"
	# Native compile on pi, targeting real hardware
	cargo run --bin piggui --features "gui","pi"
else
	# Compile for host, targeting fake hardware
	cargo run --bin piggui --features "gui","fake"
endif

.PHONY: run-piglet
run-piglet:
ifneq ($(PI),)
	@echo "Detected as running on Raspberry Pi"
	# Native compile on pi, targeting real hardware
	cargo run --bin piglet --features "pi"
else
	# Compile for host, targeting fake hardware
	cargo run --bin piglet --features "fake"
endif

# This will build all binaries on the current host, be it macos, linux or raspberry pi - with release profile
.PHONY: release-build
release-build:
ifneq ($(PI),)
	@echo "Detected as running on Raspberry Pi"
	# Native compile on pi, targeting real hardware
	cargo build --bin piggui --release --features "gui","pi"
	cargo build --bin piglet --release --features "pi"
else
	# Compile for host, targeting fake hardware
	cargo build --bin piggui --release --features "gui","fake"
	cargo build --bin piglet --release --features "fake"
	# Cross compile for pi, targeting real hardware
	CROSS_CONTAINER_OPTS="--platform linux/amd64" cross build --bin piggui --release --features "gui","pi" --target=aarch64-unknown-linux-gnu
	CROSS_CONTAINER_OPTS="--platform linux/amd64" cross build --bin piglet --release --features "pi" --target=aarch64-unknown-linux-gnu
endif

# This will only test GUI tests in piggui on the local host, whatever that is
# We'd need to think how to run tests on RPi, on piggui with GUI and GPIO functionality, and piglet with GPIO functionality
.PHONY: test
test:
ifneq ($(PI),)
	@echo "Detected as running on Raspberry Pi"
	# Native compile on pi, targeting real hardware
	cargo test --bin piggui --features "gui","pi"
	cargo test --bin piglet --features "pi"
else
	# Compile for host, targeting fake hardware
	cargo test --bin piggui --features "gui","fake"
	cargo test --bin piglet --features "fake"
	# cross can run tests on pi architecture, so we cannot run tests that depend on "pi" hardware
	# and no point in re-running tests on pi architecture on "fake" hardware that we have already ran above on host
endif

.PHONY: copy
copy: build
	scp target/aarch64-unknown-linux-gnu/release/piggui $(PI_USER)@$(PI_TARGET):~/
	scp target/aarch64-unknown-linux-gnu/release/piglet $(PI_USER)@$(PI_TARGET):~/

.PHONY: ssh
ssh:
	ssh $(PI_USER)@$(PI_TARGET)
