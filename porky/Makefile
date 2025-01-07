all: build-pico-w build-pico

clean:
	cargo clean

clippy:
	cargo clippy --features "pico_w"
	cargo clippy --features "pico"

run-pico-w:
	cargo run --release --bin porky_w --features "pico_w"

run-pico:
	cargo run --release --bin porky --features "pico"

build-pico-w:
	cargo build --release --bin porky_w --features "pico_w"

build-pico:
	cargo build --release --bin porky --features "pico"

run-w:
	cargo run --release --bin porky_w --features "pico_w"

run:
	cargo run --release --bin porky --features "pico"

install-uf2:
	curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
	cargo binstall elf2uf2-rs

uf2-pico-w: build-pico-w
	elf2uf2-rs target/thumbv6m-none-eabi/release/porky_w target/thumbv6m-none-eabi/release/porky_pico_w.uf2

uf2-pico: build-pico
	elf2uf2-rs target/thumbv6m-none-eabi/release/porky target/thumbv6m-none-eabi/release/porky_pico.uf2

uf2: uf2-pico-w