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
brew install andrewdavidmackenzie/pigg-tap/pigg
```

## Install pre-built binaries via "cargo binstall"

If you have installed a rust toolchain, then you can install `cargo-binstall` from [crates.io](https://crates.io)
and then use it to install the pre-built binaries, without building from source:

```sh
cargo binstall pigg
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

## Checking the version installed

Check the version you have installed is the latest with

```sh
piggui --version
piglet --version
```
