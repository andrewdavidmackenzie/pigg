all: build

clean:
	cargo clean

run: build
	cargo run --release

build:
	cargo build --release

config: build
	@cd .. && cargo run -p configr -- MOVISTAR_8A9E E68N8MA422GRQJQTPqjN target/thumbv6m-none-eabi/release/picomon