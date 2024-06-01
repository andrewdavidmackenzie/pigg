[![codecov](https://codecov.io/gh/andrewdavidmackenzie/pigg/graph/badge.svg?token=Lv5SstEMGO)](https://codecov.io/gh/andrewdavidmackenzie/pigg)

# pigg - Raspberry Pi GPIO GUI

A GUI for visualization/control of GPIO on Raspberry Pis.

## Chosen Tech

* [rust](https://www.rust-lang.org/)
* [Iced](https://github.com/iced-rs/iced) for GUI
* [rppal](https://github.com/golemparts/rppal) for Raspberry Pi GPIO control

## Basic / Initial Functionality

* visual representation of the GPIO connector/header with pins with numbers and names
* able to config each pin (input, output, pulled up/down, pwm etc.)
* able to set status of outputs
* able to see the status of inputs
* Able to load a config from file, and save the config that is currently set in the GUI

## Next batch of functionality

* Able to provide a time-view of inputs, so like an analyzer...

## Further out ideas

* trigger a script or WebAssembly plugin on an input event (edge, level, etc.)
* able to have UI on different device to where GPIO is and connect remotely
* hence able to connect the native UI to a remote device, where some "agent" is running
* have an "agent" able to run on a Pi Pico
* Have a web UI able to connect to an agent on a Pi or Pico

## Project Structure

### PIGGUI ("Piggy")

A binary that shows a GUI using Iced.
On Raspberry pi it will include a real GPIO hardware backend (via rppal).
On macOS and linux it will just have the UI, without GPIO.

### PIGLET ("Piglet)

A headless binary that is only built on RaspberryPi and that has no UI.

## Installing

Use

```
cargo install pigg
```

to build and install.

NOTE: `cargo` will build it for the machine where you are running it, so if you run it on Mac or Linux,
you will get a version of `piggui` with fake hardware backing it, not real Pi GPIO hardware.

To be able to interact with real Pi GPIO hardware you have two options:

* Run `cargo install --features "gui","pi"` on your Pi
* Follow the instructions before for Building from Source
    * Directly on your Raspberry Pi.
    * Use `make` to build on your machine for your Pi, but you will need `Docker`/`Podman` and `cross`
      installed

Soon, we will add support for `cargo binstall` to allow you to get a binary

## Building from Source

### Pre-requisites

We use `"cross"` to cross compile for Raspberry Pi from Linux or macOS.
Install docker or podman and `"cross"` for cross compiling rust on your host for the Raspberry Pi.

If you run `"make"` on a Raspberry Pi, it will not use `"cross"` and just compile natively.
So, to be clear `"cross"` is not a pre-requisite for Raspberry Pi native building.

### Building on host development machine

Run `"make"` on macOS or linux (or in fact RPi also) host to build these binaries:

* `target/debug/piggui` - GUI version without GPIO, to enable UI development on a host
* `target/aarch64-unknown-linux-gnu/release/piggui` - GUI version for Pi with GPIO, can be run natively from RPi command
  line
* `target/aarch64-unknown-linux-gnu/release/piglet` - Headless version for Pi with GPIO, can be run natively from RPi
  command line

Use `"make run"` to start `piggui` on the local machine - for GUI development.

### Building for Pi from macOS or Linux

If you use `make` that builds for local host AND pi (using cross).

#### Helper Env vars

There are a couple of env vars that can be setup to help you interact with your pi.

You can set these up in your env, so you always have them, or set them on the command line when invoking `make`

* `PI_TARGET` Which Pi to copy files to and ssh into
  `PI_TARGET := pizero2w0.local`

* `PI_USER` The username of your user on the pi, to be able to copy files and ssh into it
  `PI_USER := andrew`

#### Make targets

* Use `make` to run `clippy`, build for the Pi using `cross`, build for the local machine using `cargo` and to run tests
* Use `make pibuild` to build only for the Pi. This will build both `piggui` (with GUI and GPIO) and `piglet` binary
  with GPIO only
* Use `make copy` to copy the built binaries to your raspberry pi.
* Use `make ssh` to ssh into your Pi to be able to run the binaries.

### Building for Pi on a Pi!

You should be able to use `make` or `make run` directly, and it will build `piggui` with a GUI and
also build `piglet`

## Running

### Piggui

Use `make run`

One Mac/Linus this will build for the local machine, and start `piggui` with a fake hardware backend.

If you run that on a Raspberry Pi, it will detect that, and build for the Pi, with the real Pi GPIO hardware backend.

Piggui takes an optional filename argument, to attempt to load the code from. If there is an error
loading a config, the default config will be used.

To do this you can use the equivalent of what `make run` does, adding the filename:

* On a Pi: `cargo run --features "pi","gui" -- <filename>`
* On macOS and Linux: `cargo run --features "gui"  -- <filename>`