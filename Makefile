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
all: clippy format-check build build-arm build-armv7 build-aarch64 build-porky build-web test

.PHONY: clean
clean:
	@cargo clean
	@rm -rf _site

.PHONY: macos-setup
macos-setup:
	@cmake -C piggui macos-setup
	@make -C pigglet macos-setup

.PHONY: setup
setup:
ifeq ($(OSFLAG),macos)
	@echo "Running macos specific setup"
	$(MAKE) macos-setup
endif
	@cargo install cargo-all-features
	@make -C piggui setup
	@make -C pigglet setup
	@make -C porky setup

.PHONY: clippy
clippy:
	cargo clippy --tests --no-deps

.PHONY: format-check
format-check:
	cargo fmt --all -- --check

.PHONY: build
build:
	cargo build

.PHONY: run
run:
	cargo run --bin piggui

.PHONY: run-release
run-release:
	cargo run --bin piggui --release

.PHONY: run-pigglet
run-pigglet:
	cargo run --bin pigglet

.PHONY: run-release-pigglet
run-release-pigglet:
	cargo run --bin pigglet --release

.PHONY: build-release
build-release:
	cargo build --release

.PHONY: build-porky
build-porky:
	make -C porky

.PHONY: test
test:
	cargo test -- --show-output

.PHONY: build_hw_tests
build_hw_tests:
	make -C hw_test build

.PHONY: hw_tests
hw_tests:
	make -C hw_test

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
	scp target/armv7-unknown-linux-gnueabihf/debug/piggui $(PI_USER)@$(PI_TARGET):~/
	scp target/armv7-unknown-linux-gnueabihf/debug/pigglet $(PI_USER)@$(PI_TARGET):~/

.PHONY: copy-release-armv7
copy-release-armv7:
	scp target/armv7-unknown-linux-gnueabihf/release/piggui $(PI_USER)@$(PI_TARGET):~/
	scp target/armv7-unknown-linux-gnueabihf/release/pigglet $(PI_USER)@$(PI_TARGET):~/

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
	scp target/aarch64-unknown-linux-gnu/debug/pigglet $(PI_USER)@$(PI_TARGET):~/

.PHONY: copy-release-aarch64
copy-release-aarch64:
	scp target/aarch64-unknown-linux-gnu/release/piggui $(PI_USER)@$(PI_TARGET):~/
	scp target/aarch64-unknown-linux-gnu/release/pigglet $(PI_USER)@$(PI_TARGET):~/

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
	scp target/arm-unknown-linux-gnueabihf/debug/pigglet $(PI_USER)@$(PI_TARGET):~/
	scp target/arm-unknown-linux-gnueabihf/debug/piggui $(PI_USER)@$(PI_TARGET):~/

.PHONY: copy-release-arm
copy-release-arm:
	scp target/arm-unknown-linux-gnueabihf/release/pigglet $(PI_USER)@$(PI_TARGET):~/
	scp target/arm-unknown-linux-gnueabihf/release/piggui $(PI_USER)@$(PI_TARGET):~/


.PHONY: ssh
ssh:
	ssh $(PI_USER)@$(PI_TARGET)

.PHONY: usb
usb:
	@echo "Echo your root password at the prompt to copy udev rules for piggui to the system folder for them"
	sudo cp 70.pigg.rules  /etc/udev/rules.d/

.PHONY: clean-start
clean-start:
	@find . -name "*.profraw"  | xargs rm -rf {}

.PHONY: coverage
coverage: clean-start
	@echo "coverage<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<"
	@RUSTFLAGS="-C instrument-coverage" LLVM_PROFILE_FILE="pigg-%p-%m.profraw" cargo build
	cargo test
	@echo "Gathering coverage information"
	@grcov . --binary-path target/debug/ -s . -t lcov --branch --ignore-not-existing --ignore "/*" -o coverage.info
	@lcov --remove coverage.info 'target/debug/build/**' 'target/release/build/**' '**/errors.rs' '**/build.rs' '*tests/*' --ignore-errors unused,unused --ignore-errors unsupported --ignore-errors inconsistent --ignore-errors empty,empty -o coverage.info  --erase-functions "(?=^.*fmt).+"
	@find . -name "*.profraw" | xargs rm -f
	@echo "Generating coverage report"
	@genhtml -o target/coverage --quiet coverage.info
	@echo "View coverage report using 'open target/coverage/index.html'"

build-web:
	@make -C piggui trunk-build

docs:
	bundle exec jekyll build --source site --destination _site