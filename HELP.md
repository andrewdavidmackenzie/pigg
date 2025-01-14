# Help

This files lists help for common problems found

## "Permission denied (os error 13)" (Linux ONLY)

### Issue Description

This error is due to the user/application not having permissions to write to the USB device folders and
files under `/dev/bus/usb/`.

You can check the permissions using `ls -l /dev/bus/usb/*/*`. This may show something like this:

```
crw-rw-r-- 1 root root 189,   0 Jan 10 14:57 /dev/bus/usb/001/001
crw-rw-r-- 1 root root 189,   1 Jan 10 14:57 /dev/bus/usb/001/002
crw-rw-r-- 1 root root 189,   2 Jan 10 14:57 /dev/bus/usb/001/003
crw-rw-r-- 1 root root 189,   3 Jan 10 15:31 /dev/bus/usb/001/004
crw-rw-r-- 1 root root 189,   4 Jan 10 14:57 /dev/bus/usb/001/005
crw-rw-r-- 1 root root 189,   5 Jan 10 15:53 /dev/bus/usb/001/006
crw-rw-r-- 1 root root 189, 128 Jan 10 14:57 /dev/bus/usb/002/001
crw-rw-r-- 1 root root 189, 256 Jan 10 14:57 /dev/bus/usb/003/001
crw-rw-r-- 1 root root 189, 384 Jan 10 14:57 /dev/bus/usb/004/001
```

The numbers in the paths correspond to the bus number and the device address.

You can use `lsusb` to find the bus and device address of the "pigg" "porky" device:

```
Bus 001 Device 001: ID 1d6b:0002 Linux Foundation 2.0 root hub
Bus 001 Device 002: ID babe:face pigg porky
Bus 001 Device 003: ID 04f2:b67d Chicony Electronics Co., Ltd Integrated Camera
Bus 001 Device 004: ID 06cb:00bd Synaptics, Inc. Prometheus MIS Touch Fingerprint Reader
Bus 001 Device 005: ID 8087:0aaa Intel Corp. Bluetooth 9460/9560 Jefferson Peak (JfP)
Bus 001 Device 006: ID 093a:2510 Pixart Imaging, Inc. Optical Mouse
Bus 002 Device 001: ID 1d6b:0003 Linux Foundation 3.0 root hub
Bus 003 Device 001: ID 1d6b:0002 Linux Foundation 2.0 root hub
Bus 004 Device 001: ID 1d6b:0003 Linux Foundation 3.0 root hub
```

Here "pigg" "porky" device is on Bus 001 and is Device 002, which corresponds to the second line
in the folder listing above. As you can see that device belongs to the `root` user and `root`
user group and my current user (who is not `root` or in the `root` user group) only has read
permissions, not write permission.

### Fixing the Issue:

See [relevant section in INSTALLING.md](INSTALLING.md#installing-udev-rules-on-linux)