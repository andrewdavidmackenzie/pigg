## Building from Source

### Pre-requisites

None of the developers currently has a Windows machine, so we have been unable to test this on Windows.

Note that these instructions below apply equally, whether you have cloned the repo or have downloaded and exploded the
source tarball/zip of a release.

### Building all on host development machine

`cd` into the `pigg` project root folder and then run `"make"` on macOS, linux (including on a RPi),
Windows host to build these binaries:

- `piggui/target/debug/piggui` - GUI version without GPIO, to enable UI development on a host
- `piggui/target/debug/piglet` - CLI binary that interacts with Pi Hardware for remote GUI usage
- `porky/target/thumbv6m-none-eabi/release/porky` - Binary for Pi Pico W MCU as a target for remoet GUI control

### `piggui` and `piglet` on macOS/linux/Windows or Pi (with a fake Hardware backend)

`cd` into the `piggui` subfolder and run:

```
cargo build
```

NOTE: `cargo` will build for the machine where you are running it, so you will get a version of `piggui` and `piglet`
with a fake hardware backend, not real Pi GPIO hardware, but you can play with the GUI.

Use `"make run-release"` (`cargo run --release`) to run a release build of `piggui` on the local machine
or use `make run` to start a debug build of the same.

### Building `piggui` and `piglet` on Pi with real GPIO hardware backend

To be able to interact with real Pi GPIO hardware, clone the project tp your Pi, `cd` into the project's `piggui`
subfolder and run:

- Run `cargo install` to build and install `piggui` and `piglet`
- Run `make run-piglet` to run a local version of `piglet` that remote `piggui` instances can connect to
- Run `make run-release` to build and run a release-build of `piggui` GUI