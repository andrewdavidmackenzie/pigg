![GH Action](https://github.com/andrewdavidmackenzie/pigg/actions/workflows/build_and_test.yml/badge.svg)
[![codecov](https://codecov.io/gh/andrewdavidmackenzie/pigg/graph/badge.svg?token=Lv5SstEMGO)](https://codecov.io/gh/andrewdavidmackenzie/pigg)

# pigg - Raspberry Pi GPIO GUI

An app for Raspberry Pi GPIO Output control and Input visualization, built in rust using the
[Iced](https://github.com/iced-rs) GUI toolkit and [rppal](https://github.com/golemparts/rppal/) GPIO crate.

The GUI binary (Pi Gpio GUI - PIGGUI) is affectionately known as "piggy".

<table cellspacing="0" cellpadding="0" border="0">
  <tr>
    <td valign="top">
      <img alt="BCM Pin Layout Screenshot" src="assets/images/bcm_pin_layout.png" width="400" align="top" />
    </td>
    <td valign="top">
      <img alt="Board Pin Layout Screenshot" src="assets/images/board_pin_layout_1.png" width="400" align="top" />
      <br><br>
      <img alt="Board Pin Layout Screenshot" src="assets/images/board_pin_layout_2.png" width="400" align="top" />
    </td>
  </tr>
</table>

Currently, when run on a Pi, you can configure the Pi's GPIO hardware Inputs or Outputs, controlling the
level of the Outputs and view the level of the Inputs.

It runs on macOS/Linux/Windows. When we add networking support, this will allow you to control the Pi GPIO
hardware remotely.

<table cellspacing="0" cellpadding="0" border="0">
  <tr>
    <td valign="top">
      <img alt="Input" src="assets/gifs/input.gif" width="400" align="top" />
    </td>
    <td valign="top">
      <img alt="Output" src="assets/gifs/output.gif" width="400" align="top" />
    </td>
  </tr>
</table>

## Current Features

- Visual representation of the GPIO pins in two layouts, a "Board Pin Layout" that mimics the
  physical layout of the Pi's GPIO connector/header, or a "BCM Pin Layout" with only the programmable
  GPIO pins, ordered by BCM pin number
- Each pin has its board pin number, name and function.
- Drop down selector to config each pin (Currently as an Input with or without pull-up/pull-down, or
  as an Output)
- Inputs have a visualization like an LED to show its current level (Black is unknown, Red is off, Green is on),
  plus a waveform view that shows you the recent history of the level detected on the input.
- Outputs have a toggle switch that can be used to change the stable value of the output, plus a "clicker" for quick
  inversions of the stable level, plus a waveform view showing the recent history of the level set on the Output.
- GPIO configurations can be loaded at startup with a command line filename option, or loaded via
  file-picker from the UI or saved to file via file picker.

You can see more gifs and videos of features [here](assets/features.md)

## Input from Raspberry Pi users wanted

We would like input from Raspberry Pi users to help us decide the order of things to work on in the future,
and input on how integrate new functionalities (e.g. I2C buses, SPI, UART, etc.).

Please let us know what you think, and suggestions, via GitHub discussions or GH issues, or in threads where we
communicate its existence (discord, reddit, etc.).

## Short-term Roadmap

We have identified a number of areas we would like to work on after this initial release,
but would really appreciate your input on what could be most useful or just the coolest,
many have GH issues.

- The next big one we would like to work on is having the GUI ("piggui") connect to
  a Pi over the network (running either "piglet" or using the existing Remote GPIO feature
  of PiOS) to control and view the GPIO hardware from a distance, including on
  Mac, Linux, Windows machines and other Raspberry
  Pis. [Issue #106](https://github.com/andrewdavidmackenzie/pigg/issues/106)
- Automation of release process and publishing
  packages [Issue #85](https://github.com/andrewdavidmackenzie/pigg/issues/85)
- Pre-built binaries for install on Raspberry Pi [Issue #112](https://github.com/andrewdavidmackenzie/pigg/issues/112)
  and easier install [Issue #111](https://github.com/andrewdavidmackenzie/pigg/issues/111)
- Expand support beyond Inputs and Outputs ( e.g. Clocks, PWM, I2C, UART, SPI etc.).
  Issue [#53](https://github.com/andrewdavidmackenzie/pigg/issues/53),
  [#52](https://github.com/andrewdavidmackenzie/pigg/issues/52), [#5](https://github.com/andrewdavidmackenzie/pigg/issues/5)
- True logical layout, grouping pins by function [Issue #94](https://github.com/andrewdavidmackenzie/pigg/issues/94)

## Further out ideas

- Allow connections between pins [Issue #95](https://github.com/andrewdavidmackenzie/pigg/issues/95)
- Pico support for a headless hardware backend accessed over the network
- Trigger a script or WebAssembly plugin on an input event (edge, level, etc.)

## Project Structure

### PIGGUI ("Piggy")

A binary that shows a GUI for configuring pins, observing input levels and controlling output
levels.
On Raspberry Pi it has a real GPIO hardware backend (via rppal).
On macOS, linux and windows it uses a fake GPIO hardware backend.

### PIGLET ("Piglet)

A "GUI-less" binary. Currently, it has minimal functionality. It can be built on any platform and will use the fake
Hardware backend (but not be very useful!).

If built on the Pi (with the "pi_hw" feature), then it has a real GPIO hardware backend.

It takes a file command line option. It will load the GPIO configuration from the file (like "piggui" can) and
it will apply it to the hardware. But currently there is no way to interact with it after that.

## Installing

### macOS/linux/Windows or Pi (with a fake Hardware backend)

```
cargo install pigg
```

NOTE: `cargo` will build for the machine where you are running it, so you will get a version of `piggui`
with a fake hardware backend, not real Pi GPIO hardware, but you can play with the GUI.

### On Pi with real GPIO hardware backend

To be able to interact with real Pi GPIO hardware you have two options:

- Run `cargo install --features "pi_hw"` on your Pi
- Follow the instructions before for Building from Source on your Pi
    - Use `make` to build on macOS/linux/Windows and cross-compile for your Pi, but you will need `Docker`/`Podman`
      and `cross`
      installed

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
- `target/armv7-unknown-linux-gnueabihf` - GUI version for Pi with GPIO, can be run natively from RPi command line
- `target/armv7-unknown-linux-gnueabihf` -  Headless version for Pi with GPIO, can be run natively from RPi
  command line

Use `"make run"` to start `piggui` on the local machine - for GUI development.

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
- Use `make build` to build only for the Pi. This will build both `piggui` (with GUI and GPIO) and `piglet` binary
  with GPIO only
- Use `make copy` to copy the built binaries to your raspberry pi.
- Use `make ssh` to ssh into your Pi to be able to run the binaries.

### Building for Pi on a Pi!

You should be able to use `make` or `make run` directly, and it will build `piggui` with a GUI and
also build `piglet`

## Running

### Piggui

Use `make run`

One macOS/linux/Windows this will build for the local machine, and start `piggui` with a fake hardware backend.

If you run that on a Raspberry Pi, it will detect that, and build for the Pi, with the real Pi GPIO hardware backend.

Piggui takes an optional filename argument, to attempt to load the code from. If there is an error
loading a config, the default config will be used.

To do this you can use the equivalent of what `make run` does, adding the filename:

- On a Pi: `cargo run --features "pi_hw" -- <filename>`
- On macOS, linux or Windows: `cargo run -- <filename>`

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md)

## License

See [LICENSE](LICENSE)

## Code of Conduct

See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)

## Security

See [SECURITY.md](SECURITY.md)
