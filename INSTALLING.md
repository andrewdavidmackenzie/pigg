# Installing

Up-to-date instructions for installing are also be in the release notes of the
[latest release](https://github.com/andrewdavidmackenzie/pigg/releases/latest)

## Install prebuilt binaries via shell script

If your platform supports `sh` (and you have `curl` installed), then you can install the appropriate prebuilt binary
using:

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/andrewdavidmackenzie/pigg/releases/download/0.3.4/pigg-installer.sh | sh
```

(example shown is for version 0.3.4. Check
the [latest release](https://github.com/andrewdavidmackenzie/pigg/releases/latest) page
for the line to be used for the latest release, after 0.3.4)

## Install prebuilt binaries via Homebrew

For platforms that use homebrew, you can install the appropriate prebuilt binary using:

```sh
brew install andrewdavidmackenzie/pigg-tap/piggui
brew install andrewdavidmackenzie/pigg-tap/pigglet
```

## Install pre-built binaries via "cargo binstall"

If you have installed a rust toolchain, then you can install `cargo-binstall` from crates.io
and then use it to install the pre-built binaries, without building from source:

```sh
cargo binstall piggui
cargo binstall pigglet
```

## Other Installation options for Windows

See the Downloads section in the [latest release](https://github.com/andrewdavidmackenzie/pigg/releases/latest)
where you can find a downloadable "msi" installer for Windows that will install the appropriate pre-built binary.

You can also download a "zip" file with the prebuilt binary for Windows and run or install that manually.

## Other Installation options for Mac

See the Downloads section in the [latest release](https://github.com/andrewdavidmackenzie/pigg/releases/latest)
where you can find "tar.xz" downloads for x86 and Apple Silicon Macs.

Expand the downloaded file with `tar -xvf` and then run or manually install the binary

## Other Installation options for Mac

See the Downloads section in the [latest release](https://github.com/andrewdavidmackenzie/pigg/releases/latest)
where you can find "tar.xz" downloads for Linux x86 machines

Expand the downloaded file with `tar -xvf` and then run or manually install the binary

## Installing `udev` rules on Linux

If you installed `piggui` on Linux using a provided installation script, it may well have installed these rules for you
already. Try `piggui`. If it works without an error then you are done!

If `piggui` reports a USB permissions error in the UI, then the cause is most likely a lack of these `udev` rules.
The problem and the fix is covered by a section of [HELP.md](HELP.md#permission-denied-os-error-13-linux-only).

## Checking the version installed

Check the version you have installed is the latest with

```sh
piggui --version
pigglet --version
```

## Installing from source with `cargo install`

For this option you will need a working rust toolchain installed.

Clone the project https://github.com/andrewdavidmackenzie/pigg.git to a local disk, change into the root folder of the
project.

NOTE: Since the project has been moved to a workspace rust project, there are separate packages for each of the
top-level
parts of the project, the two installable binaries are:

- `piggui` - the GUI fir pigg
- `pigglet` - the CLI or background service for pigg

```
cargo install piggui
cargo install pigglet
```

`cargo` will build the binaries for the machine where you are running it, so:

- On a macOS/Windows/Linux host you will get a version of `piggui` and `pigglet` with a fake hardware backend,
  not real Pi GPIO hardware, but you can play with the GUI.
- On a Pi you will get a version of `piggui` and `pigglet` with a real GPIO hardware backend enabling you
  to interact with real Pi GPIO hardware.

### Piglet as a system service

You can install `pigglet` as a system service that runs in the background and is restarted at boot, so it is always
available.

- Find where the `pigglet` binary is. This could be in `target/debug` or `target/release`
- To install as a system service: `pigglet --install`
- To uninstall an existing service: `pigglet --uninstall`

Most OS require you to run this as admin/su using `sudo` or equivalent.
This has caused me some problems as `cargo` was not in `su` user's path. This problem should be reduced when we
produce pre-built binaries for you to use.

As mentioned above, `pigglet` will output information to help connect to it, but when running as a background
system service this will be in logs, and not simple for users to find.  
To facilitate getting these values you can run `pigglet` from the command line and it will
detect there is always another instance running, find the values associated with that instance and echo them to
the terminal. You can copy these and use them with `piggui` on the same, or a remote, machine.