# Variables used to talk to your pi. Set these up in your env, or set them on the command
# line when invoking make
# Which Pi to copy files to and ssh into
#PI_TARGET := pizero2w0.local
# The User name of your user on the pi, to be able to copy files and ssh into it
#PI_USER := andrew

# default target: "make" ran on macos or linux host should build these binaries:
# target/debug/piggui - GUI version without GPIO, to enable UI development on a host
# target/aarch64-unknown-linux-gnu/release/piggui - GUI version for Pi with GPIO, can be run natively from RPi command line
# target/armv7-unknown-linux-gnueabihf/release/piggui - GUI version for armv7 based architecture with GPIO, can be run natively

# MacOS pre-requisites for cross-compiling to armv7
# brew install arm-linux-gnueabihf-binutils
# rustup target add armv7-unknown-linux-musleabihf
#
# See: https://github.com/messense/homebrew-macos-cross-toolchains
#
# brew tap messense/macos-cross-toolchains
# brew install aarch64-unknown-linux-gnu
# brew install arm-unknown-linux-gnueabihf
# brew install arm-unknown-linux-musleabihf

# Detect if on a Raspberry Pi
$(eval PI = $(shell cat /proc/cpuinfo 2>&1 | grep "Raspberry Pi"))

OSFLAG 				:=
ifeq ($(OS),Windows_NT)
	OSFLAG:=windows
else
	UNAME_S := $(shell uname -s)
	ifeq ($(UNAME_S),Linux)
		OSFLAG:=linux
	endif
	ifeq ($(UNAME_S),Darwin)
		OSFLAG:=macos
	endif
endif

.PHONY: all
all: clippy build build-arm trunk-build test

.PHONY: clean
clean:
	@cargo clean

# install non-clang llvm and then we need to make sure it's used instead of default macos clang!
.PHONY: macos-setup
macos-setup:
	@echo "Installing non-clang llvm"
	@brew install llvm
	@echo 'export PATH="/opt/homebrew/opt/llvm/bin:$$PATH"' >> ~/.zshrc
	@export PATH="/opt/homebrew/opt/llvm/bin:$$PATH"

.PHONY: binstall
binstall:
	@echo "installing cargo binstall"
	@cargo install cargo-binstall

.PHONY: setup
setup: binstall
ifeq ($(OSFLAG),macos)
	@echo "Running macos specific setup"
	$(MAKE) macos-setup
endif
	rustup target add wasm32-unknown-unknown
	rustup target add aarch64-unknown-linux-gnu
	cargo binstall trunk

.PHONY: clippy
clippy:
	cargo clippy --tests --no-deps

.PHONY: build
build:
	cargo build

.PHONY: build-release
build-release:
	cargo build --release

.PHONY: run
run:
	cargo run

.PHONY: run-release
run-release:
	cargo run --release

# This will only test GUI tests in piggui on the local host, whatever that is
# We'd need to think how to run tests on RºPi, on piggui with GUI and GPIO functionality,
# and piglet with GPIO functionality
.PHONY: test
test:
	cargo test

.PHONY: features
features:
	cargo build-all-features

#### arm builds
.PHONY: build-arm
# Don't build build-armv7-musl locally on macOS
build-arm: build-armv7 build-aarch64

#### armv7 targets
# Don't build build-armv7-musl locally on macOS
.PHONY: armv7
armv7: clippy-armv7 build-armv7

.PHONY: clippy-armv7
clippy-armv7:
	cargo clippy --tests --no-deps --target=armv7-unknown-linux-gnueabihf

.PHONY: build-armv7
build-armv7:
	cargo build --target=armv7-unknown-linux-gnueabihf

.PHONY: build-armv7-musl
build-armv7-musl:
	cargo build --target=armv7-unknown-linux-musleabihf

.PHONY: release-build-armv7
release-build-armv7:
	cargo build --release --target=armv7-unknown-linux-gnueabihf

# NOTE: The tests will be built for armv7 architecture, so tests can only be run on that architecture
.PHONY: test-armv7
test-armv7:
	cargo test --target=armv7-unknown-linux-gnueabihf

.PHONY: copy-armv7
copy-armv7:
	scp target/armv7-unknown-linux-gnueabihf/debug/piggui $(PI_USER)@$(PI_TARGET):~/

.PHONY: copy-release-armv7
copy-release-armv7:
	scp target/armv7-unknown-linux-gnueabihf/release/piggui $(PI_USER)@$(PI_TARGET):~/


#### aarch64 targets
.PHONY: aarch64
aarch64: clippy-aarch64 build-aarch64

.PHONY: clippy-aarch64
clippy-aarch64:
	cargo clippy --tests --no-deps --target=aarch64-unknown-linux-gnu

.PHONY: build-aarch64
build-aarch64:
	cargo build --target=aarch64-unknown-linux-gnu

.PHONY: release-build-aarch64
release-build-aarch64:
	cargo build --release --target=aarch64-unknown-linux-gnu

# NOTE: The tests will be built for aarch64 architecture, so tests can only be run on that architecture
.PHONY: test-aarch64
test-aarch64:
	cargo test --target=aarch64-unknown-linux-gnu

.PHONY: copy-aarch64
copy-aarch64:
	scp target/aarch64-unknown-linux-gnu/debug/piggui $(PI_USER)@$(PI_TARGET):~/

.PHONY: copy-release-aarch64
copy-release-aarch64:
	scp target/aarch64-unknown-linux-gnu/release/piggui $(PI_USER)@$(PI_TARGET):~/

.PHONY: ssh
ssh:
	ssh $(PI_USER)@$(PI_TARGET)

.PHONY: trunk-build
trunk-build:
	RUSTFLAGS='--cfg getrandom_backend="wasm_js"' trunk build

.PHONY: web-run
web-run: trunk-build
	RUSTFLAGS='--cfg getrandom_backend="wasm_js"' trunk serve
