# Building Piggui and Piglet from source

Note that these instructions below apply equally, whether you have cloned the repo or have downloaded and exploded the
source tarball/zip of a release.

`cd` into the `pigg` project root folder (where you can see `Cargo.toml` for example) and then follow the instructions
below to build from source.

Note that once we complete some more work to create pre-built, installable, binaries for you, this work will all
go away and we will document how you install the binaries directly.

### macOS/linux/Windows or Pi (with a fake Hardware backend)

```
cargo install pigg
```

NOTE: `cargo` will build for the machine where you are running it, so you will get a version of `piggui`
with a fake hardware backend, not real Pi GPIO hardware, but you can play with the GUI.

### On Pi with real GPIO hardware backend

To be able to interact with real Pi GPIO hardware you have two options:

- Run `cargo install` on your Pi
- Follow the instructions before for Building from Source on your Pi
    - Use `make` to build on macOS/linux/Windows and cross-compile for your Pi, but you will need `Docker`/`Podman`
      and `cross` installed

Soon, we will add support for `cargo binstall` to allow you to get a binary for Pi directly.

## Building from Source

### Pre-requisites

We use `"cross"` to cross compile for Raspberry Pi from Linux or macOS or Windows.
None of the developers currently has a Windows machine, so we have been unable to test this on Windows.
Install docker or podman and `"cross"` for cross compiling rust on your host for the Raspberry Pi.

If you run `"make"` on a Raspberry Pi, it will not use `"cross"` and just compile natively.
So, to be clear `"cross"` is not a pre-requisite for Raspberry Pi native building.

### Building on host development machine

Run `"make"` on macOS, linux, Windows (or in fact RPi also) host to build these binaries:

- `target/debug/piggui` - GUI version without GPIO, to enable UI development on a host
- `target/aarch64-unknown-linux-gnu/release/piggui` - GUI version for Pi with GPIO, can be run natively from RPi command
  line
- `target/aarch64-unknown-linux-gnu/release/piglet` - Headless version for Pi with GPIO, can be run natively from RPi
  command line
- `target/armv7-unknown-linux-gnueabihf/release/piggui` - GUI version for Pi with GPIO, can be run natively from RPi
  command line
- `target/armv7-unknown-linux-gnueabihf/release/piglet` - Headless version for Pi with GPIO, can be run natively from
  RPi
  command line

Use `"make run-release"` to start a release build of `piggui` on the local machine - for GUI development, or use
`make run` to start a debug build of the same.

### Building for Pi from macOS, linux or Windows

If you use `make` that builds for local host AND pi (using cross).

If you get strange build errors from `cross`, check first that your Docker daemon is running.

#### Helper Env vars

There are a couple of env vars that can be setup to help you interact with your pi.

You can set these up in your env, so you always have them, or set them on the command line when invoking `make`

- `PI_TARGET` Which Pi to copy files to and ssh into
  `PI_TARGET := pizero2w0.local`

- `PI_USER` The username of your user on the pi, to be able to copy files and ssh into it
  `PI_USER := andrew`

#### Make targets

- Use `make` to run `clippy`, build for the Pi using `cross`, build for the local machine using `cargo` and to run tests
- Use `make cross-build` for aarch64-based Raspberry Pi models, and `make cross-build-armv7` for armv7-based Raspberry
  Pi models to build only for the Pi.
- Use `make copy` to copy the built binaries to your raspberry pi.
- Use `make ssh` to ssh into your Pi to be able to run the binaries.

### Building for Pi on a Pi!

You should be able to use `make` or `make run` directly, and it will build `piggui` with a GUI and
also build `piglet`