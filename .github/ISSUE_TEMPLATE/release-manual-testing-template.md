---
name: Release Manual testing template
about: A checklist of manual tests to do before a release
title: Release Manual Test Checklist
labels: ''
assignees: ''

---

# Manual Test Template

This template lists a set of "Text Blocks" to be executed manually in different "Scenarios" when we do a new
release as (as yet) we are unable to automate such tests.

For each milestone that will culminate in a new release, create a new issue using this template, and assign it to the
milestone. Then, after other issues that change code have been completed, but before releasing, go through the tests
manually and mark as done each one until they are all done.

At the end, we list system level tests that we have been able to automate and that are executed in CI, to avoid
doing manual testing of those. That list may grow over time (and this template should be updated when it does)

## Manual Tests Prior to Release

Below we group individual tests to execute into blocks, that are to be executed in different scenarios. The purpose
of defining the blocks is to make the list of tests per scenario shorter, but the grouping can be used to skip
entire groups of tests that don't need to be repeated in all scenarios.

### Test Blocks

#### HW Interaction

- Editing of pins connected to real HW
    - Input pin connected to switch
        - When set with pull-down it reports a low level
        - When set with pull-dup it reports a high level
        - While switch is pressed it reports a low in the UI and high when released
    - Output pin connected to LED
        - When loaded from config that has a value set in it, the LED changes to that value
        - When config set manually
            - if no value set in UI the LED does not change initial value
            - Setting to high with toggle sets the LED on
            - Setting to low with toggle sets the LED off
            - Clicking the clicker changes the value while pressed and back when released
- Same as above, but config is loaded from a file.

#### UI Interaction

- Pin can be set to be an Input
    - Pullup can be set to pull-up
    - Pullup can be set to None
    - Pullup can be set to pull-down
- Pin can be set to be an Output
    - Toggle changes the displayed value and stays there
    - Toggle changes back the displayed value and stays there
    - clicker changes value while pressed only
- Board layout toggle changes the layout and changes it back and window changes size and is not clipped
- Edit some pins and the config can be saved. If later reloaded from that file the config is loaded and applied
  including any output level set in the UI
- If no edit has been made, exiting by clicking on "X" in window manager exits immediately
- If an edit has been made, exiting by clicking on "X" in window manager causes exit dialog to show
    - If exit anyway is chosen the app exits
    - If cancel exit is chosen you return to the app
        - Trying to exit again will cause the dialog to show again
- If a config was loaded from file, exiting by clicking on "X" in window manager causes exit dialog to show
    - If after loading you make an edit, it behaves as above when edit was made without loading from file first
- If edit is made and you chose to load a config then the load config or cancel dialog is shown
    - If you cancel, you return to the app
    - If you chose to load anyway, the config is loaded and overwrites previous edits and app is in "no unsaved changes"
      state
        - Trying to exit should exit immediately

#### USB (applies to Piggui+Porky scenario only)

- Piggui can detect a USB connected porky
- the device is added to the discovered devices menu
- when the porky is flashed with a "stock" UF2 from a release:
    - the "Display device details" dialog shows it as NOT connected to wifi initially
    - the "Configure wifi" option opens the dialog but it's not prefilled with anything
    - configuring with a valid ssid, it reboots and connects (dialog displays IP and port)
    - the "reset wifi" option causes a reboot and now the device no longer has a config
      and doesn't connect to the wifi

#### Networking (only applies to Piggui+Piglet and Piggui+Porky scenarios)

- Piggui can disconnect from the current hardware that was connected at startup (fake or Pi hw)
    - The board layout is not shown
    - The option to connect to a remote piglet is shown
    - The option to reconnect to local hardware is shown
- Piggui can connect to piglet using Iroh with a nodeid via command line
- Piggui can connect to piglet using Iroh with a nodeid entered via dialog
- Piggui can connect to piglet using TCP with a ip:port via command line
- Piggui can connect to piglet using TCP with a ip:port entered via dialog
- Piggui can connect to porky using TCP with a ip:port entered via dialog
    - A Pi Pico pin layout is shown
- Piggui can connect to porky using TCP with a ip:port via command line

### Scenarios

- Piggui only
    - 1 - Piggui on Macos/Linux/Windows with fake hw
    - 2 - Piggui on Pi with real GPIO hw
- Piggui and piglet
    - 3 - Piggui on Mac/Linux/Windows + Piglet on same machine
    - 4 - Piggui on Pi + Piglet on same machine
    - 5 - Piggui on Mac/Linux/Windows + Piglet on Pi
        - a) - On Pi Zero / Zero 2 (gnu-aarch64)
        - b) - On Pi 3B (armv7 gnu and musl binaries)
        - c) - On Pi 4/400 (gnu-aarch64)
        - d) - On Pi 5 (gnu-aarch64)
- Piggui and porky
    - 6 - Piggui on Mac/Linux/Windows + porky

## Manual Test Matrix

Execute the tests blocks in the specified scenario and click the checkbox when all pass.

- Scenario: 1 - Piggui on Macos/Linux/Windows with fake hw
- Test Blocks:
    - [ ] UI Interaction


- Scenario: 2 - Piggui on Pi with real GPIO hw
- Test Blocks:
    - [ ] HW Interaction
    - [ ] UI Interaction


- Scenario: 3 - Piggui on Macos/Linux/Windows + Piglet on same machine
- Test Blocks:
    - [ ] Networking


- Scenario: 4 - Piggui on Pi + Piglet on same machine
- Test Blocks:
    - [ ] HW Interaction
    - [ ] Networking


- Scenario: 5 - Piggui on Mac/Linux/Windows + Piglet on Pi
    - a) - On Pi Zero / Zero 2 (gnu-aarch64)
    - b) - On Pi 3B (armv7 gnu and musl binaries)
    - c) - On Pi 4/400 (gnu-aarch64)
    - d) - On Pi 5 (gnu-aarch64)
- Test Blocks:
    - [ ] HW Interaction
    - [ ] Networking

- Scenario 6 - Piggui on Mac/Linux/Windows + porky
    - [ ] HW Interaction
    - [ ] Networking
    - [ ] USB

## Already automated tests

- [X] piggui connects to piglet using Iroh by supplying nodeid at the command line
- [X] piggui connects to piglet using Tcp by supplying an IP and port number at the command line
