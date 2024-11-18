# Building Piggui and Piglet from source

NOTE: For details on building `porky` the embedded binary for Pi Pico W devices,
see [porky/BUILDING.md](porky/BUILDING.md)

NOTE: These instructions below apply equally, whether you have cloned the repo or have downloaded and exploded the
source tarball/zip of a release.

NOTE: None of the developers currently has a Windows machine, so we have been unable to test this on Windows.

```sh
cd
 ``` 

into the `pigg` project root folder (where you can see `Cargo.toml` for example) and then follow the instructions
below to build from source.

## Building on host machine

```sh
make
```

This will run clippy, build, and run tests, generating these binaries:

- `target/debug/piggui` - GUI with a GPIO backend and ability to connect to remote `piglet` and `porky` devices
- `target/debug/piglet` - CLI with a GPIO backend
- `porky/target/thumbv6m-none-eabi/release/porky` - an executable for use on Raspberry Pi Pico W

By default, `cargo` (used by `make`) compiles for the machine it is running on. For cross-compiling for a Raspberry
Pi on a different host see later section.

## GPIO Backends

If built for macOS, linux, Windows (building on the same OS) the GPIO backend of `piggui` and `piglet` will be a
simulated backend to show the features and ease development.

If built for a Raspberry Pi (building on the Pi or cross compiling for it) the GPIO backend will interact with the
real GPIO hardware present.

## Running directly

- Use `make run` to start a debug build of `piggui`
- Use `"make run-release"` to start a release build of `piggui`
- Use `make run-piglet` to start a debug build of `piglet`
- Use `"make run-release-piglet"` to start a release build of `piglet`

## Building for Pi on a Pi

NOTE: For this option you will need a working rust toolchain installed on your Raspberry Pi. Also, these builds make
take a very long time. Large intermediate files are also generated and can fill-up the storage of your Pi.

On your Raspberry Pi, use `make` to build and `make run` (etc) as above.

## Building for Pi on another host

You can cross compile for a Raspberry Pi device on a macOS/Windows/Linux host and take advantage of the increased
computing power to reduce compile times.

Relevant Makefile targets for `armv7` and `aarch64` architectures are:

- [`armv7` | `aarch64`] - will run clippy and build
- [`clippy-armv7` | `clippy-aarch64`] - will run clippy
- [`build-armv7` | `build-aarch64`] - will run build
- [`release-build-armv7` | `release-build-aarch64`] - will run a release build
- [`test-armv7` | `test-aarch64`] - will run tests

These targets will build binary files for `piggui` and `piglet` in the `target/{architecture}/{release or debug}/`
directory.

Being built _for_ the Raspberry Pi, these binaries will have real GPIO backends that interact with the real Raspberry
Pi hardware.

### Helper Env vars

There are a couple of env vars that can be setup to help you interact with your Raspberry Pi.

You can set these up in your env, so you always have them, or set them on the command line when invoking `make`

- `PI_TARGET` Which Pi to copy files to and ssh into
- `PI_USER` The username of your user on the pi, to be able to copy files and ssh into it

Example: `PI_TARGET=pizero2w0.local PI_USER=andrew make copy-armv7`

### Copying and Running binaries on a Raspberry Pi from a host

- [`copy-armv7` | `copy-aarch64`] to copy the built binaries to your Raspberry Pi
- `make ssh` to ssh into your Pi to be able to run the binaries.

## Building `porky`

For more details on building and running `porky` on a Raspberry Pi Pico W device,
see [porky/BUILDING.md](porky/BUILDING.md).
