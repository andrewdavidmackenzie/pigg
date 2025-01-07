all: build-w build

clean:
	cargo clean

clippy:
	cargo clippy --bin porky_w --features "usb, wifi"
	cargo clippy --bin porky --features "usb"

run-w:
	cargo run --release --bin porky_w --features "usb, wifi"

run:
	cargo run --release --bin porky --features "usb"

build-w:
	cargo build --release --bin porky_w --features "usb, wifi"

build:
	cargo build --release --bin porky --features "usb"

install-uf2:
	curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
	cargo binstall elf2uf2-rs

uf2-w: build-w
	elf2uf2-rs target/thumbv6m-none-eabi/release/porky_w target/thumbv6m-none-eabi/release/porky_pico_w.uf2

uf2: build
	elf2uf2-rs target/thumbv6m-none-eabi/release/porky target/thumbv6m-none-eabi/release/porky_pico.uf2