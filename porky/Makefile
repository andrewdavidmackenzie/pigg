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

all: clippy build build-w build2 build-w2

clean:
	cargo clean

install-targets:
	rustup target add thumbv6m-none-eabi
	rustup target add thumbv8m.main-none-eabihf

setup: install-binstall install-targets

.PHONY: features
features:
	cargo build-all-features --target thumbv6m-none-eabi

clippy:
	cargo clippy --target thumbv6m-none-eabi --features "usb, wifi, pico1"
	cargo clippy --target thumbv6m-none-eabi --features "usb, pico1"
	cargo clippy --target thumbv8m.main-none-eabihf --features "usb, wifi, pico2"
	cargo clippy --target thumbv8m.main-none-eabihf --features "usb, pico2"

run-w:
	cargo run --release --target thumbv6m-none-eabi --features "usb, wifi, pico1"

run-w2:
	cargo run --release --target thumbv8m.main-none-eabihf --features "usb, wifi, pico2"

run:
	cargo run --release --target thumbv6m-none-eabi --features "usb, pico1"

run2:
	cargo run --release --target thumbv8m.main-none-eabihf --features "usb, pico2"

build-w:
	cargo build --release --target thumbv6m-none-eabi --features "usb, wifi, pico1"

build-w2:
	cargo build --release --target thumbv8m.main-none-eabihf --features "usb, wifi, pico2"

build:
	cargo build --release --target thumbv6m-none-eabi --features "usb, pico1"

build2:
	cargo build --release --target thumbv8m.main-none-eabihf --features "usb, pico2"

install-binstall:
	curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

install-picotool:
setup:
ifeq ($(OSFLAG),macos)
	brew install picotool
else
	mkdir $(HOME)/.bin || true
	curl -L --proto '=https' --tlsv1.2 -sSf https://github.com/raspberrypi/pico-sdk-tools/releases/download/v2.1.1-1/picotool-2.1.1-x86_64-lin.tar.gz | tar -xzv > $(HOME)/.bin/picotool
	chmod +x $(HOME)/.bin/picotool
endif

uf2s: install-targets install-picotool build-uf2s

build-uf2s: uf2-w uf2-w2 uf2 uf2-2

uf2-w: build-w # UF2 for Pi Pico (1) with Wi-FI
	picotool uf2 convert -t elf target/thumbv6m-none-eabi/release/porky target/thumbv6m-none-eabi/release/porky_pico_w.uf2

uf2-w2: build-w2 # UF2 for Pi Pico 2 with Wi-Fi
	picotool uf2 convert -t elf target/thumbv8m.main-none-eabihf/release/porky target/thumbv8m.main-none-eabihf/release/porky_pico_w2.uf2

uf2: build # UF2 for Pi Pico (1) without Wi-Fi
	picotool uf2 convert -t elf target/thumbv6m-none-eabi/release/porky target/thumbv6m-none-eabi/release/porky_pico.uf2

uf2-2: build2 # UF2 for Pi Pico 2 without WEi-Fi
	picotool uf2 convert -t elf target/thumbv8m.main-none-eabihf/release/porky target/thumbv8m.main-none-eabihf/release/porky_pico2.uf2