# Running Piglet and Piggui

`piglet` is the small command line utility that can be run to interact with the hardware, while using `piggui` (on the
same machine or a remote one) for the GUI for user interaction.

## Piglet

Use `make run-piglet` or `make run-release-piglet`

One macOS/linux/Windows this will build for the local machine, and start `piglet` with a fake hardware backend.

If you run that on a Raspberry Pi, it will detect that, and build for the Pi, with the real Pi GPIO hardware backend.

`piglet` will print out to the terminal a series of values that you can use with `piggui` to connect remotely to that
`piglet` instance, notably `nodeid`.

### Piglet as a system service

You can install `piglet` as a system service that runs in the background and is restarted at boot, so it is always
available.

To facilitate getting the values required to connect to it remotely, if you run `piglet` from the command line, it will
detect there is always another instance running, and find the values associated with that instance and echo them to
the terminal. You can copy these and use them with `piggui` on the same, or a remote, machine.

## Piggui

Use `make run`

One macOS/linux/Windows this will build for the local machine, and start `piggui` with a fake hardware backend.

If you run that on a Raspberry Pi, it will detect that, and build for the Pi, with the real Pi GPIO hardware backend.

Piggui takes an optional filename argument, to attempt to load the code from. If there is an error
loading a config, the default config will be used.

To do this you can use the equivalent of what `make run` does, adding the filename:

- On a Pi: `cargo run --features "pi_hw" -- <filename>`
- On macOS, linux or Windows: `cargo run -- <filename>`
