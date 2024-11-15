# Porky

Porky is an implementation of piglet remote GPIO client for the Raspberry Pi Pico W, supporting direct USB
connections in order to configure the Wi-Fi network it should use, and then full-functionality `piggui` connections
over TCP.

See [Building.md](BUILDING.md) on details of how to build from source.

## Installing and Running `porky` on your Raspberry Pi Pico W

### Getting a UF2 file

You should find a `porky` UF2 file as part of the pre-build binaries in a release.

If you wish to build your own (e.g. maybe you want to specify a default Wi-Fi network to connect to), please consult
the [UF2 Building section of BUILDING.md](BUILDING.md#creating-a-uf2-file)

### Connect Raspberry Pi Pico W in Mass Storage Mode

- Disconnect your Raspberry Pi Pico W from USB if it is connected.
- Press and hold the BOOTSEL (the only button there is! :-) ) button on the board while you re-connect via USB
- Release the BOOTSEL button

The Pi Pico W should start in mass storage mode. It should be detected and mounted as a USB storage device by your host
Operating System.

NOTE: You may need to mount it manually on Linux.

#### MacOs

On Mac, a new volume should appear in `/Volumes`. This is usually called `RPI-RP2`.
You might get a (helpful) alert that a new USB device was plugged in.
If not you can check using `ls /Volumes`.

### Copying the UF2 file to Raspberry Pi Pico W

Then you should be able to copy your UF2 file to the mass storage device.

#### MacOS

You should be able to drag 'n' drop or copy the UF2 file to `/Volumes/RPI-RP` using `cp`,
but I (and many others on the Internet) get an error from macOS 14.

However, this works:
`ditto --norsrc --noextattr --noacl picomon.uf2 /Volumes/RPI-RP2`

people report that `rsync` may also work.

when the download is done the Pi Pico W should reboot and run `porky`.

## Configuring Wi-Fi on a Pi Pico W `porky` device

Depending on where you got your UF2 file from, `porky` may have a default Wi-Fi network configured, or none.

You need to configure the Wi-Fi network you wish `porky` to connect to in order for it to be remotely accessible
over the network via TCP.

Follow the following steps:

- Connected the Pi Pico W via USB to a host computer where you can run the `piggui` GUI application
- Run `piggui`
- You should see an info message (in the bottom right hand corner message area) that a `porky` device was detected on
  USB
- Oen the "hardware" menu (bottom menu bar). The "Discovered devices sub-menu should have sub-menu for the device
- On the device's sub-menu select the "Configure Device Wi-Fi..." option
- In the dialog enter the details of the Wi-Fi network you wish this `porky` device to connect to
- Click the "Send" button. This will send the SSID details to the porky device and the dialog should close
- The `porky` device should reboot (you will see disconnected and then connected messages in message box in the UI)
- The `porky` device should now be attempting to connect to the specified Wi-Fi network

NOTE: You may leave the device connected via USB (to power it) or disconnect it and connect it elsewhere, including
using a USB charger (no data), as the device will attempt to connect to the specified Wi-Fi network.
The USB connection to `piggui` is no longer needed.

## Getting the `porky` device's IP and port

If the Wi-Fi network was configured correctly, `porky` should have been able to connect to the network and get an
IP address. It will be listening for TCP connections on that IP on the default port of 1234.

For the `piggui` GUI application to connect to a networked `porky` device, it needs to know the device's
IP address.

## Connecting to `porky` over Wi-Fi

Once you have the IP address, in the "hardware" menu, disconnect from any other device
(including the fake local device on the host) and then select the "Connect to remote Pi..." option.

This will open the Connection dialog. Select the "Connect using TCP" tab. Then complete the fields with the IP Address
of the `porky` device and the port number 1234, and press "Connect".

`piggui` should connect to the remote `porky` device and show the pins in the default layout. Now you may control and
view remotely the GPIO hardware of your Pi Pico W - without writing a line of code and from the comfort of your
host computer.