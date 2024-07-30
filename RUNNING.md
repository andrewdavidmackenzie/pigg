# Running Piglet and Piggui

`piglet` is the small command line utility that can be run to interact with the hardware.

`piggui` is the GUI for user interaction. `piggui` can be run natively on a raspberry
Pi and allow you to control the GPIO hardware directly from the UI.

Alternatively, you can run `piglet` to interact with the hardware (on a Pi...) and on the same machine or a remote
one run `piggui` for the GUI.

## Running Piglet

For now, while we need you to build from source:

- Use `make run-piglet` or `make run-release-piglet`

On macOS/linux/Windows this will build for the local machine, and start `piglet` with a fake hardware backend.

If you run that on a Raspberry Pi, it will detect that, and build for the Pi, with the real Pi GPIO hardware backend.

`piggui` takes an optional filename argument, to load a config from.

`piglet` will print to the terminal a series of values that you can use with `piggui` to connect remotely to that
`piglet` instance, notably `nodeid`.

### Piglet as a system service

You can install `piglet` as a system service that runs in the background and is restarted at boot, so it is always
available.

- Find where the `piglet` binary is. This could be in `target/debug` or `target/release`
- To install as a system service: `piglet --install`
- To uninstall an existing service: `piglet --uninstall`

Most OS require you to run this as admin/su using `sudo` or equivalent.
This has caused me some problems as `cargo` was not in `su` user's path. This problem should be reduced when we
produce pre-built binaries for you to use.

As mentioned above, `piglet` will output information to help connect to it, but when running as a background
system service this will be in logs, and not simple for users to find.  
To facilitate getting these values you can run `piglet` from the command line and it will
detect there is always another instance running, find the values associated with that instance and echo them to
the terminal. You can copy these and use them with `piggui` on the same, or a remote, machine.

## Piggui

For now, while we need you to build from source:

- Use `make run`

One macOS/linux/Windows this will build for the local machine
(using `cargo run --bin piggui --features "fake_hw"`), and start `piggui` with a fake hardware backend.

If you run that on a Raspberry Pi, it will detect that, and build for the Pi
(using `cargo run --bin piggui --features "pi_hw"`), with the real Pi GPIO hardware backend.

`piggui` takes an optional filename argument, to load a config from. If there is an error
loading a config, the default config will be used.

To do this you can use the equivalent of what `make run` does, adding the filename:

On a Pi:

- `cargo run --bin piggui --features "pi_hw" -- <filename>`

On macOS/Linux/Window:

- On macOS, linux or Windows: `cargo run --bin piggui --features "fake_hw" -- <filename>`

### Connecting Piggui to a remove Piglet

To connect to a remote piglet, get the `nodeid` value from the piglet instance (see above) and pass it to
`piggui` as a command line option.

When building from source, run `piggui` thus (where $nodeid is the value of the node id, quotes not required;

On a Pi:

- `cargo run --bin piggui -- --nodeid $nodeid` (builds with no hardware backend)
- `cargo run --bin piggui --features "pi_hw" -- --nodeid $nodeid` (built with a Pi hardware backend)

On macOS/Linux/Windows:

- `cargo run --bin piggui -- --nodeid $nodeid`