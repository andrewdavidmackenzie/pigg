# default target: "make" ran on macos or linux or windows and build for the host architecture
# target/aarch64-unknown-linux-gnu/release/piglet - for Pi with GPIO, can be run natively from RPi command line
# target/armv7-unknown-linux-gnueabihf/release/piglet - for armv7 based architecture with GPIO, can be run natively

# MacOS pre-requisites for cross-compiling to armv7
# brew install arm-linux-gnueabihf-binutils
# rustup target add armv7-unknown-linux-gnueabihf
# rustup target add armv7-unknown-linux-musleabihf
#
# See: https://github.com/messense/homebrew-macos-cross-toolchains
#
# brew tap messense/macos-cross-toolchains
# brew install aarch64-unknown-linux-gnu
# brew install arm-unknown-linux-gnueabihf
# brew install arm-unknown-linux-musleabihf
#
# Pi Zero is an ARMv6 CPU and so need a different target from v7 and v8 or aarch64 targets - target 'arm'

# Detect if on a Raspberry Pi
$(eval PI = $(shell cat /proc/cpuinfo 2>&1 | grep "Raspberry Pi"))

.PHONY: all
all: clippy build build-arm build-armv7 build-aarch64 test

.PHONY: clean
clean:
	@cargo clean
	@trunk clean

.PHONY: macos-setup
macos-setup: setup
	brew tap messense/macos-cross-toolchains
	brew install aarch64-unknown-linux-gnu
	brew install arm-unknown-linux-gnueabihf
	brew install arm-unknown-linux-musleabihf

.PHONY: setup
setup:
	rustup target add armv7-unknown-linux-gnueabihf
	rustup target add arm-unknown-linux-gnueabihf # For Pi Zero
	rustup target add armv7-unknown-linux-musleabihf
	rustup target add aarch64-unknown-linux-gnu

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

.PHONY: test
test:
	cargo test -- --show-output

.PHONY: features
features:
	cargo build-all-features

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
	scp ../target/armv7-unknown-linux-gnueabihf/debug/piglet $(PI_USER)@$(PI_TARGET):~/

.PHONY: copy-release-armv7
copy-release-armv7:
	scp ../target/armv7-unknown-linux-gnueabihf/release/piglet $(PI_USER)@$(PI_TARGET):~/


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
	scp ../target/aarch64-unknown-linux-gnu/debug/piglet $(PI_USER)@$(PI_TARGET):~/

.PHONY: copy-release-aarch64
copy-release-aarch64:
	scp ../target/aarch64-unknown-linux-gnu/release/piglet $(PI_USER)@$(PI_TARGET):~/


#### arm targets - useful for Raspberry Pi Zero (not Zero 2)
# Don't build build-arm-musl by default
.PHONY: arm
arm: clippy-arm build-arm

.PHONY: clippy-arm
clippy-arm:
	cargo clippy --tests --no-deps --target=arm-unknown-linux-gnueabihf

.PHONY: build-arm
build-arm:
	cargo build --target=arm-unknown-linux-gnueabihf

.PHONY: build-arm-musl
build-arm-musl:
	cargo build --target=arm-unknown-linux-musleabihf

.PHONY: release-build-arm
release-build-arm:
	cargo build --release --target=arm-unknown-linux-gnueabihf

# NOTE: The tests will be built for arm architecture, so tests can only be run on that architecture
.PHONY: test-arm
test-arm:
	cargo test --target=arm-unknown-linux-gnueabihf

.PHONY: copy-arm
copy-arm:
	scp ../target/arm-unknown-linux-gnueabihf/debug/piglet $(PI_USER)@$(PI_TARGET):~/

.PHONY: copy-release-arm
copy-release-arm:
	scp ../target/arm-unknown-linux-gnueabihf/release/piglet $(PI_USER)@$(PI_TARGET):~/


.PHONY: ssh
ssh:
	ssh $(PI_USER)@$(PI_TARGET)