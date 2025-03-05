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

will run build a release DWARF binary for the Raspberry Pi Pico, Pico W, Pico 2 and Pico 2 W:

The same binary is overwritten for each, the last build being Pi Pico 2 W, which can be found at:

- `pigg/porky/target/thumbv6m-none-eabi/release/porky`

### How SSID information is used by `porky` on Pico W/Pico 2 W

The startup process of `porky` is:

- Try to load SSID information from Flash memory
- If none is present try and use the default SSID information compiled in as part of the build
- If no default information was built in, then `porky` has no SSID information and will not try to connect to a Wi-Fi
  network
- USB connection is always started and will be able to receive SSID information from `piggui`. This will be stored in
  the Flash memory and used in the above process at next start

### Configuring with override SSID information

As described, the default information can be overridden by supplying other SSID details that are stored in Flash memory.

See the [section in README.md](README.md#configuring-wi-fi-on-a-pi-pico-w-porky-device) on that for details
of how to do it using `piggui`

### Customizing the default SSID information

It is possible to customize the default SSID (Wi-Fi network) information that `porky` tries to use to connect to a
Wi-Fi network on startup, if you are building from source. Then all devices programmed with that build will
automatically try to connect to that Wi-Fi network at startup, without having to use `piggui` and USB.

The build process looks for the presence of a valid SSID specification file (called `ssid.toml`) in TOML format in
the `porky` project root directory (i.e. In `pigg/porky/` beside `Cargo.toml`).

The format of `ssid.toml` must be like this:

```
ssid_name = "SSID Name"
ssid_pass = "SSID Password"
security = "wpa2"
```

Valid values for the `security` field are: `open`, `wpa`, `wpa2` and `wpa3`

Create this file with your SSID information in it and then run the makefile `make build-w` or `make build-w2` targets,
and it will produce a `porky` binary with that SSID as the default SSID. Run `make uf2` to generate a UF2 file for
download.

## Running Directly

NOTE: Running directly requires a debug probe attached to your device and host, and `probe-rs` installed.

If you have a debug probe for the Pico, and it is connected and also the Pico is connected to
USB then you can use probe-rs to download and debug `porky`.

NOTE: `probe-rs` still doesn't have support for Pi Pico 2 (it's being worked on) so temporarily we have fallen back
to using `picotool` to download the firmware to flash and run it. It doesn't support SWG debug yet. So, you will have to
start the device with BOOTSEL, so it's waiting for a firmware fownload.

[config.toml](./.cargo/config.toml) is where the runner for cargo is set up.

You can use `make run` or `make run-w` or `make run2` or `make run-w2` (which uses `cargo run --release`) to run `porky`
directly.

The release binary is about half the size of the debug binary, so downloading to the Pico W is much faster.

The Pi Pico W will start running `porky` and you should start seeing log messages on the terminal where
you are running `make run`. The boot process should end with `porky` connected to the configured (or default)
Wi-Fi network, and the output of its IP and the port it is listening for TCP connections on.

## Creating a UF2 file

The `porky/Makefile` has 5 targets for uf2 files:

- `uf2-w`: Build UF2 file (`target/thumbv6m-none-eabi/release/porky_pico_w.uf2`) for Pico 1 W (Wi-Fi)
- `uf2-w2`: Build UF2 file (`target/thumbv6m-none-eabi/release/porky_pico_w2.uf2`)for Pico 2 W (Wi-Fi)
- `uf2`: Build UF2 file (`target/thumbv6m-none-eabi/release/porky_pico.uf2`) for Pico 1 (no Wi-Fi)
- `uf2-2`: Build UF2 file (`target/thumbv6m-none-eabi/release/porky_pico2.uf2`) for Pico 2 (no Wi-Fi)
- `uf2s`: Build all of the above

This uses [`elf2usb2-rs`](https://github.com/JoNil/elf2uf2-rs). This can be installed using the `porky/Makefile` target
`install-uf2` or
manually:

- `cargo binstall elf2uf2-rs` if you use `cargo binstall` to install a pre-compiled binary
- `cargo install elf2uf2-rs` to build it from source and install it

You can check the generated files using: `file`.

For example: `file target/thumbv6m-none-eabi/release/porky_pico_w.uf2`:

```
target/thumbv6m-none-eabi/release/porky_pico_w.uf2: UF2 firmware image, family Raspberry Pi RP2040, address 0x10000000, 1608 total blocks
```

See [section in README.md](README.md#installing-and-running-porky-on-your-raspberry-pi-pico-w) for how to use your newly
created UF2 file.