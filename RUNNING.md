---
layout: page
title: Running
nav_order: 3
---

# Running Piggui, Pigglet, Porky

`piggui` is the GUI for user interaction. It can be run natively on a macOS, Linux or Windows host to interact
remotely with GPIO hardware running on a Raspberry Pi (via `pigglet`) or a Raspberry Pi Pico W (via `porky`).

`pigglet` is a command line utility that can be run to interact with the hardware. This can
be run on a macOS, Linux or Windows host (for development or demo purposes, with simulated GPIO hardware) or on
a Raspberry Pi (not Pico) and connected to remotely.

`porky` is an embedded application for the Raspberry Pi Pico W, and can be connected to remotely from `piggui`.

## Piggui

`piggui` takes an optional filename argument, to load a config from. If there is an error
loading a config, the default config will be used.

- `piggui -c <filename>`
- `piggui --config <filename>`

## Running Pigglet

If run on a macOS, Linux or Windows host `pigglet` will start with a fake hardware backend, for demo purposes.

If run that on a Raspberry Pi, it will start with the real Pi GPIO hardware backend.

- `pigglet`

`pigglet` will print to the terminal a series of values that you can use with `piggui` to connect remotely to that
`pigglet` instance, such `nodeid` for an Iroh connection, or IP Address and Port for a TCP connection.

`pigglet` also takes an optional filename argument, to load a config from. If there is an error
loading a config, the default config will be used.

- `piggui -c <filename>`
- `piggui --config <filename>`
-

## Running Porky

For details on how to install the embedded `porky` application binary on your Raspberry Pi Pico W and run it, refer
to `porky`'s own [README.md](porky/README.md)

### Connecting Piggui to a remote Pigglet or Porky - Command Line Options

To connect to a remote `pigglet` using the Iroh network method, get the `nodeid` value from the pigglet instance (see
above)
and pass it to
`piggui` as a command line option.

- `piggui --nodeid $nodeid`

To connect to a remote `pigglet` using TCP, get the `ip` value (ip address and port together as a string, seperated by a
':') from the pigglet instance (see above) and pass it to `piggui` as a command line option.

- `piggui --ip $ip`

To connect to a USB attached 'porky' device, you need to get the devices porky serial number (from debug logs or
a previous connection to it) and have 'piggui' connect to it using the 'usb' argument

- `piggui --usb $serial_number`

### Connecting Piggui to a remote Pigglet/Porky - Using the GUI

To connect to a remote `pigglet` using the Iroh network method, get the `nodeid` value from the pigglet instance (see
above).
Open the "hardware" menu (bottom center), select the "Disconnect" menu item to disconnect from the current device
(this maybe the simulated GPIO hardware), then chose the "Connect to remote Pi..." menu.

This will display the Conection Dialog. Enter the `nodeid` of the `pigglet` you wish to connect to and hit "Connect".

To connect to a remote `pigglet` using TCP, get the `ip` value (ip address and port together as a string, seperated by a
':') from the pigglet instance (see above), open the "Connection Dialog" as above, enter the IP Address and Port and
hit "Connect"

### Configuring then connecting to a remote Porky Pi Pico W device

As the device has no display, interacting with it is slightly more complicated, but `piggui` has you covered with
the ability to detect and configure `porky` devices from its GUI.

For more details, refer to the [porky/README.md](porky/README.md)