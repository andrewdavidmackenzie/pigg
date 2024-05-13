# pigg - Raspberry Pi GPIO GUI

A GUI for visualization/control of GPIO on Raspberry Pis.

## Idea Stage

This is just a proposal at the idea stage.

I posted the initial idea on reddit to see if any interest, and there seemed to be some, so I created this repo to help capture input.

Later, when I actually write something and issues are clean-up, I will come back and edit this readme to 
describe the actual plan!

## Provide Input

I will enable discussions on this repo, so feel free to raise something there.

Please add issues for ideas for functionality.


## Chosen Tech

For me to pursue this project, there are a few pieces of tech that I want to use, and are more or
less "non-negotiable" (or I will lose interest)

* rust
* iced for GUI (although I'm also using leptos for web, and a GUI framework in rust that also provides a web UI might be acceptable)

For GPIO on Pi I have been using [rppal](https://github.com/golemparts/rppal). 
I'm open to others, providing it's in rust.

## Basic / Initial Functionality

* visual representation of the GPIO connector/header with pins with numbers and names
* able to config each pin (input, output, pulled up/down, pwm etc)
* able to set status of outputs
* able to see the status of inputs

## Next batch of functionality

* Able to provide a time-view of inputs, so like an analyzer...


## Further out ideas

* trigger a script or WebAssembly plugin on an input event (edge, level, etc)
* able to have UI on different device to where GPIO is and connect remotely
* hence able to connect the native UI to a remote device, where some "agent" is running
* have an "agent" able to run on a Pi Pico
* Have a web UI able to connect to an agent on a Pi or Pico

## Project Structure
### PIGGUI ("Piggy")
A binary that shows a GUI using Iced.
On Raspberry pi it will include GPIO 8via rppal).
On macOS and linux it will just have the UI, without GPIO.

### PIGLET ("Piglet)
A headless binary that is only built on RaspberryPi and that has no UI.

## Building and Running
### Pre-requisites
We use "cross" to cross compile for Raspberry Pi from Linux or macOS.
Install docker or podman and "cross" for cross compiling rust on your host for the Raspberry Pi.

### Building in general on host development machine
Run `"make"` on macos or linux (or in fact RPi also) host to build these binaries:
* `target/debug/piggui` - GUI version without GPIO, to enable UI development on a host
* `target/aarch64-unknown-linux-gnu/release/piggui` - GUI version for Pi with GPIO, can be run natively from RPi command line
* `target/aarch64-unknown-linux-gnu/release/piglet` - Headless version for Pi with GPIO, can be run natively from RPi command line

### Building for Pi
#### Helper Env vars
There are a couple of env vars that can be setup to help you interact with your pi.

Set these up in your env, or set them on the command line when invoking `make`

* `PI_TARGET` Which Pi to copy files to and ssh into
`PI_TARGET := pizero2w0.local`
 
* `PI_USER` The User name of your user on the pi, to be able to copy files and ssh into it
`PI_USER := andrew`

#### Make targets
* Use `"make pibuild"` to build only for the Pi. This will build both `piggui` (with GUI and GPIO) and `piglet` binary with GPIO only
* Use `"make copy"` to copy the built binaries to your raspberry pi.
* Use `"make ssh"` to ssh into your Pi to be able to run the binaries.

### Building for Linux/macOS
Use "make build"