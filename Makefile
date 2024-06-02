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
all: clippy build pibuild test

release: release-build pibuild

.PHONY: piclippy
piclippy:
ifneq ($(PI),)
	echo "Detected as running on Raspberry Pi"
else
	CROSS_CONTAINER_OPTS="--platform linux/amd64" cross clippy --release --features "pi" --tests --no-deps --target=aarch64-unknown-linux-gnu
endif

.PHONY: clippy
clippy:
ifneq ($(PI),)
	echo "Detected as running on Raspberry Pi"
	cargo clippy --features "pi" --tests --no-deps
else
	cargo clippy --tests --no-deps
endif

# I'm currently building using release profile for Pi, as not debugging natively on it. If we want to do that then
# we may need to add another make target
# For raspberry pi, build with the "rrpal" and "iced" features.
# That should build both the "piggui" and "piglet" binaries, with GUI and GPIO in "piggui" and GPIO in "piglet"
.PHONY: pibuild
pibuild:
ifneq ($(PI),)
	echo "Detected as running on Raspberry Pi"
else
	CROSS_CONTAINER_OPTS="--platform linux/amd64" cross build --release --features "pi" --target=aarch64-unknown-linux-gnu
endif

# Enable the "iced" feature so we only build the "piggui" binary on the current host (macos, linux or raspberry pi)
# To build both binaries on a Pi directly, we will need to modify this
.PHONY: build
build:
ifneq ($(PI),)
	echo "Detected as running on Raspberry Pi"
	cargo build --features "pi"
else
	cargo build
endif

.PHONY: run
run:
ifneq ($(PI),)
	echo "Detected as running on Raspberry Pi"
	cargo run --features "pi"
else
	cargo run
endif

# This will build all binaries on the current host, be it macos, linux or raspberry pi - with release profile
.PHONY: release-build
release-build:
ifneq ($(PI),)
	echo "Detected as running on Raspberry Pi"
	cargo build --release --features "pi"
else
	cargo build --release
endif

# This will only test GUI tests in piggui on the local host, whatever that is
# We'd need to think how to run tests on RPi, on piggui with GUI and GPIO functionality, and piglet with GPIO functionality
.PHONY: test
test:
ifneq ($(PI),)
	echo "Detected as running on Raspberry Pi"
	cargo test --features "pi"
else
	cargo test
endif

.PHONY: copy
copy: pibuild
	scp target/aarch64-unknown-linux-gnu/release/piggui $(PI_USER)@$(PI_TARGET):~/
	scp target/aarch64-unknown-linux-gnu/release/piglet $(PI_USER)@$(PI_TARGET):~/

.PHONY: ssh
ssh:
	ssh $(PI_USER)@$(PI_TARGET)