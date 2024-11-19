all: build

clean:
	cargo clean

run:
	cargo run --release

build:
	cargo build --release

uf2: build
	elf2uf2-rs target/thumbv6m-none-eabi/release/porky target/thumbv6m-none-eabi/release/porky_pico_w.uf2