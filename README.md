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

## Latest NEW Feature - pre-built images and installers

We have added pre-built images for different OS and CPU architecture, along with installers, to the automated
release process using `cargo dist`. See [INSTALLING.md](INSTALLING.md) for details.

Up-to-date instructions for installing are also be in the release notes of the
[latest release](https://github.com/andrewdavidmackenzie/pigg/releases/latest)

## Other Features

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
- The GUI (`piggui`) can connect to a Pi (that is running `piglet`) over the network, to control and view the GPIO
  hardware from a distance. The GUI can run on Mac, Linux, Windows or Raspberry Pis. Events are timestamped at source
  (as close to the hardware as possible) so network delays should not affect the waveforms displayed. Provide us
  feedback and ideas related to networking in Discussions or GH issues.
- The data required to connect to a remote node via iroh-net is called the `nodeid`. `piglet` prints this out for you
  if it is started in the foreground. When `piglet` has been started as a system service, start another instance in the
  foreground and this will detect the background instance and display its `nodeid` for you then exit.
- Take the `nodeid` and either supply it as a command line option to `piggui` (`--nodeid $nodeid`, prefixed with `-- `
  if using `cargo run`) or enter it into the GUI. To connect to a remote instance from the GUI, click on the
  "hardware menu" in the left of the info bar at the bottom of the screen and select the "Connect to remote Pi..:"
  menu item. Then enter the `nodeid` into the field provided and hit "Connect"
- Here are two videos showing the two ways to use it, with piglet running on a RPi shown via VNC.
    - Video with Dialog: https://youtu.be/aToJ1aT7NeM
    - Video using CLI argument: https://youtu.be/zcEa_Oke014

You can see more gifs and videos of features [here](assets/features.md)

## Piggui (pronounced "Piggy")

`piggui` is a GUI for configuring pins, observing input levels and controlling output levels.
On Raspberry Pi it has a real GPIO hardware backend (via rppal).
On macOS, Linux and Windows it uses a fake hardware backend (mainly for development) or can connect to a remote
hardware backend that is running `piglet`.

## Piglet

`piglet` is a "headless" command line utility that interacts with the GPIO hardware, and can either apply a
config supplied from file and stop, or can listen for config changes from a remote `piggui` and report input
level changes to the GUI.

If built on the Pi (with the "pi_hw" feature), then it has a real GPIO hardware backend.

It can be built on macOS/Linux/Windows/Pi with the "fake_hw" feature for a fake hardware backend, mainly used
for development.

It takes an optional config file as a command line option. It will load the GPIO configuration from the file
(like `piggui` can) and it will apply it to the hardware then stop.

It offers the ability to interact with the hardware from a remote `piggui`instance.
It will print out connection info at startup and start listing for Iroh network connections from `piggui` instances,
then the user can interact with it and visualize inputs level changes from the `piggui` GUI.

## Input from Raspberry Pi users wanted

We would like input from Raspberry Pi users to help us decide the order of things to work on in the future,
and input on how integrate new functionalities (e.g. I2C buses, SPI, UART, etc.).

Please let us know what you think, and suggestions, via
[Discussions](https://github.com/andrewdavidmackenzie/pigg/discussions) or GH issues.

## Roadmap

We have identified a number of areas we would like to work on after this initial release,
but would really appreciate your input on what could be most useful or just the coolest,
many have GH issues.

- Pre-built binaries for install on Raspberry Pi [Issue #112](https://github.com/andrewdavidmackenzie/pigg/issues/112)
  and easier install [Issue #111](https://github.com/andrewdavidmackenzie/pigg/issues/111)
- Expand support beyond Inputs and Outputs ( e.g. Clocks, PWM, I2C, UART, SPI etc.).
  Issue [#53](https://github.com/andrewdavidmackenzie/pigg/issues/53),
  [#52](https://github.com/andrewdavidmackenzie/pigg/issues/52), [#5](https://github.com/andrewdavidmackenzie/pigg/issues/5)
- True logical layout, grouping pins by function [Issue #94](https://github.com/andrewdavidmackenzie/pigg/issues/94)
- Allow connections between pins [Issue #95](https://github.com/andrewdavidmackenzie/pigg/issues/95)
- Pico support for a headless hardware backend accessed over the network
- Trigger a script or WebAssembly plugin on an input event (edge, level, etc.)

## Installing

TODO

## Building from Source

See [BUILDING.md](BUILDING.md)

## Running Piggui and Piglet

For details on running `piglet` and `piggui` in the foreground or as a system service, on the same machine or with a
remote GUI to Pi hardware, see [RUNNING.md](RUNNING.md)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md)

## License

See [LICENSE](LICENSE)

## Code of Conduct

See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)

## Security

See [SECURITY.md](SECURITY.md)
