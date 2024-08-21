# Porky

Porky is an implementation of piglet remote GPIO client for the Raspberry Pi Pico W.

## Building

To just build you can use `make build`.

## Running using probe-rs

If you have a debug probe for the Pico, and it is connected and also the Pico is connected to
USB (for power) then you can use probe-rs to download and debug `picomon`.

[config.toml](./.cargo/config.toml) is where the runner for cargo is set up.

You can use `make run`. This uses `cargo run --release` which is better as the release version
of the binary is about half the size of the debug version of the binary, so download to the
Pico and re-flash is much faster.

This will build for the Pi Pico device and copy the built binary to it.

The Pi Pico will reboot and start running your binary, you should start seeing log messages on the terminal where
you are running `probe-rs` first from Embassy, then from Porky.

## Running using a UF2 file

### Creating a UF2 file

Use the `elf2usb2` command which you can install using cargo.
`Usage: elf2uf2-rs <INPUT> [OUTPUT]`

Input should be the ELF file in `target/thumbv6m-none-eabi/release/picomon`
Let's make the output `picomon.uf2`

### Copying UF2 to Pi Pico

Disconnect your RPi PicoW if it is connected.
Press and hold the BOOTSEL (the only button there is! :-) ) button on the board while you
connect via USB, then release BOOTSEL.

#### MacOs

It should be mounted as a USB storage device (you may need to mount it on Linux).
On Mac, a new volume should appear in `/Volumes`. This is usually called `RPI-RP2`.
You might get a (helpful) alert that a new USB device was plugged in.
If not you can check using `ls /Volumes`.

Then you should be able to copy a uf2 file to `/Volumes/RPI-RP` using `cp`
but I (and many others on the Internet) get an error from macOS 14.

However, this works:
`ditto --norsrc --noextattr --noacl picomon.uf2 /Volumes/RPI-RP2`

people report that rsync may also work.