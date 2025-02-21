all: build build-w build2 build-w2

clean:
	cargo clean

# Install required rust targets
targets:
	rustup target add thumbv8m.main-none-eabihf

.PHONY: features
features:
	cargo build-all-features --target thumbv6m-none-eabi
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

install-uf2:
	curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
	cargo binstall elf2uf2-rs

uf2s: uf2-w uf2-w2 uf2 uf22

uf2-w: build-w
	elf2uf2-rs target/thumbv6m-none-eabi/release/porky target/thumbv6m-none-eabi/release/porky_pico_w.uf2

uf2-w2: build-w2
	elf2uf2-rs target/thumbv6m-none-eabi/release/porky target/thumbv6m-none-eabi/release/porky_pico_w2.uf2

uf2: build
	elf2uf2-rs target/thumbv6m-none-eabi/release/porky target/thumbv6m-none-eabi/release/porky_pico.uf2

uf22: build2
	elf2uf2-rs target/thumbv6m-none-eabi/release/porky target/thumbv6m-none-eabi/release/porky_pico2.uf2