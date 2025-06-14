![GH Action](https://github.com/andrewdavidmackenzie/pigg/actions/workflows/build_and_test.yml/badge.svg)
[![codecov](https://codecov.io/gh/andrewdavidmackenzie/pigg/graph/badge.svg?token=Lv5SstEMGO)](https://codecov.io/gh/andrewdavidmackenzie/pigg)

<a href="https://repology.org/project/pigg-x86-64-unknown-linux-gnu/versions">
<img src="https://repology.org/badge/vertical-allrepos/pigg-x86-64-unknown-linux-gnu.svg" alt="Packaging status">
</a>
<a href="https://www.drips.network/app/projects/github/andrewdavidmackenzie/pigg" target="_blank"><img src="https://www.drips.network/api/embed/project/https%3A%2F%2Fgithub.com%2Fandrewdavidmackenzie%2Fpigg/support.png?background=blue&style=drips&text=project&stat=dependencies" alt="Support pigg on drips.network" height="32"></a>

# pigg - Raspberry Pi GPIO GUI

An app for Raspberry Pi GPIO Output control and Input visualization, with GUI and CLI Support for macOS, Linux
(including Raspberry Pi) and Windows; GPIO CLI agent for Raspberry Pi and embedded applications for Pi Pico (USB)
and Pi Pico W (USB, TCP).

The GUI (Pi Gpio GUI - PIGGUI) is affectionately known as "piggy".

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

[Website](https://mackenzie-serres.net/pigg/) (just README.md content for now...)

## What's new in Release 0.7.0 - Pico 2 Support, Improved Layouts, Bug fixing and Tests

* Pico 2 and Pico 2 W full support alongside Pico and Pico W
* New compact pin layout view that shows only configured pins, suitable for devices with small displays
* Menus to configure pins are now on the pin itself to reduce space used
* Addition of a disconnected view when no connection is present. Connecting status added to the connecting menu while a
  connection is being attempted
* FakePi hardware view (mainly for development) is only included in debug builds. Release builds will show the "
  Disconnected View" initially
* LED output display is a clickable control now to reduce space used
* Updated lots of dependencies. Almost all are on the latest versions available
* Hardware compatibility tests are run in CI. Added "hw_tests" that are a set of tests running piggui against pigglet
  and
  porky running on real Hardware (Pi Zero, Pico and Pico 2) connected to my server, using a custom GH runner
* Lots of test improvements across all subprojects
* A number of small visual improvements around menus, dialogs, sizes etc.
* Improvements in developer setup Makefile targets for any contributors
* Improvements in connection handling and switching from one device directly to another
* More robust service restart by retrying with a delay on startup and failure (pending the "proper" fix (for systemd)
  which is in the works
* Improvements to tooltips, adding connection details on connection buttons
* Many code improvements, removing all unwraps among them. Project structured as a workspace enabling more code reuse
  across projects
* Added a FUNDING.yml
* Improvements in the wasm32 build, a pre-requisite to getting a web UI working
* A number of small bug fixes
*

## Other Features

- Directly on a Pi or Pi Pico or remotely from other platforms you can configure the GPIO hardware Inputs and Outputs,
  controlling the level of the Outputs and view the level of the Inputs in the GUI.
- Pre-built images for different OS and CPU architecture, along with installers. See [INSTALLING.md](INSTALLING.md) for
  details.
- Visual representation of the GPIO pins in two layouts, a "Board Pin Layout" that mimics the
  physical layout of the Pi's GPIO connector/header, or a "BCM Pin Layout" with only the programmable
  GPIO pins, ordered by BCM pin number. Physical pin layout adapts to reflect the device that `piggui` is connected
  to as Pi and Pi Pico pin outs are different.
- Each pin has its board pin number, name and function.
- Drop down selector to config each pin (Currently as an Input with or without pull-up/pull-down, or
  as an Output)
- Inputs have a visualization like an LED to show its current level (Black is unknown, Red is off, Green is on),
  plus a waveform view that shows you the recent history of the level detected on the input.
- Outputs have a toggle switch that can be used to change the stable value of the output, plus a "clicker" for quick
  inversions of the stable level, plus a waveform view showing the recent history of the level set on the Output.
- GPIO configurations can be loaded at startup with a command line filename option, or loaded via
  file-picker from the UI or saved to file via file picker, or the device will communicate it's current configuration
  to the GUI, allowing you to continue with the configuration currently being used by the GPIO hardware.
- GUI discovery of devices using mDNS for networked `pigglet`s and `porky`s, or USB for direct connected `porky`s.
- The GUI (`piggui`) can connect to a Pi (running `pigglet`) over the network, or to a Pi Pico/Pi Pico W (over the
  network or USB direct connect) to control and view the GPIO hardware from a distance.
- The GUI can run on Mac, Linux, Windows or Raspberry Pis. Events are timestamped at source (as close to the hardware
  as possible) so network delays should not affect the waveforms displayed. Please provide us feedback and ideas related
  to networking in Discussions or GH issues.
- The data required to connect to a remote node via iroh-net is called the `nodeid`. `pigglet` prints this out for you
  if it is started in the foreground. When `pigglet` has been started as a system service, start another instance in the
  foreground and this will detect the background instance and display its `nodeid` for you then exit.
- Take the `nodeid` and either supply it as a command line option to `piggui` (`--nodeid $nodeid`, prefixed with `-- `
  if using `cargo run`) or enter it into the GUI. To connect to a remote instance from the GUI, click on the
  "hardware menu" in the left of the info bar at the bottom of the screen and select the "Connect to remote Pi..."
  menu item. Then enter the `nodeid` into the field provided and hit "Connect"
- Here are two videos showing the two ways to use it, with pigglet running on a RPi shown via VNC.
    - Video with Dialog: https://youtu.be/aToJ1aT7NeM
    - Video using CLI argument: https://youtu.be/zcEa_Oke014

You can see more gifs and videos of features [here](assets/features.md)

## Piggui (pronounced "Piggy")

`piggui` is a GUI for configuring pins, observing input levels and controlling output levels.
On Raspberry Pi it has a real GPIO hardware backend (via rppal).
On macOS, Linux and Windows it uses a fake hardware backend (mainly for development) or can connect to a remote
hardware backend that is running `pigglet`.

## Piglet

`pigglet` is a "headless" command line utility that interacts with the GPIO hardware, and can either apply a
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

## Porky

`porky` is an embedded application developer for the Raspberry Pi Pico and Pi Pico W for remote interaction with the
Pico's GPIO hardware. It can be connected to over TCP or USB.

For more details see [porky's README.md](porky/README.md)

## Supported Hardware and Operating Systems

`pigg` has a number of binaries as part of the project (see descriptions above) and they are tested in CI, or
manually or are known to work as follows:

| Application | Arch Supported | Device      | OS Supported       | Asset                                      |
|-------------|----------------|-------------|--------------------|--------------------------------------------|
| piggui      | Apple Silicon  |             | macOS 15           | pigg-aarch64-apple-darwin.tar.xz           |
|             | x86_64         |             | macOS 15           | pigg-x86_64-apple-darwin.tar.xz            
|             | x86_64         |             | Ubuntu 24.04       | pigg-x86_64-unknown-linux-gnu.tar.xz       
|             | x86_64         |             | Windows 10         | pigg-x86_64-pc-windows-msvc.msi            
|             | aarch64        | Pi400       | Pi OS              | pigg-aarch64-unknown-linux-gnu.tar.xz      
|             | aarch64        | Pi4         | Pi OS              | pigg-aarch64-unknown-linux-gnu.tar.xz      
|             | aarch64        | Pi5         | Pi OS              | pigg-aarch64-unknown-linux-gnu.tar.xz      
|             | arm            | Pi Zero     | Pi OS (32bit)      | pigg-arm-unknown-linux-gnu.tar.xz          
|             | aarch64        | Pi Zero 2   | Pi OS (64bit)      | pigg-aarch64-unknown-linux-gnu.tar.xz      
|             | armv7 musl     | Pi3B        | Ubuntu 18.04.6 LTS | pigg-armv7-unknown-linux-musleabihf.tar.xz 
|             | armv7 gnu      | Pi3B        | Ubuntu 18.04.6 LTS | pigg-armv7-unknown-linux-gnueabihf.tar.xz  
| pigglet     | Apple Silicon  |             | macOS 15           | pigg-aarch64-apple-darwin.tar.xz           
|             | x86_64         |             | macOS 15           | pigg-x86_64-apple-darwin.tar.xz            
|             | x86_64         |             | Ubuntu 24.04       | pigg-x86_64-unknown-linux-gnu.tar.xz       
|             | x86_64         |             | Windows 10         | pigg-x86_64-pc-windows-msvc.msi            
|             | aarch64        | Pi400       | Pi OS              | pigg-aarch64-unknown-linux-gnu.tar.xz      
|             | aarch64        | Pi4         | Pi OS              | pigg-aarch64-unknown-linux-gnu.tar.xz      
|             | aarch64        | Pi5         | Pi OS              | pigg-aarch64-unknown-linux-gnu.tar.xz      
|             | arm            | Pi Zero     | Pi OS (32bit)      | pigg-arm-unknown-linux-gnu.tar.xz          
|             | aarch64        | Pi Zero 2   | Pi OS (64bit)      | pigg-aarch64-unknown-linux-gnu.tar.xz      
|             | armv7 musl     | Pi3B        | Ubuntu 18.04.6 LTS | pigg-armv7-unknown-linux-musleabihf.tar.xz 
| porky_w     | armv7          | Pi Pico W   | N/A                | porky_pico_w.uf2                           
| porky       | armv7          | Pi Pico     | N/A                | porky_pico.uf2                             
| porky_w2    | armv7          | Pi Pico 2 W | N/A                | porky_pico_w2.uf2                          
| porky2      | armv7          | Pi Pico 2   | N/A                | porky_pico2.uf2                            

## Input from Raspberry Pi users wanted

We would like input from Raspberry Pi users to help us decide the order of things to work on in the future,
and input on how integrate new functionalities (e.g. I2C buses, SPI, UART, etc.).

Please let us know what you think, and suggestions, via
[Discussions](https://github.com/andrewdavidmackenzie/pigg/discussions) or GH issues.

## Roadmap

We have identified a number of areas to work on in future releases, but we would really appreciate your input
on what could be most useful or just the coolest, many have GH issues.

See issues in
milestone [0.7.0](https://github.com/andrewdavidmackenzie/pigg/issues?q=is%3Aopen+is%3Aissue+milestone%3A%220.7.0+Release%22)
for the up-to-date list and progress.

- Extend Pi Pico support:
    - Support for Pi Pico 2 and Pi Pico 2 W
- Expand support beyond Inputs and Outputs ( e.g. Clocks, PWM, I2C, UART, SPI etc.).
  Issue [#53](https://github.com/andrewdavidmackenzie/pigg/issues/53), [#52](https://github.com/andrewdavidmackenzie/pigg/issues/52), [#5](https://github.com/andrewdavidmackenzie/pigg/issues/5)
- True logical layout, grouping pins by function [Issue #94](https://github.com/andrewdavidmackenzie/pigg/issues/94)
- Custom layouts to order, group pins and only show pins in use
- Smaller window sizes for devices running `piggui` with small displays
- Allow connections between pins [Issue #95](https://github.com/andrewdavidmackenzie/pigg/issues/95)
- Trigger a script or WebAssembly plugin on an input event (edge, level, etc.)

## Installing

See [INSTALLING.md](INSTALLING.md)

## Help

See [HELP.md](HELP.md) for help with known issues. We hope to grow this and maybe link with the GUI and reported
errors.

## Building from Source

See [BUILDING.md](BUILDING.md)

## Running Piggui and Piglet

For details on running `pigglet` and `piggui` in the foreground or as a system service, on the same machine or with a
remote GUI to Pi hardware, see [RUNNING.md](RUNNING.md)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md)

## License

See [LICENSE](LICENSE)

## Code of Conduct

See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)

## Security

See [SECURITY.md](SECURITY.md)
