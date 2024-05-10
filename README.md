# pigg - Raspberry Pi GPIO GUI

A GUI for visualization/control of GPIO on Raspberry Pis.

## Idea stage

This is just a proposal at the idea stage.

I posted the initial idea on reddit to see if any interest, and there seemed to be some, so I created this repo to help capture input.

Later, when I actually write something and issues are clean-up, I will come back and edit this readme to 
describe the actual plan!

## Provide input!

I will enable discussions on this repo, so feel free to raise something there.

Please add issues for ideas for functionality.

## Chosen tech

For me to pursue this project, there are a few pieces of tech that I want to use, and are more or
less "non-negotiable" (or I will lose interest)

* rust
* iced for GUI (although I'm also using leptos for web, and a GUI framework in rust that also provides a web UI might be acceptable)


For GPIO on Pi I have been using rpal. I'm open to others, providing it's in rust.

## Basic / Initial Functionality

* visual representation of the GPIO connector/header with pins with numbers and names
* able to config each pin (input, output, pulled up/down, pwm etc)
* able to set status of outputs
* able to see the status of inputs

## Next batch of functionality

* Able to provide a time-view of inputs, so like an analyzer...

## Further out ideas

* trigger a script or WebAssembly plugin on an input event (edge, level, etc)
* able to have UI on different device to where GPIO is and connect remotely
* hence able to connect the native UI to a remote device, where some "agent" is running
* have an "agent" able to run on a Pi Pico
* Have a web UI able to connect to an agent on a Pi or Pico
