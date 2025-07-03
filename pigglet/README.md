---
layout: page
title: Pigglet
---

![GH Action](https://github.com/andrewdavidmackenzie/pigg/actions/workflows/build_and_test.yml/badge.svg)
[![codecov](https://codecov.io/gh/andrewdavidmackenzie/pigg/graph/badge.svg?token=Lv5SstEMGO)](https://codecov.io/gh/andrewdavidmackenzie/pigg)

<a href="https://repology.org/project/pigg-x86-64-unknown-linux-gnu/versions">
<img src="https://repology.org/badge/vertical-allrepos/pigg-x86-64-unknown-linux-gnu.svg" alt="Packaging status">
</a>
<a href="https://www.drips.network/app/projects/github/andrewdavidmackenzie/pigg" target="_blank"><img src="https://www.drips.network/api/embed/project/https%3A%2F%2Fgithub.com%2Fandrewdavidmackenzie%2Fpigg/support.png?background=blue&style=drips&text=project&stat=dependencies" alt="Support pigg on drips.network" height="32"></a>

# pigglet - Raspberry Pi GPIO CLI tool

`pigglet` is a "headless" command line utility that interacts with Raspberry Pi GPIO hardware, and can either
apply a config supplied from file and stop, or can listen for config changes from a remote `piggui` and report
input level changes to the GUI.

Main features:

- Runs on a Pi to allow `piggui` GUI running on other platforms to configure GPIO Outputs and visualize GPIO inputs,
  with remote connections over TCP or Iroh-net
- Pre-built images for different CPU architecture, along with installers. See [INSTALLING.md](../INSTALLING.md)
- GPIO config is saved and restored across power failure or device re-start.
- mDNS discovery supported enabling the `piggui` GUI to discover them.
- Here are two videos showing the two ways to use it, with pigglet running on a RPi shown via VNC.
    - Video with Dialog: https://youtu.be/aToJ1aT7NeM
    - Video using CLI argument: https://youtu.be/zcEa_Oke014

You can see more gifs and videos of features [here](../assets/features.md)

See what's new in [latest release](https://github.com/andrewdavidmackenzie/pigg/releases/latest)

[Website](https://mackenzie-serres.net/pigg/)

## Details

If built on the Pi (with the "pi_hw" feature), then it has a real GPIO hardware backend.

It can be built on macOS/Linux/Windows/Pi with the "fake_hw" feature for a fake hardware backend, mainly used
for development (hence enabled on `dev` build).

It takes an optional config file as a command line option. It will load the GPIO configuration from the file
(like `piggui` can) and it will apply it to the hardware then stop.

It will print out connection info at startup and start listing for Iroh network connections from `piggui` instances,
then the user can interact with it and visualize inputs level changes from the `piggui` GUI.

## Porky

`porky` is an embedded application developer for the Raspberry Pi Pico and Pi Pico W for remote interaction with the
Pico's GPIO hardware. It can be connected to over TCP or USB.

For more details see [porky's README.md](../porky/)

## Supported Hardware and Operating Systems

| Application | Arch Supported | Device    | OS Supported       | Asset                                                                                                                                                               |
|-------------|----------------|-----------|--------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| pigglet     | Apple Silicon  |           | macOS 15           | [pigglet-aarch64-apple-darwin.tar.xz](https://github.com/andrewdavidmackenzie/pigg/releases/download/0.7.2/pigglet-aarch64-apple-darwin.tar.xz)                     |
|             | x86_64         |           | macOS 15           | [pigglet-x86_64-apple-darwin.tar.xz](https://github.com/andrewdavidmackenzie/pigg/releases/download/0.7.2/pigglet-x86_64-apple-darwin.tar.xz)                       |
|             | x86_64         |           | Ubuntu 24.04       | [pigglet-x86_64-unknown-linux-gnu.tar.xz](https://github.com/andrewdavidmackenzie/pigg/releases/download/0.7.2/pigglet-x86_64-unknown-linux-gnu.tar.xz)             |
|             | x86_64         |           | Windows 10         | [pigglet-x86_64-pc-windows-msvc.msi](https://github.com/andrewdavidmackenzie/pigg/releases/download/0.7.2/pigglet-x86_64-pc-windows-msvc.msi)                       |
|             | aarch64        | Pi400     | Pi OS              | [pigglet-aarch64-unknown-linux-gnu.tar.xz](https://github.com/andrewdavidmackenzie/pigg/releases/download/0.7.2/pigglet-aarch64-unknown-linux-gnu.tar.xz)           |
|             | aarch64        | Pi4       | Pi OS              | [pigglet-aarch64-unknown-linux-gnu.tar.xz](https://github.com/andrewdavidmackenzie/pigg/releases/download/0.7.2/pigglet-aarch64-unknown-linux-gnu.tar.xz)           |
|             | aarch64        | Pi5       | Pi OS              | [pigglet-aarch64-unknown-linux-gnu.tar.xz](https://github.com/andrewdavidmackenzie/pigg/releases/download/0.7.2/pigglet-aarch64-unknown-linux-gnu.tar.xz)           |
|             | arm            | Pi Zero   | Pi OS (32bit)      | [pigglet-arm-unknown-linux-gnu.tar.xz](https://github.com/andrewdavidmackenzie/pigg/releases/download/0.7.2/pigglet-arm-unknown-linux-gnu.tar.xz)                   |
|             | aarch64        | Pi Zero 2 | Pi OS (64bit)      | [pigglet-aarch64-unknown-linux-gnu.tar.xz](https://github.com/andrewdavidmackenzie/pigg/releases/download/0.7.2/pigglet-aarch64-unknown-linux-gnu.tar.xz)           |
|             | armv7 musl     | Pi3B      | Ubuntu 18.04.6 LTS | [pigglet-armv7-unknown-linux-musleabihf.tar.xz](https://github.com/andrewdavidmackenzie/pigg/releases/download/0.7.2/pigglet-armv7-unknown-linux-musleabihf.tar.xz) |

## Installing

See [INSTALLING.md](../INSTALLING.md)

## Help

See [HELP.md](../HELP.md) for help with known issues. We hope to grow this and maybe link with the GUI and reported
errors.

## Building from Source

See [BUILDING.md](../BUILDING.md)

## Running Piggui and Pigglet

For details on running `pigglet` in the foreground or as a system service. See [RUNNING.md](../RUNNING.md)

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md)

## License

See [LICENSE](../LICENSE)

## Code of Conduct

See [CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md)

## Security

See [SECURITY.md](../SECURITY.md)
