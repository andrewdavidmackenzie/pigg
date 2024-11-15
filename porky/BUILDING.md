# Building Porky

NOTE: These instructions below apply equally, whether you have cloned the repo or have downloaded and exploded the
source tarball/zip of a release.

NOTE: None of the developers currently has a Windows machine, so we have been unable to test this on Windows.

```sh
cd pigg/porky
 ```

## Building

```sh
make
```

will run build a release DWARF binary for the Raspberry Pi Pico W here:

- `pigg/porky/target/thumbv6m-none-eabi/release/porky`

## Running Directly

NOTE: Running directly requires a debug probe attached to your device and host, and `probe-rs` installed.

If you have a debug probe for the Pico, and it is connected and also the Pico is connected to
USB then you can use probe-rs to download and debug `porky`.

[config.toml](./.cargo/config.toml) is where the runner for cargo is set up.

You can use `make run` (which uses `cargo run --release`) to run `porky` directly.

The release binary is about half the size of the debug binary, so downloading to the Pico W is much faster.

The Pi Pico will start running `porky` and you should start seeing log messages on the terminal where
you are running `make run`. The boot process should end with `porky` connected to the configured (or default)
Wi-Fi network, and the output of its IP and the port it is listening for TCP connections on.

## Creating a UF2 file

Use `make uf2` Makefile target.

This uses the `elf2usb2` command which you can install using cargo.

This ill produce the file `target/thumbv6m-none-eabi/release/porky.uf2`

You can check its type using: `file target/thumbv6m-none-eabi/release/porky.uf2`:

```
target/thumbv6m-none-eabi/release/porky.uf2: UF2 firmware image, family Raspberry Pi RP2040, address 0x10000000, 1608 total blocks
```

See [section in README.md](README.md#installing-and-running-porky-on-your-raspberry-pi-pico-w) for how to use your
newly created UF2 file.