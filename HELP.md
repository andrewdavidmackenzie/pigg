# Help

This files lists help for common problems found

## "Permission denied (os error 13)" (Linux ONLY)

### Issue Description

This error is due to the user/application not having permissions to write to the USB device folders and
files under `/dev/bus/usb/`.

In order for `piggui` to discover and control a USB connected `porky` device the user running it needs to write
to a connected USB device, via these files and folders.

This is controlled by the Linux `udev` system. The file `70.pigg.rules` gives these permissions to the user and
is part of the project and shipped as part of releases.

### Fixing the Issue:

To install the `udev` rules manually:

- Download the `70.pigg.rules` file from:
- <!---
  the [latest release]((https://github.com/andrewdavidmackenzie/pigg/releases/latest/70.pigg.rules))
  -->
    - the [repository](https://github.com/andrewdavidmackenzie/pigg/blob/master/70.pigg.rules)
- Run this command from the command line in a terminal window from the folder where you downloaded the file:
    - `sudo cp 70.pigg.rules /etc/udev/rules.d/`

If you have cloned the project repository, you can run this command from the command line in a terminal window,
in the root folder of the project:

- `make usb` (requites `root` permissions and will ask for your `root` password)

NOTE: For these rules to take affect you will need to restart your computer.
