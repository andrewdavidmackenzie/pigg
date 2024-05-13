# Variables used to talk to your pi. Set these up in your env, or set them on the command
# line when invoking make
# Which Pi to copy files to and ssh into
#PI_TARGET := pizero2w0.local
# The User name of your user on the pi, to be able to copy files and ssh into it
#PI_USER := andrew

.PHONY: all
all: build pibuild

.PHONY: piclippy
piclippy:
	CROSS_CONTAINER_OPTS="--platform linux/amd64" cross clippy --release --target=aarch64-unknown-linux-gnu

.PHONY: build
build:
	cargo build

.PHONY: pibuild
pibuild:
	CROSS_CONTAINER_OPTS="--platform linux/amd64" cross build --release --target=aarch64-unknown-linux-gnu

.PHONY: picopy
picopy: pibuild
	scp target/aarch64-unknown-linux-gnu/release/ringr $(PI_USER)@$(PI_TARGET):~/ringr

.PHONY: ssh
ssh:
	ssh $(PI_USER)@$(PI_TARGET)