all: build

clean:
	cargo clean

run:
	cargo run --release

build:
	cargo build --release

install-uf2:
	curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
	cargo binstall elf2uf2-rs

uf2: build
	elf2uf2-rs target/thumbv6m-none-eabi/release/porky target/thumbv6m-none-eabi/release/porky_pico_w.uf2